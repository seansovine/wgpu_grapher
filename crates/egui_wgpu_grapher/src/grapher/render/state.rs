use crate::grapher::{
    camera::CameraState,
    pipeline::{light::LightState, render_preferences::RenderPreferences, texture::DepthBuffer},
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};
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
