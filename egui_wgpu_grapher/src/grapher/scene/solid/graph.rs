// make scene for function graph

use super::build_scene;
use crate::grapher::{
    math::{
        FunctionHolder,
        graph::{self, GraphableFunc},
    },
    matrix::MatrixUniform,
    render::RenderState,
    scene::{RenderScene, Scene},
};

use egui_wgpu::wgpu::{Device, Queue, SurfaceConfiguration};
use meval::Expr;

pub struct GraphParameters {
    pub scale_x: f64,
    pub scale_z: f64,
    pub scale_y: f64,

    pub shift_x: f64,
    pub shift_z: f64,
    pub shift_y: f64,
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
    pub width: f64,

    // TODO: generalize this and move it to RenderScene
    pub needs_update: bool,

    // publicly adjustable parameters
    pub parameters: GraphParameters,

    // function to graph, if any
    pub function: Option<FunctionHolder>,
}

impl Default for GraphScene {
    fn default() -> Self {
        Self {
            scene: None,
            width: 6.0_f64,
            needs_update: false,
            parameters: Default::default(),
            function: None,
        }
    }
}

impl GraphScene {
    pub fn try_rebuild_scene(
        &mut self,
        device: &Device,
        surface_config: &SurfaceConfiguration,
        state: &RenderState,
    ) {
        if self.function.is_none() {
            self.scene = None;
        }
        let f = self.function.take().unwrap().f;
        let f = graph::shift_scale_input(
            f,
            self.parameters.shift_x,
            self.parameters.scale_x,
            self.parameters.shift_z,
            self.parameters.scale_z,
        );
        let f = graph::shift_scale_output(f, self.parameters.shift_y, self.parameters.scale_y);
        let f = FunctionHolder::from(f);

        self.scene = Some(build_scene_for_graph(
            device,
            surface_config,
            state,
            self.width,
            &f,
        ));
        self.function = Some(f);
    }
}

pub fn build_scene_for_graph(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
    width: f64,
    f: &impl GraphableFunc,
) -> Scene {
    const SUBDIVISIONS: u32 = 750;

    let matrix = MatrixUniform::identity();
    // Previously: MatrixUniform::translation(&[-width / 2.0_f32, 0.0f32, -width / 2.0_f32]);
    //
    // TODO: Omitting floor mesh until we fix its interaction with shadow mapping.
    //
    // let floor_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, width)
    //     .mesh_data(graph::SquareTesselation::FLOOR_COLOR);

    let func_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, width)
        .apply_function(f)
        .mesh_data(graph::SquareTesselation::FUNCT_COLOR);

    build_scene(
        device,
        surface_config,
        state,
        vec![(func_mesh, matrix)], // omitting: (floor_mesh, matrix),
    )
}

#[allow(dead_code)]
pub fn get_example_function(parameters: &GraphParameters) -> FunctionHolder {
    // Other good example functions:
    // let f = |x: f32, z: f32| (x * x + z * z).sqrt().sin() / (x * x + z * z).sqrt();
    // let f = |x: f32, z: f32| x.powi(2) + z.powi(2);
    let f = |x: f64, z: f64| 2.0_f64.powf(-(x.powi(2) + z.powi(2)).sin());
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
pub fn demo_graph_scene(
    device: &Device,
    surface_config: &SurfaceConfiguration,
    state: &RenderState,
) -> GraphScene {
    const WIDTH: f64 = 6.0;

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
    if let Ok(expr) = function_string.parse::<Expr>()
        && let Ok(func) = expr.bind2("x", "z")
    {
        function = Some(FunctionHolder { f: Box::from(func) });
    }
    // let function = Some(get_graph_func(&parameters));

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
        function,
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
