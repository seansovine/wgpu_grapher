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
