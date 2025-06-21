// The code in this module is a bridge between egui and function graphing code.

use crate::grapher::{
    mesh::{GraphScene, RenderScene},
    render::RenderState,
};

use egui::Ui;
use egui_wgpu::wgpu::{CommandEncoder, Queue, TextureView};

// The following enum replaces dynamic dispatch and allows the
// GUI to display different data and perform different actions
// depending on the particular grapher scene that is selected.

pub enum GrapherScene {
    Graph(GraphScene),
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
            GrapherScene::Graph(graph_scene) => {
                // pass scene in to state render function
                render_state.render(view, encoder, &graph_scene.scene());
            }
            _ => unimplemented!(),
        }
    }

    pub fn update(&mut self, queue: &Queue, state: &RenderState, pre_render: bool) {
        match self {
            GrapherScene::Graph(graph_scene) => {
                graph_scene.update(queue, state, pre_render);
            }
            _ => unimplemented!(),
        }
    }
}

impl GrapherScene {
    pub fn parameter_ui(&mut self, ui: &mut Ui) {
        match self {
            GrapherScene::Graph(graph_scene) => {
                // graph-specific parameter ui
                ui.label("Placeholder for graph scene parameter widgets.");
            }
            _ => unimplemented!(),
        }
    }
}
