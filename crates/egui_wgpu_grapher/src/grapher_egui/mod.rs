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
                render_state.render(view, encoder, &data.graph_scene.scene());
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
                data.graph_scene.update(queue, state, pre_render);
            }
            _ => unimplemented!(),
        }
    }
}

impl GrapherScene {
    pub fn parameter_ui(&mut self, ui: &mut Ui) {
        match self {
            GrapherScene::Graph(graph_scene) => {
                parameter_ui_graph(graph_scene, ui);
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Default)]
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
fn parameter_ui_graph(data: &mut GraphSceneData, ui: &mut Ui) {
    let scale_x = &mut data.graph_scene.parameters.scale_x;
    let scale_z = &mut data.graph_scene.parameters.scale_z;

    let needs_update = &mut data.graph_scene.needs_update;

    Grid::new("graph scale input").show(ui, |ui| {
        ui.horizontal(|ui| {
            let mut scale_x_text = &mut data.ui_data.scale_x_text;

            ui.label(format!("Graph x scale: "));
            let x_response = ui.add(egui::TextEdit::singleline(scale_x_text));

            if x_response.lost_focus() {
                // parse and update if valid
                if let Ok(f_val) = scale_x_text.parse::<f32>() {
                    *scale_x = f_val;
                    println!("Updated x scale to {scale_x}.");
                    *needs_update = true;
                } else {
                    *scale_x_text = scale_x.to_string();
                }
            }
        });
        ui.end_row();

        ui.horizontal(|ui| {
            let mut scale_z_text = &mut data.ui_data.scale_z_text;

            ui.label(format!("Graph z scale: "));
            let z_response = ui.add(egui::TextEdit::singleline(scale_z_text));

            if z_response.lost_focus() {
                // parse and update if valid
                if let Ok(f_val) = scale_z_text.parse::<f32>() {
                    *scale_z = f_val;
                    println!("Updated x scale to {scale_x}.");
                    *needs_update = true;
                } else {
                    *scale_z_text = scale_z.to_string();
                }
            }
        });
        ui.end_row();
    });
}
