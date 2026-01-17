// The code in this module is a bridge between egui and function graphing code.

#[allow(dead_code)]
pub mod graph;
#[allow(dead_code)]
pub mod image_viewer;
#[allow(dead_code)]
pub mod model;

use crate::{
    egui::ui::UiState,
    grapher::{
        pipeline::render_preferences::RenderPreferences,
        render::RenderState,
        scene::{RenderScene, solid::graph::GraphScene},
    },
    grapher_egui::image_viewer::{ImageViewerSceneData, parameter_ui_image_viewer},
};
use graph::{GraphSceneData, parameter_ui_graph};
use model::{ModelSceneData, parameter_ui_model};

use egui::Ui;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

// For use with user input.

#[allow(dead_code)]
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

// The following enum replaces dynamic dispatch and allows the
// GUI to display different data and perform different actions
// depending on the particular grapher scene that is selected.

pub enum GrapherScene {
    Changed,
    None,
    Graph(Box<GraphSceneData>),
    Model(ModelSceneData),
    ImageViewer(ImageViewerSceneData),
}

impl GrapherScene {
    pub fn is_some(&self) -> bool {
        !matches!(self, GrapherScene::None)
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
                    // pass scene in to state render function
                    render_state.render(view, encoder, data.graph_scene.scene());
                }
            }
            GrapherScene::Model(data) => {
                // pass scene in to state render function
                render_state.render(view, encoder, data.model_scene.scene());
            }
            GrapherScene::ImageViewer(data) => {
                // pass scene in to state render function
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
        pre_render: bool,
    ) {
        match self {
            GrapherScene::Graph(data) => {
                // rebuild scene if parameters changed
                if data.graph_scene.needs_update {
                    data.graph_scene
                        .try_rebuild_scene(device, surface_config, state);
                    data.graph_scene.needs_update = false;
                }
                // currently a no-op; would perform state update
                data.graph_scene.update(queue, state, pre_render);
            }
            GrapherScene::Model(data) => {
                // currently a no-op; would perform state update
                data.model_scene.update(queue, state, pre_render);
            }
            GrapherScene::ImageViewer(data) => {
                // currently a no-op; would perform state update
                data.image_viewer_scene.update(queue, state, pre_render);
            }
            _ => unimplemented!(),
        }
    }

    #[allow(unused)]
    pub fn try_save_light(&mut self) {
        match self {
            GrapherScene::Graph(data) => {
                if let Some(scene) = data.graph_scene.scene.as_mut() {
                    scene.light.save_light();
                }
            }
            GrapherScene::Model(data) => {
                data.model_scene.scene.light.save_light();
            }
            GrapherScene::ImageViewer(data) => {
                data.image_viewer_scene.scene.light.save_light();
            }
            _ => {} // no-op
        }
    }

    #[allow(unused)]
    pub fn try_restore_light(&mut self, queue: &Queue) {
        match self {
            GrapherScene::Graph(data) => {
                if let Some(scene) = data.graph_scene.scene.as_mut() {
                    scene.light.maybe_restore_light(queue);
                }
            }
            GrapherScene::Model(data) => {
                data.model_scene.scene.light.maybe_restore_light(queue);
            }
            GrapherScene::ImageViewer(data) => {
                data.image_viewer_scene
                    .scene
                    .light
                    .maybe_restore_light(queue);
            }
            _ => {} // no-op
        }
    }
}

impl GrapherScene {
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

    pub fn set_needs_update(&mut self, needs_update: bool) {
        match self {
            GrapherScene::Graph(data) => {
                data.graph_scene.needs_update = needs_update;
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
}

// Code for building the grapher renderer parameter ui.

#[derive(Default)]
pub struct RenderUiState {
    // state for ui rendering
    pub lighting_enabled: bool,
    pub use_wireframe: bool,
    pub shadow_enabled: bool,

    // was there and update that needs processed
    pub needs_prefs_update: bool,
}

impl RenderUiState {
    pub fn from_render_preferences(render_prefs: &RenderPreferences) -> Self {
        Self {
            lighting_enabled: render_prefs.lighting_enabled(),
            use_wireframe: render_prefs.wireframe_enabled(),
            shadow_enabled: render_prefs.shadow_enabled(),
            needs_prefs_update: false,
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
            // only requires updating a uniform with write_buffer
            render_ui_state.needs_prefs_update = true;
        }

        if matches!(grapher_scene, GrapherScene::Graph(_)) {
            let response = ui.checkbox(&mut render_ui_state.use_wireframe, "Wireframe ");
            if response.changed() {
                render_state
                    .render_preferences
                    .set_wireframe(render_ui_state.use_wireframe);
                // requires changing polygon mode, and so recreating pipeline
                grapher_scene.set_needs_update(true);
            }
        }
    });
    if matches!(grapher_scene, GrapherScene::Graph(_)) {
        let response = ui.checkbox(&mut render_ui_state.shadow_enabled, "Shadow ");
        if response.changed() {
            render_state
                .render_preferences
                .set_shadow_enabled(render_ui_state.shadow_enabled);
            // only requires updating a uniform with write_buffer
            render_ui_state.needs_prefs_update = true;
        }
    }
    let _ = ui.checkbox(
        &mut render_state.camera_state.camera.absolute_rotation,
        "Relative rotation",
    );
}
