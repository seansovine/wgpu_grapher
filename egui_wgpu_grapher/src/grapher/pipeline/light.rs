use cgmath::Matrix4;
use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Buffer, Device, Queue, util::DeviceExt,
};

use crate::grapher::{
    camera,
    matrix::{self, Matrix, MatrixUniform},
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding_1: u32,
    color: [f32; 3],
    _padding_2: u32,
}

pub struct LightState {
    // to pass light data to shader as uniform
    pub uniform: LightUniform,
    pub buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,

    // light view matrix used for shadow mapping
    pub camera_matrix: MatrixUniform,
    pub camera_matrix_bind_group_layout: BindGroupLayout,
    pub camera_matrix_bind_group: BindGroup,

    // one-step light state save and restore
    #[allow(unused)]
    pub previous_uniform: Option<LightUniform>,
}

impl LightState {
    pub fn set_position(&mut self, new_position: [f32; 3]) {
        self.uniform.position = new_position;
    }

    pub fn update_uniform(&mut self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

impl LightState {
    const DEFAULT_LIGHT_POS: [f32; 3] = [3.0, 4.0, 0.0];

    pub fn create(device: &Device) -> Self {
        let uniform = LightUniform {
            position: Self::DEFAULT_LIGHT_POS,
            _padding_1: 0_u32,
            color: [1.0, 1.0, 1.0],
            _padding_2: 0_u32,
        };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light UBO"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[bind_group_layout_entry],
            label: Some("light bind group layout"),
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("light bind group"),
        });

        // Create view matrix for use in shadow mapping.
        let matrix = Self::build_shadow_matrix(&uniform.position);
        let matrix_uniform = Matrix::from(matrix);
        let camera_matrix = matrix::make_matrix_state(device, matrix_uniform);

        let camera_matrix_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[camera_matrix.bind_group_layout_entry],
                label: Some("solid mesh matrix bind group layout"),
            });
        let camera_matrix_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_matrix_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_matrix.buffer.as_entire_binding(),
            }],
            label: Some("solid mesh matrix bind group"),
        });

        // TODO: Later we'll want to be able to update the light
        // during runtime, which will require updating this matrix.

        Self {
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
            //
            camera_matrix,
            camera_matrix_bind_group_layout,
            camera_matrix_bind_group,
            //
            previous_uniform: None,
        }
    }

    fn build_shadow_matrix(position: &[f32; 3]) -> Matrix4<f32> {
        let view_target = cgmath::Point3::<f32>::from([0.0, 0.0, 0.0]);
        let view_origin = cgmath::Point3::<f32>::from(*position);

        let view_up = if position[0] == 0.0 && position[2] == 0.0 {
            cgmath::Vector3::<f32>::from([1.0, 0.0, 0.0])
        } else {
            cgmath::Vector3::<f32>::from([0.0, 1.0, 0.0])
        };
        let view = cgmath::Matrix4::look_at_rh(view_origin, view_target, view_up);

        // TODO: This should be the identity matrix. Look at other possible uses.
        let projection = cgmath::ortho(-1.0_f32, 1.0_f32, -1.0_f32, 1.0_f32, -1.0, 1.0);

        camera::OPENGL_TO_WGPU_MATRIX * projection * view
    }

    pub fn camera_view_matrix(&self) -> &MatrixUniform {
        &self.camera_matrix
    }

    #[allow(unused)]
    pub fn save_light(&mut self) {
        self.previous_uniform = Some(self.uniform);
    }

    // Restores light uniform from previous state if one was saved.
    #[allow(unused)]
    pub fn maybe_restore_light(&mut self, queue: &Queue) {
        if let Some(uniform) = self.previous_uniform.take() {
            self.uniform = uniform;
            self.update_uniform(queue);
        }
    }
}
