use crate::{
    egui::{egui_tools::EguiRenderer, ui::UiState},
    grapher::{
        self, math::FunctionHolder, render::MultisampleData, scene::solid::graph::GraphScene,
    },
    grapher_egui::{GrapherScene, GrapherSceneMode, RenderUiState, graph, image_viewer, model},
};
use egui_file_dialog::FileDialog;
use egui_wgpu::wgpu::{self, Limits};
use winit::window::Window;

pub enum FileInputState {
    Hidden,
    NeedsInput,
    BadPath,
    InvalidFile,
    NeedsChecked,
}

pub enum SceneLoadingState {
    NoData,
    NeedsLoaded,
    Loaded,
}

pub struct AppState {
    // wgpu and egui state
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub egui_renderer: EguiRenderer,

    // Should scene run its updates during redraw.
    pub scene_updates_paused: bool,
    // If GUI has focus some input events are blocked.
    pub gui_has_focus: bool,

    // File picker with persistent state.
    pub file_dialog: FileDialog,

    // GUI state machine.
    pub scene_mode: GrapherSceneMode,
    pub file_input_state: FileInputState,
    pub scene_loading_state: SceneLoadingState,

    // Graphics scene state.
    pub grapher_state: grapher::render::RenderState,
    pub grapher_scene: GrapherScene,

    // ui state needed persisted across renders
    pub ui_data: UiState,
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
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: features,
                required_limits: Limits {
                    // TODO: combine bindings into fewer groups
                    max_bind_groups: 5,
                    ..Default::default()
                },
                ..Default::default()
            })
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
        let grapher_state = grapher::render::RenderState::new(&device, &surface_config).await;
        let render_ui_state =
            RenderUiState::from_render_preferences(&grapher_state.render_preferences);
        let scale_factor = 1.0;
        let ui_data = UiState {
            render_ui_state,
            selected_scene_index: initial_scene.into(),
            scale_factor,
            function_valid: true,
            ..Default::default()
        };

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            //
            scene_updates_paused: false,
            gui_has_focus: false,
            //
            file_dialog: FileDialog::new().as_modal(false),
            //
            scene_mode: initial_scene,
            file_input_state: FileInputState::Hidden,
            scene_loading_state: SceneLoadingState::NoData,
            //
            grapher_state,
            grapher_scene: GrapherScene::None,
            //
            ui_data,
        }
    }

    pub(super) fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        // Resize depth buffer texture.
        self.grapher_state.depth_buffer =
            grapher::pipeline::texture::DepthBuffer::create(&self.surface_config, &self.device);
        // Resize MSAA texture.
        self.grapher_state.msaa_data = MultisampleData::create(&self.surface_config, &self.device);

        // update camera aspect ratio
        self.grapher_state.camera_state.camera.aspect = width as f32 / height as f32;
        self.grapher_state.update(&mut self.queue);
    }

    pub fn update_graph(&mut self, function: FunctionHolder) {
        if let GrapherScene::Graph(graph_scene_data) = &mut self.grapher_scene {
            graph_scene_data.graph_scene.function = Some(function);
            graph_scene_data.graph_scene.try_rebuild_scene(
                &self.device,
                &self.surface_config,
                &self.grapher_state,
            );
        }
    }

    pub(super) fn handle_scene_changes(&mut self) {
        match self.scene_mode {
            GrapherSceneMode::Graph => {
                self.scene_change_graph();
            }
            GrapherSceneMode::Model => {
                self.scene_change_model();
            }
            GrapherSceneMode::ImageViewer => {
                self.scene_change_image();
            }
        };
    }

    pub fn hide_file_input(&mut self) {
        self.file_input_state = FileInputState::Hidden;
        self.ui_data.show_file_input = false;
    }

    pub fn show_file_input(&mut self) {
        if !self.ui_data.show_file_input {
            self.file_input_state = FileInputState::NeedsInput;
        }
        self.ui_data.show_file_input = true;
        self.file_dialog.pick_file();
    }
}

