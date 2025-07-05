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
        mesh::{solid::graph::GraphScene, RenderScene},
        pipeline::render_preferences::RenderPreferences,
        render::RenderState,
    },
    grapher_egui::image_viewer::{parameter_ui_image_viewer, ImageViewerSceneData},
};
use graph::{parameter_ui_graph, GraphSceneData};
use model::{parameter_ui_model, ModelSceneData};

use egui::Ui;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

// For use with user input.

#[allow(dead_code)]
pub enum GrapherSceneMode {
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

pub fn scene_selection_ui(
    selected_scene: &mut GrapherSceneMode,
    ui_state: &mut UiState,
    ui: &mut Ui,
) {
    let alternatives = ["graph", "model", "image"];
    let selected_scene_index = &mut ui_state.selected_scene_index;

    egui::ComboBox::from_id_salt("select scene").show_index(
        ui,
        selected_scene_index,
        alternatives.len(),
        |i| alternatives[i],
    );

    *selected_scene = (*selected_scene_index).into();
}

// The following enum replaces dynamic dispatch and allows the
// GUI to display different data and perform different actions
// depending on the particular grapher scene that is selected.

#[allow(dead_code)]
pub enum GrapherScene {
    Graph(GraphSceneData),
    Model(ModelSceneData),
    ImageViewer(ImageViewerSceneData),
}

impl GrapherScene {
    pub fn render(
        &self,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        render_state: &RenderState,
    ) {
        match self {
            GrapherScene::Graph(data) => {
                // pass scene in to state render function
                render_state.render(view, encoder, data.graph_scene.scene());
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
                        .rebuild_scene(device, surface_config, state);
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
}

impl GrapherScene {
    pub fn parameter_ui(&mut self, editing: &mut bool, ui: &mut Ui) {
        match self {
            GrapherScene::Graph(data) => {
                parameter_ui_graph(data, editing, ui);
            }
            GrapherScene::Model(data) => {
                parameter_ui_model(data, editing, ui);
            }
            GrapherScene::ImageViewer(data) => {
                parameter_ui_image_viewer(data, editing, ui);
            }
            _ => unimplemented!(),
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

fn float_edit_line(
    label: &str,
    edit_text: &mut String,
    edit_value: &mut f32,
    editing: &mut bool,
    ui: &mut Ui,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(format!("{label}: "));

        let response = ui.add(egui::TextEdit::singleline(edit_text));

        if response.gained_focus() {
            *editing = true;
        }

        if response.lost_focus() {
            // parse text and update value if valid
            if let Ok(f_val) = edit_text.parse::<f32>() {
                *edit_value = f_val;
                changed = true;
            } else {
                *edit_text = edit_value.to_string();
            }
            *editing = false;
        }
    });

    changed
}

// Code for building the grapher renderer parameter ui.

pub struct RenderUiState {
    // state for ui rendering
    pub lighting_enabled: bool,
    pub use_wireframe: bool,

    // was there and update that needs processed
    pub needs_update: bool,
}

impl RenderUiState {
    pub fn from_render_preferences(render_prefs: &RenderPreferences) -> Self {
        Self {
            lighting_enabled: render_prefs.lighting_enabled(),
            use_wireframe: render_prefs.wireframe_enabled(),
            needs_update: false,
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
            render_ui_state.needs_update = true;
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
}
