//! Build a scene that renders an image as a texture on a rectangular canvas.

use super::{SQUARE_INDICES, SQUARE_VERTICES_VERTICAL, TexturedMeshData, build_scene};
use crate::grapher::{
    camera::ProjectionType,
    matrix::Matrix,
    pipeline::texture::{Image, TextureData},
    render::RenderState,
    scene::{RenderScene, Scene3D},
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};

/// Render the scene onto both sides of a square canvas.
/// Updates camera state to one suited for viewing 2D image.
pub fn image_viewer_scene(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    state: &mut RenderState,
    image_path: &str,
) -> Option<ImageViewerScene> {
    let Ok(image) = Image::from_file(image_path) else {
        return None;
    };

    // tell shader to use texture for color
    state.render_preferences.set_use_texture(true);
    state.render_preferences.update_uniform(queue);

    // update camera settings
    state.camera_state.set_from_z(52.0);
    state.camera_state.camera.projection_type = ProjectionType::Orthographic;
    state.camera_state.update_uniform(queue);

    // create textured canvas
    let texture_data_front = TextureData::from_image(&image, device, queue);

    let mut mesh_data_front = TexturedMeshData {
        vertices: SQUARE_VERTICES_VERTICAL.clone(),
        indices: Vec::from(SQUARE_INDICES),
        texture: texture_data_front,
    };
    update_canvas_aspect_ratio(&mut mesh_data_front, image.dimensions.1, image.dimensions.0);

    let meshes: Vec<(TexturedMeshData, Matrix)> =
        vec![(mesh_data_front, Matrix::translation(&[0.0, 0.0, 0.5]))];

    let mut image_scene = ImageViewerScene {
        scene: build_scene(device, surface_config, state, meshes),
    };
    // update light position
    image_scene.scene.light.set_position([0.0, 0.0, 3.0]);
    image_scene.scene.light.update_uniform(queue);

    Some(image_scene)
}

fn update_canvas_aspect_ratio(mesh_data: &mut TexturedMeshData, height: u32, width: u32) {
    if width < height {
        let mult = width as f32 / height as f32;
        for vertex in mesh_data.vertices.iter_mut() {
            vertex.position[0] *= mult;
        }
    } else if width > height {
        let mult = height as f32 / width as f32;
        for vertex in mesh_data.vertices.iter_mut() {
            vertex.position[1] *= mult;
        }
    }
}

pub struct ImageViewerScene {
    pub scene: Scene3D,
}

impl RenderScene for ImageViewerScene {
    fn scene(&self) -> &Scene3D {
        &self.scene
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState) {}
}
