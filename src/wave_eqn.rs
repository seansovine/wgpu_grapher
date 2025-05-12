// Finite-difference wave equation solver.

use rand::Rng;
use rand::rngs::ThreadRng;

pub const X_SIZE: usize = 256;
pub const Y_SIZE: usize = 256;

const PROP_SPEED: f32 = 0.0625;
const DAMPING_FACTOR: f32 = 0.995;

const DISTURBANCE_PROB: f32 = 0.01;
const DISTURBANCE_SIZE: f32 = 50.0;

pub struct WaveEquationData {
  // current timestep data
  pub u_0: [[f32; Y_SIZE]; X_SIZE],
  // previous timestep data
  u_1: [[f32; Y_SIZE]; X_SIZE],
  // 2x previous data
  u_2: [[f32; Y_SIZE]; X_SIZE],
  // propagation speed
  k: f32,
  // random number generator
  rng: ThreadRng,
}

impl WaveEquationData {
  pub fn new() -> Self {
    Self {
      u_0: [[0.0; Y_SIZE]; X_SIZE],
      u_1: [[0.0; Y_SIZE]; X_SIZE],
      u_2: [[0.0; Y_SIZE]; X_SIZE],
      k: PROP_SPEED,
      rng: rand::rng(),
    }
  }

  pub fn update(&mut self) {
    self.add_random_disturbance();

    // shift current and previous back one timestep
    self.u_2 = self.u_1;
    self.u_1 = self.u_0;

    let u_1 = &self.u_1;
    let u_2 = &self.u_2;

    // update current internal points; boundary held at 0
    for i in 1..X_SIZE - 1 {
      for j in 1..Y_SIZE - 1 {
        // next finite difference step
        self.u_0[i][j] = self.k
          * (u_1[i - 1][j] + u_1[i + 1][j] + u_1[i][j - 1] + u_1[i][j + 1] - 4.0 * u_1[i][j])
          + 2.0 * u_1[i][j]
          - u_2[i][j];

        // add damping, following Beltoforion's example
        self.u_0[i][j] *= DAMPING_FACTOR;
      }
    }
  }

  pub fn add_random_disturbance(&mut self) {
    // following Beltoforion's example,
    // add a random disturbance to the space
    if self.rng.random::<f32>() < DISTURBANCE_PROB {
      let x: usize = self.rng.random_range(5..X_SIZE - 5);
      let y: usize = self.rng.random_range(5..Y_SIZE - 5);

      for i in x - 2..x + 2 {
        for j in y - 2..y + 2 {
          self.u_0[i][j] = DISTURBANCE_SIZE;
        }
      }
    }
  }
}
