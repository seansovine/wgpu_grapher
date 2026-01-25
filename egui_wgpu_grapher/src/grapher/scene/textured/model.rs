//! Code to build a scene from data imported from a glTF archive.

use std::path::Path;

use super::{GpuVertex, TexturedMeshData, build_scene};
use crate::grapher::{
    matrix,
    pipeline::texture::TextureData,
    render::RenderState,
    scene::{
        RenderScene, Scene,
        textured::gltf_loader::{self, read_texture},
    },
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};
use gltf::mesh::Mode;

const DEFAULT_COLOR: [f32; 3] = [1.0, 0.0, 0.0];
const DEFAULT_COLOR_U8: [u8; 4] = [255, 0, 0, 255];

pub fn load_model(device: &Device, queue: &Queue, file: &str) -> Result<Vec<TexturedMeshData>, ()> {
    let Ok((gltf, buffers, _)) = gltf::import(file) else {
        return Err(());
    };

    let mut meshes = vec![];

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            // We assume triangular meshes.
            if primitive.mode() != Mode::Triangles {
                continue;
            }
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let iter = reader
                .read_positions()
                .unwrap()
                .zip(reader.read_normals().unwrap());
            let mut vertices: Vec<GpuVertex> = vec![];

            for (position, normal) in iter {
                vertices.push(GpuVertex {
                    position,
                    color: DEFAULT_COLOR,
                    normal,
                    ..Default::default()
                });
            }

            // First set of texture coords are for vertex color.
            if let Some(iter) = reader.read_tex_coords(0) {
                vertices
                    .iter_mut()
                    .zip(iter.into_f32())
                    .for_each(|(vertex, tex_coords)| vertex.tex_coords = tex_coords);
            }

            let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

            let model_path = Path::new(file).parent().unwrap();
            let texture = read_texture(device, queue, &primitive, model_path);
            if let Ok(texture) = texture {
                meshes.push(TexturedMeshData {
                    vertices,
                    indices,
                    texture,
                });
            } else {
                meshes.push(TexturedMeshData {
                    vertices,
                    indices,
                    // TODO: We should get the color from the material (it may be in KHR_ extension data).
                    texture: TextureData::solid_color_texture(&DEFAULT_COLOR_U8, device, queue),
                });
            }

            // This version doesn't load textures.
        }
    }

    Ok(meshes)
}

const USE_NEW_LOADER: bool = true;

pub fn model_scene(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    state: &mut RenderState,
    path: &str,
) -> Option<ModelScene> {
    let mut mesh_data = vec![];

    if USE_NEW_LOADER {
        let Ok(loader) = gltf_loader::GltfLoader::create(device, queue, path) else {
            return None;
        };
        let render_scene = loader.traverse();
        for render_mesh in render_scene.meshes {
            mesh_data.push((render_mesh.data, render_mesh.matrix));
        }
    } else {
        let Ok(model_meshes) = load_model(device, queue, path) else {
            return None;
        };
        // This version doesn't individual node/mesh matrices.
        let matrix = matrix::MatrixUniform::x_rotation(0.0);
        for mesh in model_meshes {
            mesh_data.push((mesh, matrix));
        }
    }

    // Tell shader to use texture for vertex color.
    // TODO: Did we implement this?
    state.render_preferences.set_use_texture(true);
    state.render_preferences.update_uniform(queue);

    let scene = build_scene(device, surface_config, state, mesh_data);

    Some(ModelScene { scene })
}

pub struct ModelScene {
    pub scene: Scene,
}

impl RenderScene for ModelScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState) {}
}
