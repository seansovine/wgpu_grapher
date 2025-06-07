// Code to build mesh(es) for graphing a function y = f(x, z).
//
// This includes:
//
//  - a function that tesselates the (x, z) unit square with uniform subsquares
//  - a function to generate the vertex and index sets from squares
//    - we'll want to consider front & back face rendering
//  - a function to update the vertex sets above from an (x, z) -> y closure
//    - (note we're working in OpenGL coordinate system)
//  - mechanisms to decorate such closures to scale and shift inputs and outputs
//
// This will do:
//
//   This will create the MeshData and matrix uniform objects used by mesh::build_scene.

use crate::mesh::{self, MeshData};

pub struct Triangle {
  // should be ordered counter clockwise
  vertex_indices: [u16; 3],
  normal: [f32; 3],
}

impl Triangle {
  fn create(i_1: u16, i_2: u16, i_3: u16, vertices: &[Vertex]) -> Self {
    let v_1 = &vertices[i_1 as usize];
    let v_2 = &vertices[i_2 as usize];
    let v_3 = &vertices[i_3 as usize];

    // first side
    let a = [v_2[0] - v_1[0], v_2[1] - v_1[1], v_2[2] - v_1[2]];
    // second side
    let b = [v_3[0] - v_1[0], v_3[1] - v_1[1], v_3[2] - v_1[2]];

    // normal vector by cross product
    let normal = [
      a[1] * b[2] - a[2] * b[1],
      a[2] * b[0] - a[0] * b[2],
      a[0] * b[1] - a[1] * b[0],
    ];
    // normalize
    let norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    let normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];

    Self {
      vertex_indices: [i_1, i_2, i_3],
      normal,
    }
  }
}

impl From<Triangle> for [u16; 3] {
  fn from(value: Triangle) -> Self {
    value.vertex_indices
  }
}

pub struct Square {
  // vertex indices of corners CW from back-left
  corner_indices: [u16; 4],
}

impl Square {
  // #[rustfmt::skip]
  fn triangles(&self, vertices: &[Vertex]) -> [Triangle; 2] {
    let c = &self.corner_indices;
    [
      // bottom faces
      // c[0], c[2], c[3],
      // c[0], c[1], c[2],

      // TODO: We need to think about inverted normals
      // for bottom faces.

      // top faces
      Triangle::create(c[0], c[3], c[2], vertices),
      Triangle::create(c[0], c[2], c[1], vertices),
    ]
  }
}

type Vertex = [f32; 3];

pub struct UnitSquareTesselation {
  // number of squares to subdivide in each direction
  _n: u16,
  vertices: Vec<Vertex>,
  squares: Vec<Square>,
}

impl UnitSquareTesselation {
  pub const FLOOR_COLOR: [f32; 3] = [
    0.8 * 168.0f32 / 255.0f32,
    0.8 * 125.0f32 / 255.0f32,
    0.8 * 50.0f32 / 255.0f32,
  ];
  pub const FUNCT_COLOR: [f32; 3] = [1.0, 0.0, 0.0];

  /// build tesselation of (x, z) coordinate system
  pub fn generate(n: u16, width: f32) -> Self {
    let mut ticks: Vec<f32> = vec![];

    for i in 0..=n {
      ticks.push(i as f32 * (width / n as f32));
    }

    let mut vertices: Vec<Vertex> = vec![];

    // flattened order is important here:
    //  we go across rows from left to right, visiting rows from back to front
    for z in &ticks {
      for x in &ticks {
        vertices.push([*x, 0.0, *z]);
      }
    }

    let mut squares = vec![];

    // x and z are indices here, not coordinates.
    //  note: n squares per row/column means n+1 ticks
    for z in 0..n {
      for x in 0..n {
        squares.push(Square {
          corner_indices: [
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
      vertex[1] = f(vertex[0], vertex[2])
    }

    self
  }

  pub fn mesh_data(&self, color: [f32; 3]) -> MeshData {
    let mut indices: Vec<u16> = vec![];
    let mut normals: Vec<Option<[f32; 3]>> = vec![None; self.vertices.len()];

    self.squares.iter().for_each(|square| {
      let triangles = &square.triangles(&self.vertices);

      let t1 = &triangles[0];
      let t2 = &triangles[1];

      indices.extend_from_slice(&t1.vertex_indices);
      indices.extend_from_slice(&t2.vertex_indices);

      for t in &[t1, t2] {
        for v in t.vertex_indices.map(|v| v as usize) {
          if normals[v].is_none() {
            normals[v] = Some(t.normal);
          }
        }
      }
    });

    let mut vertices: Vec<mesh::Vertex> = vec![];

    for (i, vertex) in self.vertices.iter().enumerate() {
      vertices.push(mesh::Vertex {
        position: *vertex,
        color,
        normal: normals[i].take().unwrap(),
      });
    }

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
  // new closure takes ownership of old one
  move |x: f32, z: f32| f((x - x_shift) * x_scale, (z - z_shift) * z_scale)
}

pub fn shift_scale_output<F>(f: F, y_shift: f32, y_scale: f32) -> impl Fn(f32, f32) -> f32
where
  F: Fn(f32, f32) -> f32,
{
  // new closure takes ownership of old one
  move |x: f32, z: f32| f(x, z) * y_scale + y_shift
}
