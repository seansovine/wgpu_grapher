//! Reusable egui components.

use egui::{Color32, Context};

pub struct HasFocus(pub bool);

pub fn validated_text_input_window(
    context: &Context,
    title: &str,
    input: &mut String,
    mut validate: impl FnMut(&String),
    is_valid: bool,
) -> HasFocus {
    let mut text_has_focus = false;
    egui::Window::new(title)
        .resizable(true)
        .default_size([800.0, 600.0])
        .collapsible(false)
        .show(context, |ui| {
            let response = ui.add(egui::TextEdit::singleline(input).text_color({
                if !is_valid {
                    Color32::from_rgb(176, 44, 44)
                } else {
                    Color32::from_gray(208)
                }
            }));

            if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                validate(input);
            }
            text_has_focus = response.has_focus();
        });

    HasFocus(text_has_focus)
}
