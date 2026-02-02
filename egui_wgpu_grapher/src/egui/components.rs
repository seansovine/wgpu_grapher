//! A few reusable higher-level egui components.

#![allow(dead_code)]

use std::f32;

use egui::{Color32, Context, Ui};

pub struct HasFocus(pub bool);

impl HasFocus {
    pub fn has_focus(&self) -> bool {
        self.0
    }
}

pub fn validated_text_input_window(
    context: &Context,
    title: &str,
    input: &mut String,
    mut validate: impl FnMut(&String),
    is_valid: bool,
) -> HasFocus {
    let mut text_has_focus = false;
    egui::Window::new(title)
        .default_width(300.0)
        .default_pos([250.0, 15.0])
        .resizable([true, false])
        .collapsible(false)
        .show(context, |ui| {
            let response = ui.add(
                egui::TextEdit::singleline(input)
                    .text_color({
                        if !is_valid {
                            Color32::from_gray(104)
                        } else {
                            Color32::from_gray(208)
                        }
                    })
                    .desired_width(f32::INFINITY)
                    .desired_rows(1),
            );

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
    edit_value: &mut f64,
    ui: &mut Ui,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(format!("{label}: "));

        let response = ui.add(egui::TextEdit::singleline(edit_text));

        if response.lost_focus() {
            // parse text and update value if valid
            if let Ok(f_val) = edit_text.parse::<f64>() {
                *edit_value = f_val;
                changed = true;
            } else {
                *edit_text = edit_value.to_string();
            }
        }
    });

    changed
}
