use crate::{
    egui::{egui_tools::EguiRenderer, ui::UiState},
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

    // state for grapher render objects
    pub selected_scene: GrapherSceneMode,
    pub grapher_state: grapher::render::RenderState,
    pub grapher_scene: GrapherScene,

    // ui state needed persisted across renders
    pub ui_state: UiState,
}

impl AppState {
    // grapher scene to be loaded on app start
    const DEFAULT_SCENE_CHOICE: GrapherSceneMode = GrapherSceneMode::Model;

    pub(super) async fn new(
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
        let graph_scene =
            grapher::mesh::solid::graph::graph_scene(&device, &surface_config, &grapher_state);
        let grapher_scene = GrapherScene::Graph(graph::GraphSceneData::new(graph_scene));

        let render_ui_state =
            RenderUiState::from_render_preferences(&grapher_state.render_preferences);
        let ui_state = UiState {
            render_ui_state,
            selected_scene_index: Self::DEFAULT_SCENE_CHOICE.into(),
            scale_factor,
        };

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,

            selected_scene: Self::DEFAULT_SCENE_CHOICE,
            grapher_state,
            grapher_scene,

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
        let grapher_scene = match self.selected_scene {
            GrapherSceneMode::Graph => {
                if matches!(self.grapher_scene, GrapherScene::Graph(_)) {
                    return;
                }

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

                GrapherScene::Graph(graph::GraphSceneData::new(graph_scene))
            }
            GrapherSceneMode::Model => {
                if matches!(self.grapher_scene, GrapherScene::Model(_)) {
                    return;
                }

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
                );

                GrapherScene::Model(model::ModelSceneData::new(model_scene))
            }
            GrapherSceneMode::ImageViewer => {
                if matches!(self.grapher_scene, GrapherScene::ImageViewer(_)) {
                    return;
                }

                // save old camera and lightstate
                self.grapher_state.camera_state.save_camera();
                self.grapher_state.light_state.save_light();

                // TODO: hard-coded path for testing
                const TEST_IMAGE: &str = "assets/pexels-arjay-neyra-2152024526-32225792.jpg";

                let image_scene = grapher::mesh::textured::image_viewer::image_viewer_scene(
                    &self.device,
                    &self.queue,
                    &self.surface_config,
                    &mut self.grapher_state,
                    TEST_IMAGE,
                );

                GrapherScene::ImageViewer(image_viewer::ImageViewerSceneData::new(image_scene))
            }
        };

        self.grapher_scene = grapher_scene;
    }
}
