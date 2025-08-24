use crate::{
    egui::ui::{FileInputState, UiState},
    grapher::scene::textured::image_viewer::ImageViewerScene,
};

use egui::Ui;

pub struct ImageViewerSceneUiData;

pub struct ImageViewerSceneData {
    pub image_viewer_scene: ImageViewerScene,
    pub _ui_data: ImageViewerSceneUiData,
}

impl ImageViewerSceneData {
    pub fn new(image_viewer_scene: ImageViewerScene) -> Self {
        Self {
            image_viewer_scene,
            _ui_data: ImageViewerSceneUiData {},
        }
    }
}

// model-specific parameter ui
pub fn parameter_ui_image_viewer(
    _data: &mut ImageViewerSceneData,
    _editing: &mut bool,
    ui: &mut Ui,
    ui_state: &mut UiState,
) {
    if ui.add(egui::Button::new("Change file")).clicked() {
        ui_state.file_window_state = FileInputState::NeedsInput;
    }
}
