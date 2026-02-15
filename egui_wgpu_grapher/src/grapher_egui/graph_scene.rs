//! UI specific to the grapher mode.

use super::GraphScene;
use crate::egui::components::float_edit_line;

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
    pub smoothing_scale: Option<f64>,
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
            smoothing_scale: None,
        }
    }
}

// graph-specific parameter ui
pub fn parameter_ui_graph(data: &mut GraphSceneData, ui: &mut Ui) {
    let scale_x = &mut data.graph_scene.parameters.scale_x;
    let scale_z = &mut data.graph_scene.parameters.scale_z;
    let scale_y = &mut data.graph_scene.parameters.scale_y;

    let shift_x = &mut data.graph_scene.parameters.shift_x;
    let shift_z = &mut data.graph_scene.parameters.shift_z;
    let shift_y = &mut data.graph_scene.parameters.shift_y;

    let needs_update = &mut data.graph_scene.needs_rebuild;

    // TODO: Temporarily disables function scale and translation UI.
    const CLOSED_FOR_RENOVATION: bool = true;

    if !CLOSED_FOR_RENOVATION {
        Grid::new("graph parameter input").show(ui, |ui| {
            *needs_update = float_edit_line("x scale", &mut data.ui_data.scale_x_text, scale_x, ui)
                || *needs_update;
            ui.end_row();

            // scale parameter edits

            *needs_update = float_edit_line("z scale", &mut data.ui_data.scale_z_text, scale_z, ui)
                || *needs_update;
            ui.end_row();

            *needs_update = float_edit_line("y scale", &mut data.ui_data.scale_y_text, scale_y, ui)
                || *needs_update;
            ui.end_row();

            ui.separator();
            ui.end_row();

            // shift parameter edits

            *needs_update = float_edit_line("x shift", &mut data.ui_data.shift_x_text, shift_x, ui)
                || *needs_update;
            ui.end_row();

            *needs_update = float_edit_line("z shift", &mut data.ui_data.shift_z_text, shift_z, ui)
                || *needs_update;
            ui.end_row();

            *needs_update = float_edit_line("y shift", &mut data.ui_data.shift_y_text, shift_y, ui)
                || *needs_update;
            ui.end_row();
        });
    }

    let mut smoothing = data.smoothing_scale.unwrap_or_default();
    ui.label("Smoothing scale:");
    ui.add_space(2.5);

    if ui
        .add(egui::Slider::new(&mut smoothing, 0.0..=40.0))
        .changed()
    {
        if smoothing == 0.0 {
            data.smoothing_scale = None;
        } else {
            data.smoothing_scale = Some(smoothing);
        }
    }

    // TODO: Need to store function string for reuse;
    //       then we can implement this version.
    //
    // ui.add_space(5.0);
    // if ui.button("Update graph").clicked() {
    //     *needs_update = true;
    // }
}
