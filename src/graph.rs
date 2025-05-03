// TODO: Add code to build mesh(es) for graphing a function y = f(x, z).
//
// This will include:
//
//  - a function that tesselates the (x, z) unit square with uniform subsquares
//  - a function to generate the vertex and index sets from squares
//    - we'll want to consider front & back face rendering
//  - a function to update the vertex sets above from an (x, y) -> z closure
//  - mechanisms to decorate such closures to scale and shift inputs and outputs
//
// This will do:
//
//   This will create the MeshData and matrix uniform objects used by mesh::build_scene.

use crate::mesh::{self, MeshData};

pub struct Square {
  // vertex indices of corners
  //  CCW from bottom-left
  corners: [u16; 4],
}

impl Square {
  #[rustfmt::skip]
  fn triangle_vertices(&self) -> [u16; 12] {
		let c = &self.corners;
		[
			// top faces
			c[0], c[2], c[3],
			c[0], c[1], c[2],
			// bottom faces
			c[0], c[3], c[2],
			c[0], c[2], c[1],
		]
	}
}

struct Vertex([f32; 3]);

pub struct UnitSquareTesselation {
  // number of squares to subdivide in each direction
  _n: u16,
  vertices: Vec<Vertex>,
  squares: Vec<Square>,
}

impl UnitSquareTesselation {
  pub const FLOOR_COLOR: [f32; 3] = [168.0f32 / 255.0f32, 125.0f32 / 255.0f32, 50.0f32 / 255.0f32];
  pub const FUNCT_COLOR: [f32; 3] = [1.0, 0.0, 0.0];

  /// build tesselation of (x, z) coordinate system
  pub fn generate(n: u16) -> Self {
    let mut ticks: Vec<f32> = vec![];

    for i in 0..=n {
      ticks.push(i as f32 * (1.0f32 / n as f32));
    }

    let mut vertices = vec![];

    // flattened order is important here:
    //  we go across rows from left to right,
    //  visiting rows from bottom to top
    for z in &ticks {
      for x in &ticks {
        vertices.push(Vertex([*x, 0.0, *z]));
      }
    }

    let mut squares = vec![];

    // x and y are indices here, not coordinates
    // not n squares per row/column means n+1 ticks
    for z in 0..n {
      for x in 0..n {
        squares.push(Square {
          corners: [
            z * (n + 1) + x,
            z * (n + 1) + (x + 1),
            (z + 1) * (n + 1) + (x + 1),
            (z + 1) * (n + 1) + x,
          ],
        })
      }
    }

    UnitSquareTesselation {
      _n: n,
      vertices,
      squares,
    }
  }

  pub fn apply_function<F>(&mut self, f: F) -> &mut Self
  where
    F: Fn(f32, f32) -> f32,
  {
    for vertex in &mut self.vertices {
      vertex.0[1] = f(vertex.0[0], vertex.0[2])
    }

    self
  }

  pub fn mesh_data(&self, color: [f32; 3]) -> MeshData {
    let mut vertices = vec![];

    for vertex in &self.vertices {
      vertices.push(mesh::Vertex {
        position: vertex.0,
        color,
      });
    }

    let mut indices: Vec<u16> = vec![];

    self.squares.iter().for_each(|square| {
      indices.extend_from_slice(&square.triangle_vertices());
    });

    MeshData { vertices, indices }
  }
}

// function modification helpers

pub fn shift_scale_input<F>(
  f: F,
  x_shift: f32,
  x_scale: f32,
  z_shift: f32,
  z_scale: f32,
) -> impl Fn(f32, f32) -> f32
where
  F: Fn(f32, f32) -> f32,
{
  move |x: f32, z: f32| f((x - x_shift) * x_scale, (z - z_shift) * z_scale)
}

pub fn shift_scale_output<F>(f: F, y_shift: f32, y_scale: f32) -> impl Fn(f32, f32) -> f32
where
  F: Fn(f32, f32) -> f32,
{
  move |x: f32, z: f32| f(x, z) * y_scale + y_shift
}
