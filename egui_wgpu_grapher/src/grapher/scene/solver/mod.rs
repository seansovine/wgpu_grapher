//! Render data for a two=dimensional scene.
//!
//! This will be used as a canvas to display a texture that
//! is updated by a time-dependent equation-solver pipeline.

use std::sync::OnceLock;

use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device, Extent3d,
    Origin3d, Queue, RenderPipeline, SurfaceConfiguration, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, util::DeviceExt,
};

use crate::grapher::pipeline::{
    create_compute_pipeline, create_solver_pipeline, get_solver_compute_shader,
};

// --------------------------
// Solver scene uniform data.

#[repr(C)]
#[derive(Default, Copy, Clone, Pod, Zeroable)]
pub struct UniformData {
    pub timestep: u32,
    aspect_ratio: f32,
}

#[allow(dead_code)]
pub struct Uniform {
    pub data: UniformData,
    buffer: Buffer,
    pub compute_bind_group: BindGroup,
    pub compute_bind_group_layout: BindGroupLayout,
    pub render_bind_group: BindGroup,
    pub render_bind_group_layout: BindGroupLayout,
}

impl Uniform {
    pub fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        let data = UniformData {
            aspect_ratio: surface_config.height as f32 / surface_config.width as f32,
            ..Default::default()
        };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Solver Compute Bind Group Layout"),
            });
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Solver Render Bind Group Layout"),
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        Self {
            data,
            buffer,
            compute_bind_group,
            compute_bind_group_layout,
            render_bind_group,
            render_bind_group_layout,
        }
    }
}

// -----------------------------
// Texture to hold compute data.

#[allow(dead_code)]
pub struct DataTexture {
    texture: Texture,
    pub compute_bind_group: BindGroup,
    pub compute_bind_group_layout: BindGroupLayout,
    pub render_bind_group: BindGroup,
    pub render_bind_group_layout: BindGroupLayout,
}

const TEXTURE_WIDTH: u32 = 1024;
const TEXTURE_HEIGHT: u32 = 1024;
const TEXTURE_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: TEXTURE_WIDTH,
    height: TEXTURE_HEIGHT,
    depth_or_array_layers: 1,
};

impl DataTexture {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Solver Data Texture"),
            size: TEXTURE_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Solver Texture View"),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });
        init_texture(queue, &texture, TEXTURE_SIZE);

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Wave Eqn Data Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Solver Data Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Solver Data Render Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Solver Data Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            compute_bind_group,
            compute_bind_group_layout,
            render_bind_group,
            render_bind_group_layout,
        }
    }
}

fn init_texture(queue: &Queue, texture: &Texture, texture_size: Extent3d) {
    static INIT_DATA: OnceLock<Vec<[f32; 4]>> = OnceLock::new();

    let init_data = INIT_DATA.get_or_init(|| {
        let mut buffer = vec![
            [64.0f32, 64.0f32, 64.0f32, 0.0f32];
            TEXTURE_HEIGHT as usize * TEXTURE_WIDTH as usize
        ];
        for i in TEXTURE_HEIGHT / 4..TEXTURE_HEIGHT * 3 / 4 {
            for j in TEXTURE_WIDTH / 4..TEXTURE_WIDTH * 3 / 4 {
                let coord = i as usize * TEXTURE_WIDTH as usize + j as usize;
                buffer[coord][0] = 192.0;
                buffer[coord][1] = 192.0;
                buffer[coord][2] = 192.0;
            }
        }
        buffer
    });
    queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(init_data),
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(TEXTURE_WIDTH * std::mem::size_of::<[f32; 4]>() as u32),
            rows_per_image: Some(TEXTURE_HEIGHT),
        },
        texture_size,
    );
}

// --------------------------
// Top-level scene structure.

pub struct SolverScene {
    pub compute_pipeline: ComputePipeline,
    pub render_pipeline: RenderPipeline,
    pub index_buffer: Buffer,
    pub uniform: Uniform,
    pub data_texture: DataTexture,
}

const CANVAS_QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

impl SolverScene {
    pub fn new(device: &Device, queue: &Queue, surface_config: &SurfaceConfiguration) -> Self {
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&CANVAS_QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let uniform = Uniform::new(device, surface_config);
        let data_texture = DataTexture::new(device, queue);

        let compute_pipeline = create_compute_pipeline(
            device,
            get_solver_compute_shader(),
            &[
                &data_texture.compute_bind_group_layout,
                &uniform.compute_bind_group_layout,
            ],
        );
        let render_pipeline = create_solver_pipeline(
            device,
            surface_config,
            &[
                &uniform.render_bind_group_layout,
                &data_texture.render_bind_group_layout,
            ],
        );

        Self {
            compute_pipeline,
            render_pipeline,
            index_buffer,
            uniform,
            data_texture,
        }
    }

    pub fn increment_timestep(&mut self, queue: &Queue) {
        self.uniform.data.timestep += 1;
        queue.write_buffer(
            &self.uniform.buffer,
            0,
            bytemuck::bytes_of(&self.uniform.data),
        );
    }

    pub fn update_aspect_ratio(&mut self, queue: &Queue, new_ratio: f32) {
        self.uniform.data.aspect_ratio = new_ratio;
        queue.write_buffer(
            &self.uniform.buffer,
            0,
            bytemuck::bytes_of(&self.uniform.data),
        );
    }

    pub fn solver_timestep(&self, encoder: &mut CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.data_texture.compute_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.uniform.compute_bind_group, &[]);

        let workgroup_count_x = TEXTURE_WIDTH.div_ceil(8);
        let workgroup_count_y = TEXTURE_HEIGHT.div_ceil(8);
        compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
    }
}
