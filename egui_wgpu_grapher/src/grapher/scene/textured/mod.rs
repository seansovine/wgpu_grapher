// Structures and functions for building textured mesh scenes.

pub mod gltf_loader;
pub mod image_viewer;
// #[allow(dead_code)]
pub mod model;
pub mod pde;

use super::{GpuVertex, Scene};
use crate::grapher::{
    matrix::{self, MatrixState, MatrixUniform},
    pipeline::{self, light, texture::TextureData},
    render::RenderState,
};

use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Device, SurfaceConfiguration, util::DeviceExt,
};
use std::sync::LazyLock;

pub struct TexturedMeshData {
    pub vertices: Vec<GpuVertex>,
    pub indices: Vec<u32>,
    pub texture: TextureData,
}

pub struct TexturedMeshRenderData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,

    pub matrix: MatrixState,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,

    pub texture: TextureData,
}

impl TexturedMeshRenderData {
    fn from_mesh_data(
        device: &wgpu::Device,
        mesh_data: TexturedMeshData,
        matrix_uniform: MatrixUniform,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(mesh_data.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
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

        TexturedMeshRenderData {
            vertex_buffer,
            index_buffer,
            num_indices,

            matrix,
            bind_group_layout,
            bind_group,

            texture: mesh_data.texture,
        }
    }
}

// build scene from (mesh, matrix) vector

pub fn build_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
    mesh_data: Vec<(TexturedMeshData, MatrixUniform)>,
) -> Scene {
    let mut textured_meshes = vec![];

    for (mesh, matrix) in mesh_data {
        let mesh_render_data = TexturedMeshRenderData::from_mesh_data(device, mesh, matrix);
        textured_meshes.push(mesh_render_data);
    }
    // use the matrix and texture layouts from the last mesh
    let last_mesh = textured_meshes.last().unwrap();

    let light = light::LightState::create(device);
    // we'll try to get away with just one textured render pipeline
    let pipeline = pipeline::create_render_pipeline::<GpuVertex>(
        device,
        surface_config,
        pipeline::get_textured_shader(),
        &[
            &state.bind_group_layout,
            &last_mesh.bind_group_layout,
            &light.bind_group_layout,
            &last_mesh.texture.bind_group_layout,
        ],
        wgpu::PolygonMode::Fill,
    );

    Scene {
        pipeline: None,
        textured_pipeline: Some(pipeline),

        meshes: vec![],
        textured_meshes,

        light,
        shadow: None,
    }
}

// mesh data to render two-sided square

static SQUARE_VERTICES_VERTICAL: LazyLock<Vec<GpuVertex>> = LazyLock::new(|| {
    vec![
        GpuVertex {
            position: [-0.5, -0.5, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        },
        GpuVertex {
            position: [0.5, -0.5, 0.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        },
        GpuVertex {
            position: [0.5, 0.5, 0.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        },
        GpuVertex {
            position: [-0.5, 0.5, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        },
    ]
});

static SQUARE_VERTICES_FLAT: LazyLock<Vec<GpuVertex>> = LazyLock::new(|| {
    vec![
        GpuVertex {
            position: [-0.5, 0.0, 0.5],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        },
        GpuVertex {
            position: [0.5, 0.0, 0.5],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        },
        GpuVertex {
            position: [0.5, 0.0, -0.5],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        },
        GpuVertex {
            position: [-0.5, 0.0, -0.5],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        },
    ]
});

#[rustfmt::skip]
const SQUARE_INDICES: &[u32] = &[
  // top
  0, 1, 2,
  0, 2, 3,
  // bottom
  0, 2, 1,
  0, 3, 2,
];
