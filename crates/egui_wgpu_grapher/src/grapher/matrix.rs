// General code for creating matrix uniforms and associated buffers.

use egui_wgpu::wgpu::{
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

pub const X_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MatrixUniform {
    matrix: [[f32; 4]; 4],
}

impl MatrixUniform {
    #[allow(unused)]
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

    pub fn update(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.matrix = matrix.into();
    }
}

pub struct MatrixState {
    pub uniform: MatrixUniform,
    pub buffer: Buffer,
    pub bind_group_layout_entry: BindGroupLayoutEntry,
}

impl MatrixState {
    #[allow(unused)]
    pub fn set_binding_index(&mut self, binding_index: u32) {
        self.bind_group_layout_entry.binding = binding_index;
    }
}

pub(crate) fn make_matrix_state(device: &Device, matrix_uniform: MatrixUniform) -> MatrixState {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("camera buffer"),
        contents: bytemuck::cast_slice(&[matrix_uniform]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let bind_group_layout_entry = BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };

    MatrixState {
        uniform: matrix_uniform,
        buffer,
        bind_group_layout_entry,
    }
}
