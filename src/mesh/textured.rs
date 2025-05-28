// Structures and functions for building textured mesh scenes.

use super::texture::{Image, TextureData, TextureMatrix};
use super::{RenderScene, Scene};
use crate::math::wave_eqn;
use crate::matrix::{self, MatrixState, MatrixUniform};
use crate::pipeline;
use crate::render::RenderState;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedVertex {
  pub position: [f32; 3],
  pub tex_coords: [f32; 2],
}

pub struct TexturedMeshData {
  pub vertices: Vec<TexturedVertex>,
  pub indices: Vec<u16>,
  pub texture: TextureData,
}

pub struct TexturedMeshRenderData {
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_indices: u32,
  pub matrix: MatrixState,
  pub texture: TextureData,
}

impl TexturedMeshRenderData {
  fn from_mesh_data(
    device: &wgpu::Device,
    mesh_data: TexturedMeshData,
    matrix_uniform: MatrixUniform,
  ) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("vertex buffer"),
      contents: bytemuck::cast_slice(mesh_data.vertices.as_slice()),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("index buffer"),
      contents: bytemuck::cast_slice(mesh_data.indices.as_slice()),
      usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = mesh_data.indices.len() as u32;

    let matrix = matrix::make_matrix_state(device, matrix_uniform);

    TexturedMeshRenderData {
      vertex_buffer,
      index_buffer,
      num_indices,
      matrix,
      texture: mesh_data.texture,
    }
  }
}

// build scene from (mesh, matrix) vector

pub fn build_scene(
  state: &RenderState,
  mesh_data: Vec<(TexturedMeshData, MatrixUniform)>,
) -> Scene {
  let mut textured_meshes = vec![];

  for (mesh, matrix) in mesh_data {
    let mesh_render_data = TexturedMeshRenderData::from_mesh_data(&state.device, mesh, matrix);
    textured_meshes.push(mesh_render_data);
  }

  // use the matrix and texture layouts from the last mesh
  let last_mesh = textured_meshes.last().unwrap();

  // we'll try to get away with just one textured render pipeline
  let pipeline = pipeline::create_render_pipeline::<TexturedVertex>(
    &state.device,
    &state.config,
    pipeline::get_textured_shader(),
    &[
      &state.camera_state.matrix.bind_group_layout,
      &last_mesh.matrix.bind_group_layout,
      &last_mesh.texture.bind_group_layout,
    ],
    wgpu::PolygonMode::Fill,
  );

  Scene {
    meshes: vec![],
    textured_meshes,
    pipeline: None,
    textured_pipeline: Some(pipeline),
  }
}

// test scene

const TEST_VERTICES_VERTICAL: &[TexturedVertex] = &[
  TexturedVertex {
    position: [-0.5, -0.5, 0.0],
    tex_coords: [0.0, 1.0],
  },
  TexturedVertex {
    position: [0.5, -0.5, 0.0],
    tex_coords: [1.0, 1.0],
  },
  TexturedVertex {
    position: [0.5, 0.5, 0.0],
    tex_coords: [1.0, 0.0],
  },
  TexturedVertex {
    position: [-0.5, 0.5, 0.0],
    tex_coords: [0.0, 0.0],
  },
];

const TEST_VERTICES_FLAT: &[TexturedVertex] = &[
  TexturedVertex {
    position: [-0.5, 0.0, 0.5],
    tex_coords: [0.0, 1.0],
  },
  TexturedVertex {
    position: [0.5, 0.0, 0.5],
    tex_coords: [1.0, 1.0],
  },
  TexturedVertex {
    position: [0.5, 0.0, -0.5],
    tex_coords: [1.0, 0.0],
  },
  TexturedVertex {
    position: [-0.5, 0.0, -0.5],
    tex_coords: [0.0, 0.0],
  },
];

#[rustfmt::skip]
const TEST_INDICES: &[u16] = &[
  // top
  0, 1, 2,
  0, 2, 3,
  // bottom
  0, 2, 1,
  0, 3, 2,
];

/// Render the scene onto both sides of a square canvas.
pub fn image_test_scene(state: &RenderState) -> Scene {
  let image = Image::from_file("assets/pexels-arjay-neyra-2152024526-32225792.jpg");
  let texture_data = TextureData::from_image(&image, state);

  let mesh_data = TexturedMeshData {
    vertices: Vec::from(TEST_VERTICES_VERTICAL),
    indices: Vec::from(TEST_INDICES),
    texture: texture_data,
  };

  let meshes: Vec<(TexturedMeshData, MatrixUniform)> =
    vec![(mesh_data, MatrixUniform::translation(&[0.0, 0.0, 0.0]))];

  build_scene(state, meshes)
}

// experiment in animated hand-designed texture

pub struct FadingCustomTextureScene {
  texture_matrix: TextureMatrix,
  scene: Scene,
  multiplier: f32,
  decreasing: bool,
}

impl RenderScene for FadingCustomTextureScene {
  fn scene(&self) -> &Scene {
    &self.scene
  }

