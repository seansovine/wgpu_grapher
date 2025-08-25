use meval::Expr;

fn main() {
    let str_expr = String::from("sin(x^2 + z^2)");
    let Ok(expr) = str_expr.parse::<Expr>() else {
        println!("Failed to parse str_expr as mathematical expression.");
        return;
    };
    println!("Successfully parsed expression.");

    let Ok(func) = expr.bind2("x", "z") else {
        // Note: We may need to handle singl-variable cases separately.
        println!("Expression does not contain 'x' and 'z' as variables.");
        return;
    };
    println!("Successfully defined Func(x, z) = {str_expr}");
    println!();

    for x in 0..5 {
        for z in 0..5 {
            let x = (x as f64) / 2.0_f64;
            let z = (z as f64) / 2.0_f64;
            let y = func(x, z);

            println!("Func({x}, {z}) = {y}");

            let expected = (x.powi(2) + z.powi(2)).sin();

            println!("Expected: {expected}");
        }
    }
}
