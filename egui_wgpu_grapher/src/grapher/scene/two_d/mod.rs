//! Render data for a two=dimensional scene.
//!
//! This will be used as a canvas to display a texture that
//! is updated by a time-dependent equation-solver pipeline.

use egui_wgpu::wgpu::{
    self, Buffer, Device, RenderPipeline, SurfaceConfiguration, util::DeviceExt,
};

use crate::grapher::pipeline::create_2d_pipeline;

pub struct TwoDScene {
    pub pipeline: RenderPipeline,
    pub index_buffer: Buffer,
}

static INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

impl TwoDScene {
    pub fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            pipeline: create_2d_pipeline(device, surface_config, &[]),
            index_buffer,
        }
    }
}
