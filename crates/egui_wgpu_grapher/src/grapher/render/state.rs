use crate::grapher::camera::CameraState;
use crate::grapher::pipeline::light::LightState;
use crate::grapher::pipeline::render_preferences::RenderPreferences;
use crate::grapher::pipeline::texture::DepthBuffer;

use egui_wgpu::wgpu::{self, Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

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
        // make camera, light, shader preferences

        let camera_state = CameraState::init(device, surface_config);

        let light_state = LightState::create(device);

        let shader_preferences_state = RenderPreferences::create(device);

        // construct state

        let depth_buffer = DepthBuffer::create(surface_config, device);

        Self {
            camera_state,
            light_state,
            render_preferences: shader_preferences_state,
            // we target 60 fps
            framerate: 60_f32,
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
        queue.write_buffer(
            &self.camera_state.matrix.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_state.matrix.uniform]),
        );
    }
}
