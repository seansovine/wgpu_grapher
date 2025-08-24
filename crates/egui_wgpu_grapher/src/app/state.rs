use crate::{
    egui::{
        egui_tools::EguiRenderer,
        ui::{FileInputState, UiState},
    },
    grapher,
    grapher_egui::{graph, image_viewer, model, GrapherScene, GrapherSceneMode, RenderUiState},
};
use egui_wgpu::wgpu::{self, Limits};
use winit::window::Window;

pub struct AppState {
    // wgpu and egui state
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub egui_renderer: EguiRenderer,

    // whether to update scene state on each redraw event
    pub scene_updates_paused: bool,
    // indicates a ui component is focused, to block other input
    pub editing: bool,

    // state for grapher render objects
    pub selected_scene: GrapherSceneMode,
    pub grapher_state: grapher::render::RenderState,
    pub grapher_scene: Option<GrapherScene>,

    // ui state needed persisted across renders
    pub ui_state: UiState,
}

impl AppState {
    pub(super) async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
        initial_scene: GrapherSceneMode,
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
                    required_limits: Limits {
                        // TODO: combine bindings into fewer groups
                        max_bind_groups: 5,
                        ..Default::default()
                    },
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

        let render_ui_state =
            RenderUiState::from_render_preferences(&grapher_state.render_preferences);
        let ui_state = UiState {
            render_ui_state,
            selected_scene_index: initial_scene.into(),
            scale_factor,
            ..Default::default()
        };

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,

            scene_updates_paused: false,
            editing: false,

            selected_scene: initial_scene,
            grapher_state,
            grapher_scene: None,

            ui_state,
        }
    }

    pub(super) fn resize_surface(&mut self, width: u32, height: u32) {
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

    pub(super) fn update_grapher_scene(&mut self) {
        match self.selected_scene {
            GrapherSceneMode::Graph => {
                self.ui_state.file_window_state = FileInputState::Hidden;

                if matches!(self.grapher_scene, Some(GrapherScene::Graph(_))) {
                    return;
                }
                self.editing = false;

                // restore previous camera and light if they were saved
                self.grapher_state.camera_state.maybe_restore_camera();
                self.grapher_state
                    .light_state
                    .maybe_restore_light(&self.queue);

                let graph_scene = grapher::mesh::solid::graph::graph_scene(
                    &self.device,
                    &self.surface_config,
                    &self.grapher_state,
                );

                let grapher_scene = GrapherScene::Graph(graph::GraphSceneData::new(graph_scene));

                self.grapher_scene = Some(grapher_scene);
            }

            GrapherSceneMode::Model => {
                if matches!(self.grapher_scene, Some(GrapherScene::Model(_))) {
                    if !matches!(
                        self.ui_state.file_window_state,
                        FileInputState::NeedsChecked
                    ) {
                        return;
                    }
                } else {
                    // Scene doesn't match selection, so need to reload scene.
                    // First check if current filename points to valid glTF.
                    self.ui_state.file_window_state = FileInputState::NeedsChecked;
                }
                if self.ui_state.filename.is_empty() {
                    self.grapher_scene = None;
                    self.ui_state.file_window_state = FileInputState::NeedsInput;
                }
                if !matches!(
                    self.ui_state.file_window_state,
                    FileInputState::NeedsChecked
                ) {
                    return;
                }
                // TODO: maybe clean up the logic here

                // restore previous camera and light if they were saved
                self.grapher_state.camera_state.maybe_restore_camera();
                self.grapher_state
                    .light_state
                    .maybe_restore_light(&self.queue);

                let model_scene = grapher::mesh::textured::model::model_scene(
                    &self.device,
                    &self.queue,
                    &self.surface_config,
                    &mut self.grapher_state,
                    &self.ui_state.filename,
                );

                if let Some(scene) = model_scene {
                    self.grapher_scene =
                        Some(GrapherScene::Model(model::ModelSceneData::new(scene)));
                    self.ui_state.file_window_state = FileInputState::Hidden;
                    self.editing = false;
                } else {
                    self.grapher_scene = None;
                    self.ui_state.file_window_state = FileInputState::InvalidFile;
                }
            }

            GrapherSceneMode::ImageViewer => {
                if matches!(self.grapher_scene, Some(GrapherScene::ImageViewer(_))) {
                    if !matches!(
                        self.ui_state.file_window_state,
                        FileInputState::NeedsChecked
                    ) {
                        return;
                    }
                } else {
                    // Scene doesn't match selection, so need to reload scene.
                    // First check if current filename points to valid glTF.
                    self.ui_state.file_window_state = FileInputState::NeedsChecked;
                }
                if self.ui_state.filename.is_empty() {
                    self.grapher_scene = None;
                    self.ui_state.file_window_state = FileInputState::NeedsInput;
                }
                if !matches!(
                    self.ui_state.file_window_state,
                    FileInputState::NeedsChecked
                ) {
                    return;
                }
                // TODO: maybe clean up the logic here

                // save old camera and lightstate
                self.grapher_state.camera_state.save_camera();
                self.grapher_state.light_state.save_light();

                let image_scene = grapher::mesh::textured::image_viewer::image_viewer_scene(
                    &self.device,
                    &self.queue,
                    &self.surface_config,
                    &mut self.grapher_state,
                    &self.ui_state.filename,
                );

                if let Some(scene) = image_scene {
                    self.grapher_scene = Some(GrapherScene::ImageViewer(
                        image_viewer::ImageViewerSceneData::new(scene),
                    ));
                    self.ui_state.file_window_state = FileInputState::Hidden;
                    self.editing = false;
                } else {
                    self.grapher_scene = None;
                    self.ui_state.file_window_state = FileInputState::InvalidFile;
                }
            }
        };
    }
}
