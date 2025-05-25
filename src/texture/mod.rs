// We're just getting things down here for now; we can clean up later.

use crate::render::RenderState;

use image::{ImageBuffer, Rgba};

pub struct Image {
  pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
  pub dimensions: (u32, u32),
}

impl Image {
  pub fn from_file(filepath: &str) -> Self {
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
}

impl TextureData {
  pub fn from_image(image: &Image, state: &RenderState) -> Self {
    let texture = texture_from_image(image, state);
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
    }
  }
}

pub fn texture_from_image(image: &Image, state: &RenderState) -> wgpu::Texture {
  let texture_dimensions = wgpu::Extent3d {
    width: image.dimensions.0,
    height: image.dimensions.1,
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
    &image.image,
    wgpu::TexelCopyBufferLayout {
      offset: 0,
      bytes_per_row: Some(4 * image.dimensions.0),
      rows_per_image: Some(image.dimensions.1),
    },
    texture_dimensions,
  );

  texture
}
