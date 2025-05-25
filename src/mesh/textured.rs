// NOTE: The code here may be moved and cleaned up
// later; we're just getting it down first.

// build scene from (mesh, matrix) vector

use super::Scene;
use crate::matrix::{self, MatrixState, MatrixUniform};
use crate::render::RenderState;
use crate::texture::TextureData;
use crate::{pipeline, texture};

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedVertex {
  pub position: [f32; 3],
  pub tex_coords: [f32; 2],
}

pub struct TexturedMeshData {
  pub vertices: Vec<TexturedVertex>,
  pub indices: Vec<u16>,
  pub texture: TextureData,
}

pub struct TexturedMeshRenderData {
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_indices: u32,
  pub matrix: MatrixState,
  pub texture: TextureData,
}

// TODO: Update ^ and v with texture info as needed.

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

pub fn build_scene(
  state: &RenderState,
  mesh_data: Vec<(TexturedMeshData, MatrixUniform)>,
) -> Scene {
  let mut textured_meshes = vec![];

  for (mesh, matrix) in mesh_data {
    let mesh_render_data = TexturedMeshRenderData::from_mesh_data(&state.device, mesh, matrix);
    textured_meshes.push(mesh_render_data);
  }
  // TODO: we need to handle depth issues when rendering these

  // we'll use the matrix and texture layouts from this mesh
  let last_mesh = textured_meshes.last().unwrap();

  // we'll try to get away with just one texturedpipeline
  let pipeline = pipeline::create_render_pipeline::<TexturedVertex>(
    &state.device,
    &state.config,
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

// test scene

const TEST_VERTICES: &[TexturedVertex] = &[
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

#[rustfmt::skip]
const TEST_INDICES: &[u16] = &[
  // top
  0, 1, 2,
  0, 2, 3,
  // bottom
  0, 2, 1,
  0, 3, 2,
];

#[allow(unused)]
pub fn image_test_scene(state: &RenderState) -> Scene {
  let image = texture::Image::from_file("assets/pexels-arjay-neyra-2152024526-32225792.jpg");
  let texture = texture::texture_from_image(&image, state);
  let texture_data = TextureData::from_image(&image, state);

  let mesh_data = TexturedMeshData {
    vertices: Vec::from(TEST_VERTICES),
    indices: Vec::from(TEST_INDICES),
    texture: texture_data,
  };

  let mut meshes: Vec<(TexturedMeshData, MatrixUniform)> =
    vec![(mesh_data, MatrixUniform::translation(&[0.0, 0.0, 0.0]))];

  // TODO: build and return scene
  build_scene(state, meshes)
}
