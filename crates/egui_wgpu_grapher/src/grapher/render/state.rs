use crate::grapher::{
    camera::CameraState,
    pipeline::{
        self, light::LightState, render_preferences::RenderPreferences, texture::DepthBuffer,
    },
    scene::{Bufferable, solid::MeshRenderData},
};

use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Device, Extent3d, Queue, RenderPipeline, Sampler,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
};
use winit::event::WindowEvent;

// State for global rendering environment.

pub struct RenderState {
    // camera
    pub camera_state: CameraState,
    // shader preferences
    pub render_preferences: RenderPreferences,
    // bind group for things global to the renderer
    pub bind_group_layout: BindGroupLayout,
    // includes camera and render preferences
    pub bind_group: BindGroup,
    // depth buffer
    pub depth_buffer: DepthBuffer,
    // running framerate
    pub framerate: f32,
    // multisampling texture
    pub msaa_data: MultisampleData,
}

impl RenderState {
    pub async fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        let camera_state = CameraState::init(device, surface_config);
        let mut shader_preferences = RenderPreferences::create(device);
        shader_preferences.set_binding_index(1);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                camera_state.matrix.bind_group_layout_entry,
                shader_preferences.bind_group_layout_entry,
            ],
            label: Some("shared resources bind group layout"),
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_state.matrix.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: shader_preferences.buffer.as_entire_binding(),
                },
            ],
            label: Some("shared resources bind group"),
        });

        let depth_buffer = DepthBuffer::create(surface_config, device);
        let msaa_texture = MultisampleData::create(surface_config, device);

        Self {
            camera_state,
            render_preferences: shader_preferences,
            bind_group_layout,
            bind_group,
            depth_buffer,
            // we target 60fps
            framerate: 60_f32,
            msaa_data: msaa_texture,
        }
    }
}

impl RenderState {
    pub fn handle_user_input(&mut self, event: &WindowEvent) -> bool {
        self.camera_state.controller.process_events(event)
    }

    pub fn update(&mut self, queue: &mut Queue) {
        // adjust controller speed based on framerate
        self.camera_state.controller.speed = 2.125 / self.framerate;

        self.camera_state
            .controller
            .update_camera(&mut self.camera_state.camera);
        self.camera_state
            .matrix
            .uniform
            .update(self.camera_state.camera.get_matrix());

        // update camera matrix uniform
        self.camera_state.update_uniform(queue);
    }
}

// State for MSAA.

pub struct MultisampleData {
    pub _texture: Texture,
    pub view: TextureView,
}

impl MultisampleData {
    pub fn create(surface_config: &SurfaceConfiguration, device: &Device) -> Self {
        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA color texture"),
            size: Extent3d {
                width: surface_config.width.max(1),
                height: surface_config.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let msaa_view = msaa_texture.create_view(&Default::default());
        Self {
            _texture: msaa_texture,
            view: msaa_view,
        }
    }
}

// State for shadow map.

pub struct ShadowState {
    pub pipeline: RenderPipeline,

    pub _texture: wgpu::Texture,
    pub view: TextureView,
    pub _sampler: Sampler,

    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl ShadowState {
    const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        width: 4000,
        height: 4000,

        // 1 layer because we're using 1 light (for now).
        depth_or_array_layers: 1,
    };

    pub fn create<Vertex: Bufferable>(
        device: &Device,
        light: &LightState,
        mesh: &MeshRenderData,
    ) -> Self {
        let pipeline = pipeline::create_shadow_pipeline::<Vertex>(
            device,
            &[
                &light.camera_matrix_bind_group_layout,
                &mesh.bind_group_layout,
            ],
        );

        let _texture = device.create_texture(&TextureDescriptor {
            size: Self::SHADOW_SIZE,
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
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

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
            ],
            label: None,
        });

        Self {
            pipeline,
            _texture,
            view,
            _sampler,
            bind_group_layout,
            bind_group,
        }
    }
}
