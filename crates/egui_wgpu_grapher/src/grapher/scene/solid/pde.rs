use super::{MeshData, build_scene};
use crate::grapher::{
    math::{graph::SquareTesselation, pde},
    matrix::MatrixUniform,
    render::RenderState,
    scene::{RenderScene, Scene},
};
use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};

// scene for simulating the wave equation

pub struct WaveEquationScene {
    pub scene: Scene,
    pub func_mesh: SquareTesselation,
    pub mesh_data: MeshData,
    pub wave_eqn: pde::WaveEquationData,
    pub display_scale: f32,
}

pub fn wave_eqn_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> WaveEquationScene {
    const WAVE_EQN_SUBDIV: usize = 600;
    // number of squares is 1 less than number of gridpoints
    const SUBDIVISIONS: u32 = WAVE_EQN_SUBDIV as u32 - 1;
    const WIDTH: f32 = 1.0;

    let func_mesh = SquareTesselation::generate(SUBDIVISIONS, WIDTH);
    let mesh_data = func_mesh.mesh_data(SquareTesselation::FUNCT_COLOR);
    let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, 0.1_f32, -WIDTH / 2.0_f32]);

    let scene = build_scene(
        device,
        surface_config,
        state,
        vec![(mesh_data.clone(), matrix)],
    );
    let mut wave_eqn = pde::WaveEquationData::new(WAVE_EQN_SUBDIV, WAVE_EQN_SUBDIV);

    wave_eqn.disturbance_prob = 0.003;
    wave_eqn.disturbance_size = 2.0;
    wave_eqn.damping_factor = 0.998;
    wave_eqn.prop_speed = 0.15;

    let display_scale: f32 = 0.075;

    WaveEquationScene {
        scene,
        func_mesh,
        mesh_data,
        wave_eqn,
        display_scale,
    }
}

impl RenderScene for WaveEquationScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, queue: &Queue, state: &RenderState, pre_render: bool) {
        // run next finite-difference timestep
        self.wave_eqn.update();

        let n = self.wave_eqn.x_size;
        let b = 2_usize;

        // update vertex y-coordinates
        for i in b..n - b {
            for j in b..n - b {
                self.mesh_data.vertices[j + i * n].position[1] =
                    self.display_scale * self.wave_eqn.u_0[i][j];
            }
        }

        if pre_render {
            if state.render_preferences.lighting_enabled() {
                // update vertex normals
                self.func_mesh.update_normals(&mut self.mesh_data);
            }

            // update vertex buffer
            queue.write_buffer(
                &self.scene.meshes[0].vertex_buffer,
                0,
                bytemuck::cast_slice(self.mesh_data.vertices.as_slice()),
            );
        }
    }
}

// scene for simulating the heat equation

pub struct HeatEquationScene {
    pub scene: Scene,
    pub func_mesh: SquareTesselation,
    pub mesh_data: MeshData,
    pub heat_eqn: pde::HeatEquationData,
    pub display_scale: f32,

    // HACK: we don't update boundary each render,
    // but keep buffer area fixed to avoid flicker
    b: usize,
}

pub fn heat_eqn_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> HeatEquationScene {
    let b: usize = 5;

    static WAVE_EQN_SUBDIV: usize = 400;
    // number of squares is 1 less than number of gridpoints
    let subdivisions: u32 = WAVE_EQN_SUBDIV as u32 - 1 - (b as u32 * 2);
    const WIDTH: f32 = 1.0;

    let func_mesh = SquareTesselation::generate(subdivisions, WIDTH);
    let mut mesh_data = func_mesh.mesh_data(SquareTesselation::FUNCT_COLOR);

    func_mesh.update_normals(&mut mesh_data);

    let matrix = MatrixUniform::translation(&[-WIDTH / 2.0_f32, 0.1_f32, -WIDTH / 2.0_f32]);
    let scene = build_scene(
        device,
        surface_config,
        state,
        vec![(mesh_data.clone(), matrix)],
    );

    let heat_eqn = pde::HeatEquationData::new(WAVE_EQN_SUBDIV, WAVE_EQN_SUBDIV);
    let display_scale: f32 = 0.015;

    HeatEquationScene {
        scene,
        func_mesh,
        mesh_data,
        heat_eqn,
        display_scale,
        b,
    }
}

impl RenderScene for HeatEquationScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn update(&mut self, queue: &Queue, state: &RenderState, pre_render: bool) {
        // run next finite-difference timestep
        self.heat_eqn.update();

        let n = self.heat_eqn.x_size;
        let m = n - self.b * 2;

        // update vertex y-coordinates and color
        for i in 0..m {
            for j in 0..m {
                let new_height = self.display_scale
                    * self.heat_eqn.u[(i + self.b) * n + (j + self.b)][self.heat_eqn.current_index];
                let new_color: [f32; 3] = [
                    255.0,
                    (255.0 * new_height.abs().clamp(0.0, 10.0) / 10.0),
                    0.0,
                ];

                self.mesh_data.vertices[j + i * m].position[1] = new_height;
                self.mesh_data.vertices[j + i * m].color = new_color
            }
        }

        if pre_render {
            if state.render_preferences.lighting_enabled() {
                // update vertex normals
                self.func_mesh.update_normals(&mut self.mesh_data);
            }

            // update vertex buffer
            queue.write_buffer(
                &self.scene.meshes[0].vertex_buffer,
                0,
                bytemuck::cast_slice(self.mesh_data.vertices.as_slice()),
            );
        }
    }
}
