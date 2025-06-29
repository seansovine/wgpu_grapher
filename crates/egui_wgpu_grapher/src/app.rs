use crate::egui_tools::EguiRenderer;
use crate::grapher;
use crate::grapher_egui::{GraphSceneData, GrapherScene, RenderUiState};
use crate::graphics::GraphicsState;
use crate::ui::{render_window, UiState};
use egui_wgpu::wgpu::core::device;
use egui_wgpu::wgpu::SurfaceError;
use egui_wgpu::{wgpu, ScreenDescriptor};
use std::{
    sync::Arc,
    thread,
    time::{self, Instant},
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub struct AppState {
    // wgpu and egui state
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,

    // ui state needed persisted across renders
    pub ui_state: UiState,

    // state for grapher render objects
    pub grapher_state: grapher::render::RenderState,
    pub grapher_scene: GrapherScene,
}

impl AppState {
    async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::POLYGON_MODE_LINE;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, window);

        let scale_factor = 1.0;

        let grapher_state = grapher::render::RenderState::new(&device, &surface_config).await;

        let graph_scene = grapher::mesh::graph_scene(&device, &surface_config, &grapher_state);

        let grapher_scene = GrapherScene::Graph(GraphSceneData::new(graph_scene));

        let render_ui_state =
            RenderUiState::from_render_preferences(&grapher_state.render_preferences);

        let ui_state = UiState { render_ui_state };

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
            ui_state,
            grapher_state,
            grapher_scene,
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        // resize depth buffer
        self.grapher_state.depth_buffer =
            grapher::pipeline::texture::DepthBuffer::create(&self.surface_config, &self.device);

        // update camera aspect ratio
        self.grapher_state.camera_state.camera.aspect = width as f32 / height as f32;
        self.grapher_state.update(&mut self.queue);
    }
}

pub struct App {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,

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
}

impl App {
    // time between state updates; helps control CPU usage and simulation timing
    const RENDER_TIMEOUT: time::Duration = time::Duration::from_millis(20);
    // only render after this much time has elapsed; for accumulator
    const RENDER_TIME_INCR: f32 = 1.0 / 60.0;
    // after how many frame renders to update the framerate estimate
    const REPORT_FRAMES_INTERVAL: usize = 100;

    pub fn new() -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

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
            last_update_time,
            last_render_time,
            accumulated_secs,
            render_count,
            avg_framerate,
            scene_updates_paused,
            editing,
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
            initial_width,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
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
                * state.scale_factor,
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

        state
            .grapher_scene
            .render(&surface_view, &mut encoder, &state.grapher_state);

        let window = self.window.as_ref().unwrap();

        {
            state.egui_renderer.begin_frame(window);

            let editing = &mut self.editing;

            let context = &state.egui_renderer.context();
            egui::Window::new("Settings")
                .resizable(true)
                .default_size([200.0, 300.00])
                .vscroll(true)
                .default_open(true)
                .show(context, |ui| {
                    render_window(
                        &mut state.scale_factor,
                        context.pixels_per_point(),
                        ui,
                        editing,
                        &mut state.grapher_scene,
                        &mut state.grapher_state,
                        &mut state.ui_state,
                    );
                });

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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
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
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
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

                if !self.scene_updates_paused {
                    state.grapher_scene.update(
                        &state.device,
                        &state.surface_config,
                        &state.queue,
                        &state.grapher_state,
                        do_render,
                    );
                }

                if state.ui_state.render_ui_state.needs_update {
                    state.grapher_state.render_preferences.update(&state.queue);
                    state.ui_state.render_ui_state.needs_update = false;
                }

                if do_render {
                    self.accumulated_secs -= Self::RENDER_TIME_INCR;

                    // allow grapher to do things like updating the camera
                    state.grapher_state.update(&mut state.queue);

                    self.handle_redraw();
                    self.render_count += 1;

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
