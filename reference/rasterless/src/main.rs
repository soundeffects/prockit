use log::LevelFilter;
use std::error::Error;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod camera;
mod graphics;

use app::AppState;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter(Some(module_path!()), LevelFilter::Info)
        .parse_default_env()
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AppState::new();
    event_loop.run_app(&mut app).map_err(Into::into)
}
