//! Code for meshes with color provided per-vertex; currently only graph mode.

pub mod graph;
#[allow(dead_code)]
pub mod pde;

use super::{GpuVertex, Scene};
use crate::grapher::{
    matrix::{self, MatrixState, MatrixUniform},
    pipeline::{self, light},
    render::{RenderState, ShadowState},
};

use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Buffer, Device, SurfaceConfiguration, util::DeviceExt,
};
use std::sync::LazyLock;

#[derive(Clone)]
pub struct MeshData {
    pub vertices: Vec<GpuVertex>,
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
    pub _matrix: MatrixState,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
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

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[matrix.bind_group_layout_entry],
            label: Some("solid mesh matrix bind group layout"),
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: matrix.buffer.as_entire_binding(),
            }],
            label: Some("solid mesh matrix bind group"),
        });

        MeshRenderData {
            vertex_buffer,
            index_buffer,
            num_indices,
            _matrix: matrix,
            bind_group_layout,
            bind_group,
        }
    }
}

// build scene from (mesh, matrix) vector.

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

    let light = light::LightState::create(device);
    let shadow = ShadowState::create::<GpuVertex>(surface_config, device, &light, last_mesh);

    let pipeline = pipeline::create_render_pipeline::<GpuVertex>(
        device,
        surface_config,
        pipeline::get_shader(),
        &[
            &state.bind_group_layout,
            &last_mesh.bind_group_layout,
            &light.bind_group_layout,
            &shadow.bind_group_layout,
            &light.camera_matrix_bind_group_layout,
        ],
        state.render_preferences.polygon_mode,
    );

    Scene {
        pipeline: Some(pipeline),
        textured_pipeline: None,

        meshes,
        textured_meshes: vec![],

        light,
        shadow: Some(shadow),
    }
}

// Simple test scene for development use.

static TEST_MESH: LazyLock<MeshData> = LazyLock::new(|| MeshData {
    vertices: Vec::from([
        GpuVertex {
            position: [0.0, 1.0, 0.0],
            color: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        },
        GpuVertex {
            position: [-0.5, 0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        },
        GpuVertex {
            position: [0.5, 0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
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
