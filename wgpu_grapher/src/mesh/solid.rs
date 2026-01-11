// Structures and functions for meshes with color provided per-vertex.

use super::{RenderScene, Scene};
use crate::{
    math::{
        graph::{self, SquareTesselation},
        pde,
    },
    matrix::{self, MatrixState, MatrixUniform},
    pipeline,
    render::RenderState,
};

use std::sync::LazyLock;

use wgpu::{Buffer, Device, Queue, SurfaceConfiguration, util::DeviceExt};

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

    // use this pipeline for all solid meshes
    let pipeline = pipeline::create_render_pipeline::<Vertex>(
        device,
        surface_config,
        pipeline::get_shader(),
        &[
            &state.camera_state.matrix.bind_group_layout,
            &last_mesh.matrix.bind_group_layout,
            &state.light_state.bind_group_layout,
            &state.render_preferences.bind_group_layout,
        ],
        state.render_preferences.polygon_mode,
    );

    Scene {
        meshes,
        textured_meshes: vec![],
        pipeline: Some(pipeline),
        textured_pipeline: None,
    }
}

// test data

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

// make scene for function graph

pub fn graph_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> Scene {
    static SUBDIVISIONS: u32 = 750;
    static WIDTH: f32 = 6.0;

    let floor_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH)
        .mesh_data(graph::SquareTesselation::FLOOR_COLOR);
    let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, 0.0f32, -WIDTH / 2.0_f32]);

    // example functions (uncomment one)

    // let f = |x: f32, z: f32| (x * x + z * z).sqrt().sin() / (x * x + z * z).sqrt();
    // let f = graph::shift_scale_input(f, 2.0, 40.0, 2.0, 40.0);
    // let f = graph::shift_scale_output(f, 0.25, 1.25);

    // let f = |x: f32, z: f32| x.powi(2) + z.powi(2);
    // let f = graph::shift_scale_input(f, WIDTH / 2.0_f32, SCALE, WIDTH / 2.0_f32, SCALE);
    // let f = graph::shift_scale_output(f, 0.001, 0.025);

    const SCALE: f32 = 2.0;

    let f = |x: f32, z: f32| 2.0_f32.powf(-(x.powi(2) + z.powi(2)).sin());
    let f = graph::shift_scale_input(f, WIDTH / 2.0_f32, SCALE, WIDTH / 2.0_f32, SCALE);
    let f = graph::shift_scale_output(f, 0.25, 0.5);

    let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH)
        .apply_function(f)
        .mesh_data(graph::SquareTesselation::FUNCT_COLOR);

    build_scene(
        device,
        surface_config,
        state,
        vec![(floor_mesh, matrix), (func_mesh, matrix)],
    )
}

// scene for simulating the wave equation

pub struct WaveEquationScene {
    pub scene: Scene,
    pub func_mesh: SquareTesselation,
    pub mesh_data: MeshData,
    pub wave_eqn: pde::WaveEquationData,
    pub display_scale: f32,
}

pub fn wave_eqn_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> WaveEquationScene {
    const WAVE_EQN_SUBDIV: usize = 600;
    // number of squares is 1 less than number of gridpoints
    const SUBDIVISIONS: u32 = WAVE_EQN_SUBDIV as u32 - 1;
    const WIDTH: f32 = 1.0;

    let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, WIDTH);
    let mesh_data = func_mesh.mesh_data(graph::SquareTesselation::FUNCT_COLOR);
    let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, 0.1_f32, -WIDTH / 2.0_f32]);

    let scene = build_scene(
        device,
        surface_config,
        state,
        vec![(mesh_data.clone(), matrix)],
    );
    let mut wave_eqn = pde::WaveEquationData::new(WAVE_EQN_SUBDIV, WAVE_EQN_SUBDIV);

    wave_eqn.disturbance_prob = 0.003;
    wave_eqn.disturbance_size = 2.0;
    wave_eqn.damping_factor = 0.998;
    wave_eqn.prop_speed = 0.15;

    let display_scale: f32 = 0.075;

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

    fn update(&mut self, queue: &Queue, state: &RenderState, pre_render: bool) {
        // run next finite-difference timestep
        self.wave_eqn.update();

        let n = self.wave_eqn.x_size;
        let b = 2_usize;

        // update vertex y-coordinates
        for i in b..n - b {
            for j in b..n - b {
                self.mesh_data.vertices[j + i * n].position[1] =
                    self.display_scale * self.wave_eqn.u_0[i][j];
            }
        }

        if pre_render {
            if state.render_preferences.lighting_enabled() {
                // update vertex normals
                self.func_mesh.update_normals(&mut self.mesh_data);
            }

            // update vertex buffer
            queue.write_buffer(
                &self.scene.meshes[0].vertex_buffer,
                0,
                bytemuck::cast_slice(self.mesh_data.vertices.as_slice()),
            );
        }
    }
}

// scene for simulating the heat equation

pub struct HeatEquationScene {
    pub scene: Scene,
    pub func_mesh: SquareTesselation,
    pub mesh_data: MeshData,
    pub heat_eqn: pde::HeatEquationData,
    pub display_scale: f32,

    // HACK: we don't update boundary each render,
    // but keep buffer area fixed to avoid flicker
    b: usize,
}

pub fn heat_eqn_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> HeatEquationScene {
    let b: usize = 5;

    static WAVE_EQN_SUBDIV: usize = 400;
    // number of squares is 1 less than number of gridpoints
    let subdivisions: u32 = WAVE_EQN_SUBDIV as u32 - 1 - (b as u32 * 2);
    const WIDTH: f32 = 1.0;

    let func_mesh = graph::SquareTesselation::generate(subdivisions, WIDTH);
    let mut mesh_data = func_mesh.mesh_data(graph::SquareTesselation::FUNCT_COLOR);

    func_mesh.update_normals(&mut mesh_data);

    let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, 0.1_f32, -WIDTH / 2.0_f32]);
    let scene = build_scene(
        device,
        surface_config,
        state,
        vec![(mesh_data.clone(), matrix)],
    );

    let heat_eqn = pde::HeatEquationData::new(WAVE_EQN_SUBDIV, WAVE_EQN_SUBDIV);
    let display_scale: f32 = 0.015;

    HeatEquationScene {
        scene,
        func_mesh,
        mesh_data,
        heat_eqn,
        display_scale,
        b,
    }
}

impl RenderScene for HeatEquationScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, queue: &Queue, state: &RenderState, pre_render: bool) {
        // run next finite-difference timestep
        self.heat_eqn.update();

        let n = self.heat_eqn.x_size;
        let m = n - self.b * 2;

        // update vertex y-coordinates and color
        for i in 0..m {
            for j in 0..m {
                let new_height = self.display_scale
                    * self.heat_eqn.u[(i + self.b) * n + (j + self.b)][self.heat_eqn.current_index];
                let new_color: [f32; 3] = [
                    255.0,
                    (255.0 * new_height.abs().clamp(0.0, 10.0) / 10.0),
                    0.0,
                ];

                self.mesh_data.vertices[j + i * m].position[1] = new_height;
                self.mesh_data.vertices[j + i * m].color = new_color
            }
        }

        if pre_render {
            if state.render_preferences.lighting_enabled() {
                // update vertex normals
                self.func_mesh.update_normals(&mut self.mesh_data);
            }

            // update vertex buffer
            queue.write_buffer(
                &self.scene.meshes[0].vertex_buffer,
                0,
                bytemuck::cast_slice(self.mesh_data.vertices.as_slice()),
            );
        }
    }
}
