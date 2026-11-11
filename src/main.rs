use std::{collections::HashMap, io::stdin, str::FromStr};

use num::BigRational;

use crate::solver::LinearSystem;

mod parser;
mod solver;
mod display;

/// x + y + z = sum
#[derive(Default, Debug, Clone, PartialEq, Eq)]
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

        for line in buf.lines() {
            match Equation::from_str(line.trim()) {
                Ok(eq) => equations.push(eq),
                Err(err_msg) => println!("Parse error: {err_msg}"),
            }
        }

        buf.clear();
    }

    let mut linear_system = LinearSystem::from_equation_vec(equations);

    println!("Linear system:\n{}", linear_system);

    linear_system.row_echelon_form();

    println!("Row Echelon Form Linear system:\n{}", linear_system);

    linear_system.reduced_row_echelon_form();

    println!("Reduced Row Echelon Form Linear system:\n{}", linear_system);
}
