mod state;
pub use state::*;

use crate::mesh::{self, MeshRenderData, TexturedMeshRenderData};

use wgpu::{RenderPipeline, TextureView};

pub fn render(state: &RenderState, scene: &mesh::Scene) -> Result<(), wgpu::SurfaceError> {
  let output = state.surface.get_current_texture()?;
  let view = output
    .texture
    .create_view(&wgpu::TextureViewDescriptor::default());

  if let Some(pipeline) = &scene.pipeline {
    for mesh in &scene.meshes {
      render_solid(state, &view, pipeline, mesh)?;
    }
  }
  if let Some(pipeline) = &scene.textured_pipeline {
    for mesh in &scene.textured_meshes {
      render_textured(state, &view, pipeline, mesh)?;
    }
  }

  output.present();

  Ok(())
}

pub fn render_solid(
  state: &RenderState,
  view: &TextureView,
  pipeline: &RenderPipeline,
  mesh: &MeshRenderData,
) -> Result<(), wgpu::SurfaceError> {
  let mut encoder = state
    .device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("solid render encoder"),
    });

  let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("solid render pass"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
      },
    })],
    depth_stencil_attachment: None,
    occlusion_query_set: None,
    timestamp_writes: None,
  });

  render_pass.set_pipeline(pipeline);

  render_pass.set_bind_group(0, &state.camera_state.matrix.bind_group, &[]);
  render_pass.set_bind_group(1, &mesh.matrix.bind_group, &[]);

  render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
  render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
  render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);

  // release borrow of encoder
  drop(render_pass);

  state.queue.submit(std::iter::once(encoder.finish()));

  Ok(())
}

// TODO: These can be merged.

pub fn render_textured(
  state: &RenderState,
  view: &TextureView,
  pipeline: &RenderPipeline,
  mesh: &TexturedMeshRenderData,
) -> Result<(), wgpu::SurfaceError> {
  let mut encoder = state
    .device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("textured render encoder"),
    });

  let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("textured render pass"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
      },
    })],
    depth_stencil_attachment: None,
    occlusion_query_set: None,
    timestamp_writes: None,
  });

  render_pass.set_pipeline(pipeline);

  render_pass.set_bind_group(0, &state.camera_state.matrix.bind_group, &[]);
  render_pass.set_bind_group(1, &mesh.matrix.bind_group, &[]);
  render_pass.set_bind_group(2, &mesh.texture.bind_group, &[]);

  render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
  render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
  render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);

  // release borrow of encoder
  drop(render_pass);

  state.queue.submit(std::iter::once(encoder.finish()));

  Ok(())
}
