// Structures and functions for meshes with color provided per-vertex.

#[allow(dead_code)]
pub mod graph;
#[allow(dead_code)]
pub mod model;
#[allow(dead_code)]
pub mod pde;

use super::{Bufferable, Scene};
use crate::grapher::{
    matrix::{self, MatrixState, MatrixUniform},
    pipeline,
    render::RenderState,
};

use egui_wgpu::wgpu::{self, util::DeviceExt, Buffer, Device, SurfaceConfiguration};
use std::sync::LazyLock;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
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
            ],
        }
    }
}

#[derive(Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn set_uniform_color(&mut self, rgb: [f32; 3]) {
        for vertex in &mut self.vertices {
            vertex.color = rgb;
        }
    }
}

pub struct MeshRenderData {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
    pub matrix: MatrixState,
}

impl MeshRenderData {
    fn from_mesh_data(device: &Device, mesh_data: MeshData, matrix_uniform: MatrixUniform) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(mesh_data.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(mesh_data.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = mesh_data.indices.len() as u32;

        let matrix = matrix::make_matrix_state(device, matrix_uniform);

        MeshRenderData {
            vertex_buffer,
            index_buffer,
            num_indices,
            matrix,
        }
    }
}

// build scene from (mesh, matrix) vector

pub fn build_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
    mesh_data: Vec<(MeshData, MatrixUniform)>,
) -> Scene {
    let mut meshes = vec![];

    for (mesh, matrix) in mesh_data {
        let mesh_render_data = MeshRenderData::from_mesh_data(device, mesh, matrix);
        meshes.push(mesh_render_data);
    }

    let last_mesh = meshes.last().unwrap();

    // use this pipeline for all solid meshes
    let pipeline = pipeline::create_render_pipeline::<Vertex>(
        device,
        surface_config,
        pipeline::get_shader(),
        &[
            &state.camera_state.matrix.bind_group_layout,
            &last_mesh.matrix.bind_group_layout,
            &state.light_state.bind_group_layout,
            &state.render_preferences.bind_group_layout,
        ],
        state.render_preferences.polygon_mode,
    );

    Scene {
        meshes,
        textured_meshes: vec![],
        pipeline: Some(pipeline),
        textured_pipeline: None,
    }
}

// test data

static TEST_MESH: LazyLock<MeshData> = LazyLock::new(|| MeshData {
    vertices: Vec::from([
        Vertex {
            position: [0.0, 1.0, 0.0],
            color: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        },
    ]),
    indices: Vec::from([0, 1, 2, 0, 2, 1]),
});

#[allow(unused)]
pub fn test_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> Scene {
    let mut meshes: Vec<(MeshData, MatrixUniform)> = vec![];

    let mut back_mesh = (*TEST_MESH).clone();
    let gold = [168.0f32 / 255.0f32, 125.0f32 / 255.0f32, 50.0f32 / 255.0f32];
    back_mesh.set_uniform_color(gold);
    meshes.push((back_mesh, MatrixUniform::translation(&[0.0, -0.5, -0.5])));

    let front_mesh = (*TEST_MESH).clone();
    meshes.push((front_mesh, MatrixUniform::translation(&[0.0, -0.5, 0.5])));

    build_scene(device, surface_config, state, meshes)
}
