use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, util::DeviceExt};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _pad1: u32,
    color: [f32; 3],
    _pad2: u32,
}

pub struct LightState {
    // in case we later update light during render
    pub _uniform: LightUniform,
    pub _buffer: Buffer,

    // to create pipeline
    pub bind_group_layout: BindGroupLayout,
    // bound during render passes
    pub bind_group: BindGroup,
}

impl LightState {
    pub fn create(device: &Device) -> Self {
        let uniform = LightUniform {
            position: [0.0, 11.0, 0.0],
            _pad1: 0,
            color: [1.0, 1.0, 1.0],
            _pad2: 0,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            _uniform: uniform,
            _buffer: buffer,
            bind_group_layout,
            bind_group,
        }
    }
}
