use crate::mesh;
use crate::render_state::RenderState;

pub fn render(state: &mut RenderState, scene: &mesh::Scene) -> Result<(), wgpu::SurfaceError> {
  let output = state.surface.get_current_texture()?;
  let view = output
    .texture
    .create_view(&wgpu::TextureViewDescriptor::default());

  for (mesh, matrix) in scene.meshes.iter().zip(scene.matrices.iter()) {
    let mut encoder = state
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render encoder"),
      });

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("render pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
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

    render_pass.set_pipeline(&scene.pipeline);

    render_pass.set_bind_group(0, &state.camera_state.matrix.bind_group, &[]);
    render_pass.set_bind_group(1, &matrix.bind_group, &[]);

    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);

    // release borrow of encoder
    drop(render_pass);

    state.queue.submit(std::iter::once(encoder.finish()));
  }

  output.present();

  Ok(())
}
