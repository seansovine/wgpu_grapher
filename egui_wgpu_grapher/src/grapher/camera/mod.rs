pub mod controller;

use super::matrix::{self, MatrixState, MatrixUniform, X_AXIS, Y_AXIS};

use cgmath::SquareMatrix;
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

    // translations
    pub translation_x: f32,
    pub translation_y: f32,

    pub alpha: f32,
    pub gamma: f32,

    // Current user rotation for relative rotation.
    pub user_rotation: cgmath::Matrix4<f32>,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    pub const ABSOLUTE_ROTAITON: bool = true;

    pub fn get_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
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

        let user_rotation = if Self::ABSOLUTE_ROTAITON {
            let alpha_rot = cgmath::Matrix4::from_axis_angle(Y_AXIS, cgmath::Rad(self.alpha));
            let gamma_rot = cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Rad(self.gamma));
            gamma_rot * alpha_rot
        } else {
            self.user_rotation
        };

        OPENGL_TO_WGPU_MATRIX * proj * view * translation * user_rotation
    }

    pub fn get_perspective_proj(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }

    pub fn default(surface_config: &SurfaceConfiguration) -> Self {
        Self {
            eye: (0.0, 0.0, 8.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Y_AXIS,
            //
            projection_type: ProjectionType::Perspective,
            //
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            //
            left: -0.5,
            right: 0.5,
            top: 0.5,
            bottom: -0.5,
            ortho_scale: 1.0,
            //
            translation_x: 0.0,
            translation_y: 0.0,
            //
            alpha: 0.0,
            gamma: 0.0,
            //
            user_rotation: cgmath::Matrix4::identity(),
        }
    }

    pub fn increment_user_rotation(&mut self, alpha: f32, gamma: f32) {
        if Self::ABSOLUTE_ROTAITON {
            self.alpha = (self.alpha + alpha).rem_euclid(2.0 * PI);
            self.gamma = (self.gamma + gamma).rem_euclid(2.0 * PI);
        } else {
            let alpha_rot = cgmath::Matrix4::from_axis_angle(Y_AXIS, cgmath::Rad(alpha));
            let gamma_rot = cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Rad(gamma));
            self.user_rotation = gamma_rot * alpha_rot * self.user_rotation;
        }
    }
}

pub struct CameraState {
    pub camera: Camera,
    pub matrix: MatrixState,
    pub controller: controller::CameraController,

    // provides a basic undo for camera changes
    #[allow(unused)]
    pub previous_camera: Option<Camera>,
}

impl CameraState {
    pub fn init(device: &Device, surface_config: &SurfaceConfiguration) -> CameraState {
        let camera = Camera::default(surface_config);

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

    pub fn reset_camera(&mut self, queue: &Queue, surface_config: &SurfaceConfiguration) {
        self.camera = Camera::default(surface_config);
        self.update_uniform(queue);
    }

    /// Set camera at positive z-direction, looking forward.
    pub fn set_from_z(&mut self, distance: f32) {
        self.camera.eye = (0.0, 0.0, distance).into();
        self.camera.user_rotation = cgmath::Matrix4::identity();
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

    #[allow(unused)]
    pub fn save_camera(&mut self) {
        self.previous_camera = Some(self.camera.clone());
    }

    #[allow(unused)]
    // Restores camera state from previous if one was saved.
    pub fn maybe_restore_camera(&mut self) {
        if let Some(camera) = self.previous_camera.take() {
            self.camera = camera;
        }
    }
}
