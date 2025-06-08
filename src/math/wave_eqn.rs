// Finite-difference wave equation solver.

use rand::Rng;
use rand::rngs::ThreadRng;

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

  #[allow(unused)]
  pub fn default() -> Self {
    WaveEquationData::new(X_SIZE, Y_SIZE)
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
          * (u_1[i - 1][j] + u_1[i + 1][j] + u_1[i][j - 1] + u_1[i][j + 1] - 4.0 * u_1[i][j])
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
      let x: usize = self.rng.random_range(5..self.x_size - 5);
      let y: usize = self.rng.random_range(5..self.y_size - 5);

      for i in x - 2..x + 2 {
        for j in y - 2..y + 2 {
          self.u_0[i][j] = self.disturbance_size;
        }
      }
    }
  }
}
