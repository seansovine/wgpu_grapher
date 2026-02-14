pub mod light;
pub mod render_preferences;

#[allow(dead_code)]
pub mod texture;

use super::scene::Bufferable;
use texture::DepthBuffer;

use egui_wgpu::wgpu::{
    self, BindGroupLayout, ComputePipeline, Device, PipelineLayoutDescriptor, RenderPipeline,
    ShaderSource, SurfaceConfiguration,
};

// -------------------------------
// Include shaders as static data.

pub fn get_shader() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into())
}

pub fn get_shadow_shader() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shaders/shadow_shader.wgsl").into())
}

pub fn get_textured_shader() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shaders/textured_shader.wgsl").into())
}

pub fn get_solver_shader() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shaders/solver_shader.wgsl").into())
}

pub fn get_solver_compute_shader() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shaders/solver.wgsl").into())
}

// -------------------------
// Create a render pipeline.

pub fn create_render_pipeline<Vertex: Bufferable>(
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
            buffers: &[Vertex::buffer_layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            count: 4,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}

// -----------------------------------
// Create pipeline for shadow mapping.

pub fn create_shadow_pipeline<Vertex: Bufferable>(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> RenderPipeline {
    let shader = get_shadow_shader();
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shadow map shader"),
        source: shader,
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow pipeline layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::buffer_layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: device
                .features()
                .contains(wgpu::Features::DEPTH_CLIP_CONTROL),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DepthBuffer::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            // parameters from wgpu official example
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}

// ---------------------------------------
// Create pipeline setup for 2D rendering.

pub fn create_2d_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    bind_group_layouts: &[&BindGroupLayout],
) -> RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("a shader"),
        source: get_solver_shader(),
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
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 4,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}

// --------------------------
// Create a compute pipeline.

pub fn create_compute_pipeline(
    device: &Device,
    shader_source: ShaderSource,
    bind_group_layouts: &[&BindGroupLayout],
) -> ComputePipeline {
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("A Compute Shader"),
        source: shader_source,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("run"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}
