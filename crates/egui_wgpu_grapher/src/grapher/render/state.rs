use crate::grapher::{
    camera::CameraState,
    pipeline::{
        self, light::LightState, render_preferences::RenderPreferences, texture::DepthBuffer,
    },
    scene::{Bufferable, solid::MeshRenderData},
};

use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Device, Queue, RenderPipeline, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureUsages, TextureView,
};
use winit::event::WindowEvent;

pub struct RenderState {
    // camera
    pub camera_state: CameraState,
    // shader preferences
    pub render_preferences: RenderPreferences,
    // bind group for things global to the render
    pub bind_group_layout: BindGroupLayout,
    // includes preferences and camera
    pub bind_group: BindGroup,
    // depth buffer
    pub depth_buffer: DepthBuffer,
    // running framerate
    pub framerate: f32,
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

        Self {
            camera_state,
            render_preferences: shader_preferences,
            bind_group_layout,
            bind_group,
            depth_buffer,
            // we target 60fps
            framerate: 60_f32,
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

pub struct ShadowState {
    pub pipeline: RenderPipeline,
    _texture: wgpu::Texture,
    pub view: TextureView,
}

impl ShadowState {
    const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        width: 512,
        height: 512,
        // NOTE: Because we're using 1 light for now.
        depth_or_array_layers: 1,
    };

    pub fn create<Vertex: Bufferable>(
        device: &Device,
        light: &LightState,
        mesh: &MeshRenderData,
    ) -> Self {
        let shadow_texture = device.create_texture(&TextureDescriptor {
            size: Self::SHADOW_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::SHADOW_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let pipeline = pipeline::create_shadow_pipeline::<Vertex>(
            device,
            &[
                &light.camera_matrix_bind_group_layout,
                &mesh.bind_group_layout,
            ],
        );

        // TODO: Shortly we'll also need a sampler for the shaders to read this.
        Self {
            pipeline,
            _texture: shadow_texture,
            view: shadow_view,
        }
    }
}
