use winit::event_loop::{ControlFlow, EventLoop};

use crate::engine::App;

mod engine;
mod ui;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Event loop error: {e}");
        std::process::exit(1);
    }
}
