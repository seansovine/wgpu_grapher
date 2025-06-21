pub mod controller;

use super::matrix::{self, MatrixState, MatrixUniform};
use egui_wgpu::wgpu::{Device, SurfaceConfiguration};
use std::f32::consts::PI;

pub struct Camera {
    // for look-at matrix
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    // for perspective matrix
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    // rotation euler angles
    pub alpha: f32,
    pub gamma: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const Y_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
const X_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(1.0, 0.0, 0.0);

impl Camera {
    pub fn get_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let alpha_rot = cgmath::Matrix4::from_axis_angle(Y_AXIS, cgmath::Rad(self.alpha));
        let gamma_rot = cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Rad(self.gamma));

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view * gamma_rot * alpha_rot
    }
}

pub struct CameraState {
    pub camera: Camera,
    pub matrix: MatrixState,
    pub controller: controller::CameraController,
}

impl CameraState {
    pub fn init(device: &Device, config: &SurfaceConfiguration) -> CameraState {
        let camera = Camera {
            eye: (0.0, 0.0, 8.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Y_AXIS,
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            alpha: PI / 15.0,
            gamma: PI / 4.75,
        };

        let mut uniform = MatrixUniform::identity();
        uniform.update(camera.get_matrix());
        let matrix = matrix::make_matrix_state(device, uniform);

        let controller = controller::CameraController::new(0.00125);

        CameraState {
            camera,
            matrix,
            controller,
        }
    }
}
