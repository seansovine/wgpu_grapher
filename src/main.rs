mod camera;
mod graph;
mod matrix;
mod mesh;
mod pipeline;
mod render;
mod state;

use winit::{
  event::*,
  event_loop::EventLoop,
  keyboard::{KeyCode, PhysicalKey},
  window::WindowBuilder,
};

// program main

fn main() {
  pollster::block_on(run_event_loop());
}

// implement main event loop

pub async fn run_event_loop() {
  env_logger::init();

  let event_loop = EventLoop::new().unwrap();
  let window = WindowBuilder::new().build(&event_loop).unwrap();
  window.set_title("wgpu grapher");

  let mut state = state::RenderState::new(&window).await;
  let scene = mesh::graph_scene(&state);

  log::info!("Starting event loop!");

  event_loop
    .run(move |event, control_flow| match event {
      Event::WindowEvent {
        ref event,
        window_id,
      } if window_id == state.window().id() => {
        if !state.handle_user_input(event) {
          match event {
            // window closed or escape pressed
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
              event:
                KeyEvent {
                  state: ElementState::Pressed,
                  physical_key: PhysicalKey::Code(KeyCode::Escape),
                  ..
                },
              ..
            } => control_flow.exit(),

            // handle window resize
            WindowEvent::Resized(physical_size) => {
              state.resize(*physical_size);
            }

            // handle redraw
            WindowEvent::RedrawRequested => {
              // request another redraw event after this one for continuous update
              state.window().request_redraw();

              state.update();
              match render::render(&mut state, &scene) {
                Ok(_) => {}
                // swap chain needs updated or recreated (wgpu docs)
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                  state.resize(state.size)
                }

                // out of memory or other error considered fatal
                Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                  log::error!("Out of memory or other error.");
                  control_flow.exit();
                }

                // present has taken too long
                Err(wgpu::SurfaceError::Timeout) => {
                  log::warn!("Surface timeout.")
                }
              }
            }

            _ => {} // other window event
          }
        }
      }

      _ => {} // non-window event
    })
    .unwrap();
}
