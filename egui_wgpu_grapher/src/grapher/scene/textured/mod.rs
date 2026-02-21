//! Code for building textured mesh scenes.

pub mod image_viewer;
pub mod model;
pub mod pde_2d_cpu;

use super::{GpuVertex, Scene3D};
use crate::grapher::{
    matrix::{self, Matrix, MatrixUniform},
    pipeline::{self, light, texture::TextureData},
    render::RenderState,
};

use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Device, SurfaceConfiguration, util::DeviceExt,
};
use std::sync::{LazyLock, OnceLock};

// ------------------------------------------------------
// Structures for raw textured mesh data and render data.

pub struct TexturedMeshData {
    pub vertices: Vec<GpuVertex>,
    pub indices: Vec<u32>,
    pub texture: TextureData,
}

pub struct TexturedMeshRenderData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,

    pub matrix: MatrixUniform,
    pub matrix_bind_group: BindGroup,

    pub texture: TextureData,
}

impl TexturedMeshRenderData {
    fn matrix_bgl(device: &Device) -> &'static BindGroupLayout {
        static BGL: OnceLock<BindGroupLayout> = OnceLock::new();
        BGL.get_or_init(|| {
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[*MatrixUniform::bind_group_layout_entry()],
                label: Some("solid mesh matrix bind group layout"),
            })
        })
    }

    fn from_mesh_data(
        device: &wgpu::Device,
        mesh_data: TexturedMeshData,
        matrix_uniform: Matrix,
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

        let matrix = matrix::make_matrix_uniform(device, matrix_uniform);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: Self::matrix_bgl(device),
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
            //
            matrix,
            matrix_bind_group: bind_group,
            //
            texture: mesh_data.texture,
        }
    }
}

// ---------------------------------------
// Build scene from (mesh, matrix) vector.

pub fn build_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
    mesh_data: Vec<(TexturedMeshData, Matrix)>,
) -> Scene3D {
    let textured_meshes: Vec<TexturedMeshRenderData> = mesh_data
        .into_iter()
        .map(|(mesh, matrix)| TexturedMeshRenderData::from_mesh_data(device, mesh, matrix))
        .collect();

    let light = light::LightState::create(device);

    let pipeline = pipeline::create_render_pipeline::<GpuVertex>(
        device,
        surface_config,
        pipeline::get_textured_shader(),
        &[
            &state.bind_group_layout,
            TexturedMeshRenderData::matrix_bgl(device),
            &light.bind_group_layout,
            TextureData::bind_group_layout(device),
        ],
        wgpu::PolygonMode::Fill,
    );

    Scene3D {
        pipeline: None,
        textured_pipeline: Some(pipeline),
        //
        meshes: vec![],
        textured_meshes,
        //
        light,
        shadow: None,
    }
}

// -------------------------------------
// Mesh data for simple square canvases.

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
