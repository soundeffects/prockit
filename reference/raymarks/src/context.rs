use crate::shaders::Shader;
use log::info;
use std::{
    fs::File,
    io::Write,
    mem::replace,
    path::{Path, PathBuf},
};
use wgpu::*;

/// Reusable device info and utilities for all benchmarks.
pub struct BenchmarkContext {
    device: Device,
    queue: Queue,
    commands: CommandEncoder,
    render_target: Texture,
    output_staging_buffer: Buffer,
}

impl BenchmarkContext {
    /// Create a new benchmark context, requesting a high-perfomance device which has all features
    /// required for all benchmarks.
    pub async fn new() -> Self {
        let instance: Instance = Instance::default();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let commands = device.create_command_encoder(&CommandEncoderDescriptor::default());

        // Create default render target of size 1024x1024
        let render_target = Self::render_target(&device, (1024, 1024));
        let output_staging_buffer = Self::output_staging_buffer(&device, (1024, 1024));

        info!(
            "Context initialized. GPU adapter info: {:?}",
            adapter.get_info()
        );

        Self {
            device,
            queue,
            commands,
            render_target,
            output_staging_buffer,
        }
    }

    /// Create a new benchmark context, blocking the current thread until the GPU is ready.
    pub fn new_sync() -> Self {
        pollster::block_on(Self::new())
    }

    /// Load a shader from the `src/shaders` directory.
    pub fn load_shader(&self, shader: Shader) -> ShaderModule {
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(shader.load_source().into()),
            })
    }

    /// Create a new rasterization pipeline.
    pub fn rasterization_pipeline(&self) -> RenderPipeline {
        let shader = self.load_shader(Shader::Rasterization);

        self.device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vertex_shader"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment_shader"),
                    compilation_options: Default::default(),
                    targets: &[Some(TextureFormat::Rgba8UnormSrgb.into())],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
                cache: None,
            })
    }

    /// Resize the render target and output staging buffer to the given size.
    pub fn resize_render_target(&mut self, size: (u32, u32)) {
        self.render_target.destroy();
        self.render_target = Self::render_target(&self.device, size);
        self.output_staging_buffer.destroy();
        self.output_staging_buffer = Self::output_staging_buffer(&self.device, size);
    }

    /// Create a new rasterization pass.
    pub fn rasterization_pass(&mut self) {
        let pipeline = self.rasterization_pipeline();

        // First, render to the render target
        {
            let mut render_pass = self.commands.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self
                        .render_target
                        .create_view(&TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&pipeline);
            render_pass.draw(0..3, 0..1);
        }

        // Then, copy the render target to the output staging buffer
        self.commands.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &self.render_target,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyBuffer {
                buffer: &self.output_staging_buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    // This needs to be a multiple of 256. Normally we would need to pad
                    // it but we here know it will work out anyways.
                    bytes_per_row: Some(self.render_target.width() * 4),
                    rows_per_image: Some(self.render_target.height()),
                },
            },
            Extent3d {
                width: self.render_target.width(),
                height: self.render_target.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    /// To queue all written commands and passes, we swap the old command encoder with a new one, and submit the old one.
    pub fn submit(&mut self) {
        let old_commands = replace(
            &mut self.commands,
            self.device
                .create_command_encoder(&CommandEncoderDescriptor::default()),
        );
        self.queue.submit(Some(old_commands.finish()));
    }

    /// Save the current render target to a PNG file.
    pub async fn save_render_target(&self, filename: &str) {
        let width = self.render_target.width();
        let height = self.render_target.height();
        let mut texture_data = Vec::<u8>::with_capacity((width * height * 4) as usize);
        let buffer_slice = self.output_staging_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(MapMode::Read, move |r| sender.send(r).unwrap());
        self.device.poll(Maintain::wait()).panic_on_timeout();
        receiver.recv_async().await.unwrap().unwrap();
        {
            let view = buffer_slice.get_mapped_range();
            texture_data.extend_from_slice(&view[..]);
        }

        let mut png_data = Vec::<u8>::with_capacity(texture_data.len());
        let mut encoder = png::Encoder::new(std::io::Cursor::new(&mut png_data), width, height);
        encoder.set_color(png::ColorType::Rgba);
        let mut png_writer = encoder.write_header().unwrap();
        png_writer.write_image_data(&texture_data[..]).unwrap();
        png_writer.finish().unwrap();

        let mut file = File::create(
            Self::image_directory().join(format!("{}_{}x{}.png", filename, width, height)),
        )
        .unwrap();
        file.write_all(&png_data[..]).unwrap();
    }

    /// Save the current render target to a PNG file, blocking the current thread until the data has been read from the GPU.
    pub fn save_render_target_sync(&self, filename: &str) {
        pollster::block_on(self.save_render_target(filename))
    }

    /// Get the directory containing the images saved by the benchmark.
    pub fn image_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("images")
    }

    /// Private method to create a render target texture.
    fn render_target(device: &Device, size: (u32, u32)) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[TextureFormat::Rgba8UnormSrgb],
        })
    }

    /// Private method to create an output staging buffer.
    fn output_staging_buffer(device: &Device, size: (u32, u32)) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: None,
            size: size.0 as u64 * size.1 as u64 * 4,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }
}
