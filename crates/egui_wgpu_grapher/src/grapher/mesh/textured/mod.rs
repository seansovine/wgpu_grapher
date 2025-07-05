// Structures and functions for building textured mesh scenes.

pub mod image_viewer;
pub mod pde;

use super::{Bufferable, Scene};
use crate::grapher::{
    matrix::{self, MatrixState, MatrixUniform},
    pipeline,
    pipeline::texture::TextureData,
    render::RenderState,
};

use egui_wgpu::wgpu::{self, util::DeviceExt, Device, SurfaceConfiguration};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Bufferable for TexturedVertex {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct TexturedMeshData {
    pub vertices: Vec<TexturedVertex>,
    pub indices: Vec<u32>,
    pub texture: TextureData,
}

pub struct TexturedMeshRenderData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub matrix: MatrixState,
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

        TexturedMeshRenderData {
            vertex_buffer,
            index_buffer,
            num_indices,
            matrix,
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

    // we'll try to get away with just one textured render pipeline
    let pipeline = pipeline::create_render_pipeline::<TexturedVertex>(
        device,
        surface_config,
        pipeline::get_textured_shader(),
        &[
            &state.camera_state.matrix.bind_group_layout,
            &last_mesh.matrix.bind_group_layout,
            &last_mesh.texture.bind_group_layout,
        ],
        wgpu::PolygonMode::Fill,
    );

    Scene {
        meshes: vec![],
        textured_meshes,
        pipeline: None,
        textured_pipeline: Some(pipeline),
    }
}

// mesh data to render two-sided square

const SQUARE_VERTICES_VERTICAL: &[TexturedVertex] = &[
    TexturedVertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    },
    TexturedVertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    },
    TexturedVertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    },
    TexturedVertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
];

const SQUARE_VERTICES_FLAT: &[TexturedVertex] = &[
    TexturedVertex {
        position: [-0.5, 0.0, 0.5],
        tex_coords: [0.0, 1.0],
    },
    TexturedVertex {
        position: [0.5, 0.0, 0.5],
        tex_coords: [1.0, 1.0],
    },
    TexturedVertex {
        position: [0.5, 0.0, -0.5],
        tex_coords: [1.0, 0.0],
    },
    TexturedVertex {
        position: [-0.5, 0.0, -0.5],
        tex_coords: [0.0, 0.0],
    },
];

#[rustfmt::skip]
const SQUARE_INDICES: &[u32] = &[
  // top
  0, 1, 2,
  0, 2, 3,
  // bottom
  0, 2, 1,
  0, 3, 2,
];
