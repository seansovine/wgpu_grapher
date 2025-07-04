mod app;
mod egui_tools;
mod grapher;
#[allow(unreachable_patterns)]
mod grapher_egui;
mod ui;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = app::App::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}
