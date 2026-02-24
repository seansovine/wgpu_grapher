//! Code to manage the different modes of the app, to dispatch high-level
//! calls based on the current mode, and to map GUI-modified state to
//! internal handler functions for the current mode.

pub mod graph_scene;
pub mod image_scene;
pub mod model_scene;
pub mod solver_scene;

use std::{
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
    },
    thread::JoinHandle,
};

use crate::{
    egui::ui::UiState,
    grapher::{
        pipeline::render_preferences::RenderPreferences,
        render::{ShadowState, render_2d},
        scene::{
            GpuVertex, RenderScene,
            solid::{
                MeshRenderData,
                graph::{GraphScene, try_build_mesh_from_string},
            },
        },
    },
    grapher_egui::{
        image_scene::{ImageViewerSceneData, parameter_ui_image_viewer},
        solver_scene::SolverSceneData,
    },
};
use graph_scene::{GraphSceneData, parameter_ui_graph};
use model_scene::{ModelSceneData, parameter_ui_model};

use egui::Ui;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

pub use crate::grapher::render::RenderState;

// --------------------------------
// Grapher mode chosen by the user.

#[derive(clap::ValueEnum, Debug, Default, Clone, Copy)]
pub enum GrapherSceneMode {
    #[default]
    Graph,
    Model,
    ImageViewer,
    Solver,
}

impl From<GrapherSceneMode> for usize {
    fn from(value: GrapherSceneMode) -> Self {
        match value {
            GrapherSceneMode::Graph => 0,
            GrapherSceneMode::Model => 1,
            GrapherSceneMode::ImageViewer => 2,
            GrapherSceneMode::Solver => 3,
        }
    }
}

impl From<usize> for GrapherSceneMode {
    fn from(value: usize) -> Self {
        match value {
            0 => GrapherSceneMode::Graph,
            1 => GrapherSceneMode::Model,
            2 => GrapherSceneMode::ImageViewer,
            3 => GrapherSceneMode::Solver,
            _ => unimplemented!(),
        }
    }
}

pub struct Changed(bool);

impl Changed {
    pub fn changed(&self) -> bool {
        self.0
    }
}

pub fn scene_selection_ui(
    selected_scene: &mut GrapherSceneMode,
    ui_state: &mut UiState,
    ui: &mut Ui,
) -> Changed {
    let alternatives = ["graph", "model", "image", "solver"];
    let selected_scene_index = &mut ui_state.selected_scene_index;
    let response = egui::ComboBox::from_id_salt("select scene").show_index(
        ui,
        selected_scene_index,
        alternatives.len(),
        |i| alternatives[i],
    );
    if response.changed() {
        *selected_scene = (*selected_scene_index).into();
        Changed(true)
    } else {
        Changed(false)
    }
}

// -------------------------------------------
// Structures to manage background task state.

pub struct BackgroundTask {
    // State of any background task.
    // - 0 = no background task
    // - 1 = background task running
    // - 2 = background task ready
    // - 3 = background task cancelled
    pub task_state: Arc<AtomicU16>,
    pub thread_handle: Option<JoinHandle<()>>,
}

pub const BACKGROUND_TASK_NONE: u16 = 0;
pub const BACKGROUND_TASK_RUNNING: u16 = 1;
pub const BACKGROUND_TASK_READY: u16 = 2;
pub const BACKGROUND_TASK_CANCELLED: u16 = 3;

impl BackgroundTask {
    pub fn reset(&mut self) {
        self.task_state
            .store(BACKGROUND_TASK_NONE, Ordering::Relaxed);
        let thread = self.thread_handle.take();
        if let Some(handle) = thread {
            handle.join().expect("Background thread panicked.");
        }
    }

    pub fn check_for_crash(&mut self) -> bool {
        if self.thread_handle.is_none() {
            return false;
        }
        if let Some(handle) = &self.thread_handle
            && !handle.is_finished()
        {
            return false;
        }
        let handle = self.thread_handle.take().unwrap();
        return handle.join().is_err();
    }
}

