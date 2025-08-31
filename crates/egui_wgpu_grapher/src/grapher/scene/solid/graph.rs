// make scene for function graph

use super::build_scene;
use crate::grapher::{
    math::graph::{self, GraphableFunc},
    matrix::MatrixUniform,
    render::RenderState,
    scene::{RenderScene, Scene},
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};
use meval::Expr;

pub struct GraphParameters {
    pub scale_x: f32,
    pub scale_z: f32,
    pub scale_y: f32,

    pub shift_x: f32,
    pub shift_z: f32,
    pub shift_y: f32,
}

impl Default for GraphParameters {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_z: 1.0,
            scale_y: 1.0,

            shift_x: 0.0,
            shift_y: 0.0,
            shift_z: 0.0,
        }
    }
}

pub struct GraphScene {
    // all the data for rendering
    pub scene: Option<Scene>,

    // size of rectangular domain of graph
    pub width: f32,

    // TODO: generalize this and move it to RenderScene
    pub needs_update: bool,

    // publicly adjustable parameters
    pub parameters: GraphParameters,

    // string representation of function to graph
    pub function_string: String,

    // does the string represent a valid function def
    pub function_valid: bool,

    // possible function to graph
    pub function: Option<FunctionHolder>,
}

impl Default for GraphScene {
    fn default() -> Self {
        Self {
            scene: None,
            width: 6.0_f32,
            needs_update: false,
            parameters: Default::default(),
            function_string: Default::default(),
            function_valid: false,
            function: None,
        }
    }
}

fn build_scene_for_graph(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
    width: f32,
    f: &impl GraphableFunc,
) -> Scene {
    const SUBDIVISIONS: u32 = 750;

    let floor_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, width)
        .mesh_data(graph::SquareTesselation::FLOOR_COLOR);
    let matrix = MatrixUniform::translation(&[-width / 2.0_f32, 0.0f32, -width / 2.0_f32]);

    let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, width)
        .apply_function(f)
        .mesh_data(graph::SquareTesselation::FUNCT_COLOR);

    build_scene(
        device,
        surface_config,
        state,
        vec![(floor_mesh, matrix), (func_mesh, matrix)],
    )
}

pub struct FunctionHolder {
    f: Box<dyn Fn(f32, f32) -> f32>,
}

impl GraphableFunc for FunctionHolder {
    fn eval(&self, x: f32, y: f32) -> f32 {
        (self.f)(x, y)
    }
}

// This is a placeholder providing a default function,
// until we implement a math expression parser in the UI.
pub fn get_graph_func(parameters: &GraphParameters) -> FunctionHolder {
    // Other good example functions:
    // let f = |x: f32, z: f32| (x * x + z * z).sqrt().sin() / (x * x + z * z).sqrt();
    // let f = |x: f32, z: f32| x.powi(2) + z.powi(2);

    let f = |x: f32, z: f32| 2.0_f32.powf(-(x.powi(2) + z.powi(2)).sin());
    let f = graph::shift_scale_input(
        f,
        parameters.shift_x,
        parameters.scale_x,
        parameters.shift_z,
        parameters.scale_z,
    );
    let f = graph::shift_scale_output(f, parameters.shift_y, parameters.scale_y);

    FunctionHolder { f: Box::from(f) }
}

#[allow(dead_code)]
pub fn graph_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> GraphScene {
    const WIDTH: f32 = 6.0;

    let parameters = GraphParameters {
        scale_x: 2.0,
        scale_z: 2.0,
        scale_y: 0.5,

        shift_x: WIDTH / 2.0,
        shift_z: WIDTH / 2.0,
        shift_y: 0.25,
    };

    let function_string = "2.0^(-sin(x^2 + z^2))".to_string();
    let mut function = None;
    let mut function_valid = false;
    if let Ok(expr) = function_string.parse::<Expr>() {
        if let Ok(func) = expr.bind2("x", "z") {
            let closure = move |x: f32, z: f32| -> f32 { func(x as f64, z as f64) as f32 };
            function = Some(FunctionHolder {
                f: Box::from(closure),
            });
            function_valid = true;
        }
    }

    // let f = get_graph_func(&parameters);

    let mut scene = None;
    if let Some(f) = function.as_ref() {
        scene = Some(build_scene_for_graph(
            device,
            surface_config,
            state,
            WIDTH,
            f,
        ));
    }

    let needs_update = false;

    GraphScene {
        scene,
        width: WIDTH,
        needs_update,
        parameters,
        function_string,
        function_valid,
        function,
    }
}

impl GraphScene {
    // will be called when gui updates graph parameters, etc.
    pub fn rebuild_scene(
        &mut self,
        device: &Device,
        surface_config: &SurfaceConfiguration,
        state: &RenderState,
    ) {
        let f = get_graph_func(&self.parameters);
        self.scene = Some(build_scene_for_graph(
            device,
            surface_config,
            state,
            self.width,
            &f,
        ));
    }
}

impl RenderScene for GraphScene {
    fn scene(&self) -> &Scene {
        self.scene.as_ref().unwrap()
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState, _pre_render: bool) {
        // no-op for now
    }
}
