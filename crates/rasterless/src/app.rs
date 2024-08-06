use crate::graphics::Graphics;
use log::{error, warn};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

pub(crate) struct AppState {
    render_attempts: u8,
    init_attempts: u8,
    graphics: Option<Graphics>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            render_attempts: 0,
            init_attempts: 0,
            graphics: None,
        }
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_none() {
            let graphics = pollster::block_on(Graphics::init(event_loop));
            if graphics.is_err() {
                self.init_attempts += 1;
                if self.init_attempts < 3 {
                    warn!(
                        "Failed to initialize the GPU device on try {}. Will try again.",
                        self.init_attempts
                    );
                } else {
                    error!(
                        "Failed to initialized the GPU device after 3 tries! Closing application."
                    );
                    event_loop.exit();
                }
            } else {
                self.graphics = Some(graphics.unwrap());
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let Some(graphics) = self.graphics.as_mut() {
                    graphics.resize(new_size);
                } else {
                    warn!("Resize requested before graphics context has been successfully initialized!");
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(graphics) = &self.graphics {
                    let render_result = graphics.draw();
                    if render_result.is_err() {
                        self.render_attempts += 1;
                        if self.render_attempts < 3 {
                            warn!(
                                "Failed to acquire swap texture on try {}. Will try again.",
                                self.render_attempts
                            );
                        } else {
                            error!("Failed to acquire swap texture after 3 tries! Closing application.");
                            event_loop.exit();
                        }
                        return;
                    } else {
                        self.render_attempts = 0;
                    }
                } else {
                    warn!("Redraw requested before graphics context has been successfully initialized!");
                }
            }
            _ => (),
        }
    }
}
