use std::sync::OnceLock;

use cgmath::Matrix4;
use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, Device, Extent3d, Queue,
    RenderPipeline, Sampler, SurfaceConfiguration, TextureDescriptor, TextureDimension,
    TextureUsages, TextureView, util::DeviceExt,
};

use crate::grapher::{
    camera,
    matrix::{self, Matrix, MatrixUniform},
    pipeline::create_shadow_pipeline,
    scene::Bufferable,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding_1: u32,
    color: [f32; 3],
    _padding_2: u32,
}

impl LightUniform {
    pub fn bind_group_layout_entry() -> &'static BindGroupLayoutEntry {
        static BGL_ENTRY: OnceLock<BindGroupLayoutEntry> = OnceLock::new();
        BGL_ENTRY.get_or_init(|| BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
    }
}

pub struct LightState {
    pub uniform: LightUniform,
    pub buffer: Buffer,
    pub bind_group: BindGroup,

    // Light view matrix used for shadow mapping.
    pub light_view: MatrixUniform,
    pub light_view_bind_group_layout: BindGroupLayout,
    pub light_view_bind_group: BindGroup,
}

impl LightState {
    const DEFAULT_LIGHT_POS: [f32; 3] = [3.0, 4.0, 0.0];

    pub fn light_bgl(device: &Device) -> &'static BindGroupLayout {
        static BGL: OnceLock<BindGroupLayout> = OnceLock::new();
        BGL.get_or_init(|| {
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[*LightUniform::bind_group_layout_entry()],
                label: Some("light bind group layout"),
            })
        })
    }

    pub fn create(device: &Device) -> Self {
        let uniform = LightUniform {
            position: Self::DEFAULT_LIGHT_POS,
            _padding_1: 0_u32,
            color: [1.0, 1.0, 1.0],
            _padding_2: 0_u32,
        };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light UBO"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: Self::light_bgl(device),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("light bind group"),
        });

        // Create view matrix for use in shadow mapping.
        let matrix = Self::build_shadow_matrix(&uniform.position);
        let matrix_uniform = Matrix::from(matrix);
        let light_view = matrix::make_matrix_uniform(device, matrix_uniform);

        let light_view_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[*MatrixUniform::bind_group_layout_entry()],
                label: Some("solid mesh matrix bind group layout"),
            });
        let light_view_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &light_view_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: light_view.buffer.as_entire_binding(),
            }],
            label: Some("solid mesh matrix bind group"),
        });

        Self {
            uniform,
            buffer,
            bind_group,
            //
            light_view,
            light_view_bind_group_layout,
            light_view_bind_group,
        }
    }

    fn build_shadow_matrix(position: &[f32; 3]) -> Matrix4<f32> {
        let view_target = cgmath::Point3::<f32>::from([0.0, 0.0, 0.0]);
        let view_origin = cgmath::Point3::<f32>::from(*position);
        let view_up = if position[0] == 0.0 && position[2] == 0.0 {
            cgmath::Vector3::<f32>::from([1.0, 0.0, 0.0])
        } else {
            cgmath::Vector3::<f32>::from([0.0, 1.0, 0.0])
        };
        let view = cgmath::Matrix4::look_at_rh(view_origin, view_target, view_up);

        let projection = cgmath::ortho(-1.5_f32, 1.5_f32, -1.5_f32, 1.5_f32, -1.0, 1.0);

        camera::OPENGL_TO_WGPU_MATRIX * projection * view
    }

    pub fn set_position(&mut self, new_position: [f32; 3]) {
        self.uniform.position = new_position;
        self.light_view
            .matrix
            .update_value(Self::build_shadow_matrix(&new_position));
    }

    pub fn position(&self) -> [f32; 3] {
        self.uniform.position
    }

    pub fn update_uniform(&mut self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        self.light_view.write_buffer(queue);
    }

    pub fn light_view_matrix(&self) -> &MatrixUniform {
        &self.light_view
    }
}

// State for shadow map.

pub struct ShadowState {
    pub shadow_pass_pipeline: RenderPipeline,

    pub _texture: wgpu::Texture,
    pub view: TextureView,
    pub _sampler: Sampler,

    pub render_pass_bind_group_layout: BindGroupLayout,
    pub render_pass_bind_group: BindGroup,
}

impl ShadowState {
    const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create<Vertex: Bufferable>(
        _surface_config: &SurfaceConfiguration,
        device: &Device,
        light: &LightState,
        model_matrix_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let pipeline = create_shadow_pipeline::<Vertex>(
            device,
            &[
                &light.light_view_bind_group_layout,
                model_matrix_bind_group_layout,
            ],
        );

        let max_tex_size = device.limits().max_texture_dimension_2d;
        let _texture = device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: max_tex_size,
                height: max_tex_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::SHADOW_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        });
        let view = _texture.create_view(&wgpu::TextureViewDescriptor::default());

        let _sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToBorder,
            address_mode_v: wgpu::AddressMode::ClampToBorder,
            address_mode_w: wgpu::AddressMode::ClampToBorder,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            border_color: Some(wgpu::SamplerBorderColor::OpaqueBlack),
            ..Default::default()
        });

        let mut camera_view_bgl_entry = *MatrixUniform::bind_group_layout_entry();
        camera_view_bgl_entry.binding = 2;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                camera_view_bgl_entry,
            ],
            label: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light.light_view_matrix().buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        Self {
            shadow_pass_pipeline: pipeline,
            _texture,
            view,
            _sampler,
            render_pass_bind_group_layout: bind_group_layout,
            render_pass_bind_group: bind_group,
        }
    }
}
