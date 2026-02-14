//! Scene to render equation solver on 2d canvas.

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};

use crate::grapher::scene::two_d::TwoDScene;

pub struct SolverSceneData {
    pub scene: TwoDScene,
    pub updates_paused: bool,
}

impl SolverSceneData {
    pub fn new(device: &Device, queue: &Queue, surface_config: &SurfaceConfiguration) -> Self {
        Self {
            scene: TwoDScene::new(device, queue, surface_config),
            updates_paused: false,
        }
    }

    pub fn update(&mut self, _: &Queue) {}

    pub fn run_solver(&mut self, device: &Device, queue: &Queue) {
        const TIMESTEPS_PER_FRAME: usize = 4;

        if !self.updates_paused {
            for _ in 0..TIMESTEPS_PER_FRAME {
                let mut encoder = device.create_command_encoder(&Default::default());
                self.scene.increment_timestep(queue);
                self.scene.solver_timestep(&mut encoder);

                // We seem to need to submit the queue each time we run this
                // to make it actually run repeatedly. Will follow up (TODO).
                queue.submit(Some(encoder.finish()));
            }
        }
    }
}
