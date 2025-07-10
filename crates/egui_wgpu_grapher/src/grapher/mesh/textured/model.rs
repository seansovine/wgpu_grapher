// Code to build a scene from data imported from a glTF archive.

use std::path::Path;

use super::{build_scene, TexturedMeshData, Vertex};
use crate::grapher::{
    matrix,
    mesh::{RenderScene, Scene},
    pipeline::texture::{Image, TextureData},
    render::RenderState,
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};
use gltf::{
    image::Source,
    mesh::{Mode, Primitive},
};

#[allow(unused)]
const TEST_FILE_1: &str = "/home/sean/Code_projects/wgpu_grapher/scratch/gltf_sphere/scene.gltf";
#[allow(unused)]
const TEST_FILE_2: &str = "/home/sean/Code_projects/wgpu_grapher/scratch/gltf_head/scene.gltf";
#[allow(unused)]
const TEST_FILE_3: &str = "/home/sean/Code_projects/wgpu_grapher/scratch/the_mimic/scene.gltf";
#[allow(unused)]
const TEST_FILE_4: &str = "/home/sean/Code_projects/wgpu_grapher/scratch/beast/scene.gltf";

const TEST_COLOR: [f32; 3] = [1.0, 0.0, 0.0];

pub fn load_model(device: &Device, queue: &Queue, file: &str) -> Vec<TexturedMeshData> {
    let (gltf, buffers, _) = gltf::import(file).unwrap();

    let mut meshes = vec![];

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            // We assume triangular meshes.
            if primitive.mode() != Mode::Triangles {
                continue;
            }

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

            let model_path = Path::new(file).parent().unwrap();
            let texture = read_texture(device, queue, &primitive, model_path);
            if let Some(texture) = texture {
                meshes.push(TexturedMeshData {
                    vertices,
                    indices,
                    texture,
                });
            } else {
                meshes.push(TexturedMeshData {
                    vertices,
                    indices,
                    texture: TextureData::solid_color_texture(&[255, 0, 0, 255], device, queue),
                });
            }

            // TODO: consider loading matrices as well
        }
    }

    meshes
}

fn read_texture(
    device: &Device,
    queue: &Queue,
    primitive: &Primitive<'_>,
    model_dir: &Path,
) -> Option<TextureData> {
    let pbr_metallic = primitive.material().pbr_metallic_roughness();

    if let Some(info) = pbr_metallic.base_color_texture() {
        let image_source = info.texture().source().source();
        match image_source {
            Source::Uri { uri, .. } => {
                let img_path = model_dir.join(Path::new(uri));
                let image = Image::from_file(img_path.to_str().unwrap());
                let texture = TextureData::from_image(&image, device, queue);

                return Some(texture);
            }
            Source::View { .. } => {}
        }
    }

    None
}

pub fn model_scene(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    state: &mut RenderState,
) -> ModelScene {
    let mut mesh_data = vec![];
    let model_meshes = load_model(device, queue, TEST_FILE_4);

    // Chosen for particular test examples; need to implement camera movement.
    let matrix = matrix::MatrixUniform::x_rotation(-135.0);

    for mesh in model_meshes {
        mesh_data.push((mesh, matrix));
    }

    // tell shader to use texture for color
    state.render_preferences.set_use_texture(true);
    state.render_preferences.update_uniform(queue);

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
