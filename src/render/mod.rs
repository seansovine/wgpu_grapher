mod state;

// re-export state
pub use state::*;

use crate::mesh::Scene;

use wgpu::{BindGroup, BufferSlice, RenderPipeline, SurfaceError, TextureView};

pub fn render(state: &RenderState, scene: &Scene) -> Result<(), SurfaceError> {
  let output = state.surface.get_current_texture()?;
  let view = output
    .texture
    .create_view(&wgpu::TextureViewDescriptor::default());
  let camera_bind_group = &state.camera_state.matrix.bind_group;
  let light_bind_group = &state.light_state.bind_group;

  // want to clear depth buffer on first render only
  let mut depth_load_op = wgpu::LoadOp::Clear(1.0);

  // render solid meshes if configured
  if let Some(pipeline) = &scene.pipeline {
    for mesh in &scene.meshes {
      render_detail(
        state,
        &view,
        pipeline,
        mesh.vertex_buffer.slice(..),
        mesh.index_buffer.slice(..),
        mesh.num_indices,
        &[camera_bind_group, &mesh.matrix.bind_group, light_bind_group],
        depth_load_op,
      )?;
      depth_load_op = wgpu::LoadOp::Load;
    }
  }

  // render textured meshes if configured
  if let Some(pipeline) = &scene.textured_pipeline {
    for mesh in &scene.textured_meshes {
      render_detail(
        state,
        &view,
        pipeline,
        mesh.vertex_buffer.slice(..),
        mesh.index_buffer.slice(..),
        mesh.num_indices,
        &[
          camera_bind_group,
          &mesh.matrix.bind_group,
          &mesh.texture.bind_group,
        ],
        depth_load_op,
      )?;
      depth_load_op = wgpu::LoadOp::Load;
    }
  }

  output.present();

  Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_detail(
  state: &RenderState,
  view: &TextureView,
  pipeline: &RenderPipeline,
  vertex_buffer: BufferSlice,
  index_buffer: BufferSlice,
  num_indices: u32,
  bind_groups: &[&BindGroup],
  depth_load_op: wgpu::LoadOp<f32>,
) -> Result<(), SurfaceError> {
  let mut encoder = state
    .device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render encoder"),
    });

  let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("render pass"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
      },
    })],
    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
      view: &state.depth_buffer.view,
      depth_ops: Some(wgpu::Operations {
        load: depth_load_op,
        store: wgpu::StoreOp::Store,
      }),
      stencil_ops: None,
    }),
    occlusion_query_set: None,
    timestamp_writes: None,
  });

  render_pass.set_pipeline(pipeline);

  for (index, bind_group) in bind_groups.iter().enumerate() {
    render_pass.set_bind_group(index as u32, *bind_group, &[]);
  }

  render_pass.set_vertex_buffer(0, vertex_buffer);
  render_pass.set_index_buffer(index_buffer, wgpu::IndexFormat::Uint16);
  render_pass.draw_indexed(0..num_indices, 0, 0..1);

  // release borrow of encoder
  drop(render_pass);

  state.queue.submit(std::iter::once(encoder.finish()));

  Ok(())
}
