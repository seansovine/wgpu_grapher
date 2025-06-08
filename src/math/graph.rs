// Code to build mesh(es) for graphing a function y = f(x, z).
//
// This includes:
//
//  - a function that tesselates an (x, z) square with uniform subsquares
//  - a function to generate the vertex and index sets from squares
//  - a function to update the vertex sets above from an (x, z) -> y closure
//     (note we're working in OpenGL coordinate system)
//  - mechanisms to decorate function closures to scale and shift inputs and outputs
//
// This will create the MeshData object used by mesh::build_scene.
//

use crate::mesh::{self, MeshData};

// geometric data

pub struct Triangle {
  // should be ordered counter clockwise
  vertex_indices: [u32; 3],

  // to give bottom of graph same lighting as top
  reflect_normal: bool,
}

impl Triangle {
  fn create(i_1: u32, i_2: u32, i_3: u32, reflect_normal: bool) -> Self {
    Self {
      vertex_indices: [i_1, i_2, i_3],
      reflect_normal,
    }
  }

  fn compute_normal(&self, vertices: &[Vertex]) -> [f32; 3] {
    let v_1 = &vertices[self.vertex_indices[0] as usize];
    let v_2 = &vertices[self.vertex_indices[1] as usize];
    let v_3 = &vertices[self.vertex_indices[2] as usize];

    // first side
    let b = [v_2[0] - v_1[0], v_2[1] - v_1[1], v_2[2] - v_1[2]];
    // second side
    let a = [v_3[0] - v_1[0], v_3[1] - v_1[1], v_3[2] - v_1[2]];

    // normal vector by cross product
    let normal = [
      a[1] * b[2] - a[2] * b[1],
      a[2] * b[0] - a[0] * b[2],
      a[0] * b[1] - a[1] * b[0],
    ];
    // normalize
    let mut norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    if self.reflect_normal {
      norm *= -1.0;
    }

    [normal[0] / norm, normal[1] / norm, normal[2] / norm]
  }
}

pub struct Square {
  // vertex indices of corners CW from back-left
  corner_indices: [u32; 4],
}

impl Square {
  fn triangles(&self) -> [Triangle; 4] {
    let c = &self.corner_indices;
    [
      // top faces
      Triangle::create(c[0], c[2], c[3], false),
      Triangle::create(c[0], c[1], c[2], false),
      // bottom faces w/ reflected normals
      Triangle::create(c[0], c[3], c[2], true),
      Triangle::create(c[0], c[2], c[1], true),
    ]
  }
}

// square tesselation

type Vertex = [f32; 3];

pub struct SquareTesselation {
  // # of squares to subdivide into in each direction
  #[allow(unused)]
  n: u32,
  vertices: Vec<Vertex>,
  squares: Vec<Square>,
}

impl SquareTesselation {
  // color to use for (x, z) plane "floor" mesh
  pub const FLOOR_COLOR: [f32; 3] = [
    0.8 * 168.0f32 / 255.0f32,
    0.8 * 125.0f32 / 255.0f32,
    0.8 * 50.0f32 / 255.0f32,
  ];

  // color to use for function mesh
  pub const FUNCT_COLOR: [f32; 3] = [1.0, 0.0, 0.0];

  /// Build tesselation of \[0, width\] x \[0, width\] square
  /// in \(x, z\) coordinate system by smaller squares.
  pub fn generate(n: u32, width: f32) -> Self {
    let mut ticks: Vec<f32> = vec![];
    let mut vertices: Vec<Vertex> = vec![];
    let mut squares = vec![];

    // compute axis subdivision points
    for i in 0..=n {
      ticks.push(i as f32 * (width / n as f32));
    }

    // NOTES:
    // - Flattened order is important here: We go across rows
    //   from left to right, visiting rows from back to front.
    for z in &ticks {
      for x in &ticks {
        vertices.push([*x, 0.0, *z]);
      }
    }

    // NOTES:
    // - x and z are indices here, not coordinates.
    // - n squares per row/column means n+1 ticks
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

    SquareTesselation {
      n,
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
    let mut indices: Vec<u32> = vec![];
    let mut normals: Vec<Option<[f32; 3]>> = vec![None; self.vertices.len()];
    let mut vertices: Vec<mesh::Vertex> = vec![];

    for square in &self.squares {
      let triangles = square.triangles();

      for t in &triangles {
        indices.extend_from_slice(&t.vertex_indices);
        for v in t.vertex_indices.map(|v| v as usize) {
          if normals[v].is_none() {
            normals[v] = Some(t.compute_normal(&self.vertices));
          }
        }
      }
    }

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
