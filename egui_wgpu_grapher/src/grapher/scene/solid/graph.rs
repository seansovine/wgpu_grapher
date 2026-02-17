// make scene for function graph

use super::build_scene;
use crate::grapher::{
    math::{
        FunctionHolder, SmoothingFunctionWrapper,
        graph::{self, GraphableFunc},
    },
    matrix::Matrix,
    render::RenderState,
    scene::{RenderScene, Scene3D},
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
    pub scene: Option<Scene3D>,

    // size of rectangular domain of graph
    pub width: f64,

    // have parameters changed that require mesh regen
    pub needs_rebuild: bool,

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
            needs_rebuild: false,
            parameters: Default::default(),
            function: None,
        }
    }
}

const GRAPH_SUBDIVISIONS: u32 = 750;

impl GraphScene {
    pub fn try_rebuild_scene(
        &mut self,
        device: &Device,
        surface_config: &SurfaceConfiguration,
        state: &RenderState,
        smoothing_scale: Option<f64>,
    ) {
        if self.function.is_none() {
            self.scene = None;
        }
        let f = self.function.take().unwrap().f;

        // TODO: This is currently disabled until we get
        //       an updated UI that works better for it.
        //
        // let f = graph::shift_scale_input(
        //     f,
        //     self.parameters.shift_x,
        //     self.parameters.scale_x,
        //     self.parameters.shift_z,
        //     self.parameters.scale_z,
        // );
        // let f = graph::shift_scale_output(f, self.parameters.shift_y, self.parameters.scale_y);

        let f = if let Some(scale) = smoothing_scale {
            let f = SmoothingFunctionWrapper::from(f, scale / GRAPH_SUBDIVISIONS as f64);
            FunctionHolder::from(move |x: f64, z: f64| f.eval(x, z))
        } else {
            FunctionHolder::from(f)
        };

        self.scene = Some(build_scene_for_graph(
            device,
            surface_config,
            state,
            self.width,
            &f,
            smoothing_scale.is_none(),
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
    direct_normals: bool,
) -> Scene3D {
    // TODO: Add GUI parameter for floor mesh.
    //
    // let floor_mesh = graph::SquareTesselation::generate(SUBDIVISIONS, width)
    //     .mesh_data(graph::SquareTesselation::FLOOR_COLOR);

    let grid = graph::SquareTesselation::generate(GRAPH_SUBDIVISIONS, width, f);

    let func_mesh = if direct_normals {
        grid.mesh_data_direct_normals(graph::SquareTesselation::FUNC_COLOR, f)
    } else {
        grid.mesh_data(graph::SquareTesselation::FUNC_COLOR)
    };

    build_scene(
        device,
        surface_config,
        state,
        vec![(func_mesh, Matrix::identity())],
    )
}

#[allow(dead_code)]
pub fn get_example_function(parameters: &GraphParameters) -> FunctionHolder {
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

    let mut scene = None;
    if let Some(f) = function.as_ref() {
        scene = Some(build_scene_for_graph(
            device,
            surface_config,
            state,
            WIDTH,
            f,
            false,
        ));
    }

    let needs_update = false;

    GraphScene {
        scene,
        width: WIDTH,
        needs_rebuild: needs_update,
        parameters,
        function,
    }
}

impl RenderScene for GraphScene {
    fn scene(&self) -> &Scene3D {
        self.scene.as_ref().unwrap()
    }

    fn update(&mut self, _queue: &Queue, _state: &RenderState) {}
}
