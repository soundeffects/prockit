use log::{error, info, warn, LevelFilter};
use std::{borrow::Cow, error::Error, sync::Arc};
use wgpu::{util::initialize_adapter_from_env_or_default, *};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter(Some(module_path!()), LevelFilter::Info)
        .parse_default_env()
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AppState::default();
    event_loop.run_app(&mut app).map_err(Into::into)
}

#[derive(Default)]
struct AppState {
    render_attempts: u8,
    init_attempts: u8,
    context: Option<InitializedContext>,
}

struct InitializedContext {
    window: Arc<Window>,
    _instance: Instance,
    surface: Surface<'static>,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    _shader: ShaderModule,
    _pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
    config: SurfaceConfiguration,
}

impl InitializedContext {
    async fn init(event_loop: &ActiveEventLoop) -> Result<Self, Box<dyn Error>> {
        // Create a new window
        let attributes = Window::default_attributes().with_title("Rasterless");
        let window = Arc::new(event_loop.create_window(attributes)?);

        // Ensure dimensions of the window are positive
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        // Get the adapter and surface
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = initialize_adapter_from_env_or_default(&instance, Some(&surface))
            .await
            .ok_or("Failed to find an appropriate adapter")?;

        info!("Selected adapter: {:?}", adapter.get_info());

        // Get the device and queue
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await?;

        // Load in the shader module from the wgsl file
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        // Create objects needed for render pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor::default());
        let capabilites = surface.get_capabilities(&adapter);
        let format = capabilites.formats[0];

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(format.into())],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        // Configure surface
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or("Surface config construction failed")?;
        surface.configure(&device, &config);

        Ok(Self {
            window,
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            _shader: shader,
            _pipeline_layout: pipeline_layout,
            render_pipeline,
            config,
        })
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_none() {
            let context = pollster::block_on(InitializedContext::init(event_loop));
            if context.is_err() {
                if self.init_attempts > 3 {
                    error!("Could not initialize the GPU device after 3 attempts! Closing application.");
                    event_loop.exit();
                } else {
                    self.init_attempts += 1;
                    warn!(
                        "Failed to initialized the GPU device on try {}. Will try again.",
                        self.init_attempts
                    );
                }
            } else {
                self.context = Some(context.unwrap());
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(context) = self.context.as_mut() {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(new_size) => {
                    context.config.width = new_size.width.max(1);
                    context.config.height = new_size.height.max(1);
                    context.surface.configure(&context.device, &context.config);

                    info!(
                        "Window resized to width {} and height {}.",
                        context.config.width, context.config.height
                    );

                    // On macos the window needs to be redrawn manually after resizing
                    context.window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    let frame_result = context.surface.get_current_texture();
                    if frame_result.is_err() {
                        self.render_attempts += 1;
                        warn!(
                            "Failed to acquire swap texture on try {}. Will try again.",
                            self.render_attempts
                        );
                        return;
                    }
                    let frame = frame_result.unwrap();
                    let view = frame.texture.create_view(&TextureViewDescriptor::default());
                    let mut encoder = context
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor { label: None });
                    {
                        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(RenderPassColorAttachment {
                                view: &view,
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
                        render_pass.set_pipeline(&context.render_pipeline);
                        render_pass.draw(0..3, 0..1);
                    }

                    context.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                _ => (),
            }
        }
    }
}
