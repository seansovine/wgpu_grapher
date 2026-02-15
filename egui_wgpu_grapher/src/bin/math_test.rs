//! Dev test file for math functions.

use egui_wgpu_grapher::grapher::math;

fn main() {
    let coefficients = math::gaussian_coefficients();
    let mut sum = 0.0f64;
    for i in 0..coefficients.dim {
        for j in 0..coefficients.dim {
            print!("{:8.6}", coefficients[(i, j)]);
            sum += coefficients[(i, j)];
        }
        println!();
    }
    println!("Coefficient sum: {sum}");
}
