// Preferences passed to shaders as a uniform.

use egui_wgpu::wgpu::{
    self, util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, PolygonMode, Queue,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

pub struct ShaderPreferencesUniform {
    // 1-bit - enable lighting
    // 2-bit - use texture
    pub flags: u32,
}

pub struct RenderPreferences {
    // data for uniform passed to shader
    pub uniform: ShaderPreferencesUniform,
    pub buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,

    // render pipeline preferences
    pub polygon_mode: PolygonMode,
}

impl RenderPreferences {
    pub fn lighting_enabled(&self) -> bool {
        self.uniform.flags & 1 == 1
    }

    pub fn set_lighting_enabled(&mut self, enabled: bool) {
        if enabled {
            self.uniform.flags |= 1_u32;
        } else {
            self.uniform.flags &= !1_u32;
        }
    }

    pub fn set_use_texture(&mut self, enabled: bool) {
        if enabled {
            self.uniform.flags |= 2_u32;
        } else {
            self.uniform.flags &= !2_u32;
        }
    }

    pub fn wireframe_enabled(&self) -> bool {
        self.polygon_mode == PolygonMode::Line
    }

    pub fn set_wireframe(&mut self, enabled: bool) {
        if enabled {
            self.polygon_mode = PolygonMode::Line;
        } else {
            self.polygon_mode = PolygonMode::Fill;
        }
    }

    pub fn update_uniform(&mut self, queue: &Queue) {
        // update uniform buffer
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

impl RenderPreferences {
    pub fn create(device: &Device) -> Self {
        // pipeline preferences

        let polygon_mode = PolygonMode::Fill;

        // shader preferences

        let uniform = ShaderPreferencesUniform {
            flags: 1_u32, // lighting enabled by default
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
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
            //
            polygon_mode,
        }
    }
}
