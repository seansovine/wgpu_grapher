use crate::{
    CliArgs, Command,
    mesh::{self, RenderScene},
    render,
};

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use std::{thread, time};

// time between state updates; helps control CPU usage and simulation timing
const RENDER_TIMEOUT: time::Duration = time::Duration::from_millis(3);

// implement main event loop

/// Setup render state and run event loop.
pub async fn run(args: CliArgs) {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("wgpu grapher");

    let mut gpu_state = render::GpuState::new(&window).await;
    let mut state = render::RenderState::new(&gpu_state.device, &gpu_state.config).await;

    let mut scene: Box<dyn RenderScene> = match args.command {
        Command::Graph => Box::from(mesh::graph_scene(
            &gpu_state.device,
            &gpu_state.config,
            &state,
        )),
        Command::WaveEquation => Box::from(mesh::wave_eqn_scene(
            &gpu_state.device,
            &gpu_state.config,
            &state,
        )),
        Command::HeatEquation => Box::from(mesh::heat_eqn_scene(
            &gpu_state.device,
            &gpu_state.config,
            &state,
        )),
        Command::Image(args) => Box::from(mesh::image_viewer_scene(
            &gpu_state.device,
            &gpu_state.queue,
            &gpu_state.config,
            &state,
            &args.path,
        )),
        Command::WaveEquationTexture => Box::from(mesh::wave_eqn_texture_scene(
            &gpu_state.device,
            &gpu_state.queue,
            &gpu_state.config,
            &state,
        )),
    };

    log::info!("Starting event loop!");

    let mut last_update_time = time::Instant::now();
    let mut last_render_time = time::Instant::now();
    let mut render_count = 0_usize;

    let mut accumulated_time = 0.0_f32;
    const RENDER_TIME_INCR: f32 = 1.0 / 60.0;

    let mut updates_paused = false;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == gpu_state.window().id() => {
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

                        // pause state updates
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::KeyP),
                                    ..
                                },
                            ..
                        } => updates_paused = !updates_paused,

                        // handle window resize
                        WindowEvent::Resized(physical_size) => {
                            gpu_state.resize(*physical_size, &mut state);
                        }

                        // handle redraw
                        WindowEvent::RedrawRequested => {
                            // request another redraw event after this one for continuous update
                            gpu_state.window().request_redraw();

                            accumulated_time += last_update_time.elapsed().as_secs_f32();
                            last_update_time = time::Instant::now();

                            let do_render = accumulated_time >= RENDER_TIME_INCR;

                            if !updates_paused {
                                scene.update(&gpu_state.queue, &state, do_render);
                            }

                            if do_render {
                                accumulated_time -= RENDER_TIME_INCR;
                                state.update(&mut gpu_state.queue);

                                match render::render(&mut gpu_state, &state, scene.scene()) {
                                    Ok(_) => {}
                                    // swap chain needs updated or recreated (wgpu docs)
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => gpu_state.resize(gpu_state.size, &mut state),

                                    // out of memory or other error considered fatal
                                    Err(
                                        wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other,
                                    ) => {
                                        log::error!("Out of memory or other error.");
                                        control_flow.exit();
                                    }

                                    // present has taken too long
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout.")
                                    }
                                }

                                // update framerate
                                render_count += 1;
                                const REPORT_FRAMES: usize = 100;

                                if render_count == REPORT_FRAMES {
                                    state.framerate = REPORT_FRAMES as f32
                                        / last_render_time.elapsed().as_secs_f32();
                                    log::info!("FPS: {}", state.framerate);
                                    render_count = 0;
                                    last_render_time = time::Instant::now();
                                }
                            }

                            thread::sleep(RENDER_TIMEOUT);
                        } // RedrawRequested

                        _ => {} // other window events ignored
                    } // match event
                } // if !state.handle_user_input(event)
            } // Event::WindowEvent

            _ => {} // non-window event
        })
        .unwrap();
}
