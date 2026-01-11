//! UI specific to the glTF viewer mode.

use crate::{egui::ui::UiState, grapher::scene::textured::model::ModelScene};

use egui::Ui;

pub struct ModelSceneUiData;

pub struct ModelSceneData {
    pub model_scene: ModelScene,
    pub _ui_data: ModelSceneUiData,
}

impl ModelSceneData {
    pub fn new(model_scene: ModelScene) -> Self {
        Self {
            model_scene,
            _ui_data: ModelSceneUiData {},
        }
    }
}

// model-specific parameter ui
pub fn parameter_ui_model(
    _data: &mut ModelSceneData,
    _editing: &mut bool,
    ui: &mut Ui,
    ui_state: &mut UiState,
) {
    if ui.add(egui::Button::new("Change file")).clicked() {
        ui_state.show_file_input = true;
    }
}
