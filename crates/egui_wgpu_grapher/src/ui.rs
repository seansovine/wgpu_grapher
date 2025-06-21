use egui::{RichText, Ui};

use crate::grapher_egui::GrapherScene;

pub fn render_window(
    scale_factor: &mut f32,
    pixels_per_point: f32,
    ui: &mut Ui,
    editing: &mut bool,
    grapher_scene: &mut GrapherScene,
) {
    // parameters for the grapher scene

    const AFTER_LABEL_SPACE: f32 = 5.0;

    ui.separator();

    ui.label(RichText::new("Grapher parameters").strong());
    ui.add_space(AFTER_LABEL_SPACE);

    grapher_scene.parameter_ui(editing, ui);

    // ui scale parameter

    ui.separator();

    ui.label(RichText::new("UI settings").strong());
    ui.add_space(AFTER_LABEL_SPACE);

    ui.horizontal(|ui| {
        ui.label(format!("Pixels per point: {}", pixels_per_point));
        if ui.button("-").clicked() {
            *scale_factor = (*scale_factor - 0.1).max(0.3);
        }
        if ui.button("+").clicked() {
            *scale_factor = (*scale_factor + 0.1).min(3.0);
        }
    });
}
