use crate::mesh::{TexturedVertex, Vertex};

use wgpu::{BindGroupLayout, Device, RenderPipeline, SurfaceConfiguration};

// vertex buffer layouts

pub(crate) trait Bufferable {
  fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

impl Bufferable for Vertex {
  fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
      ],
    }
  }
}

impl Bufferable for TexturedVertex {
  fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}

// create a render pipeline

pub fn get_shader() -> wgpu::ShaderSource<'static> {
  wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
}

pub fn get_textured_shader() -> wgpu::ShaderSource<'static> {
  wgpu::ShaderSource::Wgsl(include_str!("textured_shader.wgsl").into())
}

pub(crate) fn create_render_pipeline<T: Bufferable>(
  device: &Device,
  config: &SurfaceConfiguration,
  shader: wgpu::ShaderSource<'static>,
  bind_group_layouts: &[&BindGroupLayout],
  polygon_mode: wgpu::PolygonMode,
) -> RenderPipeline {
  let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("solid color shader"),
    source: shader,
  });

  let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("a solid color render pipeline layout"),
    bind_group_layouts,
    push_constant_ranges: &[],
  });

  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("a solid color render pipeline"),
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
    depth_stencil: None,
    multisample: wgpu::MultisampleState {
      count: 1,
      mask: !0,
      alpha_to_coverage_enabled: false,
    },
    multiview: None,
    cache: None,
  })
}
