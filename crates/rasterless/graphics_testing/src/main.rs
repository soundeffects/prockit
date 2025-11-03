use std::sync::Arc;
use winit::{
    application::ApplicationHandler, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}, event::WindowEvent
};

mod model;
mod graphics_context;
use graphics_context::GraphicsContext;

#[derive(Default)]
struct App {
    graphics_context: Option<GraphicsContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let graphics_context = pollster::block_on(GraphicsContext::new(window.clone()));
        self.graphics_context = Some(graphics_context.unwrap());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let graphics_context = self.graphics_context.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                graphics_context.render().unwrap();
            }
            WindowEvent::Resized(size) => {
                graphics_context.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default()).unwrap();
}