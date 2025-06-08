use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};

const X_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(1.0, 0.0, 0.0);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MatrixUniform {
  view_proj: [[f32; 4]; 4],
}

impl MatrixUniform {
  pub fn identity() -> Self {
    use cgmath::SquareMatrix;
    Self {
      view_proj: cgmath::Matrix4::identity().into(),
    }
  }

  pub fn translation(coords: &[f32]) -> Self {
    Self {
      view_proj: cgmath::Matrix4::from_translation(cgmath::Vector3 {
        x: coords[0],
        y: coords[1],
        z: coords[2],
      })
      .into(),
    }
  }

  pub fn x_rotation(degrees: f32) -> Self {
    Self {
      view_proj: cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Deg(degrees)).into(),
    }
  }

  pub fn update(&mut self, matrix: cgmath::Matrix4<f32>) {
    self.view_proj = matrix.into();
  }
}

pub struct MatrixState {
  pub uniform: MatrixUniform,
  pub buffer: Buffer,
  pub bind_group_layout: BindGroupLayout,
  pub bind_group: BindGroup,
}

pub(crate) fn make_matrix_state(device: &Device, matrix_uniform: MatrixUniform) -> MatrixState {
  let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("camera buffer"),
    contents: bytemuck::cast_slice(&[matrix_uniform]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
  });

  let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    entries: &[wgpu::BindGroupLayoutEntry {
      binding: 0,
      visibility: wgpu::ShaderStages::VERTEX,
      ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    }],
    label: Some("a matrix bind group layout"),
  });

  let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &bind_group_layout,
    entries: &[wgpu::BindGroupEntry {
      binding: 0,
      resource: buffer.as_entire_binding(),
    }],
    label: Some("a matrix bind group"),
  });

  MatrixState {
    uniform: matrix_uniform,
    buffer,
    bind_group_layout,
    bind_group,
  }
}
