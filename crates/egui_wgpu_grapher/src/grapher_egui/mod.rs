// The code in this module is a bridge between egui and function graphing code.

use crate::grapher::{
    mesh::{GraphScene, RenderScene},
    render::RenderState,
};

use egui::{Grid, Ui};
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

// The following enum replaces dynamic dispatch and allows the
// GUI to display different data and perform different actions
// depending on the particular grapher scene that is selected.

pub enum GrapherScene {
    Graph(GraphSceneData),
    // TODO: add other available scene types
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
            _ => unimplemented!(),
        }
    }
}

impl GrapherScene {
    pub fn parameter_ui(&mut self, editing: &mut bool, ui: &mut Ui) {
        match self {
            GrapherScene::Graph(graph_scene) => {
                parameter_ui_graph(graph_scene, editing, ui);
            }
            _ => unimplemented!(),
        }
    }
}

pub struct GraphSceneUiData {
    scale_x_text: String,
    scale_z_text: String,
}

pub struct GraphSceneData {
    pub graph_scene: GraphScene,
    pub ui_data: GraphSceneUiData,
}

impl GraphSceneData {
    pub fn new(graph_scene: GraphScene) -> Self {
        let scale_x_text = graph_scene.parameters.scale_x.to_string();
        let scale_z_text = graph_scene.parameters.scale_z.to_string();

        Self {
            graph_scene,
            ui_data: GraphSceneUiData {
                scale_x_text,
                scale_z_text,
            },
        }
    }
}

// graph-specific parameter ui
fn parameter_ui_graph(data: &mut GraphSceneData, editing: &mut bool, ui: &mut Ui) {
    let scale_x = &mut data.graph_scene.parameters.scale_x;
    let scale_z = &mut data.graph_scene.parameters.scale_z;

    let needs_update = &mut data.graph_scene.needs_update;

    Grid::new("graph scale input").show(ui, |ui| {
        *needs_update = float_edit_line(
            "Graph x scale",
            &mut data.ui_data.scale_x_text,
            scale_x,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        *needs_update = float_edit_line(
            "Graph z scale",
            &mut data.ui_data.scale_z_text,
            scale_z,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();
    });
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
