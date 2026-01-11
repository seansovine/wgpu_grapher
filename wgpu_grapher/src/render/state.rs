use crate::{
    camera::CameraState,
    pipeline::{light::LightState, render_preferences::RenderPreferences, texture::DepthBuffer},
};

use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct GpuState<'a> {
    // wgpu
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub depth_buffer: DepthBuffer,

    // winit
    pub size: PhysicalSize<u32>,
    pub window: &'a Window,
}

impl<'a> GpuState<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // create surface, device, queue, config

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let depth_buffer = DepthBuffer::create(&config, &device);

        GpuState {
            surface,
            device,
            queue,
            config,
            depth_buffer,
            size,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        render_state: &mut RenderState,
    ) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // update camera aspect ratio
            render_state.camera_state.camera.aspect =
                self.config.width as f32 / self.config.height as f32;

            // update depth buffer size
            self.depth_buffer = DepthBuffer::create(&self.config, &self.device);

            render_state.update(&mut self.queue)
        }
    }
}

pub struct RenderState {
    // camera
    pub camera_state: CameraState,
    // light
    pub light_state: LightState,
    // shader preferences
    pub render_preferences: RenderPreferences,
    // running framerate
    pub framerate: f32,
}

impl RenderState {
    pub async fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        // make camera, light, shader preferences

        let camera_state = CameraState::init(device, surface_config);

        let light_state = LightState::create(device);

        let shader_preferences_state = RenderPreferences::create(device);

        // construct state

        Self {
            camera_state,
            light_state,
            render_preferences: shader_preferences_state,
            // we target 60 fps
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
        queue.write_buffer(
            &self.camera_state.matrix.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_state.matrix.uniform]),
        );
    }
}
