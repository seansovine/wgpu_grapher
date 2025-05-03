pub mod controller;

use crate::matrix::{self, MatrixState, MatrixUniform};

use wgpu::{Device, SurfaceConfiguration};

pub struct Camera {
  pub eye: cgmath::Point3<f32>,
  pub target: cgmath::Point3<f32>,
  pub up: cgmath::Vector3<f32>,
  pub aspect: f32,
  pub fovy: f32,
  pub znear: f32,
  pub zfar: f32,
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
    let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

    OPENGL_TO_WGPU_MATRIX * proj * view
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
      eye: (0.0, 1.0, 2.0).into(),
      target: (0.0, 0.0, 0.0).into(),
      up: cgmath::Vector3::unit_y(),
      aspect: config.width as f32 / config.height as f32,
      fovy: 45.0,
      znear: 0.1,
      zfar: 100.0,
    };

    let mut uniform = MatrixUniform::identity();
    uniform.update(camera.get_matrix());

    let state = matrix::make_matrix_state(device, uniform);

    let controller = controller::CameraController::new(0.0025);

    CameraState {
      camera,
      matrix: state,
      controller,
    }
  }
}
