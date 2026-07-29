use crate::grapher::{
    camera::CameraState,
    matrix::MatrixUniform,
    pipeline::{render_preferences::RenderPreferences, texture::DepthBuffer},
};

use egui_wgpu::wgpu::{
    self, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Device, Extent3d, Queue, SurfaceConfiguration, Texture, TextureView,
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
    // does light move with time?
    pub light_motion: bool,
}

impl RenderState {
    pub async fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        let camera_state = CameraState::init(device, surface_config);
        let mut shader_preferences = RenderPreferences::create(device);
        shader_preferences.set_binding_index(1);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                *MatrixUniform::bind_group_layout_entry(),
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
            light_motion: false,
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
            .matrix
            .update_value(self.camera_state.camera.get_matrix());
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
