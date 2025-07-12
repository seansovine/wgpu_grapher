mod state;
use state::*;

use crate::{
    egui::ui::{render_file_window, render_window, FileInputState},
    grapher_egui::{validate_path, GrapherSceneMode},
};
use egui_wgpu::{
    wgpu::{self, SurfaceError},
    ScreenDescriptor,
};
use std::{
    sync::Arc,
    thread,
    time::{self, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

pub struct App {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
    window_attributes: WindowAttributes,

    // timing variables
    last_update_time: Instant,
    last_render_time: Instant,
    accumulated_secs: f32,
    render_count: usize,
    avg_framerate: f32,

    // whether to update scene state on each redraw event
    scene_updates_paused: bool,
    // indicates a ui component is focused, to block other input
    editing: bool,

    // initial_scene
    initial_scene: GrapherSceneMode,
}

impl App {
    // time between state updates; helps control CPU usage and simulation timing
    const RENDER_TIMEOUT: time::Duration = time::Duration::from_millis(20);
    // only render after this much time has elapsed; for accumulator
    const RENDER_TIME_INCR: f32 = 1.0 / 60.0;
    // after how many frame renders to update the framerate estimate
    const REPORT_FRAMES_INTERVAL: usize = 100;

    pub fn new(initial_scene: Option<GrapherSceneMode>) -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let window_attributes = Window::default_attributes().with_title("WGPU Grapher");

        let last_update_time = time::Instant::now();
        let last_render_time = time::Instant::now();
        let accumulated_secs = 0.0_f32;
        let render_count = 0_usize;
        let avg_framerate = 60.0f32;

        let scene_updates_paused = false;
        let editing = false;

        Self {
            instance,
            state: None,
            window: None,
            window_attributes,

            last_update_time,
            last_render_time,
            accumulated_secs,
            render_count,
            avg_framerate,

            scene_updates_paused,
            editing,

            initial_scene: initial_scene.unwrap_or_default(),
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1360;
        let initial_height = 768;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let state = AppState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_height,
            self.initial_scene,
        )
        .await;

        // Docs: gracefully handle redundant... Resumed events
        if self.window.is_none() {
            self.window.replace(window);
            self.state.replace(state);
        }
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.state.as_mut().unwrap().resize_surface(width, height);
        }
    }

    fn handle_redraw(&mut self) {
        // if minimized don't redraw
        if let Some(window) = self.window.as_ref() {
            if let Some(min) = window.is_minimized() {
                if min {
                    return;
                }
            }
        }

        let state = self.state.as_mut().unwrap();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32
                * state.ui_state.scale_factor,
        };

        let surface_texture = state.surface.get_current_texture();

        match surface_texture {
            Err(SurfaceError::Outdated) => {
                // template authors: for resizing and minimization handling
                return;
            }
            Err(_) => {
                // panic on other errors (for now)
                surface_texture.expect("Failed to acquire next swap chain texture");
                return;
            }
            Ok(_) => {}
        };

        let surface_texture = surface_texture.unwrap();

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if let Some(grapher_scene) = state.grapher_scene.as_mut() {
            grapher_scene.render(&surface_view, &mut encoder, &state.grapher_state);
        }

        let window = self.window.as_ref().unwrap();

        {
            state.egui_renderer.begin_frame(window);

            let editing = &mut self.editing;
            let context = &state.egui_renderer.context();

            egui::Window::new("Settings")
                .resizable(true)
                .default_size([200.0, 330.00])
                .vscroll(true)
                .default_open(true)
                .show(context, |ui| {
                    render_window(
                        context.pixels_per_point(),
                        ui,
                        editing,
                        state.grapher_scene.as_mut(),
                        &mut state.grapher_state,
                        &mut state.ui_state,
                        &mut state.selected_scene,
                    );
                });

            if !matches!(state.ui_state.file_window_state, FileInputState::Hidden) {
                *editing = true;
                let is_valid = !matches!(
                    state.ui_state.file_window_state,
                    FileInputState::BadPath | FileInputState::InvalidFile,
                );
                render_file_window(
                    context,
                    &mut state.ui_state.filename,
                    |filename| {
                        if !validate_path(filename) {
                            state.ui_state.file_window_state = FileInputState::BadPath;
                        } else {
                            state.ui_state.file_window_state = FileInputState::NeedsChecked;
                        }
                    },
                    is_valid,
                );
            }

            state.egui_renderer.end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

impl ApplicationHandler for App {
    /// Handles startup and resume from system suspsend. See discussion in
    ///  [winit docs](https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html#portability).
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(self.window_attributes.clone())
            .unwrap();
        pollster::block_on(self.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let Some(window) = self.window.as_mut() else {
            return;
        };

        // let egui render to process the event first
        state.egui_renderer.handle_input(window, &event);

        // short-circuits if editing
        if !self.editing && state.grapher_state.handle_user_input(&event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                window.request_redraw();

                // allow state updates and renders to have different frequencies
                self.accumulated_secs += self.last_update_time.elapsed().as_secs_f32();
                self.last_update_time = time::Instant::now();

                let do_render = self.accumulated_secs >= Self::RENDER_TIME_INCR;

                if !self.scene_updates_paused && state.grapher_scene.is_some() {
                    state.grapher_scene.as_mut().unwrap().update(
                        &state.device,
                        &state.surface_config,
                        &state.queue,
                        &state.grapher_state,
                        do_render,
                    );
                }

                if state.ui_state.render_ui_state.needs_update {
                    state
                        .grapher_state
                        .render_preferences
                        .update_uniform(&state.queue);
                    state.ui_state.render_ui_state.needs_update = false;
                }

                if do_render {
                    self.accumulated_secs -= Self::RENDER_TIME_INCR;

                    // allow grapher to do things like updating the camera
                    state.grapher_state.update(&mut state.queue);

                    self.handle_redraw();
                    self.render_count += 1;

                    // check if scene needs changed; reborrow to satisfy checker
                    self.state
                        .as_mut()
                        .unwrap()
                        .update_grapher_scene(&mut self.editing);

                    // update framerate estimate
                    if self.render_count == Self::REPORT_FRAMES_INTERVAL {
                        self.avg_framerate = Self::REPORT_FRAMES_INTERVAL as f32
                            / self.last_render_time.elapsed().as_secs_f32();
                        self.render_count = 0;
                        self.last_render_time = time::Instant::now();
                    }
                }

                // send/handle events less often for efficiency
                thread::sleep(Self::RENDER_TIMEOUT);
            }

            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