  fn update(&mut self, state: &RenderState) {
    const DIM_FACTOR: f32 = 0.96;
    if self.decreasing {
      self.multiplier *= DIM_FACTOR;

      if self.multiplier < 0.01 {
        self.decreasing = false;
      }
    }

    const BRIGHT_FACTOR: f32 = 1.02;
    if !self.decreasing {
      self.multiplier *= BRIGHT_FACTOR;

      if self.multiplier > 0.9 {
        self.decreasing = true;
      }
    }

    let dims = self.texture_matrix.dimensions;
    let mut matrix = self.texture_matrix.clone();

    for i in 0..dims.0 {
      for j in 0..dims.1 {
        for r in 0..3 {
          matrix.get(i, j)[r] = ((matrix.get(i, j)[r] as f32) * self.multiplier) as u8;
        }
      }
    }

    let texture = &self.scene.textured_meshes[0].texture.texture;

    // write updated bytes into texture
    state.queue.write_texture(
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

/// Render the scene onto both sides of a square canvas.
pub fn custom_fading_texture_scene(state: &RenderState) -> FadingCustomTextureScene {
  const TEXTURE_DIMS: (u32, u32) = (500, 500);
  let mut texture_matrix = TextureMatrix::new(TEXTURE_DIMS.0, TEXTURE_DIMS.1);

  const GRAY_FACTOR: f32 = 0.7;
  let gray = |value: u8, factor: f32| ((value as f32) * factor) as u8;

  for i in 0..TEXTURE_DIMS.0 {
    for j in 0..TEXTURE_DIMS.1 {
      if i < TEXTURE_DIMS.0 / 2 && j < TEXTURE_DIMS.1 / 2 {
        let entry = texture_matrix.get(i, j);
        entry[0] = gray(entry[0], GRAY_FACTOR * GRAY_FACTOR);
        entry[1] = gray(entry[1], GRAY_FACTOR * GRAY_FACTOR);
        entry[2] = gray(entry[2], GRAY_FACTOR * GRAY_FACTOR);
      }
      if i >= TEXTURE_DIMS.0 / 2 && j >= TEXTURE_DIMS.1 / 2 {
        let entry = texture_matrix.get(i, j);
        entry[0] = gray(entry[0], GRAY_FACTOR);
        entry[1] = gray(entry[1], GRAY_FACTOR);
        entry[2] = gray(entry[2], GRAY_FACTOR);
      }
    }
  }

  let texture_data = TextureData::from_matrix(&texture_matrix, state);

  let mesh_data = TexturedMeshData {
    vertices: Vec::from(TEST_VERTICES_VERTICAL),
    indices: Vec::from(TEST_INDICES),
    texture: texture_data,
  };

  let meshes: Vec<(TexturedMeshData, MatrixUniform)> =
    vec![(mesh_data, MatrixUniform::translation(&[0.0, 0.0, 0.0]))];

  let scene = build_scene(state, meshes);

  FadingCustomTextureScene {
    texture_matrix,
    scene,
    multiplier: 1.0,
    decreasing: true,
  }
}

// wave equation rendered into texture

pub fn wave_eqn_texture_scene(state: &RenderState) -> WaveEquationTextureScene {
  let texture_dims: (u32, u32) = (wave_eqn::X_SIZE as u32, wave_eqn::Y_SIZE as u32);

  let mut texture_matrix = TextureMatrix::new(texture_dims.0, texture_dims.1);

  for i in 0..texture_dims.0 {
    for j in 0..texture_dims.1 {
      let entry = texture_matrix.get(i, j);
      entry[0] = 0;
      entry[1] = 0;
      entry[2] = 0;
    }
  }

  let texture_data = TextureData::from_matrix(&texture_matrix, state);

  let mesh_data = TexturedMeshData {
    vertices: Vec::from(TEST_VERTICES_FLAT),
    indices: Vec::from(TEST_INDICES),
    texture: texture_data,
  };

  let meshes: Vec<(TexturedMeshData, MatrixUniform)> =
    vec![(mesh_data, MatrixUniform::translation(&[0.0, 0.0, 0.0]))];

  let scene = build_scene(state, meshes);
  let wave_eqn = wave_eqn::WaveEquationData::new();

  WaveEquationTextureScene {
    texture_matrix,
    scene,
    wave_eqn,
  }
}

pub struct WaveEquationTextureScene {
  texture_matrix: TextureMatrix,
  scene: Scene,
  pub wave_eqn: wave_eqn::WaveEquationData,
}

impl RenderScene for WaveEquationTextureScene {
  fn scene(&self) -> &Scene {
    &self.scene
  }

  fn update(&mut self, state: &RenderState) {
    // run next finite-difference timestep
    self.wave_eqn.update();

    let matrix = &mut self.texture_matrix;

    // update vertex data
    let n = matrix.dimensions.0;
    for i in 0..n {
      for j in 0..n {
        let new_val = float_to_scaled_u8(self.wave_eqn.u_0[i as usize][j as usize]);
        let entry = matrix.get(i, j);
        entry[0] = new_val;
        entry[1] = new_val;
        entry[2] = new_val;
      }
    }

    let texture = &self.scene.textured_meshes[0].texture.texture;

    // write updated bytes into texture
    state.queue.write_texture(
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

fn float_to_scaled_u8(x: f32) -> u8 {
  const SCALE: f32 = 2.0;
  const SHIFT: f32 = 128.0;

  (x * SCALE + SHIFT).clamp(0.0, 255.0) as u8
}