impl Default for BackgroundTask {
    fn default() -> Self {
        Self {
            task_state: AtomicU16::new(BACKGROUND_TASK_NONE).into(),
            thread_handle: None,
        }
    }
}

// ----------------------------------
// Grapher mode and associated state.

pub enum GrapherScene {
    // Means user has chosen new mode that needs loaded.
    Changed,
    // Means that no state has been loaded.
    None,

    Graph(Box<GraphSceneData>),
    Model(ModelSceneData),
    ImageViewer(ImageViewerSceneData),
    Solver(SolverSceneData),
}

pub enum UpdateEffect {
    None,
    BackgroundTaskStarted,
}

impl GrapherScene {
    pub fn is_some(&self) -> bool {
        !matches!(self, GrapherScene::None | GrapherScene::Changed)
    }

    pub fn render(
        &self,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        render_state: &RenderState,
    ) {
        match self {
            GrapherScene::Graph(data) => {
                if data.graph_scene.scene.is_some() {
                    render_state.render(view, encoder, data.graph_scene.scene());
                }
            }
            GrapherScene::Model(data) => {
                render_state.render(view, encoder, data.model_scene.scene());
            }
            GrapherScene::ImageViewer(data) => {
                render_state.render(view, encoder, data.image_viewer_scene.scene());
            }
            GrapherScene::Solver(data) => {
                render_2d(view, encoder, &data.scene, render_state);
            }
            _ => unimplemented!(),
        }
    }

    pub fn compute(&mut self, device: &Device, queue: &Queue) {
        if let GrapherScene::Solver(data) = self {
            data.run_solver(device, queue);
        }
    }

    pub fn update(
        &mut self,
        _: &Device,
        _: &SurfaceConfiguration,
        queue: &Queue,
        state: &RenderState,
        background_task: &BackgroundTask,
    ) -> UpdateEffect {
        match self {
            GrapherScene::Graph(data) => {
                // Rebuild scene if non-uniform parameters changed.
                if data.graph_scene.needs_rebuild && !data.function_string.is_empty() {
                    let function_string = data.function_string.clone();
                    self.start_update_graph(function_string, background_task.task_state.clone());
                    // Re-borrow here so we can use self to update graph.
                    if let GrapherScene::Graph(data) = self {
                        data.graph_scene.needs_rebuild = false;
                    }
                    return UpdateEffect::BackgroundTaskStarted;
                }
                data.graph_scene.needs_rebuild = false;
                data.graph_scene.update(queue, state);
            }
            GrapherScene::Model(data) => {
                data.model_scene.update(queue, state);
            }
            GrapherScene::ImageViewer(data) => {
                data.image_viewer_scene.update(queue, state);
            }
            GrapherScene::Solver(data) => {
                data.update(queue);
            }
            _ => unimplemented!(),
        }
        UpdateEffect::None
    }

    pub fn start_update_graph(
        &mut self,
        function_string: String,
        background_task_state: Arc<AtomicU16>,
    ) -> Option<JoinHandle<()>> {
        if let GrapherScene::Graph(data) = self {
            data.function_string = function_string.clone();
            background_task_state.store(BACKGROUND_TASK_RUNNING, Ordering::Relaxed);

            // We copy these so the thread can take ownership.
            let smoothing_scale = data.smoothing_scale;
            let width = data.graph_scene.width;
            let mesh_data = data.mesh_data.clone();
            let handle = std::thread::spawn(move || {
                // We rebuild from the string here because our function parsing
                // and evaluation library produces objects that don't have Rust's
                // thread-safety marker traits.
                let Some(mesh) =
                    try_build_mesh_from_string(&function_string, smoothing_scale, width)
                else {
                    background_task_state.store(BACKGROUND_TASK_NONE, Ordering::Relaxed);
                    return;
                };
                *mesh_data.lock() = Some(mesh);
                background_task_state.store(BACKGROUND_TASK_READY, Ordering::Relaxed);
            });
            Some(handle)
        } else {
            None
        }
    }

