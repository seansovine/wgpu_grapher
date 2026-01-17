//! Camera state controller. Originally based on Learn Wgpu tutorial example.

use crate::grapher::camera::{self, ProjectionType};

use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use std::f32::consts::PI;

// TODO: Rework this as an event queue.
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

    // modifiers (see TODO)
    pub shift_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            //
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            //
            z_pressed: false,
            x_pressed: false,
            //
            f_pressed: false,
            g_pressed: false,
            h_pressed: false,
            t_pressed: false,
            //
            shift_pressed: false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut camera::Camera) {
        let zoom_incr: f32 = if self.shift_pressed { 120.0 } else { 1.2 };
        let zoom_incr = zoom_incr * self.speed;

        match camera.projection_type {
            ProjectionType::Perspective => {
                use cgmath::InnerSpace;
                let forward = camera.target - camera.eye;
                let forward_norm = forward.normalize();
                let forward_mag = forward.magnitude();

                // use of look-at from Learn WGPU
                if self.z_pressed && forward_mag > self.speed {
                    camera.eye += forward_norm * zoom_incr;
                }
                if self.x_pressed {
                    camera.eye -= forward_norm * zoom_incr;
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
            self.speed * 25.0
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

                    // TODO: This seems to work, but is it the
                    //       best way to handle modifiers?
                    KeyCode::ShiftRight => {
                        self.shift_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
