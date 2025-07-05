pub mod light;
pub mod render_preferences;

#[allow(dead_code)]
pub mod texture;

use super::mesh::Bufferable;
use texture::DepthBuffer;

use egui_wgpu::wgpu::{self, BindGroupLayout, Device, RenderPipeline, SurfaceConfiguration};

// include shaders and make accessor functions

pub fn get_shader() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into())
}

pub fn get_textured_shader() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shaders/textured_shader.wgsl").into())
}

// create a render pipeline

pub fn create_render_pipeline<T: Bufferable>(
    device: &Device,
    config: &SurfaceConfiguration,
    shader: wgpu::ShaderSource<'static>,
    bind_group_layouts: &[&BindGroupLayout],
    polygon_mode: wgpu::PolygonMode,
) -> RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("a shader"),
        source: shader,
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("a render pipeline layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("a render pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[T::buffer_layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DepthBuffer::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
