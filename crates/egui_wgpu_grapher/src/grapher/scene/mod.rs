pub mod solid;

#[allow(dead_code)]
pub mod textured;

use super::render::RenderState;

use egui_wgpu::wgpu::{self, Queue, RenderPipeline};

pub struct Scene {
    pub meshes: Vec<solid::MeshRenderData>,
    pub textured_meshes: Vec<textured::TexturedMeshRenderData>,
    pub pipeline: Option<RenderPipeline>,
    pub textured_pipeline: Option<RenderPipeline>,
}

// trait to abstract scene behavior in render loop

pub trait RenderScene {
    /// get associated Scene reference
    fn scene(&self) -> &Scene;

    /// perform any timestep state updates
    fn update(&mut self, queue: &Queue, state: &RenderState, pre_render: bool);
}

impl RenderScene for Scene {
    fn scene(&self) -> &Scene {
        self
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState, _pre_render: bool) {
        // no-op; basic scene is static
    }
}

// trait for structs that can provide a vertex buffer layout

pub(crate) trait Bufferable {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

// Vertex passed to GPU in buffer.

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Bufferable for Vertex {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 0.0, 1.0],
            normal: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
        }
    }
}
