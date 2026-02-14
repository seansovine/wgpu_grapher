//! Code to manage the different modes of the app, to dispatch high-level
//! calls based on the current mode, and to map GUI-modified state to
//! internal handler functions for the current mode.

pub mod graph_scene;
pub mod image_scene;
pub mod model_scene;
pub mod solver_scene;

use crate::{
    egui::ui::UiState,
    grapher::{
        pipeline::render_preferences::RenderPreferences,
        render::{ShadowState, render_2d},
        scene::{GpuVertex, RenderScene, solid::graph::GraphScene},
    },
    grapher_egui::{
        image_scene::{ImageViewerSceneData, parameter_ui_image_viewer},
        solver_scene::SolverSceneData,
    },
};
use graph_scene::{GraphSceneData, parameter_ui_graph};
use model_scene::{ModelSceneData, parameter_ui_model};

use egui::Ui;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

pub use crate::grapher::render::RenderState;

// --------------------------------
// Grapher mode chosen by the user.

#[derive(clap::ValueEnum, Debug, Default, Clone, Copy)]
pub enum GrapherSceneMode {
    #[default]
    Graph,
    Model,
    ImageViewer,
    Solver,
}

impl From<GrapherSceneMode> for usize {
    fn from(value: GrapherSceneMode) -> Self {
        match value {
            GrapherSceneMode::Graph => 0,
            GrapherSceneMode::Model => 1,
            GrapherSceneMode::ImageViewer => 2,
            GrapherSceneMode::Solver => 3,
        }
    }
}

impl From<usize> for GrapherSceneMode {
    fn from(value: usize) -> Self {
        match value {
            0 => GrapherSceneMode::Graph,
            1 => GrapherSceneMode::Model,
            2 => GrapherSceneMode::ImageViewer,
            3 => GrapherSceneMode::Solver,
            _ => unimplemented!(),
        }
    }
}

pub struct Changed(bool);

impl Changed {
    pub fn changed(&self) -> bool {
        self.0
    }
}

pub fn scene_selection_ui(
    selected_scene: &mut GrapherSceneMode,
    ui_state: &mut UiState,
    ui: &mut Ui,
) -> Changed {
    let alternatives = ["graph", "model", "image", "solver"];
    let selected_scene_index = &mut ui_state.selected_scene_index;
    let response = egui::ComboBox::from_id_salt("select scene").show_index(
        ui,
        selected_scene_index,
        alternatives.len(),
        |i| alternatives[i],
    );
    if response.changed() {
        *selected_scene = (*selected_scene_index).into();
        Changed(true)
    } else {
        Changed(false)
    }
}

// ----------------------------------
// Grapher mode and associated state.

pub enum GrapherScene {
    // Means user has chosen new mode that needs loaded.
    Changed,
    // Means that no state has been loaded.
    None,

    Graph(Box<GraphSceneData>),
    Model(ModelSceneData),
    ImageViewer(ImageViewerSceneData),
    Solver(SolverSceneData),
}

impl GrapherScene {
    pub fn is_some(&self) -> bool {
        !matches!(self, GrapherScene::None | GrapherScene::Changed)
    }

    pub fn render(
        &self,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        render_state: &RenderState,
    ) {
        match self {
            GrapherScene::Graph(data) => {
                if data.graph_scene.scene.is_some() {
                    render_state.render(view, encoder, data.graph_scene.scene());
                }
            }
            GrapherScene::Model(data) => {
                render_state.render(view, encoder, data.model_scene.scene());
            }
            GrapherScene::ImageViewer(data) => {
                render_state.render(view, encoder, data.image_viewer_scene.scene());
            }
            GrapherScene::Solver(data) => {
                render_2d(view, encoder, &data.scene, render_state);
            }
            _ => unimplemented!(),
        }
    }

    pub fn compute(&mut self, device: &Device, queue: &Queue) {
        if let GrapherScene::Solver(data) = self {
            data.run_solver(device, queue);
        }
    }

