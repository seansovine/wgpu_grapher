pub mod graph;

use graph::GraphableFunc;
use meval::Expr;

#[allow(dead_code)]
pub mod pde;

pub struct FunctionHolder {
    pub f: Box<dyn Fn(f32, f32) -> f32>,
}

impl<F> From<F> for FunctionHolder
where
    F: Fn(f32, f32) -> f32 + 'static,
{
    fn from(value: F) -> Self {
        Self {
            f: Box::from(value),
        }
    }
}

impl GraphableFunc for FunctionHolder {
    fn eval(&self, x: f32, y: f32) -> f32 {
        (self.f)(x, y)
    }
}

pub fn try_parse_function_string(function_string: &str) -> Option<FunctionHolder> {
    let mut function = None;
    if let Ok(expr) = function_string.parse::<Expr>() {
        if let Ok(func) = expr.bind2("x", "z") {
            let closure = move |x: f32, z: f32| -> f32 { func(x as f64, z as f64) as f32 };
            function = Some(FunctionHolder {
                f: Box::from(closure),
            });
        }
    }

    function
}
