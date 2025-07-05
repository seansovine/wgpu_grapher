// Code to build a scene that renders an image as a texture on a square canvas.

use super::{build_scene, TexturedMeshData, SQUARE_INDICES, SQUARE_VERTICES_VERTICAL};
use crate::grapher::{
    matrix::MatrixUniform,
    mesh::Scene,
    pipeline::texture::{Image, TextureData},
    render::RenderState,
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};

/// Render the scene onto both sides of a square canvas.
pub fn image_viewer_scene(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
    image_path: &str,
) -> Scene {
    let image = Image::from_file(image_path);

    let texture_data_front = TextureData::from_image(&image, device, queue);

    let mesh_data_front = TexturedMeshData {
        vertices: Vec::from(SQUARE_VERTICES_VERTICAL),
        indices: Vec::from(SQUARE_INDICES),
        texture: texture_data_front,
    };

    // second image behind first, to test depth buffer

    let texture_data_back = TextureData::from_image(&image, device, queue);

    let mesh_data_back = TexturedMeshData {
        vertices: Vec::from(SQUARE_VERTICES_VERTICAL),
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

    build_scene(device, surface_config, state, meshes)
}
