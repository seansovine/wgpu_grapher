// Structures and functions for meshes with color provided per-vertex.

use super::{RenderScene, Scene};
use crate::grapher::{
    math::graph,
    matrix::{self, MatrixState, MatrixUniform},
    pipeline,
    render::RenderState,
};

use egui_wgpu::wgpu::{self, util::DeviceExt, Buffer, Device, Queue, SurfaceConfiguration};
use std::sync::LazyLock;

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

pub struct GraphParameters {
    pub scale_x: f32,
    pub scale_z: f32,
    pub scale_y: f32,

    pub shift_x: f32,
    pub shift_z: f32,
    pub shift_y: f32,
}

pub struct GraphScene {
    // all the data for rendering
    pub scene: Scene,
    pub width: f32,

    // TODO: generalize this and move it to RenderScene
    pub needs_update: bool,

    // publicly adjustable parameters
    pub parameters: GraphParameters,
}

fn build_scene_for_graph(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
    width: f32,
    f: impl Fn(f32, f32) -> f32,
) -> Scene {
    const SUBDIVISIONS: u32 = 750;

    let floor_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, width)
        .mesh_data(graph::SquareTesselation::FLOOR_COLOR);
    let matrix = MatrixUniform::translation(&[-width / 2.0_f32, 0.0f32, -width / 2.0_f32]);

    let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, width)
        .apply_function(f)
        .mesh_data(graph::SquareTesselation::FUNCT_COLOR);

    build_scene(
        device,
        surface_config,
        state,
        vec![(floor_mesh, matrix), (func_mesh, matrix)],
    )
}

// placeholder for now until we find a better solution
pub fn get_graph_func(parameters: &GraphParameters) -> impl Fn(f32, f32) -> f32 {
    // other example functions (uncomment one)

    let f = |x: f32, z: f32| (x * x + z * z).sqrt().sin() / (x * x + z * z).sqrt();

    // let f = |x: f32, z: f32| x.powi(2) + z.powi(2);

    // let f = |x: f32, z: f32| 2.0_f32.powf(-(x.powi(2) + z.powi(2)).sin());

    let f = graph::shift_scale_input(
        f,
        parameters.shift_x,
        parameters.scale_x,
        parameters.shift_z,
        parameters.scale_z,
    );
    graph::shift_scale_output(f, parameters.shift_y, parameters.scale_y)
}

pub fn graph_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> GraphScene {
    const WIDTH: f32 = 6.0;

    let parameters = GraphParameters {
        scale_x: 2.0,
        scale_z: 2.0,
        scale_y: 0.5,

        shift_x: WIDTH / 2.0,
        shift_z: WIDTH / 2.0,
        shift_y: 0.25,
    };

    let f = get_graph_func(&parameters);

    let scene = build_scene_for_graph(device, surface_config, state, WIDTH, f);

    let needs_update = false;

    GraphScene {
        scene,
        width: WIDTH,
        needs_update,
        parameters,
    }
}

impl GraphScene {
    // will be called when gui updates graph parameters, etc.
    pub fn rebuild_scene(
        &mut self,
        device: &Device,
        surface_config: &SurfaceConfiguration,
        state: &RenderState,
    ) {
        let f = get_graph_func(&self.parameters);
        self.scene = build_scene_for_graph(device, surface_config, state, self.width, f);
    }
}

impl RenderScene for GraphScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState, _pre_render: bool) {
        // no-op for now
    }
}
