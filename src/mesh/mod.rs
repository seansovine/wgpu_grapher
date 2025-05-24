mod solid;
mod textured;

pub use solid::*;
pub use textured::*;

use wgpu::RenderPipeline;

pub struct Scene {
  pub meshes: Vec<MeshRenderData>,
  pub textured_meshes: Vec<TexturedMeshRenderData>,
  pub pipeline: RenderPipeline,
}
