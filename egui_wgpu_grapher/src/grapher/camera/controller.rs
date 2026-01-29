//! Camera state controller, originally based on the Learn Wgpu example.

use crate::grapher::camera::{self, ProjectionType};

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use std::f32::consts::PI;

#[derive(Default)]
pub struct CameraController {
    pub speed: f32,

    // rotation keys
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,

    // zoom keys
    pub z_pressed: bool,
    pub x_pressed: bool,

    // translation keys
    pub t_pressed: bool,
    pub f_pressed: bool,
    pub g_pressed: bool,
    pub h_pressed: bool,

    // modifiers
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,

    // mouse sate
    pub left_down: bool,
    pub last_pos: PhysicalPosition<f64>,
    pub last_drag: Option<[f64; 2]>,
    pub last_mouse_scroll: Option<f32>,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            left_down: false,
            ..Default::default()
        }
    }

    pub fn update_camera(&mut self, camera: &mut camera::Camera) {
        let zoom_incr: f32 = if self.shift_pressed { 6.0 } else { 1.2 };
        let zoom_incr = zoom_incr * self.speed;
        const MOUSE_SCROLL_RATE: f32 = 5.0;
        match camera.projection_type {
            ProjectionType::Perspective => {
                use cgmath::InnerSpace;
                let forward = camera.target - camera.eye;
                let forward_norm = forward.normalize();
                let forward_mag = forward.magnitude();
                if self.z_pressed && forward_mag > self.speed {
                    camera.eye += forward_norm * zoom_incr;
                }
                if self.x_pressed {
                    camera.eye -= forward_norm * zoom_incr;
                }
                if let Some(scroll) = self.last_mouse_scroll.take() {
                    camera.eye += scroll * MOUSE_SCROLL_RATE * forward_norm * zoom_incr;
                }
            }
            ProjectionType::Orthographic => {
                const INCR_ADJUSTMENT: f32 = 50.0;
                if self.z_pressed {
                    camera.ortho_scale *= 1.0 + zoom_incr / INCR_ADJUSTMENT;
                }
                if self.x_pressed {
                    camera.ortho_scale *= 1.0 - zoom_incr / INCR_ADJUSTMENT;
                }
                if let Some(scroll) = self.last_mouse_scroll.take() {
                    camera.ortho_scale *=
                        1.0 + scroll * MOUSE_SCROLL_RATE * zoom_incr / INCR_ADJUSTMENT;
                }
            }
        }

        if let Some(incr) = self.last_drag.take() {
            const MOUSE_ROTATION_RATE: f32 = 0.05;
            const MOUSE_TRANSLATION_RATE: f32 = 0.25;
            if !self.ctrl_pressed {
                camera.increment_user_rotation(
                    incr[0] as f32 * MOUSE_ROTATION_RATE,
                    incr[1] as f32 * MOUSE_ROTATION_RATE,
                );
            } else {
                camera.translation_x +=
                    incr[0] as f32 * MOUSE_TRANSLATION_RATE / camera.ortho_scale;
                camera.translation_y -=
                    incr[1] as f32 * MOUSE_TRANSLATION_RATE / camera.ortho_scale;
            }
        }

        if matches!(camera.projection_type, ProjectionType::Perspective) {
            let angle_incr = self.speed * PI / 4.0;

            if self.right_pressed {
                camera.increment_user_rotation(angle_incr, 0.0);
            }
            if self.left_pressed {
                camera.increment_user_rotation(-angle_incr, 0.0);
            }
            if self.up_pressed {
                camera.increment_user_rotation(0.0, angle_incr);
            }
            if self.down_pressed {
                camera.increment_user_rotation(0.0, -angle_incr);
            }
        }

        let trans_incr = if self.shift_pressed {
            self.speed * 6.0
        } else {
            self.speed * 0.5
        };

        if self.t_pressed {
            camera.translation_y += trans_incr / camera.ortho_scale;
        }
        if self.g_pressed {
            camera.translation_y -= trans_incr / camera.ortho_scale;
        }
        if self.f_pressed {
            camera.translation_x -= trans_incr / camera.ortho_scale;
        }
        if self.h_pressed {
            camera.translation_x += trans_incr / camera.ortho_scale;
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;

                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.up_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.down_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.right_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyZ => {
                        self.z_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyX => {
                        self.x_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyT => {
                        self.t_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyF => {
                        self.f_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyG => {
                        self.g_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyH => {
                        self.h_pressed = is_pressed;
                        true
                    }

                    KeyCode::ShiftRight | KeyCode::ShiftLeft => {
                        self.shift_pressed = is_pressed;
                        true
                    }
                    KeyCode::ControlLeft | KeyCode::ControlRight => {
                        self.ctrl_pressed = is_pressed;
                        true
                    }

                    _ => false,
                }
            }

            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                if *button == MouseButton::Left {
                    self.left_down = state.is_pressed();
                }
                true
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                if self.left_down {
                    let drag_x = position.x - self.last_pos.x;
                    let drag_y = position.y - self.last_pos.y;
                    if let Some(drag) = self.last_drag.as_mut() {
                        drag[0] += drag_x;
                        drag[1] += drag_y;
                    } else {
                        self.last_drag = Some([drag_x, drag_y]);
                    }
                }
                self.last_pos = *position;
                true
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                self.last_mouse_scroll = Some(*y);
                true
            }

            _ => false,
        }
    }
}
