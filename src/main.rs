use std::{collections::HashMap, io::stdin, str::FromStr};

use num::BigRational;

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

    println!("{equations:?}");
}
