// wave equation rendered into texture

use super::{build_scene, TexturedMeshData, SQUARE_INDICES, SQUARE_VERTICES_FLAT};

use crate::grapher::{
    math::pde,
    matrix::MatrixUniform,
    mesh::{RenderScene, Scene},
    pipeline::texture::{TextureData, TextureMatrix},
    render::RenderState,
};

use egui_wgpu::wgpu::{self, Device, Queue, SurfaceConfiguration};

pub fn wave_eqn_texture_scene(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> WaveEquationTextureScene {
    let texture_dims: (u32, u32) = (pde::X_SIZE as u32, pde::Y_SIZE as u32);

    let mut texture_matrix = TextureMatrix::new(texture_dims.0, texture_dims.1);

    for i in 0..texture_dims.0 {
        for j in 0..texture_dims.1 {
            let entry = texture_matrix.get(i, j);
            entry[0] = 0;
            entry[1] = 0;
            entry[2] = 0;
        }
    }

    let texture_data = TextureData::from_matrix(&texture_matrix, device, queue);

    let mesh_data = TexturedMeshData {
        vertices: Vec::from(SQUARE_VERTICES_FLAT),
        indices: Vec::from(SQUARE_INDICES),
        texture: texture_data,
    };

    let meshes = vec![(mesh_data, MatrixUniform::x_rotation(90.0))];

    let scene = build_scene(device, surface_config, state, meshes);
    let mut wave_eqn = pde::WaveEquationData::new(1000, 1000);

    // update solver properties
    wave_eqn.disturbance_prob = 0.01;
    wave_eqn.disturbance_size = 50.0;
    wave_eqn.damping_factor = 0.997;
    wave_eqn.prop_speed = 0.25;

    WaveEquationTextureScene {
        texture_matrix,
        scene,
        wave_eqn,
    }
}

pub struct WaveEquationTextureScene {
    texture_matrix: TextureMatrix,
    scene: Scene,
    pub wave_eqn: pde::WaveEquationData,
}

impl RenderScene for WaveEquationTextureScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, queue: &Queue, _state: &RenderState, _pre_render: bool) {
        // run next finite-difference timestep
        self.wave_eqn.update();

        let matrix = &mut self.texture_matrix;

        // update vertex data
        let n = matrix.dimensions.0;
        for i in 0..n {
            for j in 0..n {
                let new_val =
                    float_to_scaled_u8_color_pixel(self.wave_eqn.u_0[i as usize][j as usize]);
                let entry = matrix.get(i, j);

                entry[0] = new_val[0];
                entry[1] = new_val[1];
                entry[2] = new_val[2];
            }
        }

        let texture = &self.scene.textured_meshes[0].texture.texture;

        // write updated bytes into texture
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &matrix.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * matrix.dimensions.0),
                rows_per_image: Some(matrix.dimensions.1),
            },
            texture.size(),
        );
    }
}

#[inline(always)]
#[allow(unused)]
fn float_to_scaled_u8_grayscale_pixel(x: f32) -> [u8; 3] {
    const SCALE: f32 = 3.0;
    const SHIFT: f32 = 128.0;

    let value = (x * SCALE + SHIFT).clamp(0.0, 255.0) as u8;

    [value, value, value]
}

#[inline(always)]
#[allow(unused)]
fn float_to_scaled_u8_color_pixel(x: f32) -> [u8; 3] {
    const SCALE: f32 = 10.0;
    const SHIFT: f32 = 128.0;

    let value = (x * SCALE + SHIFT).clamp(0.0, 255.0) as u8;

    [0, value, 255 - value]
}
