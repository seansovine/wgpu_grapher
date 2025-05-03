// Data representing a mesh and the vertices that they're made of.

use crate::graph;
use crate::matrix;
use crate::matrix::{MatrixState, MatrixUniform};
use crate::pipeline;
use crate::state::RenderState;

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
      usage: wgpu::BufferUsages::VERTEX,
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

pub struct Scene {
  pub meshes: Vec<MeshRenderData>,
  pub pipeline: RenderPipeline,
}

// build scene from (mesh, vector) vector

pub fn build_scene(state: &RenderState, mesh_data: Vec<(MeshData, MatrixUniform)>) -> Scene {
  let mut meshes = vec![];

  for (mesh, matrix) in mesh_data {
    let mesh_render_data = MeshRenderData::from_mesh_data(&state.device, mesh, matrix);
    meshes.push(mesh_render_data);
  }
  // TODO: we need to handle depth issues when rendering these

  let last_mesh = meshes.last().unwrap();

  // only uses matrix layout, so should only need one
  let pipeline = pipeline::create_render_pipeline(
    &state.device,
    &state.config,
    &[
      &state.camera_state.matrix.bind_group_layout,
      &last_mesh.matrix.bind_group_layout,
    ],
  );

  Scene { meshes, pipeline }
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

#[allow(unused)]
pub fn test_scene(state: &RenderState) -> Scene {
  let mut meshes: Vec<(MeshData, MatrixUniform)> = vec![];

  let mut back_mesh = (*TEST_MESH).clone();
  let gold = [168.0f32 / 255.0f32, 125.0f32 / 255.0f32, 50.0f32 / 255.0f32];
  back_mesh.set_uniform_color(gold);
  meshes.push((back_mesh, MatrixUniform::translation(&[0.0, -0.5, -0.5])));

  let front_mesh = (*TEST_MESH).clone();
  meshes.push((front_mesh, MatrixUniform::translation(&[0.0, -0.5, 0.5])));

  build_scene(state, meshes)
}

// graph data

#[allow(unused)]
pub fn graph_scene(state: &RenderState) -> Scene {
  static SUBDIVISIONS: u16 = 64;

  let floor_mesh = graph::UnitSquareTesselation::generate(SUBDIVISIONS)
    .mesh_data(graph::UnitSquareTesselation::FLOOR_COLOR);
  let matrix = MatrixUniform::translation(&[-0.5, -0.5, -0.5]);

  // example function
  let mut f = |x: f32, z: f32| x.sin() * z.cos();
  let f = graph::shift_scale_input(f, 0.5, 8.0, 0.5, 8.0);
  let f = graph::shift_scale_output(f, 0.55, 0.5);

  let func_mesh = graph::UnitSquareTesselation::generate(SUBDIVISIONS)
    .apply_function(f)
    .mesh_data(graph::UnitSquareTesselation::FUNCT_COLOR);

  build_scene(state, vec![(floor_mesh, matrix), (func_mesh, matrix)])
}
