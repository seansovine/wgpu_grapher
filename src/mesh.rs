// Data representing a mesh and the vertices that they're made of.

use crate::graph;
use crate::matrix;
use crate::matrix::{MatrixState, MatrixUniform};
use crate::pipeline;
use crate::state::RenderState;
use crate::wave_eqn;

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

pub struct Scene {
  pub meshes: Vec<MeshRenderData>,
  pub pipeline: RenderPipeline,
}

// build scene from (mesh, matrix) vector

pub fn build_scene(state: &RenderState, mesh_data: Vec<(MeshData, MatrixUniform)>) -> Scene {
  let mut meshes = vec![];

  for (mesh, matrix) in mesh_data {
    let mesh_render_data = MeshRenderData::from_mesh_data(&state.device, mesh, matrix);
    meshes.push(mesh_render_data);
  }
  // TODO: we need to handle depth issues when rendering these

  let last_mesh = meshes.last().unwrap();

  // only uses matrix layout, so only need one
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

// trait to encapsulate scene behavior

#[allow(unused)]
pub trait RenderScene {
  /// get associated Scene reference
  fn scene(&self) -> &Scene;
  /// perform any timestep state updates
  fn update(&mut self, state: &RenderState);
}

impl RenderScene for Scene {
  fn scene(&self) -> &Scene {
    self
  }

  fn update(&mut self, _state: &RenderState) {}
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

// make scene for function graph

#[allow(unused)]
pub fn graph_scene(state: &RenderState) -> Scene {
  static SUBDIVISIONS: u16 = 64;
  static WIDTH: f32 = 2.0;

  let floor_mesh = graph::UnitSquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .mesh_data(graph::UnitSquareTesselation::FLOOR_COLOR);
  let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, -WIDTH / 2.0_f32, -WIDTH / 2.0_f32]);

  // example function
  let mut f = |x: f32, z: f32| x.sin() * z.cos();
  let f = graph::shift_scale_input(f, 0.5, 8.0, 0.5, 8.0);
  let f = graph::shift_scale_output(f, 0.55, 0.5);

  let func_mesh = graph::UnitSquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .apply_function(f)
    .mesh_data(graph::UnitSquareTesselation::FUNCT_COLOR);

  build_scene(state, vec![(floor_mesh, matrix), (func_mesh, matrix)])
}

// test a scene with changing buffers

pub struct MeltingScene {
  pub scene: Scene,
  pub func_mesh: MeshData,
}

impl MeltingScene {
  const SCALE_FACTOR: f32 = 0.9995;
}

impl RenderScene for MeltingScene {
  fn scene(&self) -> &Scene {
    &self.scene
  }

  fn update(&mut self, state: &RenderState) {
    for vertex in &mut self.func_mesh.vertices {
      vertex.position[1] *= MeltingScene::SCALE_FACTOR;
    }

    state.queue.write_buffer(
      &self.scene.meshes[1].vertex_buffer,
      0,
      bytemuck::cast_slice(self.func_mesh.vertices.as_slice()),
    );
  }
}

#[allow(unused)]
pub fn melting_graph_scene(state: &RenderState) -> MeltingScene {
  static SUBDIVISIONS: u16 = 200;
  static WIDTH: f32 = 2.0;

  let floor_mesh = graph::UnitSquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .mesh_data(graph::UnitSquareTesselation::FLOOR_COLOR);
  let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, -0.2_f32, -WIDTH / 2.0_f32]);

  // example function
  let mut f = |x: f32, z: f32| x.sin() * z.cos();
  let f = graph::shift_scale_input(f, 0.5, 8.0, 0.5, 8.0);
  let f = graph::shift_scale_output(f, 0.55, 0.5);

  let func_mesh = graph::UnitSquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .apply_function(f)
    .mesh_data(graph::UnitSquareTesselation::FUNCT_COLOR);

  let scene = build_scene(
    state,
    vec![(floor_mesh, matrix), (func_mesh.clone(), matrix)],
  );

  MeltingScene { scene, func_mesh }
}

// create a scene for simulating the wave equation

#[allow(unused)]
pub struct WaveEquationScene {
  pub scene: Scene,
  pub func_mesh: MeshData,
  pub wave_eqn: wave_eqn::WaveEquationData,
}

#[allow(unused)]
pub fn wave_eqn_scene(state: &RenderState) -> WaveEquationScene {
  // number of squares is 1 less than number of gridpoints
  // NOTE: we assume wave_eqn::X_SIZE == wave_eqn::Y_SIZE
  static SUBDIVISIONS: u16 = wave_eqn::X_SIZE as u16 - 1;
  static WIDTH: f32 = 2.0;

  let func_mesh = graph::UnitSquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .mesh_data(graph::UnitSquareTesselation::FUNCT_COLOR);
  let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, -0.2_f32, -WIDTH / 2.0_f32]);

  let scene = build_scene(state, vec![(func_mesh.clone(), matrix)]);
  let wave_eqn = wave_eqn::WaveEquationData::new();

  WaveEquationScene {
    scene,
    func_mesh,
    wave_eqn,
  }
}

impl RenderScene for WaveEquationScene {
  fn scene(&self) -> &Scene {
    &self.scene
  }

  fn update(&mut self, state: &RenderState) {
    // run next finite-difference timestep
    self.wave_eqn.update();

    static SCALE: f32 = 0.002;

    // update vertex data
    let n = wave_eqn::X_SIZE;
    for i in 0..n {
      for j in 0..n {
        self.func_mesh.vertices[j + i * n].position[1] = SCALE * self.wave_eqn.u_0[i][j];
      }
    }

    // update vertex buffer
    state.queue.write_buffer(
      &self.scene.meshes[0].vertex_buffer,
      0,
      bytemuck::cast_slice(self.func_mesh.vertices.as_slice()),
    );
  }
}
