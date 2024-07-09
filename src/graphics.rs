use crate::camera::Camera;
use bytemuck::cast_slice;
use glam::Vec3;
use log::info;
use std::{borrow::Cow, error::Error, sync::Arc};
use util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

/// The `Graphics` object is a container for all the context and functionality related to rendering in
/// the `rasterless` engine. Creating the `Graphics` object will grab the GPU device and set up the
/// rendering pipeline, and calling the `draw` function on the object runs the pipeline.
///
/// The rendering pipeline consists of a single compute pass, which marches rays through the top-level
/// volume hierarchy. The single compute pass handles all graphical effects, including primary visibility,
/// lighting, and even post-processing.
///
/// The only `wgpu` feature your GPU must support in order to create a `Graphics` object is the
/// `BGRA8UNORM_STORAGE` feature, which should be available on all devices running DX12, Vulkan, and Metal;
/// both on web and native.
pub(crate) struct Graphics {
    window: Arc<Window>,
    camera: Camera,
    camera_uniform: Buffer,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    raymarch_bind_group_layout: BindGroupLayout,
    raymarch_pipeline: ComputePipeline,
}

impl Graphics {
    /// Create a `Graphics` object. This will create a window, as well as initializing all needed GPU context
    /// and handles rendering. Returns an `Error` result if window creation fails, if surface creation (the
    /// texture that will be rendered to and displayed on the window) fails, or if searching for a valid
    /// adapter/device (the GPU) fails.
    ///
    /// The only `wgpu` feature your GPU must support in order to create a `Graphics` object is the
    /// `BGRA8UNORM_STORAGE` feature, which should be available on all devices running DX12, Vulkan, and Metal;
    /// both on web and native.
    pub(crate) async fn init(event_loop: &ActiveEventLoop) -> Result<Self, Box<dyn Error>> {
        // Create a new window
        let attributes = Window::default_attributes().with_title("Rasterless");
        let window = Arc::new(event_loop.create_window(attributes)?);

        // Init the shader output texture to be large enough to cover any monitor on the host system
        let mut full_width = 1;
        let mut full_height = 1;
        for monitor in window.available_monitors() {
            full_width = full_width.max(monitor.size().width);
            full_height = full_height.max(monitor.size().height);
        }

        // Ensure the current dimensions of the window are positive
        let mut current_size = window.inner_size();
        current_size.width = current_size.width.max(1);
        current_size.height = current_size.height.max(1);

        // Configure device
        // ----------------
        let instance: Instance = Instance::default();

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    // The BGRA8UNORM_STORAGE feature should be available on all modern platforms
                    // and graphics backends, including the web. It is required because the surface
                    // texture we write to must be of the Bgra8Unorm format--the only format
                    // guaranteed to be supported by all platforms. Compute shaders can only write
                    // to storage textures, and and using a texture of this format as a storage
                    // texture is not allowed without this feature.
                    required_features: Features::BGRA8UNORM_STORAGE,
                    required_limits: Limits::default(),
                },
                None,
            )
            .await?;

        let config = SurfaceConfiguration {
            // We use the surface texture as a storage texture so that we can write to it directly
            // from the compute shader.
            usage: TextureUsages::STORAGE_BINDING,
            // Bgra8Unorm (and the Srgb variant) is the only guaranteed format that all platforms
            // will support.
            format: TextureFormat::Bgra8UnormSrgb,
            width: current_size.width,
            height: current_size.height,
            // AutoVsync is supported everywhere because of fallbacks which allow it to gracefully
            // fail when no Vsync is available.
            present_mode: PresentMode::AutoVsync,
            // Only buffer 2 frames ahead.
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            // Note that only view format conversions that are guaranteed are the original format
            // of the texture and the Srgb/Non-Srgb variant of the format.
            view_formats: vec![TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb],
        };
        surface.configure(&device, &config);

        // Create raymarching (compute pass) pipeline
        // ------------------------------------------
        let camera_position = Vec3::new(0.0, 3.0, -3.0);
        let camera = Camera::new(
            camera_position,
            (-camera_position).normalize(),
            Vec3::Y,
            90.0,
            100.0,
            current_size.width,
            current_size.height,
        );

        let camera_uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: &camera.to_uniform_data(),
            // Add the copy destionation usage so that we can send write_buffer commands for
            // when the camera object changes.
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let raymarch_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Raymarch (Compute Pass) Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
        });

        let raymarch_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Raymarch (Compute Pass) Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            // Note that write-only is the only valid access pattern for storage
                            // textures on web.
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Bgra8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let raymarch_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Raymarch (Compute Pass) Pipeline Layout"),
            bind_group_layouts: &[&raymarch_bind_group_layout],
            push_constant_ranges: &[],
        });

        let raymarch_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Raymarch (Compute Pass) Pipeline"),
            layout: Some(&raymarch_pipeline_layout),
            module: &raymarch_shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        info!(
            "Rasterless graphics context initialized. Selected adapter: {:?}.",
            adapter.get_info()
        );

        Ok(Self {
            window,
            camera,
            camera_uniform,
            surface,
            device,
            queue,
            config,
            raymarch_bind_group_layout,
            raymarch_pipeline,
        })
    }

    /// This method will update the surface texture (which is the texture that gets rendered to) and the
    /// camera aspect ratio to the size provided. It's purpose is to update the rendering context when the
    /// window resizes.
    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // Ensure that the window dimensions are positive
        let width = new_size.width.max(1);
        let height = new_size.height.max(1);

        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        self.camera.resize(width, height);

        // Queue a write_buffer command, which will be executed on demand, just before the next compute pass.
        // This will update the camera uniform buffer with the new screen size.
        self.queue.write_buffer(
            &self.camera_uniform,
            0,
            cast_slice(&self.camera.to_uniform_data()),
        );

        // On macOS the window needs to be redrawn manually after resizing. There's negligible drawbacks for
        // rendering an additional frame on other platforms, so this functionality has not been isolated to
        // macOS.
        self.window.request_redraw();
    }

    /// The `draw` method will submit a new raymarching compute pass to the GPU, render a new frame, and
    /// display the frame on the window. The rendering pipeline consists of a single compute pass, which
    /// marches rays through the top-level volume hierarchy. The single compute pass handles all graphical
    /// effects, including primary visibility, lighting, and even post-processing.
    ///
    /// Note that the compute pass writes directly to the surface texture, which is what will eventually
    /// be displayed on the window. It does not make a separate texture or copy any textures.
    ///
    /// This method will return an `Error` result if it cannot get the current surface texture for any
    /// reason.
    pub(crate) fn draw(&self) -> Result<(), Box<dyn Error>> {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Command Encoder"),
            });

        let frame = self.surface.get_current_texture()?;

        // We construct a new binding every frame because our reference to the surface texture is only
        // valid for a single frame. It should not be a costly operation to construct the bind group--
        // its size is small and construction is simple.
        let raymarch_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Raymarch (Compute Pass) Bind Group"),
            layout: &self.raymarch_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.camera_uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&frame.texture.create_view(
                        &TextureViewDescriptor {
                            label: Some("Raymarch (Compute Pass) Surface Texture View"),
                            format: Some(TextureFormat::Bgra8Unorm),
                            dimension: Some(TextureViewDimension::D2),
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        // Open a scoped block so that all values we allocate in it are dropped at the end of the block.
        // We drop the compute pass because it borrows and continues to borrow the encoder after we
        // call `begin_compute_pass`.
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Raymarching Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_bind_group(0, &raymarch_bind_group, &[]);
            compute_pass.set_pipeline(&self.raymarch_pipeline);

            // TODO: Workgroup sizes should be tested. Currently a workgroup is dispatched for every pixel,
            // but it may be faster to dispatch workgroups of larger sizes.
            compute_pass.dispatch_workgroups(self.config.width, self.config.height, 1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
