use std::collections::HashMap;

use num::BigRational;

mod display;
mod parser;
pub mod solver;

/// An equation of the form `x + y + z = sum`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    variables: HashMap<String, BigRational>,
    sum: BigRational,
}
