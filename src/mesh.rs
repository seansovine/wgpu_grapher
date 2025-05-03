// TODO:
//
// - Add structures needed to render a mesh.
// - Add structures to hold current render data,
//   including meshes and render pipline(s).
// - We will also want a function to generate the
//   meshes for plotting a function.

use crate::matrix;
use crate::matrix::{MatrixState, MatrixUniform};
use crate::pipeline;
use crate::render_state::RenderState;

use std::sync::LazyLock;

use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, RenderPipeline};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub color: [f32; 3],
}

#[derive(Clone)]
pub struct MeshData {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

pub struct MeshRenderData {
  pub vertex_buffer: Buffer,
  pub index_buffer: Buffer,
  pub num_indices: u32,
}

impl MeshRenderData {
  fn from_mesh_data(device: &Device, mesh_data: MeshData) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(mesh_data.vertices.as_slice()),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(mesh_data.indices.as_slice()),
      usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = mesh_data.indices.len() as u32;

    MeshRenderData {
      vertex_buffer,
      index_buffer,
      num_indices,
    }
  }
}

pub struct Scene {
  pub meshes: Vec<MeshRenderData>,
  pub matrices: Vec<MatrixState>,
  pub pipeline: RenderPipeline,
}

// test data

#[allow(unused)]
static TEST_MESH: LazyLock<MeshData> = LazyLock::new(|| MeshData {
  vertices: Vec::from([
    Vertex {
      position: [0.0, 1.0, 0.0],
      color: [1.0, 0.0, 0.0],
    },
    Vertex {
      position: [-0.5, 0.0, 0.0],
      color: [1.0, 0.0, 0.0],
    },
    Vertex {
      position: [0.5, 0.0, 0.0],
      color: [1.0, 0.0, 0.0],
    },
  ]),
  indices: Vec::from([0, 1, 2, 0, 2, 1]),
});

pub fn test_scene(state: &RenderState) -> Scene {
  let mut meshes = vec![];
  let mut matrices = vec![];

  let matrix = matrix::make_matrix_state(
    &state.device,
    MatrixUniform::_translation(&[0.0, -0.5, 0.5]),
  );

  // only uses matrix layout, so should only need one
  let pipeline = pipeline::create_render_pipeline(
    &state.device,
    &state.config,
    &[
      &state.camera_state.matrix.bind_group_layout,
      &matrix.bind_group_layout,
    ],
  );

  matrices.push(matrix);

  let mesh_render_data = MeshRenderData::from_mesh_data(&state.device, (*TEST_MESH).clone());
  meshes.push(mesh_render_data);

  let matrix = matrix::make_matrix_state(
    &state.device,
    MatrixUniform::_translation(&[0.0, -0.5, -0.5]),
  );
  matrices.push(matrix);

  let mesh_render_data = MeshRenderData::from_mesh_data(&state.device, (*TEST_MESH).clone());
  meshes.push(mesh_render_data);

  Scene {
    meshes,
    matrices,
    pipeline,
  }
}
