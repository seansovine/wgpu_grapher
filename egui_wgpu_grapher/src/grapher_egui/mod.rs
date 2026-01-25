//! The code in this module manages the different modes of the app,
//! including user interface code specific to each of the three modes
//! and code that maps GUI-modified state to internal handler functions.

pub mod graph_ui;
pub mod image_ui;
pub mod model_ui;

use crate::{
    egui::ui::UiState,
    grapher::{
        pipeline::render_preferences::RenderPreferences,
        render::{RenderState, ShadowState},
        scene::{GpuVertex, RenderScene, solid::graph::GraphScene},
    },
    grapher_egui::image_ui::{ImageViewerSceneData, parameter_ui_image_viewer},
};
use graph_ui::{GraphSceneData, parameter_ui_graph};
use model_ui::{ModelSceneData, parameter_ui_model};

use egui::Ui;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

/// Indicates the mode that the user has chosen,
/// which may or may not have been loaded yet.
#[derive(clap::ValueEnum, Debug, Default, Clone, Copy)]
pub enum GrapherSceneMode {
    #[default]
    Graph,
    Model,
    ImageViewer,
}

impl From<GrapherSceneMode> for usize {
    fn from(value: GrapherSceneMode) -> Self {
        match value {
            GrapherSceneMode::Graph => 0,
            GrapherSceneMode::Model => 1,
            GrapherSceneMode::ImageViewer => 2,
        }
    }
}

impl From<usize> for GrapherSceneMode {
    fn from(value: usize) -> Self {
        match value {
            0 => GrapherSceneMode::Graph,
            1 => GrapherSceneMode::Model,
            2 => GrapherSceneMode::ImageViewer,
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
    let alternatives = ["graph", "model", "image"];
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

/// Manages the state for each of the supported modes:
/// function grapher; model viewer; image viewer.
///
/// `None` indicates that no state has been loaded,
/// and `Changed` indicates that the user has chosen a
/// new mode but data for that mode hasn't been loaded.
pub enum GrapherScene {
    Changed,
    None,
    Graph(Box<GraphSceneData>),
    Model(ModelSceneData),
    ImageViewer(ImageViewerSceneData),
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
            _ => unimplemented!(),
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
                // rebuild scene if non-uniform parameters changed
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
            _ => unimplemented!(),
        }
    }

    pub fn parameter_ui(&mut self, editing: &mut bool, ui: &mut Ui, ui_state: &mut UiState) {
        match self {
            GrapherScene::Graph(data) => {
                parameter_ui_graph(data, editing, ui);
            }
            GrapherScene::Model(data) => {
                parameter_ui_model(data, editing, ui, ui_state);
            }
            GrapherScene::ImageViewer(data) => {
                parameter_ui_image_viewer(data, editing, ui, ui_state);
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

    pub fn rebuild_shadow_state(&mut self, device: &Device, surface_config: &SurfaceConfiguration) {
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

// Code for building the grapher renderer parameter ui.

#[derive(Default)]
pub struct RenderUiState {
    pub lighting_enabled: bool,
    pub use_wireframe: bool,
    pub shadow_enabled: bool,
    pub needs_prefs_uniform_write: bool,
}

impl RenderUiState {
    pub fn from_render_preferences(render_prefs: &RenderPreferences) -> Self {
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
