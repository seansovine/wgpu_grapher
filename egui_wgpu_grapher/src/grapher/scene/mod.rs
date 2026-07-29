//! This module has code for building and representing scenes that we render.

pub mod solid;
pub mod solver;
pub mod textured;

use super::render::RenderState;
use crate::grapher::{
    light::{LightState, ShadowState},
    pipeline::{self},
};

use bytemuck::{Pod, Zeroable};
use cgmath::{Deg, Quaternion, Rotation, Rotation3, Vector3};
use egui_wgpu::wgpu::{self, BindGroupLayout, Device, Queue, RenderPipeline, SurfaceConfiguration};

// -----------------------------------------
// Pipelines and render data for a 3D scene.

pub struct Scene3D {
    // solid and textured render pipelines
    pub pipeline: Option<RenderPipeline>,
    pub textured_pipeline: Option<RenderPipeline>,
    // meshes
    pub meshes: Vec<solid::MeshRenderData>,
    pub textured_meshes: Vec<textured::TexturedMeshRenderData>,

    // For drawing debug items in the scene.
    pub debug_pipeline: RenderPipeline,

    // light
    pub light: LightState,
    // shadow
    pub shadow: Option<ShadowState>,
}

fn debug_data(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    _light: &LightState,
    camera_layout: &BindGroupLayout,
) -> RenderPipeline {
    pipeline::create_render_pipeline::<GpuVertex>(
        device,
        surface_config,
        pipeline::get_debug_shader(),
        &[camera_layout, LightState::light_bgl(device)],
        wgpu::PolygonMode::Fill,
    )
}

// ------------------------------------------------
// Trait to abstract scene behavior in render loop.

pub trait RenderScene {
    /// get associated Scene reference
    fn scene(&self) -> &Scene3D;

    /// perform any timestep state updates
    fn update(&mut self, queue: &Queue, state: &RenderState);
}

impl RenderScene for Scene3D {
    fn scene(&self) -> &Scene3D {
        self
    }

    fn update(&mut self, queue: &Queue, state: &RenderState) {
        if state.light_motion {
            let rotation = Quaternion::from_axis_angle(Vector3::from([0.0, 1.0, 0.0]), Deg(0.5));
            let old_position: Vector3<f32> = self.light.position().into();
            let new_position = rotation.rotate_vector(old_position);
            self.light.set_position(new_position.into());
            self.light.update_uniform(queue);
        }
    }
}

// ----------------------------------------------------------
// Trait for structs that can provide a vertex buffer layout.

pub trait Bufferable {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

// --------------------------------------------------
// Vertex mapped to GPU buffer in 3D scene framework.

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Bufferable for GpuVertex {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as wgpu::BufferAddress,
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

impl Default for GpuVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 0.0, 1.0],
            normal: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
        }
    }
}