    pub fn finish_update_graph(
        &mut self,
        device: &Device,
        surface_config: &SurfaceConfiguration,
        state: &RenderState,
    ) {
        if let GrapherScene::Graph(data) = self
            && let Some(mesh_data) = data.mesh_data.lock().take()
        {
            data.graph_scene
                .rebuild_scene_from_mesh(device, surface_config, state, mesh_data);
        }
    }

    pub fn parameter_ui(&mut self, ui: &mut Ui, ui_state: &mut UiState) {
        match self {
            GrapherScene::Graph(data) => {
                parameter_ui_graph(data, ui);
            }
            GrapherScene::Model(data) => {
                parameter_ui_model(data, ui, ui_state);
            }
            GrapherScene::ImageViewer(data) => {
                parameter_ui_image_viewer(data, ui, ui_state);
            }
            GrapherScene::Solver(data) => {
                data.parameter_ui(ui);
            }
            _ => {}
        }
    }

    pub fn set_needs_rebuild(&mut self, needs_update: bool) {
        match self {
            GrapherScene::Graph(data) => {
                data.graph_scene.needs_rebuild = needs_update;
            }
            GrapherScene::Model(_data) => {
                // no-op
            }
            GrapherScene::ImageViewer(_data) => {
                // no-op
            }
            _ => unimplemented!(),
        }
    }

    pub fn handle_resize(
        &mut self,
        device: &Device,
        queue: &Queue,
        surface_config: &SurfaceConfiguration,
    ) {
        self.rebuild_shadow_state(device, surface_config);
        if let GrapherScene::Solver(data) = self {
            data.handle_resize(queue, surface_config);
        }
    }

    fn rebuild_shadow_state(&mut self, device: &Device, surface_config: &SurfaceConfiguration) {
        if let GrapherScene::Graph(data) = self
            && let Some(scene) = &mut data.graph_scene.scene
            && !scene.meshes.is_empty()
        {
            let shadow = ShadowState::create::<GpuVertex>(
                surface_config,
                device,
                &scene.light,
                MeshRenderData::matrix_bgl(device),
            );
            scene.shadow = Some(shadow);
        }
    }
}

// ------------------------------
// Grapher renderer parameter ui.

#[derive(Default)]
pub struct RenderUiState {
    pub lighting_enabled: bool,
    pub use_wireframe: bool,
    pub shadow_enabled: bool,
    pub needs_prefs_uniform_write: bool,
}

impl From<&RenderPreferences> for RenderUiState {
    fn from(render_prefs: &RenderPreferences) -> Self {
        Self {
            lighting_enabled: render_prefs.lighting_enabled(),
            use_wireframe: render_prefs.wireframe_enabled(),
            shadow_enabled: render_prefs.shadow_enabled(),
            needs_prefs_uniform_write: false,
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
            render_ui_state.needs_prefs_uniform_write = true;
        }

        if matches!(grapher_scene, GrapherScene::Graph(_)) {
            let response = ui.checkbox(&mut render_ui_state.use_wireframe, "Wireframe ");
            if response.changed() {
                render_state
                    .render_preferences
                    .set_wireframe(render_ui_state.use_wireframe);
                // we recreate the pipeline on (rare) change of poly mode
                grapher_scene.set_needs_rebuild(true);
            }
        }
    });
    if matches!(grapher_scene, GrapherScene::Graph(_)) {
        let response = ui.checkbox(&mut render_ui_state.shadow_enabled, "Shadow ");
        if response.changed() {
            render_state
                .render_preferences
                .set_shadow_enabled(render_ui_state.shadow_enabled);
            render_ui_state.needs_prefs_uniform_write = true;
        }
    }
    let response = ui.checkbox(
        &mut render_state.camera_state.camera.relative_rotation,
        "Relative rotation",
    );
    if response.changed() {
        render_state
            .camera_state
            .camera
            .on_relative_rotation_change();
    }
}
