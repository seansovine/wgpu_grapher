//! Code for converting between 4x4 matrix types and making matrix uniforms.

use std::ops::Mul;

use egui_wgpu::wgpu::{
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

pub const X_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix {
    matrix: [[f32; 4]; 4],
    // TODO: We should store a cgmath::Matrix4 here. It also
    //       has repr(c) with the same layout, and that would
    //       avoid some converseions, though the conversions
    //       are done pretty rarely.
}

impl From<[[f32; 4]; 4]> for Matrix {
    fn from(value: [[f32; 4]; 4]) -> Self {
        Self { matrix: value }
    }
}

impl From<Matrix> for cgmath::Matrix4<f32> {
    fn from(value: Matrix) -> Self {
        value.matrix.into()
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Matrix {
    type Output = Self;

    // For convenience; this is rarely used.
    fn mul(self, rhs: Self) -> Self::Output {
        let cg_self: cgmath::Matrix4<_> = self.matrix.into();
        let cg_other: cgmath::Matrix4<_> = rhs.matrix.into();
        Self {
            matrix: (cg_self * cg_other).into(),
        }
    }
}

impl Matrix {
    pub fn identity() -> Self {
        use cgmath::SquareMatrix;
        Self {
            matrix: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn from(matrix: cgmath::Matrix4<f32>) -> Self {
        Self {
            matrix: matrix.into(),
        }
    }

    pub fn translation(coords: &[f32]) -> Self {
        Self {
            matrix: cgmath::Matrix4::from_translation(cgmath::Vector3 {
                x: coords[0],
                y: coords[1],
                z: coords[2],
            })
            .into(),
        }
    }

    pub fn x_rotation(degrees: f32) -> Self {
        Self {
            matrix: cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Deg(degrees)).into(),
        }
    }

    pub fn update_inner(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.matrix = matrix.into();
    }

    pub fn mat4_left_mul(&mut self, lhs: &cgmath::Matrix4<f32>) {
        let matrix_cg: cgmath::Matrix4<_> = self.matrix.into();
        self.matrix = (lhs * matrix_cg).into();
    }
}

pub struct MatrixUniform {
    pub uniform: Matrix,
    pub buffer: Buffer,
    pub bind_group_layout_entry: BindGroupLayoutEntry,
}

impl MatrixUniform {
    #[allow(unused)]
    pub fn set_binding_index(&mut self, binding_index: u32) {
        self.bind_group_layout_entry.binding = binding_index;
    }
}

pub(crate) fn make_matrix_state(device: &Device, matrix_uniform: Matrix) -> MatrixUniform {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("camera buffer"),
        contents: bytemuck::cast_slice(&[matrix_uniform]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let bind_group_layout_entry = BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };

    MatrixUniform {
        uniform: matrix_uniform,
        buffer,
        bind_group_layout_entry,
    }
}
