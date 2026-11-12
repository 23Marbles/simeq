use std::collections::HashMap;

use num::BigRational;

mod display;
mod parser;
pub mod solver;

/// An equation of the form `x + y + z = sum`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    pub variables: HashMap<String, BigRational>,
    pub sum: BigRational,
}

impl Equation {
    pub fn add_term(&mut self, name: String, coefficient: BigRational) {
        let entry = self.variables.entry(name);

        match entry {
            std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                let current_coefficient = occupied_entry.into_mut();

                *current_coefficient += coefficient
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert_entry(coefficient);
            }
        }
    }
}
