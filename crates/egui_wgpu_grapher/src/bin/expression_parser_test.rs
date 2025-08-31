use std::io;

use meval::Expr;

fn main() {
    loop {
        println!("Enter a function of x and z to evaluate: ");
        let mut str_expr = String::new();
        io::stdin()
            .read_line(&mut str_expr)
            .expect("Failed to read input.");

        let Ok(expr) = str_expr.parse::<Expr>() else {
            println!("Failed to parse str_expr as mathematical expression.");
            continue;
        };
        println!("Successfully parsed expression.");

        let Ok(func) = expr.bind2("x", "z") else {
            println!("Expression does not contain 'x' and 'z' as its only variables.");
            continue;
        };
        println!("Successfully defined Func(x, z) = {str_expr}");
        println!();

        for x in 0..5 {
            for z in 0..5 {
                let x = (x as f64) / 2.0_f64;
                let z = (z as f64) / 2.0_f64;
                let y = func(x, z);

                println!("Func({x}, {z}) = {y}");
            }
        }
        println!();
    }
}
