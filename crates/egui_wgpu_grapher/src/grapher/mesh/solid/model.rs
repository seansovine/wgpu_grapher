// Code to build a scene from data imported from a glTF archive.

use super::{build_scene, MeshData, Vertex};
use crate::grapher::{
    matrix,
    mesh::{RenderScene, Scene},
    render::RenderState,
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};

#[allow(unused)]
const TEST_FILE_1: &str = "/home/sean/Code_projects/wgpu_grapher/scratch/gltf_sphere/scene.gltf";
#[allow(unused)]
const TEST_FILE_2: &str = "/home/sean/Code_projects/wgpu_grapher/scratch/gltf_head/scene.gltf";

const TEST_COLOR: [f32; 3] = [1.0, 0.0, 0.0];

pub fn load_solid_model() -> Vec<MeshData> {
    let (gltf, buffers, _) = gltf::import(TEST_FILE_2).unwrap();

    let mut meshes = vec![];

    for mesh in gltf.meshes() {
        let primitive = mesh.primitives().next().unwrap();

        // we assume the only primitive is a triangle
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let iter = reader
            .read_positions()
            .unwrap()
            .zip(reader.read_normals().unwrap());

        let mut vertices: Vec<Vertex> = vec![];

        for (position, normal) in iter {
            vertices.push(Vertex {
                position,
                color: TEST_COLOR,
                normal,
                ..Default::default()
            });
        }

        // add "first" mesh text coords if present
        if let Some(iter) = reader.read_tex_coords(0) {
            vertices
                .iter_mut()
                .zip(iter.into_f32())
                .for_each(|(vertex, tex_coords)| vertex.tex_coords = tex_coords);
        }

        let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

        meshes.push(MeshData { vertices, indices });

        // TODO: consider loading matrices as well
    }

    meshes
}

pub fn model_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> ModelScene {
    let mut mesh_data = vec![];
    let model_meshes = load_solid_model();

    // chosen to make model look good; need to allow
    // rotating model and/or light through ui to see
    let matrix = matrix::MatrixUniform::x_rotation(-135.0);

    for mesh in model_meshes {
        mesh_data.push((mesh, matrix));
    }

    let scene = build_scene(device, surface_config, state, mesh_data);

    ModelScene { scene }
}

pub struct ModelScene {
    scene: Scene,
}

impl RenderScene for ModelScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState, _pre_render: bool) {
        // no-op for now
    }
}
