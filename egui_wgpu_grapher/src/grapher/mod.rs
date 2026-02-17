//! Code for building, storing, and rendering 3D and 2D scenes using Wgpu.
//!
//! We currently have two main scene formats:
//!  - A 3D scene with separate pipelines for textured and vertex-colored meshes.
//!  - A 2D scene for running a finite-difference solver compute pipeline and
//!    rendering the results to a fixed 2D canvas texture.
//!
//! Within the 3D scene format there are several types, including a function
//! grapher and a glTF model viewer.

mod camera;
mod gltf_loader;
mod matrix;

pub mod math;
pub mod pipeline;
pub mod render;
pub mod scene;
