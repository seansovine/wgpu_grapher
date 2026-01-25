pub mod graph;

use graph::GraphableFunc;
use meval::Expr;

#[allow(dead_code)]
pub mod pde;

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

pub fn try_parse_function_string(function_string: &str) -> Option<FunctionHolder> {
    let mut function = None;
    if let Ok(expr) = function_string.parse::<Expr>()
        && let Ok(func) = expr.bind2("x", "z")
    {
        function = Some(FunctionHolder { f: Box::from(func) });
    }
    function
}
