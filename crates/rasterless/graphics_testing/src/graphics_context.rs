use std::sync::Arc;
use anyhow::Result;
use bytemuck::cast_slice;
use winit::{
    dpi::PhysicalSize,
    window::Window,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt}, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites, CompositeAlphaMode, Device, DeviceDescriptor, Face, FragmentState, FrontFace, IndexFormat, Instance, InstanceDescriptor, LoadOp, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor, VertexState
};

use crate::model::{ColoredVertex, Model, PENTAGON_INDICES, PENTAGON_VERTICES};
pub struct GraphicsContext {
    window: Arc<Window>,
    gpu_device: Device,
    gpu_queue: Queue,
    render_surface: Surface<'static>,
    render_surface_format: TextureFormat,
    render_surface_size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    model: Model,
}

impl GraphicsContext {
    /// Test
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = Instance::new(&InstanceDescriptor::default());
        let adapter = instance.request_adapter(&RequestAdapterOptions::default()).await?;
        let (gpu_device, gpu_queue) = adapter.request_device(&DeviceDescriptor::default()).await?;
        let render_surface = instance.create_surface(window.clone())?;
        let render_surface_format = render_surface.get_capabilities(&adapter).formats[0];
        let render_surface_size = window.inner_size();
        let shader = gpu_device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout = gpu_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = gpu_device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                buffers: &[ColoredVertex::buffer_layout()],
                compilation_options: PipelineCompilationOptions::default(),
                entry_point: Some("vs_main"),
            },
            fragment: Some(FragmentState {
                module: &shader,
                targets: &[Some(ColorTargetState {
                    format: render_surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
                entry_point: Some("fs_main"),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        let model = Model::pentagon(&gpu_device);
        let mut context = Self {
            window,
            gpu_device,
            gpu_queue,
            render_surface,
            render_surface_format,
            render_surface_size,
            render_pipeline,
            model,
        };
        context.configure_render_surface();
        Ok(context)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.render_surface_size = new_size;
        self.configure_render_surface();
    }

    fn configure_render_surface(&mut self) {
        self.render_surface.configure(&self.gpu_device, &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.render_surface_format,
            view_formats: vec![self.render_surface_format.add_srgb_suffix()],
            alpha_mode: CompositeAlphaMode::Auto,
            width: self.render_surface_size.width,
            height: self.render_surface_size.height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
        });
    }

    pub fn render(&mut self) -> Result<()> {
        let surface_texture = self.render_surface.get_current_texture()?;
        let texture_view = surface_texture.texture
            .create_view(&TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.render_surface_format.add_srgb_suffix()),
                ..Default::default()
            });
        let mut encoder = self.gpu_device.create_command_encoder(&Default::default());
        let mut render_pass= encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.model.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.model.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.model.index_count, 0, 0..1);
        drop(render_pass);
        self.gpu_queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
        self.window.request_redraw();
        Ok(())
    }
}