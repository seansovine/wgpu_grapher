pub mod controller;

use super::matrix::{self, MatrixState, MatrixUniform, X_AXIS, Y_AXIS};
use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};
use std::f32::consts::PI;

#[derive(Default, Clone)]
pub enum ProjectionType {
    Orthographic,
    #[default]
    Perspective,
}

#[derive(Clone)]
pub struct Camera {
    // for look-at matrix
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,

    // orthographic or perspective
    pub projection_type: ProjectionType,

    // for perspective matrix
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    // for orthographic matrix
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub ortho_scale: f32,

    // rotation euler angles
    pub alpha: f32,
    pub gamma: f32,

    // translations
    pub translation_x: f32,
    pub translation_y: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    pub fn get_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let alpha_rot = cgmath::Matrix4::from_axis_angle(Y_AXIS, cgmath::Rad(self.alpha));
        let gamma_rot = cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Rad(self.gamma));
        let translation = cgmath::Matrix4::from_translation(cgmath::Vector3 {
            x: self.translation_x,
            y: self.translation_y,
            z: 0.0,
        });
        let proj = match self.projection_type {
            ProjectionType::Perspective => self.get_perspective_proj(),
            ProjectionType::Orthographic => cgmath::ortho(
                self.left * self.aspect / self.ortho_scale,
                self.right * self.aspect / self.ortho_scale,
                self.bottom / self.ortho_scale,
                self.top / self.ortho_scale,
                2.0, // znear
                self.zfar,
            ),
        };

        OPENGL_TO_WGPU_MATRIX * proj * view * translation * gamma_rot * alpha_rot
    }

    pub fn get_perspective_proj(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }
}

pub struct CameraState {
    pub camera: Camera,
    pub matrix: MatrixState,
    pub controller: controller::CameraController,

    // provides a basic undo for camera changes
    pub previous_camera: Option<Camera>,
}

impl CameraState {
    pub fn init(device: &Device, config: &SurfaceConfiguration) -> CameraState {
        let camera = Camera {
            eye: (0.0, 0.0, 8.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Y_AXIS,

            projection_type: ProjectionType::Perspective,

            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,

            left: -0.5,
            right: 0.5,
            top: 0.5,
            bottom: -0.5,
            ortho_scale: 1.0,

            alpha: PI / 15.0,
            gamma: PI / 4.75,

            translation_x: 0.0,
            translation_y: 0.0,
        };

        let uniform = MatrixUniform::from(camera.get_matrix());
        let matrix = matrix::make_matrix_state(device, uniform);
        let controller = controller::CameraController::new(0.00125);

        CameraState {
            camera,
            matrix,
            controller,
            previous_camera: None,
        }
    }

    /// Set camera at positive z-direction, looking forward.
    pub fn set_from_z(&mut self, distance: f32) {
        self.camera.eye = (0.0, 0.0, distance).into();
        self.camera.alpha = 0.0;
        self.camera.gamma = 0.0;
        self.camera.translation_x = 0.0;
        self.camera.translation_y = 0.0;
    }

    pub fn update_uniform(&mut self, queue: &Queue) {
        queue.write_buffer(
            &self.matrix.buffer,
            0,
            bytemuck::cast_slice(&[self.matrix.uniform]),
        );
    }

    pub fn save_camera(&mut self) {
        self.previous_camera = Some(self.camera.clone());
    }

    // Restores camera state from previous if one was saved.
    pub fn maybe_restore_camera(&mut self) {
        if let Some(camera) = self.previous_camera.take() {
            self.camera = camera;
        }
    }
}
