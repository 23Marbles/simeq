use std::{collections::HashMap, io::stdin, str::FromStr};

use num::BigRational;

use crate::solver::LinearSystem;

mod parser;
mod solver;

#[derive(Debug, Default)]
struct Equation {
    variables: HashMap<String, BigRational>,
    sum: BigRational,
}

fn main() {
    let mut equations = Vec::new();

    let mut buf = String::new();

    loop {
        let bytes_read = stdin().read_line(&mut buf).expect("Failed to read line");

        if bytes_read == 0 || buf.trim().is_empty() {
            break;
        }

        match Equation::from_str(&buf) {
            Ok(eq) => equations.push(eq),
            Err(err_msg) => println!("Parse error: {err_msg}"),
        }

        buf.clear();
    }

    let linear_system = LinearSystem::from_equation_vec(equations);

    let mut matrix = linear_system.augmented_matrix().clone();

    println!("Augmented matrix:\n{}", matrix);

    matrix.reorder_triangle();

    println!("Augmented matrix:\n{}", matrix);

    matrix.row_echelon_form();

    println!("Augmented matrix:\n{}", matrix);
}
