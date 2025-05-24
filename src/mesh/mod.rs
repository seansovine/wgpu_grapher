mod solid;
pub mod textured;

pub use solid::*;

use wgpu::RenderPipeline;

pub struct Scene {
  pub meshes: Vec<MeshRenderData>,
  pub pipeline: RenderPipeline,
}
