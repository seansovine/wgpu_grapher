use crate::grapher::camera;

use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use std::f32::consts::PI;

pub struct CameraController {
    pub speed: f32,

    // rotation keys
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,

    // zoom keys
    pub is_z_pressed: bool,
    pub is_x_pressed: bool,

    // translation keys
    pub is_t_pressed: bool,
    pub is_f_pressed: bool,
    pub is_g_pressed: bool,
    pub is_h_pressed: bool,

    // modifier
    pub is_shift_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,

            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,

            is_z_pressed: false,
            is_x_pressed: false,

            is_f_pressed: false,
            is_g_pressed: false,
            is_h_pressed: false,
            is_t_pressed: false,

            is_shift_pressed: false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut camera::Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        let multipler: f32 = if self.is_shift_pressed { 120.0 } else { 1.2 };

        // use of look-at from Learn WGPU
        if self.is_z_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed * multipler;
        }
        if self.is_x_pressed {
            camera.eye -= forward_norm * self.speed * multipler;
        }

        let angle_incr = self.speed * PI / 4.0;

        if self.is_right_pressed {
            camera.alpha += angle_incr;
        }
        if self.is_left_pressed {
            camera.alpha -= angle_incr;
        }
        if self.is_up_pressed {
            camera.gamma += angle_incr;
        }
        if self.is_down_pressed {
            camera.gamma -= angle_incr;
        }

        let trans_incr = if self.is_shift_pressed {
            self.speed * 25.0
        } else {
            self.speed * 2.5
        };

        if self.is_t_pressed {
            camera.translation_y += trans_incr;
        }
        if self.is_g_pressed {
            camera.translation_y -= trans_incr;
        }
        if self.is_f_pressed {
            camera.translation_x -= trans_incr;
        }
        if self.is_h_pressed {
            camera.translation_x += trans_incr;
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
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyZ => {
                        self.is_z_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyX => {
                        self.is_x_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyT => {
                        self.is_t_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyF => {
                        self.is_f_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyG => {
                        self.is_g_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyH => {
                        self.is_h_pressed = is_pressed;
                        true
                    }
                    KeyCode::ShiftRight => {
                        self.is_shift_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
