use crate::camera::CameraState;
use crate::mesh::texture::DepthBuffer;

use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct RenderState<'a> {
  // wgpu
  pub surface: wgpu::Surface<'a>,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  pub depth_buffer: DepthBuffer,
  // winit
  pub size: PhysicalSize<u32>,
  pub window: &'a Window,
  // camera
  pub camera_state: CameraState,
  // running framerate
  pub framerate: f32,
}

impl<'a> RenderState<'a> {
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

    // make camera and depth buffer

    let camera_state = CameraState::init(&device, &config);

    let depth_buffer = DepthBuffer::create(&config, &device);

    Self {
      surface,
      device,
      queue,
      config,
      depth_buffer,
      size,
      window,
      camera_state,
      framerate: 1_f32,
    }
  }
}

impl RenderState<'_> {
  pub fn window(&self) -> &Window {
    self.window
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);

      // update camera aspect ratio
      self.camera_state.camera.aspect = self.config.width as f32 / self.config.height as f32;

      // update depth buffer size
      self.depth_buffer = DepthBuffer::create(&self.config, &self.device);

      self.update()
    }
  }

  pub fn handle_user_input(&mut self, event: &WindowEvent) -> bool {
    self.camera_state.controller.process_events(event)
  }

  pub fn update(&mut self) {
    // adjust controller speed based on framerate
    self.camera_state.controller.speed = 2.125 / self.framerate;

    self
      .camera_state
      .controller
      .update_camera(&mut self.camera_state.camera);

    self
      .camera_state
      .matrix
      .uniform
      .update(self.camera_state.camera.get_matrix());

    // update camera matrix uniform
    self.queue.write_buffer(
      &self.camera_state.matrix.buffer,
      0,
      bytemuck::cast_slice(&[self.camera_state.matrix.uniform]),
    );
  }
}
