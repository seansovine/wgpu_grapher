mod state;
use egui_file_dialog::DialogState;
use state::*;

use crate::{
    egui::{
        components::{self, HasFocus, validate_path},
        ui::create_gui,
    },
    grapher,
    grapher_egui::GrapherSceneMode,
};
use egui_wgpu::{
    ScreenDescriptor,
    wgpu::{self, SurfaceError},
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

        // egui docs: gracefully handle redundant... Resumed events
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
        if let Some(window) = self.window.as_ref()
            && let Some(min) = window.is_minimized()
            && min
        {
            return;
        }

        let state = self.state.as_mut().unwrap();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32
                * state.ui_data.scale_factor,
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

        // Render graphics scene.
        if state.grapher_scene.is_some() {
            state
                .grapher_scene
                .render(&surface_view, &mut encoder, &state.grapher_state);
        }

        // Render GUI.
        {
            let window = self.window.as_ref().unwrap();
            state.egui_renderer.begin_frame(window);
            Self::build_gui(state);
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

    fn build_gui(state: &mut AppState) {
        match state.file_input_state {
            FileInputState::NeedsInput => {
                let context = &state.egui_renderer.context();
                state.file_dialog.update(context);
                // Check if the user picked a file.
                if let Some(path) = state.file_dialog.take_picked() {
                    state.ui_data.filename = path.to_string_lossy().to_string();
                    state.file_input_state = FileInputState::NeedsChecked;
                }
                if matches!(state.file_dialog.state(), DialogState::Cancelled) {
                    state.hide_file_input();
                }
            }
            FileInputState::InvalidFile => {
                let context = state.egui_renderer.context();
                let modal = egui::containers::Modal::new("file_load_failed_modal".into());
                let mut close_clicked = false;
                let _ = modal.show(context, |ui| {
                    ui.heading("Load Failed");
                    ui.label("Failed to load the selected file.");
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
                if close_clicked {
                    state.file_input_state = FileInputState::Hidden;
                    state.show_file_input();
                }
            }
            _ => {}
        }

        let context = &state.egui_renderer.context();
        let editing = &mut state.gui_has_focus;

        // main settings window
        egui::Window::new("Settings")
            .resizable(true)
            .default_size([200.0, 225.0])
            .default_pos([15.0, 15.0])
            .vscroll(true)
            .default_open(true)
            .show(context, |ui| {
                create_gui(
                    context.pixels_per_point(),
                    ui,
                    editing,
                    &mut state.grapher_scene,
                    &mut state.grapher_state,
                    &mut state.ui_data,
                    &mut state.scene_mode,
                );
            });

        // TODO: We're in process of replacing with a file picker library.
        const DISABLED: bool = true;
        // maybe show file input window
        if !DISABLED && state.ui_data.show_file_input {
            *editing = true;
            let is_valid = !matches!(
                state.file_input_state,
                FileInputState::BadPath | FileInputState::InvalidFile,
            );
            components::validated_text_input_window(
                context,
                "File",
                &mut state.ui_data.filename,
                |filename| {
                    if !validate_path(filename) {
                        state.file_input_state = FileInputState::BadPath;
                    } else {
                        state.file_input_state = FileInputState::NeedsChecked;
                    }
                },
                is_valid,
            );
        } else {
            *editing = false;
        }

        // show function input in graph mode
        if matches!(state.scene_mode, GrapherSceneMode::Graph) {
            let mut is_valid = state.ui_data.function_valid;
            let mut function = None;
            {
                let is_valid_ref = &mut is_valid;
                let HasFocus(has_focus) = components::validated_text_input_window(
                    context,
                    "Function",
                    &mut state.ui_data.function_string,
                    |func_str| {
                        function = grapher::math::try_parse_function_string(func_str);
                        *is_valid_ref = function.is_some();
                    },
                    state.ui_data.function_valid,
                );
                *editing = has_focus;
            }
            if let Some(func) = function {
                state.update_graph(func);
            }
            state.ui_data.function_valid = is_valid;
        }
    }
}

impl ApplicationHandler for App {
    /// Handles startup and resume from system suspsend. See discussion in
    /// [winit docs](https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html#portability).
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

        // Let egui process event first.
        state.egui_renderer.handle_input(window, &event);

        // Stop here if GUI has focus or if the event was handled input.
        if !state.gui_has_focus && state.grapher_state.handle_user_input(&event) {
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

                // Accumulate time for render timeout.
                self.accumulated_secs += self.last_update_time.elapsed().as_secs_f32();
                self.last_update_time = time::Instant::now();

                // Throttle rendering for efficiency.
                let do_render = self.accumulated_secs >= Self::RENDER_TIME_INCR;

                // Let scene run any of its own internal updates, which include things like
                // key press events that were recorded since last redraw and need handling.
                if !state.scene_updates_paused && state.grapher_scene.is_some() {
                    state.grapher_scene.update(
                        &state.device,
                        &state.surface_config,
                        &state.queue,
                        &state.grapher_state,
                        do_render,
                    );
                }

                // Update preference uniform if needed.
                if state.ui_data.render_ui_state.needs_prefs_update {
                    state
                        .grapher_state
                        .render_preferences
                        .update_uniform(&state.queue);
                    state.ui_data.render_ui_state.needs_prefs_update = false;
                }

                // Re-render the scene.
                if do_render {
                    self.accumulated_secs -= Self::RENDER_TIME_INCR;

                    // Let grapher handle internal updates, which would be
                    // used for things like time-dependent scenes or animations.
                    state.grapher_state.update(&mut state.queue);

                    self.handle_redraw();
                    self.render_count += 1;
                    self.state.as_mut().unwrap().handle_scene_changes();

                    if self.render_count == Self::REPORT_FRAMES_INTERVAL {
                        self.avg_framerate = Self::REPORT_FRAMES_INTERVAL as f32
                            / self.last_render_time.elapsed().as_secs_f32();
                        self.render_count = 0;
                        self.last_render_time = time::Instant::now();
                    }
                }

                // Throttle event handling often for efficiency.
                thread::sleep(Self::RENDER_TIMEOUT);
            }

            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