    pub fn update(
        &mut self,
        device: &Device,
        surface_config: &SurfaceConfiguration,
        queue: &Queue,
        state: &RenderState,
    ) {
        match self {
            GrapherScene::Graph(data) => {
                // Rebuild scene if non-uniform parameters changed.
                if data.graph_scene.needs_rebuild {
                    data.graph_scene
                        .try_rebuild_scene(device, surface_config, state);
                    data.graph_scene.needs_rebuild = false;
                }
                data.graph_scene.update(queue, state);
            }
            GrapherScene::Model(data) => {
                data.model_scene.update(queue, state);
            }
            GrapherScene::ImageViewer(data) => {
                data.image_viewer_scene.update(queue, state);
            }
            GrapherScene::Solver(data) => {
                data.update(queue);
            }
            _ => unimplemented!(),
        }
    }

    pub fn parameter_ui(&mut self, ui: &mut Ui, ui_state: &mut UiState) {
        match self {
            GrapherScene::Graph(data) => {
                parameter_ui_graph(data, ui);
            }
            GrapherScene::Model(data) => {
                parameter_ui_model(data, ui, ui_state);
            }
            GrapherScene::ImageViewer(data) => {
                parameter_ui_image_viewer(data, ui, ui_state);
            }
            _ => {}
        }
    }

    pub fn set_needs_rebuild(&mut self, needs_update: bool) {
        match self {
            GrapherScene::Graph(data) => {
                data.graph_scene.needs_rebuild = needs_update;
            }
            GrapherScene::Model(_data) => {
                // no-op
            }
            GrapherScene::ImageViewer(_data) => {
                // no-op
            }
            _ => unimplemented!(),
        }
    }

    pub fn handle_resize(
        &mut self,
        device: &Device,
        queue: &Queue,
        surface_config: &SurfaceConfiguration,
    ) {
        self.rebuild_shadow_state(device, surface_config);
        if let GrapherScene::Solver(data) = self {
            data.handle_resize(queue, surface_config);
        }
    }

    fn rebuild_shadow_state(&mut self, device: &Device, surface_config: &SurfaceConfiguration) {
        if let GrapherScene::Graph(data) = self
            && let Some(scene) = &mut data.graph_scene.scene
            && !scene.meshes.is_empty()
        {
            let last_mesh = scene.meshes.last().unwrap();
            let shadow = ShadowState::create::<GpuVertex>(
                surface_config,
                device,
                &scene.light,
                &last_mesh.bind_group_layout,
            );
            scene.shadow = Some(shadow);
        }
    }
}

// ------------------------------
// Grapher renderer parameter ui.

#[derive(Default)]
pub struct RenderUiState {
    pub lighting_enabled: bool,
    pub use_wireframe: bool,
    pub shadow_enabled: bool,
    pub needs_prefs_uniform_write: bool,
}

impl From<&RenderPreferences> for RenderUiState {
    fn from(render_prefs: &RenderPreferences) -> Self {
        Self {
            lighting_enabled: render_prefs.lighting_enabled(),
            use_wireframe: render_prefs.wireframe_enabled(),
            shadow_enabled: render_prefs.shadow_enabled(),
            needs_prefs_uniform_write: false,
        }
    }
}

pub fn render_parameter_ui(
    render_state: &mut RenderState,
    render_ui_state: &mut RenderUiState,
    grapher_scene: &mut GrapherScene,
    ui: &mut Ui,
) {
    ui.horizontal(|ui| {
        let response = ui.checkbox(&mut render_ui_state.lighting_enabled, "Lighting ");
        if response.changed() {
            render_state
                .render_preferences
                .set_lighting_enabled(render_ui_state.lighting_enabled);
            render_ui_state.needs_prefs_uniform_write = true;
        }

        if matches!(grapher_scene, GrapherScene::Graph(_)) {
            let response = ui.checkbox(&mut render_ui_state.use_wireframe, "Wireframe ");
            if response.changed() {
                render_state
                    .render_preferences
                    .set_wireframe(render_ui_state.use_wireframe);
                // we recreate the pipeline on (rare) change of poly mode
                grapher_scene.set_needs_rebuild(true);
            }
        }
    });
    if matches!(grapher_scene, GrapherScene::Graph(_)) {
        let response = ui.checkbox(&mut render_ui_state.shadow_enabled, "Shadow ");
        if response.changed() {
            render_state
                .render_preferences
                .set_shadow_enabled(render_ui_state.shadow_enabled);
            render_ui_state.needs_prefs_uniform_write = true;
        }
    }
    let response = ui.checkbox(
        &mut render_state.camera_state.camera.relative_rotation,
        "Relative rotation",
    );
    if response.changed() {
        render_state
            .camera_state
            .camera
            .on_relative_rotation_change();
    }
}
