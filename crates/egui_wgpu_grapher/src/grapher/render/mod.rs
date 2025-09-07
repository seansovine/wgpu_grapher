//! Top-level code for tracking render state and executing render passes.

mod state;
// re-export state
pub use state::*;

use super::scene::Scene;

use egui_wgpu::wgpu::{self, BindGroup, BufferSlice, CommandEncoder, RenderPipeline, TextureView};

impl RenderState {
    pub fn render(&self, view: &TextureView, encoder: &mut CommandEncoder, scene: &Scene) {
        if let Some(shadow_state) = &scene.shadow_state
            && scene.pipeline.is_some()
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &shadow_state.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&shadow_state.pipeline);
            pass.set_bind_group(0, &scene.light.camera_matrix_bind_group, &[]);

            // Shadows are currently drawn for solid scene objects only.
            for mesh in &scene.meshes {
                pass.set_bind_group(1, &mesh.bind_group, &[]);
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }

        // want to clear depth buffer on first render only
        let mut depth_load_op = wgpu::LoadOp::Clear(1.0);

        // render solid meshes if configured
        if let Some(pipeline) = &scene.pipeline {
            for mesh in &scene.meshes {
                render_detail(
                    encoder,
                    view,
                    &self.depth_buffer.view,
                    pipeline,
                    mesh.vertex_buffer.slice(..),
                    mesh.index_buffer.slice(..),
                    mesh.num_indices,
                    &[
                        &self.bind_group, //
                        &mesh.bind_group, //
                        &scene.light.bind_group,
                    ],
                    depth_load_op,
                );
                depth_load_op = wgpu::LoadOp::Load;
            }
        }

        // render textured meshes if configured
        if let Some(pipeline) = &scene.textured_pipeline {
            for mesh in &scene.textured_meshes {
                render_detail(
                    encoder,
                    view,
                    &self.depth_buffer.view,
                    pipeline,
                    mesh.vertex_buffer.slice(..),
                    mesh.index_buffer.slice(..),
                    mesh.num_indices,
                    &[
                        &self.bind_group,
                        &mesh.bind_group,
                        &scene.light.bind_group,
                        &mesh.texture.bind_group,
                    ],
                    depth_load_op,
                );
                depth_load_op = wgpu::LoadOp::Load;
            }
        }
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn render_detail(
    encoder: &mut CommandEncoder,
    view: &TextureView,
    depth_buffer_view: &TextureView,
    pipeline: &RenderPipeline,
    vertex_buffer: BufferSlice,
    index_buffer: BufferSlice,
    num_indices: u32,
    bind_groups: &[&BindGroup],
    depth_load_op: wgpu::LoadOp<f32>,
) {
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
            view: depth_buffer_view,
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
    render_pass.set_index_buffer(index_buffer, wgpu::IndexFormat::Uint32);
    render_pass.draw_indexed(0..num_indices, 0, 0..1);
}
