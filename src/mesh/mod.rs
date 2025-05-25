mod solid;
mod textured;

pub use solid::*;
pub use textured::*;

use crate::render::RenderState;

use wgpu::RenderPipeline;

pub struct Scene {
  pub meshes: Vec<MeshRenderData>,
  pub textured_meshes: Vec<TexturedMeshRenderData>,
  pub pipeline: Option<RenderPipeline>,
  pub textured_pipeline: Option<RenderPipeline>,
}

// trait to encapsulate scene behavior

pub trait RenderScene {
  /// get associated Scene reference
  fn scene(&self) -> &Scene;
  /// perform any timestep state updates
  fn update(&mut self, state: &RenderState);
}

impl RenderScene for Scene {
  fn scene(&self) -> &Scene {
    self
  }

  fn update(&mut self, _state: &RenderState) {}
}
