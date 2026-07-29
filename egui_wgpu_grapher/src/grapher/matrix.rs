//! Code for converting between 4x4 matrix types and making matrix uniforms.

use std::{ops::Mul, sync::OnceLock};

use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu::{
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

// ----------------
// 4x4 matrix type.

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Matrix {
    data: [[f32; 4]; 4],
    // TODO: We should store a cgmath::Matrix4 here. It also
    //       has repr(c) with the same layout, and that would
    //       avoid some converseions, though the conversions
    //       are done pretty rarely.
}

impl From<[[f32; 4]; 4]> for Matrix {
    fn from(value: [[f32; 4]; 4]) -> Self {
        Self { data: value }
    }
}

impl From<Matrix> for cgmath::Matrix4<f32> {
    fn from(value: Matrix) -> Self {
        value.data.into()
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
        let cg_self: cgmath::Matrix4<_> = self.data.into();
        let cg_other: cgmath::Matrix4<_> = rhs.data.into();
        Self {
            data: (cg_self * cg_other).into(),
        }
    }
}

pub const X_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 0.0, 1.0);

impl Matrix {
    pub fn identity() -> Self {
        use cgmath::SquareMatrix;
        Self {
            data: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn from(matrix: cgmath::Matrix4<f32>) -> Self {
        Self {
            data: matrix.into(),
        }
    }

    pub fn translation(coords: &[f32]) -> Self {
        Self {
            data: cgmath::Matrix4::from_translation(cgmath::Vector3 {
                x: coords[0],
                y: coords[1],
                z: coords[2],
            })
            .into(),
        }
    }

    pub fn x_rotation(degrees: f32) -> Self {
        Self {
            data: cgmath::Matrix4::from_axis_angle(X_AXIS, cgmath::Deg(degrees)).into(),
        }
    }

    pub fn update_value(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.data = matrix.into();
    }

    pub fn mat4_left_mul(&mut self, lhs: &cgmath::Matrix4<f32>) {
        let matrix_cg: cgmath::Matrix4<_> = self.data.into();
        self.data = (lhs * matrix_cg).into();
    }
}

// ----------------------------
// Uniform data for 4x4 matrix.

pub struct MatrixUniform {
    pub matrix: Matrix,
    pub buffer: Buffer,
}

impl MatrixUniform {
    pub fn bind_group_layout_entry() -> &'static BindGroupLayoutEntry {
        static BGL_ENTRY: OnceLock<BindGroupLayoutEntry> = OnceLock::new();
        BGL_ENTRY.get_or_init(|| BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
    }

    pub fn write_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.matrix]));
    }
}

pub(crate) fn make_matrix_uniform(device: &Device, matrix_uniform: Matrix) -> MatrixUniform {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("camera buffer"),
        contents: bytemuck::cast_slice(&[matrix_uniform]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    MatrixUniform {
        matrix: matrix_uniform,
        buffer,
    }
}
