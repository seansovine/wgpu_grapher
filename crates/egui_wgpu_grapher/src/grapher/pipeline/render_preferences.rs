// Preferences passed to shaders as a uniform.

use egui_wgpu::wgpu::{
    self, BindGroupLayoutEntry, Buffer, Device, PolygonMode, Queue, util::DeviceExt,
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
    pub bind_group_layout_entry: BindGroupLayoutEntry,
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
            // lighting enabled by default
            flags: 1_u32,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
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

        Self {
            uniform,
            buffer,
            bind_group_layout_entry,
            polygon_mode,
        }
    }

    pub fn set_binding_index(&mut self, binding_index: u32) {
        self.bind_group_layout_entry.binding = binding_index;
    }
}
