use egui_wgpu_grapher::{app, grapher_egui};

use clap::Parser;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Parser, Clone, Debug, Default)]
struct Args {
    #[arg(long)]
    scene: Option<grapher_egui::GrapherSceneMode>,
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let args = Args::parse();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let display_handle = Box::from(event_loop.owned_display_handle());
    let mut app = app::App::new(args.scene, display_handle);
    event_loop
        .run_app(&mut app)
        .expect("Winit event loop failed to start.");
}