impl AppState {
    fn scene_change_graph(&mut self) {
        self.hide_file_input();

        // Detect change of mode.
        if matches!(self.grapher_scene, GrapherScene::Changed) {
            self.grapher_scene = GrapherScene::None;
            self.scene_loading_state = SceneLoadingState::NoData;
        }

        #[allow(clippy::single_match)]
        match self.scene_loading_state {
            SceneLoadingState::NoData => {
                self.grapher_state
                    .camera_state
                    .reset_camera(&self.queue, &self.surface_config);

                let graph_scene = GraphScene::default();
                self.grapher_scene =
                    GrapherScene::Graph(Box::from(graph::GraphSceneData::new(graph_scene)));
                self.scene_loading_state = SceneLoadingState::Loaded;
                self.gui_has_focus = false;
            }

            SceneLoadingState::NeedsLoaded => {
                // Nothing to do.
            }

            SceneLoadingState::Loaded => {
                // Nothing to do.
            }
        }
    }

    fn scene_change_model(&mut self) {
        // Detect change of mode.
        if matches!(self.grapher_scene, GrapherScene::Changed) {
            self.grapher_scene = GrapherScene::None;
            self.scene_loading_state = SceneLoadingState::NoData;
            self.ui_data.filename = "".into();
            self.show_file_input();
        }

        #[allow(clippy::single_match)]
        match self.scene_loading_state {
            SceneLoadingState::NoData => match self.file_input_state {
                FileInputState::NeedsChecked => {
                    self.scene_loading_state = SceneLoadingState::NeedsLoaded;
                }
                _ => {}
            },

            SceneLoadingState::NeedsLoaded => {
                self.grapher_state
                    .camera_state
                    .reset_camera(&self.queue, &self.surface_config);

                // Try loading scene from file.
                let model_scene = grapher::scene::textured::model::model_scene(
                    &self.device,
                    &self.queue,
                    &self.surface_config,
                    &mut self.grapher_state,
                    &self.ui_data.filename,
                );

                if let Some(scene) = model_scene {
                    self.grapher_scene = GrapherScene::Model(model::ModelSceneData::new(scene));
                    self.hide_file_input();
                    self.scene_loading_state = SceneLoadingState::Loaded;
                    self.gui_has_focus = false;
                } else {
                    self.grapher_scene = GrapherScene::None;
                    self.file_input_state = FileInputState::InvalidFile;
                    self.scene_loading_state = SceneLoadingState::NoData;
                }
            }

            SceneLoadingState::Loaded => match self.file_input_state {
                FileInputState::NeedsChecked => {
                    self.scene_loading_state = SceneLoadingState::NeedsLoaded;
                }
                _ => {}
            },
        }
    }

    fn scene_change_image(&mut self) {
        // Detect change of mode.
        if matches!(self.grapher_scene, GrapherScene::Changed) {
            self.grapher_scene = GrapherScene::None;
            self.scene_loading_state = SceneLoadingState::NoData;
            self.ui_data.filename = "".into();
            self.show_file_input();
        }

        #[allow(clippy::single_match)]
        match self.scene_loading_state {
            SceneLoadingState::NoData => match self.file_input_state {
                FileInputState::NeedsChecked => {
                    self.scene_loading_state = SceneLoadingState::NeedsLoaded;
                }
                _ => {}
            },

            SceneLoadingState::NeedsLoaded => {
                // Sets up the camera for 2D image display.
                let image_scene = grapher::scene::textured::image_viewer::image_viewer_scene(
                    &self.device,
                    &self.queue,
                    &self.surface_config,
                    &mut self.grapher_state,
                    &self.ui_data.filename,
                );

                if let Some(scene) = image_scene {
                    self.grapher_scene =
                        GrapherScene::ImageViewer(image_viewer::ImageViewerSceneData::new(scene));
                    self.hide_file_input();
                    self.scene_loading_state = SceneLoadingState::Loaded;
                    self.gui_has_focus = false;
                } else {
                    self.grapher_scene = GrapherScene::None;
                    self.file_input_state = FileInputState::InvalidFile;
                    self.scene_loading_state = SceneLoadingState::NoData;
                }
            }

            SceneLoadingState::Loaded => match self.file_input_state {
                FileInputState::NeedsChecked => {
                    self.scene_loading_state = SceneLoadingState::NeedsLoaded;
                }
                _ => {}
            },
        }
    }
}
