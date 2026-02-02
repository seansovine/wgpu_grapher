use egui::{RichText, Ui};

use crate::grapher_egui::{
    GrapherScene, GrapherSceneMode, RenderState, RenderUiState, render_parameter_ui,
    scene_selection_ui,
};

/// Store persistent data modified by GUI interaction during render passes.
#[derive(Default)]
pub struct UiState {
    pub render_ui_state: RenderUiState,
    pub selected_scene_index: usize,
    pub scale_factor: f32,
    pub filename: String,
    pub function_string: String,
    pub function_valid: bool,
    pub show_file_input: bool,
}

// Create contents of main GUI window.
#[allow(clippy::too_many_arguments)]
pub fn create_gui(
    pixels_per_point: f32,
    ui: &mut Ui,
    grapher_scene: &mut GrapherScene,
    render_state: &mut RenderState,
    ui_state: &mut UiState,
    scene_mode: &mut GrapherSceneMode,
) {
    const AFTER_LABEL_SPACE: f32 = 5.0;

    // grapher scenee selection
    ui.label(RichText::new("Select scene").strong());
    ui.add_space(AFTER_LABEL_SPACE);

    if scene_selection_ui(scene_mode, ui_state, ui).changed() {
        *grapher_scene = GrapherScene::Changed;
    }

    // parameters for the grapher scene
    if grapher_scene.is_some() {
        ui.separator();
        ui.label(RichText::new("Scene parameters").strong());
        ui.add_space(AFTER_LABEL_SPACE);

        // TODO: editing param may be no longer needed here.
        grapher_scene.parameter_ui(ui, ui_state);
    }

    // general rendere parameters
    ui.separator();
    ui.label(RichText::new("Render parameters").strong());
    ui.add_space(AFTER_LABEL_SPACE);

    render_parameter_ui(
        render_state,
        &mut ui_state.render_ui_state,
        grapher_scene,
        ui,
    );

    // ui scale parameter
    ui.separator();
    ui.label(RichText::new("UI settings").strong());
    ui.add_space(AFTER_LABEL_SPACE);

    let scale_factor = &mut ui_state.scale_factor;

    ui.horizontal(|ui| {
        ui.label(format!("Pixels per point: {pixels_per_point}"));
        if ui.button("-").clicked() {
            *scale_factor = (*scale_factor - 0.1).max(0.3);
        }
        if ui.button("+").clicked() {
            *scale_factor = (*scale_factor + 0.1).min(3.0);
        }
    });
}
