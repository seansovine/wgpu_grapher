//! Finite-difference wave equation solver using Wgpu compute shader.
//!
//! We'll start as one linear block of code, then organize it more later.

use std::sync::{LazyLock, Mutex, OnceLock};

use image::{ImageBuffer, Luma};
use wgpu::{
    Buffer, Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
};

const TEXTURE_WIDTH: u32 = 1024;
const TEXTURE_HEIGHT: u32 = 1024;

fn main() -> Result<(), ()> {
    env_logger::init();

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Wgpu failed to create adapter.");
    log::info!("Wgpu is using adapter: {:?}", adapter);

    if !adapter
        .get_downlevel_capabilities()
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        log::error!("Adapter does not support compute pipelines.");
        return Err(());
    }

    let Ok((device, queue)) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
            | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
        required_limits: wgpu::Limits::downlevel_defaults(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    })) else {
        log::error!("Wgpu failed to create device.");
        return Err(());
    };

    // Create textures to hold solution data at three timesteps.

    let texture_size = wgpu::Extent3d {
        width: TEXTURE_WIDTH,
        height: TEXTURE_HEIGHT,
        // Layer for timesteps t, t-1, t-2.
        depth_or_array_layers: 1,
    };

    let texture_1 = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Wave Eqn Data Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texture_view_1 = texture_1.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Wave Eqn Data Texture Array View"),
        dimension: Some(wgpu::TextureViewDimension::D2),
        ..Default::default()
    });
    init_texture(&queue, &texture_1, texture_size);

    let texture_2 = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Wave Eqn Data Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texture_view_2 = texture_2.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Wave Eqn Data Texture Array View"),
        dimension: Some(wgpu::TextureViewDimension::D2),
        ..Default::default()
    });
    init_texture(&queue, &texture_2, texture_size);

    let texture_3 = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Wave Eqn Data Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texture_view_3 = texture_3.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Wave Eqn Data Texture Array View"),
        dimension: Some(wgpu::TextureViewDimension::D2),
        ..Default::default()
    });
    init_texture(&queue, &texture_3, texture_size);

    // Create compute pipeline.

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Wave Eqn Data Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::R32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::R32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::R32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Wave Eqn Data Compute Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view_1),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view_2),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&texture_view_3),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let module = device.create_shader_module(wgpu::include_wgsl!("shaders/solver.wgsl"));
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("advance"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // Staging buffer for copying from device to host.

    let staging_buffer_size =
        TEXTURE_HEIGHT as u64 * TEXTURE_HEIGHT as u64 * std::mem::size_of::<f32>() as u64;
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: staging_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // Save texture data to image to check initialization.

    save_texture_to_image(
        &device,
        &queue,
        &staging_buffer,
        &texture_1,
        texture_size,
        "scratch/init_data.jpg",
    );

    // Do a compute pass.

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: None,
        timestamp_writes: None,
    });
    compute_pass.set_pipeline(&pipeline);
    compute_pass.set_bind_group(0, &bind_group, &[]);

    let workgroup_count_x = TEXTURE_WIDTH.div_ceil(8);
    let workgroup_count_y = TEXTURE_HEIGHT.div_ceil(8);
    compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);

    // Release encoder borrow.
    drop(compute_pass);
    let command_buffer = encoder.finish();
    queue.submit([command_buffer]);

    // Let shader run to completion.
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    // Write another image from second texture to check shader.

    save_texture_to_image(
        &device,
        &queue,
        &staging_buffer,
        &texture_2,
        texture_size,
        "scratch/after_data.jpg",
    );

    log::info!("Compute pipeline ran successfully!");

    // TODO:
    //
    // This gives the basic framework to run a compute shader on the device
    // with textures bound for data storage. Now we need to do the following:
    //
    // 1. Add a uniform binding to the pipeline / shader to pass current timestep, etc.
    // 2. Add code in the shader to do the computations.
    // 3. Run for N timesteps and write image every M timesteps.

    Ok(())
}

static INIT_DATA: OnceLock<Vec<f32>> = OnceLock::new();

fn init_texture(queue: &Queue, texture: &Texture, texture_size: Extent3d) {
    let init_data = INIT_DATA.get_or_init(|| {
        let mut buffer = vec![64.0f32; TEXTURE_HEIGHT as usize * TEXTURE_WIDTH as usize];
        for i in TEXTURE_HEIGHT / 4..TEXTURE_HEIGHT * 3 / 4 {
            for j in TEXTURE_WIDTH / 4..TEXTURE_WIDTH * 3 / 4 {
                buffer[i as usize * TEXTURE_WIDTH as usize + j as usize] = 192.0;
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
            bytes_per_row: Some(TEXTURE_WIDTH * std::mem::size_of::<f32>() as u32),
            rows_per_image: Some(TEXTURE_HEIGHT),
        },
        texture_size,
    );
}

fn save_texture_to_image(
    device: &Device,
    queue: &Queue,
    staging_buffer: &Buffer,
    texture: &Texture,
    texture_size: Extent3d,
    filename: &str,
) {
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: staging_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(TEXTURE_WIDTH * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(TEXTURE_HEIGHT),
            },
        },
        texture_size,
    );

    // Block until transfer is done.
    queue.submit(std::iter::once(encoder.finish()));
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    static DONE: LazyLock<Mutex<bool>> = LazyLock::new(|| false.into());
    staging_buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, |_| {
            *(DONE.lock().unwrap()) = true;
        });

    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    // This check shouldn't be needed; when poll returns all work should be done.
    let mut done = DONE.lock().unwrap();
    assert!(*done);
    *done = false;

    let mapped_data = staging_buffer.slice(..).get_mapped_range();
    let data: &[f32] = bytemuck::cast_slice(&mapped_data);

    let image_buf = ImageBuffer::<Luma<u8>, Vec<u8>>::from_vec(
        TEXTURE_WIDTH,
        TEXTURE_HEIGHT,
        data.iter()
            .map(|val| val.clamp(0.0, 255.0) as u8)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    image_buf
        .save_with_format(filename, image::ImageFormat::Bmp)
        .unwrap();

    drop(mapped_data);
    staging_buffer.unmap();
}
