use egui::Ui;

use crate::grapher_egui::GrapherScene;

pub fn render_window(
    scale_factor: &mut f32,
    pixels_per_point: f32,
    ui: &mut Ui,
    grapher_scene: &mut GrapherScene,
) {
    ui.label("Label!");

    if ui.button("Button!").clicked() {
        println!("boom!")
    }

    // parameters for the grapher scene

    ui.separator();
    grapher_scene.parameter_ui(ui);

    // ui scale parameter

    ui.separator();
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
