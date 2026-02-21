pub mod controller;

use super::matrix::{self, Matrix, MatrixUniform, X_AXIS, Y_AXIS};

use cgmath::{Euler, Matrix3, Quaternion, Rad, SquareMatrix};
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

    // For absolute rotation vs. relative to previous.
    pub relative_rotation: bool,
    pub euler_y: f32,
    pub euler_x: f32,
    pub euler_z: f32,

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

        let user_rotation = if self.relative_rotation {
            self.user_rotation
        } else {
            self.get_absolute_rotation()
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
            relative_rotation: false,
            euler_y: 0.0,
            euler_x: 0.0,
            euler_z: 0.0,
            //
            user_rotation: cgmath::Matrix4::identity(),
        }
    }

    fn store_absolute_rotation(&mut self) {
        self.user_rotation = self.get_absolute_rotation();
    }

    fn set_euler_angles(&mut self) {
        #[rustfmt::skip]
        let rotation_part = Matrix3::new(
            self.user_rotation.x.x, self.user_rotation.x.y, self.user_rotation.x.z, //
            self.user_rotation.y.x, self.user_rotation.y.y, self.user_rotation.y.z, //
            self.user_rotation.z.x, self.user_rotation.z.y, self.user_rotation.z.z, //
        );
        let quaternion = Quaternion::from(rotation_part);
        let euler_angles: Euler<Rad<_>> = Euler::from(quaternion);
        self.euler_x = euler_angles.x.0;
        self.euler_y = euler_angles.y.0;
        self.euler_z = euler_angles.z.0;
    }

    pub fn on_relative_rotation_change(&mut self) {
        if self.relative_rotation {
            self.store_absolute_rotation();
        } else {
            self.set_euler_angles();
        }
    }

    pub fn get_absolute_rotation(&self) -> cgmath::Matrix4<f32> {
        let euler_angles = Euler {
            x: Rad(self.euler_x),
            y: Rad(self.euler_y),
            z: Rad(self.euler_z),
        };
        let quaternion = Quaternion::from(euler_angles);
        quaternion.into()
    }

    pub fn increment_user_rotation(&mut self, alpha: f32, gamma: f32) {
        if self.relative_rotation {
            let alpha_rot = cgmath::Matrix4::from_axis_angle(Y_AXIS, cgmath::Rad(alpha));
            let gamma_rot = cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Rad(gamma));
            self.user_rotation = alpha_rot * gamma_rot * self.user_rotation;
        } else {
            self.euler_y = (self.euler_y + alpha).rem_euclid(2.0 * PI);
            self.euler_x = (self.euler_x + gamma).rem_euclid(2.0 * PI);
        }
    }
}

pub struct CameraState {
    pub camera: Camera,
    pub matrix: MatrixUniform,
    pub controller: controller::CameraController,
}

impl CameraState {
    pub fn init(device: &Device, surface_config: &SurfaceConfiguration) -> CameraState {
        let camera = Camera::default(surface_config);

        let uniform = Matrix::from(camera.get_matrix());
        let matrix = matrix::make_matrix_uniform(device, uniform);
        let controller = controller::CameraController::new(0.00125);

        CameraState {
            camera,
            matrix,
            controller,
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
}
