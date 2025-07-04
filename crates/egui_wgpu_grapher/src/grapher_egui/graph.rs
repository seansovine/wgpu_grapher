use super::{float_edit_line, GraphScene};

use egui::{Grid, Ui};

pub struct GraphSceneUiData {
    scale_x_text: String,
    scale_z_text: String,
    scale_y_text: String,

    shift_x_text: String,
    shift_z_text: String,
    shift_y_text: String,
}

pub struct GraphSceneData {
    pub graph_scene: GraphScene,
    pub ui_data: GraphSceneUiData,
}

impl GraphSceneData {
    pub fn new(graph_scene: GraphScene) -> Self {
        let scale_x_text = graph_scene.parameters.scale_x.to_string();
        let scale_z_text = graph_scene.parameters.scale_z.to_string();
        let scale_y_text = graph_scene.parameters.scale_y.to_string();

        let shift_x_text = graph_scene.parameters.shift_x.to_string();
        let shift_z_text = graph_scene.parameters.shift_z.to_string();
        let shift_y_text = graph_scene.parameters.shift_y.to_string();

        Self {
            graph_scene,
            ui_data: GraphSceneUiData {
                scale_x_text,
                scale_z_text,
                scale_y_text,

                shift_x_text,
                shift_z_text,
                shift_y_text,
            },
        }
    }
}

// graph-specific parameter ui
pub fn parameter_ui_graph(data: &mut GraphSceneData, editing: &mut bool, ui: &mut Ui) {
    let scale_x = &mut data.graph_scene.parameters.scale_x;
    let scale_z = &mut data.graph_scene.parameters.scale_z;
    let scale_y = &mut data.graph_scene.parameters.scale_y;

    let shift_x = &mut data.graph_scene.parameters.shift_x;
    let shift_z = &mut data.graph_scene.parameters.shift_z;
    let shift_y = &mut data.graph_scene.parameters.shift_y;

    let needs_update = &mut data.graph_scene.needs_update;

    Grid::new("graph parameter input").show(ui, |ui| {
        *needs_update = float_edit_line(
            "Graph x scale",
            &mut data.ui_data.scale_x_text,
            scale_x,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        // scale parameter edits

        *needs_update = float_edit_line(
            "Graph z scale",
            &mut data.ui_data.scale_z_text,
            scale_z,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        *needs_update = float_edit_line(
            "Graph y scale",
            &mut data.ui_data.scale_y_text,
            scale_y,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        ui.separator();
        ui.end_row();

        // shift parameter edits

        *needs_update = float_edit_line(
            "Graph x shift",
            &mut data.ui_data.shift_x_text,
            shift_x,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        *needs_update = float_edit_line(
            "Graph z shift",
            &mut data.ui_data.shift_z_text,
            shift_z,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        *needs_update = float_edit_line(
            "Graph y shift",
            &mut data.ui_data.shift_y_text,
            shift_y,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();
    });
}
