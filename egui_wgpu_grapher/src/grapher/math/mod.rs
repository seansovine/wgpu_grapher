//! Code to build scenes containing mathematical objects.
//! Currently used for building a 3D function graph scene.

pub mod graph;

use graph::GraphableFunc;
use meval::Expr;

#[allow(dead_code)]
pub mod pde;

// ----------------------------------------------
// Abstract over different function object types.

pub struct FunctionHolder {
    pub f: Box<dyn Fn(f64, f64) -> f64>,
}

impl<F> From<F> for FunctionHolder
where
    F: Fn(f64, f64) -> f64 + 'static,
{
    fn from(value: F) -> Self {
        Self {
            f: Box::from(value),
        }
    }
}

impl GraphableFunc for FunctionHolder {
    fn eval(&self, x: f64, y: f64) -> f64 {
        (self.f)(x, y)
    }
}

// ----------------------------------------------
// Try to create function object from user input.

pub fn try_parse_function_string(function_string: &str) -> Option<FunctionHolder> {
    let mut function = None;
    if let Ok(expr) = function_string.parse::<Expr>()
        && let Ok(func) = expr.bind2("x", "z")
    {
        function = Some(FunctionHolder { f: Box::from(func) });
    }
    function
}

// ----------------------------------------------
// Function wrapper that convolves with Gaussian.

pub struct SmoothingFunctionWrapper {
    pub f: Box<dyn Fn(f64, f64) -> f64>,

    coefficients: SmoothingKernel,
    increment: f64,
}

impl SmoothingFunctionWrapper {
    /// `radius`: Scale of square on which to evaluate average values.
    pub fn from<F>(value: F, radius: f64) -> Self
    where
        F: Fn(f64, f64) -> f64 + 'static,
    {
        let coefficients = gaussian_coefficients();
        let increment = radius / coefficients.dim as f64;
        Self {
            f: Box::from(value),
            coefficients,
            increment,
        }
    }
}

impl GraphableFunc for SmoothingFunctionWrapper {
    fn eval(&self, x: f64, z: f64) -> f64 {
        let mut result = 0.0f64;
        let dim = self.coefficients.dim;
        for i in 0..dim {
            for j in 0..dim {
                let x_e = x + self.increment * (i as isize - dim as isize / 2) as f64;
                let z_e = z + self.increment * (j as isize - dim as isize / 2) as f64;
                result += (self.f)(x_e, z_e) * self.coefficients[(i, j)];
            }
        }
        result
    }
}

pub struct SmoothingKernel {
    pub dim: usize,
    pub coefficients: Vec<f64>,
}

/// Make a smoothing kernel by sampling from bivariate Gaussian
/// on grid in region containing >= 95% of probability mass.
pub fn gaussian_coefficients() -> SmoothingKernel {
    let std_normal = |x: f64, z: f64| -> f64 {
        std::f64::consts::E.powf(-0.5 * (x.powi(2) + z.powi(2))) //
    };

    const RADIUS: f64 = 2.5;
    const DIM: usize = 9;
    let mut coefficients = vec![0.0f64; DIM * DIM];
    let increment: f64 = RADIUS / ((DIM / 2) as f64);

    let mut sum = 0.0f64;
    for i in 0..DIM {
        for j in 0..DIM {
            let x = (i as i32 - DIM as i32 / 2) as f64 * increment;
            let y = (j as i32 - DIM as i32 / 2) as f64 * increment;
            coefficients[i * DIM + j] = std_normal(x, y);
            sum += coefficients[i * DIM + j];
        }
    }
    coefficients.iter_mut().for_each(|c| *c /= sum);

    SmoothingKernel {
        dim: DIM,
        coefficients,
    }
}

impl std::ops::Index<(usize, usize)> for SmoothingKernel {
    type Output = f64;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.coefficients[index.0 * self.dim + index.1]
    }
}
