// General code for creating matrix uniforms and associated buffers.

use egui_wgpu::wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
};

pub const X_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MatrixUniform {
    matrix: [[f32; 4]; 4],
}

impl MatrixUniform {
    pub fn identity() -> Self {
        use cgmath::SquareMatrix;
        Self {
            matrix: cgmath::Matrix4::identity().into(),
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

    pub fn update(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.matrix = matrix.into();
    }
}

pub struct MatrixState {
    pub uniform: MatrixUniform,
    pub buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

pub(crate) fn make_matrix_state(device: &Device, matrix_uniform: MatrixUniform) -> MatrixState {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("camera buffer"),
        contents: bytemuck::cast_slice(&[matrix_uniform]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("a matrix bind group layout"),
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[BindGroupEntry {
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
