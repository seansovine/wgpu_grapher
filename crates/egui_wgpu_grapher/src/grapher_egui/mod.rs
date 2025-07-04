// The code in this module is a bridge between egui and function graphing code.

use crate::grapher::{
    mesh::{solid::GraphScene, RenderScene},
    pipeline::render_preferences::RenderPreferences,
    render::RenderState,
};

use egui::{Grid, Ui};
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

// Code for building the grapher renderer parameter ui.

pub struct RenderUiState {
    // state for ui rendering
    pub lighting_enabled: bool,
    pub use_wireframe: bool,

    // was there and update that needs processed
    pub needs_update: bool,
}

impl RenderUiState {
    pub fn from_render_preferences(render_prefs: &RenderPreferences) -> Self {
        Self {
            lighting_enabled: render_prefs.lighting_enabled(),
            use_wireframe: render_prefs.wireframe_enabled(),
            needs_update: false,
        }
    }
}

pub fn render_parameter_ui(
    render_state: &mut RenderState,
    render_ui_state: &mut RenderUiState,
    grapher_scene: &mut GrapherScene,
    ui: &mut Ui,
) {
    ui.horizontal(|ui| {
        let response = ui.checkbox(&mut render_ui_state.lighting_enabled, "Lighting ");

        if response.changed() {
            render_state
                .render_preferences
                .set_lighting_enabled(render_ui_state.lighting_enabled);

            // only requires updating a uniform with write_buffer
            render_ui_state.needs_update = true;
        }

        let response = ui.checkbox(&mut render_ui_state.use_wireframe, "Wireframe ");

        if response.changed() {
            render_state
                .render_preferences
                .set_wireframe(render_ui_state.use_wireframe);

            // requires changing polygon mode, and so recreating pipeline
            grapher_scene.set_needs_update(true);
        }
    });
}

// The following enum replaces dynamic dispatch and allows the
// GUI to display different data and perform different actions
// depending on the particular grapher scene that is selected.

pub enum GrapherScene {
    Graph(GraphSceneData),
    // TODO: add other available scene types
}

impl GrapherScene {
    pub fn render(
        &self,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        render_state: &RenderState,
    ) {
        match self {
            GrapherScene::Graph(data) => {
                // pass scene in to state render function
                render_state.render(view, encoder, data.graph_scene.scene());
            }
            _ => unimplemented!(),
        }
    }

    pub fn update(
        &mut self,
        device: &Device,
        surface_config: &SurfaceConfiguration,
        queue: &Queue,
        state: &RenderState,
        pre_render: bool,
    ) {
        match self {
            GrapherScene::Graph(data) => {
                // rebuild scene if parameters changed
                if data.graph_scene.needs_update {
                    data.graph_scene
                        .rebuild_scene(device, surface_config, state);
                    data.graph_scene.needs_update = false;
                }
                // currently a no-op; would perform state update
                data.graph_scene.update(queue, state, pre_render);
            }
            _ => unimplemented!(),
        }
    }
}

impl GrapherScene {
    pub fn parameter_ui(&mut self, editing: &mut bool, ui: &mut Ui) {
        match self {
            GrapherScene::Graph(graph_scene) => {
                parameter_ui_graph(graph_scene, editing, ui);
            }
            _ => unimplemented!(),
        }
    }

    pub fn set_needs_update(&mut self, needs_update: bool) {
        match self {
            GrapherScene::Graph(data) => {
                data.graph_scene.needs_update = needs_update;
            }
            _ => unimplemented!(),
        }
    }
}

pub struct GraphSceneUiData {
    scale_x_text: String,
    scale_z_text: String,
    scale_y_text: String,

    shift_x_text: String,
    shift_z_text: String,
    shift_y_text: String,
}

pub struct GraphSceneData {
    pub graph_scene: GraphScene,
    pub ui_data: GraphSceneUiData,
}

impl GraphSceneData {
    pub fn new(graph_scene: GraphScene) -> Self {
        let scale_x_text = graph_scene.parameters.scale_x.to_string();
        let scale_z_text = graph_scene.parameters.scale_z.to_string();
        let scale_y_text = graph_scene.parameters.scale_y.to_string();

        let shift_x_text = graph_scene.parameters.shift_x.to_string();
        let shift_z_text = graph_scene.parameters.shift_z.to_string();
        let shift_y_text = graph_scene.parameters.shift_y.to_string();

        Self {
            graph_scene,
            ui_data: GraphSceneUiData {
                scale_x_text,
                scale_z_text,
                scale_y_text,

                shift_x_text,
                shift_z_text,
                shift_y_text,
            },
        }
    }
}

// graph-specific parameter ui
fn parameter_ui_graph(data: &mut GraphSceneData, editing: &mut bool, ui: &mut Ui) {
    let scale_x = &mut data.graph_scene.parameters.scale_x;
    let scale_z = &mut data.graph_scene.parameters.scale_z;
    let scale_y = &mut data.graph_scene.parameters.scale_y;

    let shift_x = &mut data.graph_scene.parameters.shift_x;
    let shift_z = &mut data.graph_scene.parameters.shift_z;
    let shift_y = &mut data.graph_scene.parameters.shift_y;

    let needs_update = &mut data.graph_scene.needs_update;

    Grid::new("graph parameter input").show(ui, |ui| {
        *needs_update = float_edit_line(
            "Graph x scale",
            &mut data.ui_data.scale_x_text,
            scale_x,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        // scale parameter edits

        *needs_update = float_edit_line(
            "Graph z scale",
            &mut data.ui_data.scale_z_text,
            scale_z,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        *needs_update = float_edit_line(
            "Graph y scale",
            &mut data.ui_data.scale_y_text,
            scale_y,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        ui.separator();
        ui.end_row();

        // shift parameter edits

        *needs_update = float_edit_line(
            "Graph x shift",
            &mut data.ui_data.shift_x_text,
            shift_x,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        *needs_update = float_edit_line(
            "Graph z shift",
            &mut data.ui_data.shift_z_text,
            shift_z,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();

        *needs_update = float_edit_line(
            "Graph y shift",
            &mut data.ui_data.shift_y_text,
            shift_y,
            editing,
            ui,
        ) || *needs_update;
        ui.end_row();
    });
}

fn float_edit_line(
    label: &str,
    edit_text: &mut String,
    edit_value: &mut f32,
    editing: &mut bool,
    ui: &mut Ui,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(format!("{label}: "));

        let response = ui.add(egui::TextEdit::singleline(edit_text));

        if response.gained_focus() {
            *editing = true;
        }

        if response.lost_focus() {
            // parse text and update value if valid
            if let Ok(f_val) = edit_text.parse::<f32>() {
                *edit_value = f_val;
                changed = true;
            } else {
                *edit_text = edit_value.to_string();
            }
            *editing = false;
        }
    });

    changed
}
