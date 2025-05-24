// NOTE: The code here may be moved and cleaned up
// later; we're just getting it down first.

// build scene from (mesh, matrix) vector

use super::Scene;
use crate::matrix::{self, MatrixState, MatrixUniform};
use crate::pipeline;
use crate::render::RenderState;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TexturedVertex {
  pub position: [f32; 3],
  pub color: [f32; 3], // TODO: make these texture coords
}

#[derive(Clone)]
pub struct TexturedMeshData {
  pub vertices: Vec<TexturedVertex>,
  pub indices: Vec<u16>,
}

pub struct TexturedMeshRenderData {
  // TODO: store additional info needed for texture
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_indices: u32,
  pub matrix: MatrixState,
}

impl TexturedMeshRenderData {
  // TODO: update to fill in needed texture info
  fn from_mesh_data(
    device: &wgpu::Device,
    mesh_data: TexturedMeshData,
    matrix_uniform: MatrixUniform,
  ) -> Self {
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

    TexturedMeshRenderData {
      vertex_buffer,
      index_buffer,
      num_indices,
      matrix,
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

  let last_mesh = textured_meshes.last().unwrap();

  // only uses matrix layout, so only need one
  let pipeline = pipeline::create_render_pipeline(
    &state.device,
    &state.config,
    &[
      &state.camera_state.matrix.bind_group_layout,
      &last_mesh.matrix.bind_group_layout,
    ],
  );

  Scene {
    meshes: vec![],
    textured_meshes,
    pipeline,
  }
}
