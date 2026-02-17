//! Code to build mesh(es) for graphing a function y = f(x, z).
//!
//! This includes:
//!
//!  - a function that tessellates an (x, z) square with uniform subsquares
//!  - a function to generate the vertex and index sets from squares
//!  - a function to update the vertex sets above from an (x, z) -> y closure
//!  - mechanisms to decorate function closures to scale and shift inputs and outputs
//!  - functions to compute normal vectors for mesh triangles
//!
//! Note that we're working in OpenGL coordinate system, so y is "up".
//!
//! This will create the MeshData object used by mesh::build_scene.

use crate::grapher::scene::{self, solid::MeshData};

use std::vec;

// geometric data

type Vertex = [f32; 3];

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

        triangle_normal(v_1, v_2, v_3, self.reflect_normal)
    }
}

#[inline(always)]
fn triangle_normal(v_1: &Vertex, v_2: &Vertex, v_3: &Vertex, reflect: bool) -> [f32; 3] {
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
    if reflect {
        norm *= -1.0;
    }

    [normal[0] / norm, normal[1] / norm, normal[2] / norm]
}

#[inline(always)]
fn normal_from_function<F: GraphableFunc>(v: &Vertex, f: &F) -> [f32; 3] {
    const H: f64 = 1e-6;
    let dydx: f64 =
        (f.eval(v[0] as f64 + H, v[2] as f64) - f.eval(v[0] as f64 - H, v[2] as f64)) / (2.0 * H);
    let dzdx: f64 =
        (f.eval(v[0] as f64, v[2] as f64 + H) - f.eval(v[0] as f64, v[2] as f64 - H)) / (2.0 * H);
    let mag = (dydx.powi(2) + 1.0 + dydx.powi(2)).sqrt();
    [(-dydx / mag) as f32, 1.0 / mag as f32, (-dzdx / mag) as f32]
}

pub struct Square {
    // vertex indices of corners CW from back-left
    corner_indices: [u32; 4],
}

