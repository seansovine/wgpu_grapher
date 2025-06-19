use crate::render::RenderState;

use image::{ImageBuffer, Rgba};
use wgpu::{Device, SurfaceConfiguration, Texture, TextureView};

pub struct Image {
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub dimensions: (u32, u32),
}

impl Image {
    pub fn from_file(filepath: &str) -> Self {
        // TODO: pass error up call stack vs panicking
        let image_bytes = std::fs::read(filepath)
            .unwrap_or_else(|_| panic!("Unable to read image at path: {}", filepath));
        let image = image::load_from_memory(&image_bytes).unwrap().to_rgba8();
        let dimensions = image.dimensions();

        Self { image, dimensions }
    }
}

pub struct TextureData {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub texture: wgpu::Texture,
}

impl TextureData {
    pub fn from_texture(texture: Texture, state: &RenderState) -> Self {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("texture bind group layout"),
                });

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
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
            label: Some("texture bind group"),
        });

        Self {
            bind_group_layout,
            bind_group,
            texture,
        }
    }

    pub fn from_image(image: &Image, state: &RenderState) -> Self {
        let texture = texture_from_image(image, state);
        TextureData::from_texture(texture, state)
    }

    pub fn from_matrix(matrix: &TextureMatrix, state: &RenderState) -> Self {
        let texture = texture_from_matrix(matrix, state);
        TextureData::from_texture(texture, state)
    }
}

pub fn texture_from_data_and_dims(
    data: &[u8],
    dims: (u32, u32),
    state: &RenderState,
) -> wgpu::Texture {
    let texture_dimensions = wgpu::Extent3d {
        width: dims.0,
        height: dims.1,
        depth_or_array_layers: 1,
    };

    let texture = state.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_dimensions,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("image texture"),
        view_formats: &[],
    });

    // write image bytes into texture
    state.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dims.0),
            rows_per_image: Some(dims.1),
        },
        texture_dimensions,
    );

    texture
}

pub fn texture_from_image(image: &Image, state: &RenderState) -> wgpu::Texture {
    texture_from_data_and_dims(&image.image, image.dimensions, state)
}

// texture matrix

/// Represents texture data as a matrix of RGBA bytes.
#[derive(Clone)]
pub struct TextureMatrix {
    pub dimensions: (u32, u32),
    pub data: Vec<u8>,
}

impl TextureMatrix {
    pub fn new(x_dim: u32, y_dim: u32) -> Self {
        let data_len = (x_dim * y_dim * 4) as usize;
        let data = vec![255_u8; data_len];

        Self {
            dimensions: (x_dim, y_dim),
            data,
        }
    }

    pub fn get(&mut self, x: u32, y: u32) -> &mut [u8] {
        let index = (y * 4 * self.dimensions.0 + 4 * x) as usize;
        &mut self.data[index..index + 4]
    }
}

pub fn texture_from_matrix(matrix: &TextureMatrix, state: &RenderState) -> wgpu::Texture {
    texture_from_data_and_dims(&matrix.data, matrix.dimensions, state)
}

// depth buffer

pub struct DepthBuffer {
    pub _texture: Texture,
    pub view: TextureView,
}

impl DepthBuffer {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create(config: &SurfaceConfiguration, device: &Device) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("depth buffer"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let _texture = device.create_texture(&desc);

        let view = _texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { _texture, view }
    }
}
