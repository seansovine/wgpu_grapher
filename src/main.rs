mod camera;
mod math;
mod matrix;
mod mesh;
mod pipeline;
mod render;

use crate::mesh::RenderScene;

use clap::{Parser, Subcommand};

use winit::{
  event::*,
  event_loop::EventLoop,
  keyboard::{KeyCode, PhysicalKey},
  window::WindowBuilder,
};

use std::{thread, time};

// command line args

#[derive(Debug, Parser)]
pub struct CliArgs {
  #[clap(subcommand)]
  pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
  Graph,
  MeltingGraph,
  WaveEquation,
}

// program main

fn main() {
  pollster::block_on(run_event_loop());
}

// between-frame delay; aiming for ~60 fps
const RENDER_TIMEOUT: time::Duration = time::Duration::from_nanos(16_666_667);

// implement main event loop

pub async fn run_event_loop() {
  env_logger::init();
  let args = CliArgs::parse();

  let event_loop = EventLoop::new().unwrap();
  let window = WindowBuilder::new().build(&event_loop).unwrap();
  window.set_title("wgpu grapher");

  let mut state = render::RenderState::new(&window).await;

  #[allow(unused_mut)]
  let mut scene: Box<dyn RenderScene> = match args.command {
    Command::Graph => Box::from(mesh::graph_scene(&state)),
    Command::MeltingGraph => Box::from(mesh::melting_graph_scene(&state)),
    Command::WaveEquation => Box::from(mesh::wave_eqn_scene(&state)),
  };

  log::info!("Starting event loop!");

  let mut time = time::Instant::now();
  let mut framecount = 0_usize;

  event_loop
    .run(move |event, control_flow| match event {
      Event::WindowEvent {
        ref event,
        window_id,
      } if window_id == state.window().id() => {
        if !state.handle_user_input(event) {
          let elapsed = time.elapsed().as_millis();
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

              // update framerate
              framecount += 1;
              state.framerate = 1000_f32 * framecount as f32 / elapsed as f32;

              // log framerate once per second
              if elapsed >= 1000 {
                log::info!("FPS: {}", state.framerate);
                framecount = 0;
                time = time::Instant::now();
              }

              scene.update(&state);
              state.update();

              match render::render(&state, scene.scene()) {
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

              thread::sleep(RENDER_TIMEOUT);
            }

            _ => {} // other window event
          }
        }
      }

      _ => {} // non-window event
    })
    .unwrap();
}
