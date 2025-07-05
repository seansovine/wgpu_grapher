// Code to build a scene that renders an image as a texture on a square canvas.

use super::{build_scene, TexturedMeshData, SQUARE_INDICES, SQUARE_VERTICES_VERTICAL};
use crate::grapher::{
    matrix::MatrixUniform,
    mesh::{RenderScene, Scene},
    pipeline::texture::{Image, TextureData},
    render::RenderState,
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};

/// Render the scene onto both sides of a square canvas.
pub fn image_viewer_scene(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    state: &mut RenderState,
    image_path: &str,
) -> ImageViewerScene {
    let image = Image::from_file(image_path);

    // update light position
    state.light_state.set_position([0.0, 0.0, 3.0]);
    state.light_state.update_uniform(queue);

    // tell shader to use texture for color
    state.render_preferences.set_use_texture(true);
    state.render_preferences.update_uniform(queue);

    // update camera position and distance for this
    state.camera_state.set_from_z(1.5);
    state.camera_state.update_uniform(queue);
    // TODO: add orthographic projection camera matrix for this scene

    // main image being displayed

    let texture_data_front = TextureData::from_image(&image, device, queue);

    let mesh_data_front = TexturedMeshData {
        vertices: SQUARE_VERTICES_VERTICAL.clone(),
        indices: Vec::from(SQUARE_INDICES),
        texture: texture_data_front,
    };

    // second image behind first, to test depth buffer

    let texture_data_back = TextureData::from_image(&image, device, queue);

    let mesh_data_back = TexturedMeshData {
        vertices: SQUARE_VERTICES_VERTICAL.clone(),
        indices: Vec::from(SQUARE_INDICES),
        texture: texture_data_back,
    };

    let meshes: Vec<(TexturedMeshData, MatrixUniform)> = vec![
        (
            mesh_data_front,
            MatrixUniform::translation(&[0.0, 0.0, 0.5]),
        ),
        (
            mesh_data_back,
            MatrixUniform::translation(&[0.0, 0.0, -0.5]),
        ),
    ];

    ImageViewerScene {
        scene: build_scene(device, surface_config, state, meshes),
    }
}

pub struct ImageViewerScene {
    scene: Scene,
}

impl RenderScene for ImageViewerScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState, _pre_render: bool) {
        // no-op for now
    }
}
