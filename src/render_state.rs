use crate::camera::CameraState;

use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct RenderState<'a> {
  // wgpu
  pub surface: wgpu::Surface<'a>,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  // winit
  pub size: PhysicalSize<u32>,
  pub window: &'a Window,
  // camera
  pub camera_state: CameraState,
}

impl<'a> RenderState<'a> {
  // some of the wgpu api is async
  pub async fn new(window: &'a Window) -> Self {
    let size = window.inner_size();

    // create surface, device, queue, config

    // note that we could target WASM
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

    // make camera

    let camera_state = CameraState::init(&device, &config);

    // construct return object

    Self {
      surface,
      device,
      queue,
      config,
      size,
      window,
      camera_state,
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
    }
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    self.camera_state.controller.process_events(event)
  }

  pub fn update(&mut self) {
    self
      .camera_state
      .controller
      .update_camera(&mut self.camera_state.camera);
    self
      .camera_state
      .matrix
      .uniform
      .update(self.camera_state.camera.get_matrix());
    self.queue.write_buffer(
      &self.camera_state.matrix.buffer,
      0,
      bytemuck::cast_slice(&[self.camera_state.matrix.uniform]),
    );
  }
}
