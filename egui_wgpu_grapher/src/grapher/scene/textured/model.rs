//! Code to build a scene from data imported from a glTF archive.

use super::build_scene;
use crate::grapher::{
    gltf_loader::{self},
    render::RenderState,
    scene::{RenderScene, Scene3D},
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};

pub fn model_scene(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    state: &mut RenderState,
    path: &str,
) -> Option<ModelScene> {
    let Ok(loader) = gltf_loader::GltfLoader::create(device, queue, path) else {
        return None;
    };
    let mut mesh_data = vec![];
    match loader.traverse() {
        Ok(render_scene) => {
            for render_mesh in render_scene.meshes {
                mesh_data.push((render_mesh.data, render_mesh.matrix));
            }
        }
        Err(err) => {
            println!("Error while reading glTF scene: {err:?}");
            return None;
        }
    }

    // Tell shader to use texture for vertex color.
    state.render_preferences.set_use_texture(true);
    state.render_preferences.update_uniform(queue);

    Some(ModelScene {
        scene: build_scene(device, surface_config, state, mesh_data),
    })
}

pub struct ModelScene {
    pub scene: Scene3D,
}

impl RenderScene for ModelScene {
    fn scene(&self) -> &Scene3D {
        &self.scene
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState) {}
}
