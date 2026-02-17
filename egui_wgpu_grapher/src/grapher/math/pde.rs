//! Finite-difference solvers for the wave and heat equations.
//!
//! These are currently unused, but there is a GPU version of the
//! wave equation solver in the `scene::solver` module.

use rand::{Rng, rngs::ThreadRng};

pub const X_SIZE: usize = 500;
pub const Y_SIZE: usize = 500;

const PROP_SPEED: f32 = 0.35;
const DAMPING_FACTOR: f32 = 0.995;

const DISTURBANCE_PROB: f32 = 0.02;
const DISTURBANCE_SIZE: f32 = 80.0;

pub struct WaveEquationData {
    // current timestep data
    pub u_0: Vec<Vec<f32>>,

    // previous timestep data
    u_1: Vec<Vec<f32>>,

    // 2x previous data
    u_2: Vec<Vec<f32>>,

    // random number generator
    rng: ThreadRng,

    // grid size
    pub x_size: usize,
    pub y_size: usize,

    // parameters
    pub prop_speed: f32,
    pub damping_factor: f32,
    pub disturbance_prob: f32,
    pub disturbance_size: f32,
}

impl WaveEquationData {
    pub fn new(x_size: usize, y_size: usize) -> Self {
        Self {
            u_0: vec![vec![0.0; y_size]; x_size],
            u_1: vec![vec![0.0; y_size]; x_size],
            u_2: vec![vec![0.0; y_size]; x_size],
            rng: rand::rng(),
            //
            x_size,
            y_size,
            //
            prop_speed: PROP_SPEED,
            damping_factor: DAMPING_FACTOR,
            disturbance_prob: DISTURBANCE_PROB,
            disturbance_size: DISTURBANCE_SIZE,
        }
    }

    pub fn update(&mut self) {
        self.add_random_disturbance();

        for i in 0..self.x_size {
            // shift current and previous back one timestep
            self.u_2[i][0..self.y_size].copy_from_slice(&self.u_1[i]);
            self.u_1[i][0..self.y_size].copy_from_slice(&self.u_0[i]);
        }

        let u_1 = &self.u_1;
        let u_2 = &self.u_2;

        // update current internal points; boundary held at 0
        for i in 1..self.x_size - 1 {
            for j in 1..self.y_size - 1 {
                // next finite difference step
                self.u_0[i][j] = self.prop_speed
                    * (u_1[i - 1][j] + u_1[i + 1][j] + u_1[i][j - 1] + u_1[i][j + 1]
                        - 4.0 * u_1[i][j])
                    + 2.0 * u_1[i][j]
                    - u_2[i][j];

                // add damping, following Beltoforion's example
                self.u_0[i][j] *= self.damping_factor;
            }
        }
    }

    pub fn add_random_disturbance(&mut self) {
        // following Beltoforion's example,
        // add a random disturbance to the space
        if self.rng.random::<f32>() < self.disturbance_prob {
            let x: usize = self.rng.random_range(4..self.x_size - 5);
            let y: usize = self.rng.random_range(4..self.y_size - 5);

            const B: usize = 5;

            // add random bump decaying like 1 / r^3
            for i in B..self.y_size - B {
                for j in B..self.x_size - B {
                    let dist = ((j - x).pow(2) as f64 + (i - y).pow(2) as f64)
                        .powf(3.0 / 2.0)
                        .max(2.0) as f32;
                    self.u_0[i][j] += self.disturbance_size / dist;
                }
            }
        }
    }
}

// Finite-difference heat equation solver.

pub struct HeatEquationData {
    // current timestep data
    pub u: Vec<[f32; 2]>,
    pub current_index: usize,

    // grid size
    pub x_size: usize,
    pub y_size: usize,

    // parameters

    // time difference increment
    pub k: f32,

    // space difference increment
    pub h: f32,

    // diffusivity constant
    pub d: f32,
    // NOTE: For stability we need d * k / h^2 < 1/2.
}

impl HeatEquationData {
    pub fn new(x_size: usize, y_size: usize) -> Self {
        let mut new_self = Self {
            u: vec![[0.0, 0.0]; x_size * y_size],
            current_index: 0,
            //
            x_size,
            y_size,
            //
            k: 0.25, // want dk / h^2 < 1/2
            h: 1.0,
            d: 1.0,
        };

        // x, z width
        let init_width = 150_usize;
        // y height
        let init_height = 10.0_f32;

        // set initial condition
        for i in 0..init_width {
            for j in 0..init_width {
                let offset = (new_self.y_size / 2 - init_width / 2 + i) * new_self.x_size
                    + (new_self.x_size / 2 - init_width / 2 + j);
                new_self.u[offset][0] = init_height;
            }
        }

        // add a boundary condition
        for i in 0..x_size {
            new_self.u[i][0] = init_height * (i as f32 / 20.0).sin() / 2.0;
            new_self.u[i + (y_size - 1) * x_size][0] = init_height * (i as f32 / 20.0).sin() / 2.0;
        }

        new_self
    }

    pub fn update(&mut self) {
        // previous time index
        let t_0 = self.current_index;
        // new time index
        let t = (self.current_index + 1) % 2;

        // update interior points
        for y in 1..self.y_size - 1 {
            for x in 1..self.x_size - 1 {
                // du/dt = v + CD * Laplacian(u)
                self.u[y * self.x_size + x][t] = self.u[y * self.x_size + x][t_0]
                    + self.k
                        * (self.d
                // discrete laplacian
                * (-4.0 * self.u[y * self.x_size + x][t_0]
                  + self.u[y * self.x_size + x - 1][t_0]
                  + self.u[y * self.x_size + x + 1][t_0]
                  + self.u[(y - 1) * self.x_size + x][t_0]
                  + self.u[(y + 1) * self.x_size + x][t_0])
                            / self.h.powi(2));
            }
        }

        // set current time index to new
        self.current_index = t;
    }
}
