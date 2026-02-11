//! Finite-difference wave equation solver using Wgpu compute shader.
//!
//! We'll start as one linear block of code, then organize it more later.

use std::sync::{LazyLock, Mutex, OnceLock};

use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, Luma};
use wgpu::{
    BindGroup, Buffer, ComputePipeline, Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, util::DeviceExt,
};

const TEXTURE_WIDTH: u32 = 1024;
const TEXTURE_HEIGHT: u32 = 1024;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
struct Uniform {
    timestep: u32,
}

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
            | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
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

    let data_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Wave Eqn Data Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let texture_view = data_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Wave Eqn Data Texture Array View"),
        dimension: Some(wgpu::TextureViewDimension::D2),
        ..Default::default()
    });
    init_texture(&queue, &data_texture, texture_size);

    // Create uniform buffer and bind group.

    let mut uniform = Uniform::default();
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout =
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
            label: Some("Uniform Bind Group Layout"),
        });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("Uniform Bind Group"),
    });

    // Create compute pipeline.

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Wave Eqn Data Compute Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
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
        TEXTURE_HEIGHT as u64 * TEXTURE_HEIGHT as u64 * std::mem::size_of::<[f32; 4]>() as u64;
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
        &data_texture,
        0,
        texture_size,
        &format!("scratch/data_at_timestep_{:04}.jpg", uniform.timestep),
    );

    // Do a couple compute passes.

    const NUM_STEPS: usize = 4000;
    const IMAGE_STEPS: usize = 50;

    for i in 0..NUM_STEPS {
        compute_pass(
            &device,
            &queue,
            &pipeline,
            &[&bind_group, &uniform_bind_group],
        );

        // Update uniform timestep value.
        uniform.timestep += 1;
        queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        if i.is_multiple_of(IMAGE_STEPS) {
            println!("Writing image for timestep {}...", uniform.timestep);
            save_texture_to_image(
                &device,
                &queue,
                &staging_buffer,
                &data_texture,
                1,
                texture_size,
                &format!("scratch/data_at_timestep_{:04}.jpg", uniform.timestep),
            );
        }
    }

    // Block until all pipeline commands have run.
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    log::info!("Compute pipeline ran successfully!");
    Ok(())
}

fn compute_pass(
    device: &Device,
    queue: &Queue,
    pipeline: &ComputePipeline,
    bind_groups: &[&BindGroup],
) {
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: None,
        timestamp_writes: None,
    });
    compute_pass.set_pipeline(pipeline);

    for (i, bg) in bind_groups.iter().enumerate() {
        compute_pass.set_bind_group(i as u32, *bg, &[]);
    }

    let workgroup_count_x = TEXTURE_WIDTH.div_ceil(8);
    let workgroup_count_y = TEXTURE_HEIGHT.div_ceil(8);
    compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);

    // Release encoder borrow.
    drop(compute_pass);
    queue.submit([encoder.finish()]);
}

static INIT_DATA: OnceLock<Vec<[f32; 4]>> = OnceLock::new();

fn init_texture(queue: &Queue, texture: &Texture, texture_size: Extent3d) {
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

fn save_texture_to_image(
    device: &Device,
    queue: &Queue,
    staging_buffer: &Buffer,
    texture: &Texture,
    component: usize,
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
                bytes_per_row: Some(TEXTURE_WIDTH * std::mem::size_of::<[f32; 4]>() as u32),
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
    let data: &[[f32; 4]] = bytemuck::cast_slice(&mapped_data);
    let selected_data: Vec<u8> = data
        .iter()
        .map(|v| v[component].clamp(0.0, 255.0) as u8)
        .collect();

    let image_buf =
        ImageBuffer::<Luma<u8>, Vec<u8>>::from_vec(TEXTURE_WIDTH, TEXTURE_HEIGHT, selected_data)
            .unwrap();
    image_buf
        .save_with_format(filename, image::ImageFormat::Bmp)
        .unwrap();

    drop(mapped_data);
    staging_buffer.unmap();
}