impl Square {
    fn triangles(&self, flip: bool) -> [Triangle; 4] {
        let c = &self.corner_indices;
        if flip {
            [
                // top faces
                Triangle::create(c[0], c[1], c[3], false),
                Triangle::create(c[1], c[2], c[3], false),
                // bottom faces w/ reflected normals
                Triangle::create(c[0], c[3], c[1], true),
                Triangle::create(c[1], c[3], c[2], true),
            ]
        } else {
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
}

// Graphable function trait.

pub trait GraphableFunc {
    fn eval(&self, x: f64, y: f64) -> f64;
}

// square tesselation

pub struct SquareTesselation {
    #[allow(unused)]
    // # of squares to subdivide into in each direction
    n: u32,

    // all vertices in mesh
    vertices: Vec<Vertex>,

    // list of squares in the tesselation
    squares: Vec<Square>,
}

impl SquareTesselation {
    // color to use for (x, z) plane "floor" mesh
    pub const _FLOOR_COLOR: [f32; 3] = [
        0.8 * 168.0f32 / 255.0f32,
        0.8 * 125.0f32 / 255.0f32,
        0.8 * 50.0f32 / 255.0f32,
    ];

    // color to use for function mesh
    pub const FUNC_COLOR: [f32; 3] = [1.0, 0.0, 0.0];

    /// Build tesselation of \[0, width\] x \[0, width\] square
    /// in \(x, z\) coordinate system by smaller squares.
    pub fn generate<F: GraphableFunc>(n: u32, width: f64, f: &F) -> Self {
        let mut ticks: Vec<f64> = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let mut squares: Vec<Square> = vec![];

        // compute axis subdivision points
        for i in 0..=n {
            ticks.push(i as f64 * (width / n as f64) - width / 2.0);
        }

        // NOTES:
        // - Flattened order is important here: We go across rows
        //   from left to right, visiting rows from back to front.
        for z in &ticks {
            for x in &ticks {
                vertices.push([*x as f32, f.eval(*x, *z) as f32, *z as f32]);
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

    #[allow(unused)]
    pub fn apply_function<F: GraphableFunc>(&mut self, f: &F) -> &mut Self
    where
        F:,
    {
        for vertex in &mut self.vertices {
            vertex[1] = f.eval(vertex[0] as f64, vertex[2] as f64) as f32
        }

        self
    }

    pub fn mesh_data(&self, color: [f32; 3]) -> MeshData {
        let mut indices: Vec<u32> = vec![];
        let mut normals: Vec<Option<[f32; 3]>> = vec![None; self.vertices.len()];
        let mut vertices: Vec<scene::GpuVertex> = vec![];

        for square in &self.squares {
            let diag_1 = (self.vertices[square.corner_indices[0] as usize][1]
                - self.vertices[square.corner_indices[2] as usize][1])
                .abs();
            let diag_2 = (self.vertices[square.corner_indices[1] as usize][1]
                - self.vertices[square.corner_indices[3] as usize][1])
                .abs();
            let flip = diag_1 > diag_2;
            for t in square.triangles(flip) {
                indices.extend_from_slice(&t.vertex_indices);
                for v in t.vertex_indices.map(|v| v as usize) {
                    if normals[v].is_none() {
                        normals[v] = Some(t.compute_normal(&self.vertices));
                    }
                }
            }
        }

        for (i, vertex) in self.vertices.iter().enumerate() {
            vertices.push(scene::GpuVertex {
                position: *vertex,
                color,
                normal: normals[i].take().unwrap(),
                ..Default::default()
            });
        }

        MeshData { vertices, indices }
    }

    pub fn mesh_data_direct_normals<F: GraphableFunc>(&self, color: [f32; 3], f: &F) -> MeshData {
        let mut indices: Vec<u32> = vec![];
        let mut normals: Vec<Option<[f32; 3]>> = vec![None; self.vertices.len()];
        let mut vertices: Vec<scene::GpuVertex> = vec![];

        for square in &self.squares {
            let diag_1 = (self.vertices[square.corner_indices[0] as usize][1]
                - self.vertices[square.corner_indices[2] as usize][1])
                .abs();
            let diag_2 = (self.vertices[square.corner_indices[1] as usize][1]
                - self.vertices[square.corner_indices[3] as usize][1])
                .abs();
            let flip = diag_1 > diag_2;
            for t in square.triangles(flip) {
                indices.extend_from_slice(&t.vertex_indices);
            }
        }

        self.vertices.iter().enumerate().for_each(|(i, vert)| {
            normals[i] = Some(normal_from_function(vert, f));
        });

        for (i, vertex) in self.vertices.iter().enumerate() {
            vertices.push(scene::GpuVertex {
                position: *vertex,
                color,
                normal: normals[i].take().unwrap(),
                ..Default::default()
            });
        }

        MeshData { vertices, indices }
    }

    pub fn update_normals(&self, mesh_data: &mut MeshData) {
        for square in &self.squares {
            // TODO: If this is used we should set flip correctly.
            for t in square.triangles(false) {
                let i_1 = t.vertex_indices[0] as usize;
                let i_2 = t.vertex_indices[1] as usize;
                let i_3 = t.vertex_indices[2] as usize;

                let v_1 = &mesh_data.vertices[i_1].position;
                let v_2 = &mesh_data.vertices[i_2].position;
                let v_3 = &mesh_data.vertices[i_3].position;

                let new_normal = triangle_normal(v_1, v_2, v_3, t.reflect_normal);

                // just update all three normals; some will be updated more than once
                mesh_data.vertices[i_1].normal = new_normal;
                mesh_data.vertices[i_2].normal = new_normal;
                mesh_data.vertices[i_3].normal = new_normal;
            }
        }
    }
}

// function modification helpers

pub fn shift_scale_input<F>(
    f: F,
    x_shift: f64,
    x_scale: f64,
    z_shift: f64,
    z_scale: f64,
) -> impl Fn(f64, f64) -> f64
where
    F: Fn(f64, f64) -> f64,
{
    // new closure takes ownership of old one
    move |x: f64, z: f64| f((x - x_shift) * x_scale, (z - z_shift) * z_scale)
}

pub fn shift_scale_output<F>(f: F, y_shift: f64, y_scale: f64) -> impl Fn(f64, f64) -> f64
where
    F: Fn(f64, f64) -> f64,
{
    // new closure takes ownership of old one
    move |x: f64, z: f64| f(x, z) * y_scale + y_shift
}
