// Structures and functions for meshes with color provided per-vertex.

use crate::math::graph;
use crate::math::graph::SquareTesselation;
use crate::math::wave_eqn;
use crate::matrix;
use crate::matrix::{MatrixState, MatrixUniform};
use crate::pipeline;
use crate::render::RenderState;

use super::{RenderScene, Scene};

use std::sync::LazyLock;

use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub color: [f32; 3],
  pub normal: [f32; 3],
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

pub fn build_scene(state: &RenderState, mesh_data: Vec<(MeshData, MatrixUniform)>) -> Scene {
  let mut meshes = vec![];

  for (mesh, matrix) in mesh_data {
    let mesh_render_data = MeshRenderData::from_mesh_data(&state.device, mesh, matrix);
    meshes.push(mesh_render_data);
  }

  let last_mesh = meshes.last().unwrap();

  // only uses matrix layout, so only need one
  let pipeline = pipeline::create_render_pipeline::<Vertex>(
    &state.device,
    &state.config,
    pipeline::get_shader(),
    &[
      &state.camera_state.matrix.bind_group_layout,
      &last_mesh.matrix.bind_group_layout,
      &state.light_state.bind_group_layout,
    ],
    // TODO: add option to render as wireframe
    wgpu::PolygonMode::Fill,
  );

  Scene {
    meshes,
    textured_meshes: vec![],
    pipeline: Some(pipeline),
    textured_pipeline: None,
  }
}

// test data

#[allow(unused)]
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
  static SUBDIVISIONS: u32 = 750;
  static WIDTH: f32 = 6.0;

  let floor_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .mesh_data(graph::SquareTesselation::FLOOR_COLOR);
  let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, 0.0f32, -WIDTH / 2.0_f32]);

  // example function(s)

  // let f = |x: f32, z: f32| (x * x + z * z).sqrt().sin() / (x * x + z * z).sqrt();
  // let f = graph::shift_scale_input(f, 2.0, 40.0, 2.0, 40.0);
  // let f = graph::shift_scale_output(f, 0.25, 1.25);

  const SCALE: f32 = 2.0;

  let f = |x: f32, z: f32| 2.0_f32.powf(-(x.powi(2) + z.powi(2)).sin());
  let f = graph::shift_scale_input(f, WIDTH / 2.0_f32, SCALE, WIDTH / 2.0_f32, SCALE);
  let f = graph::shift_scale_output(f, 0.25, 0.5);

  // let f = |x: f32, z: f32| x.powi(2) + z.powi(2);
  // let f = graph::shift_scale_input(f, WIDTH / 2.0_f32, SCALE, WIDTH / 2.0_f32, SCALE);
  // let f = graph::shift_scale_output(f, 0.001, 0.025);

  let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .apply_function(f)
    .mesh_data(graph::SquareTesselation::FUNCT_COLOR);

  // omitting:
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
  static SUBDIVISIONS: u32 = 200;
  static WIDTH: f32 = 2.0;

  let floor_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .mesh_data(graph::SquareTesselation::FLOOR_COLOR);
  let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, -0.2_f32, -WIDTH / 2.0_f32]);

  // example function
  let mut f = |x: f32, z: f32| x.sin() * z.cos();
  let f = graph::shift_scale_input(f, 0.5, 8.0, 0.5, 8.0);
  let f = graph::shift_scale_output(f, 0.55, 0.5);

  let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH)
    .apply_function(f)
    .mesh_data(graph::SquareTesselation::FUNCT_COLOR);

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
  pub func_mesh: SquareTesselation,
  pub mesh_data: MeshData,
  pub wave_eqn: wave_eqn::WaveEquationData,
  pub display_scale: f32,
}

#[allow(unused)]
pub fn wave_eqn_scene(state: &RenderState) -> WaveEquationScene {
  static WAVE_EQN_SUBDIV: usize = 500;
  // number of squares is 1 less than number of gridpoints
  // NOTE: we assume wave_eqn::X_SIZE == wave_eqn::Y_SIZE
  static SUBDIVISIONS: u32 = WAVE_EQN_SUBDIV as u32 - 1;
  static WIDTH: f32 = 1.0;

  let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH);
  let mesh_data = func_mesh.mesh_data(graph::SquareTesselation::FUNCT_COLOR);
  let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, 0.1_f32, -WIDTH / 2.0_f32]);

  let scene = build_scene(state, vec![(mesh_data.clone(), matrix)]);
  let mut wave_eqn = wave_eqn::WaveEquationData::new(WAVE_EQN_SUBDIV, WAVE_EQN_SUBDIV);

  wave_eqn.disturbance_prob = 0.001;
  wave_eqn.disturbance_size = 50.0;
  wave_eqn.damping_factor = 0.9965;
  wave_eqn.prop_speed = 0.25;

  let display_scale: f32 = 0.01;

  WaveEquationScene {
    scene,
    func_mesh,
    mesh_data,
    wave_eqn,
    display_scale,
  }
}

impl RenderScene for WaveEquationScene {
  fn scene(&self) -> &Scene {
    &self.scene
  }

  fn update(&mut self, state: &RenderState) {
    // run next finite-difference timestep
    self.wave_eqn.update();

    // update vertex y-coordinates
    let n = self.wave_eqn.x_size;
    for i in 0..n {
      for j in 0..n {
        self.mesh_data.vertices[j + i * n].position[1] =
          self.display_scale * self.wave_eqn.u_0[i][j];
      }
    }

    // now update vertex normals
    self.func_mesh.update_normals(&mut self.mesh_data);

    // update vertex buffer
    state.queue.write_buffer(
      &self.scene.meshes[0].vertex_buffer,
      0,
      bytemuck::cast_slice(self.mesh_data.vertices.as_slice()),
    );
  }
}
