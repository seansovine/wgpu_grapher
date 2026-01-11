//! Reusable egui components.

use std::path::Path;

use egui::{Color32, Context, Ui};

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
        .default_pos([250.0, 15.0])
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

pub fn float_edit_line(
    label: &str,
    edit_text: &mut String,
    edit_value: &mut f32,
    editing: &mut bool,
    ui: &mut Ui,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(format!("{label}: "));

        let response = ui.add(egui::TextEdit::singleline(edit_text));

        if response.gained_focus() {
            *editing = true;
        }

        if response.lost_focus() {
            // parse text and update value if valid
            if let Ok(f_val) = edit_text.parse::<f32>() {
                *edit_value = f_val;
                changed = true;
            } else {
                *edit_text = edit_value.to_string();
            }
            *editing = false;
        }
    });

    changed
}

pub fn validate_path(path: &str) -> bool {
    Path::new(path).exists()
}
