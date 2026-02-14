use crate::grapher::{
    camera::CameraState,
    pipeline::{
        self, light::LightState, render_preferences::RenderPreferences, texture::DepthBuffer,
    },
    scene::Bufferable,
};

use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Device, Extent3d, Queue, RenderPipeline, Sampler,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
};
use winit::event::{DeviceEvent, WindowEvent};

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

    pub fn handle_user_input(&mut self, event: &WindowEvent) -> bool {
        // All currently handled events affect the camera.
        self.camera_state.controller.process_events(event)
    }

    pub fn handle_device_input(&mut self, event: &DeviceEvent) {
        self.camera_state.controller.process_device_events(event);
    }

    pub fn update_camera(&mut self, queue: &mut Queue) {
        // adjust controller speed based on framerate
        self.camera_state.controller.speed = 2.125 / self.framerate;
        self.camera_state
            .controller
            .update_camera(&mut self.camera_state.camera);
        self.camera_state
            .matrix
            .uniform
            .update_matrix(self.camera_state.camera.get_matrix());
        // we write the uniform every frame
        self.camera_state.update_uniform(queue);
    }

    pub fn handle_resize(&mut self, device: &Device, surface_config: &SurfaceConfiguration) {
        // Resize depth buffer texture.
        self.depth_buffer = DepthBuffer::create(surface_config, device);
        // Resize MSAA texture.
        self.msaa_data = MultisampleData::create(surface_config, device);
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
        surface_config: &SurfaceConfiguration,
        device: &Device,
        light: &LightState,
        model_matrix_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let pipeline = pipeline::create_shadow_pipeline::<Vertex>(
            device,
            &[
                &light.camera_matrix_bind_group_layout,
                model_matrix_bind_group_layout,
            ],
        );

        let surface_width = surface_config.width.max(1);
        let surface_height = surface_config.height.max(1);
        let max_tex_size = device.limits().max_texture_dimension_2d;
        let mut texture_size_multiplier = 4;
        // We use a shadow texture larger than the render surface to reduce aliasing.

        // set texture size factor
        #[allow(clippy::ifs_same_cond)]
        if surface_width * texture_size_multiplier > max_tex_size
            || surface_height * texture_size_multiplier > max_tex_size
        {
            texture_size_multiplier = 2;
        } else if surface_width * texture_size_multiplier > max_tex_size
            || surface_height * texture_size_multiplier > max_tex_size
        {
            texture_size_multiplier = 1;
        }

        let _texture = device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: surface_config.width.max(1) * texture_size_multiplier,
                height: surface_config.height.max(1) * texture_size_multiplier,
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
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let camera_view_matrix = light.camera_view_matrix();
        let mut camera_view_bgl_entry = camera_view_matrix.bind_group_layout_entry;
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
                    resource: camera_view_matrix.buffer.as_entire_binding(),
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
