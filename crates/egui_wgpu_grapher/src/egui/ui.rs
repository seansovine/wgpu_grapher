use egui::{RichText, Ui};

use crate::{
    grapher::render::RenderState,
    grapher_egui::{
        render_parameter_ui, scene_selection_ui, GrapherScene, GrapherSceneMode, RenderUiState,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn create_gui(
    pixels_per_point: f32,
    ui: &mut Ui,
    editing: &mut bool,
    mut grapher_scene: Option<&mut GrapherScene>,
    render_state: &mut RenderState,
    ui_state: &mut UiState,
    selected_scene: &mut GrapherSceneMode,
) {
    const AFTER_LABEL_SPACE: f32 = 5.0;

    // grapher scenee selection

    ui.label(RichText::new("Select scene").strong());
    ui.add_space(AFTER_LABEL_SPACE);

    scene_selection_ui(selected_scene, ui_state, ui);

    // parameters for the grapher scene

    if let Some(grapher_scene) = grapher_scene.as_mut() {
        ui.separator();
        ui.label(RichText::new("Scene parameters").strong());
        ui.add_space(AFTER_LABEL_SPACE);

        grapher_scene.parameter_ui(editing, ui, ui_state);
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

// Place to put persistent ui state that doesn't fit elsewhere.

#[derive(Default)]
pub enum FileInputState {
    #[default]
    Hidden,
    NeedsInput,
    BadPath,
    InvalidFile,
    NeedsChecked,
}

#[derive(Default)]
pub struct UiState {
    pub render_ui_state: RenderUiState,
    pub selected_scene_index: usize,
    pub scale_factor: f32,
    pub file_window_state: FileInputState,
    pub filename: String,
    pub function_string: String,
    pub function_valid: bool,
}
