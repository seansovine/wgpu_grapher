use crate::grapher::{
    camera::CameraState,
    pipeline::{
        self, light::LightState, render_preferences::RenderPreferences, texture::DepthBuffer,
    },
    scene::Vertex,
};

use egui_wgpu::wgpu::{
    self, BindGroupLayout, Device, Queue, RenderPipeline, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureUsages, TextureView,
};
use winit::event::WindowEvent;

pub struct RenderState {
    // camera
    pub camera_state: CameraState,
    // light
    pub light_state: LightState,
    // shader preferences
    pub render_preferences: RenderPreferences,
    // running framerate
    pub framerate: f32,
    // depth buffer
    pub depth_buffer: DepthBuffer,
}

impl RenderState {
    pub async fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        let camera_state = CameraState::init(device, surface_config);
        let light_state = LightState::create(device);
        let shader_preferences_state = RenderPreferences::create(device);
        let depth_buffer = DepthBuffer::create(surface_config, device);

        Self {
            camera_state,
            light_state,
            render_preferences: shader_preferences_state,
            framerate: 60_f32, // we target 60 fps
            depth_buffer,
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

    pub fn create(device: &Device, bind_group_layouts: &[&BindGroupLayout]) -> Self {
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

        // TODO: Later make generic over texture.
        let pipeline = pipeline::create_shadow_pipeline::<Vertex>(device, bind_group_layouts);

        // TODO: Shortly we'll also need a sampler for the shaders to read this.
        Self {
            pipeline,
            _texture: shadow_texture,
            view: shadow_view,
        }
    }
}
