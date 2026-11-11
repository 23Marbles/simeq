use std::collections::{HashMap, HashSet};

use num::BigRational;

use crate::{Equation, solver::augmented_matrix::AugmentedMatrix};

pub mod augmented_matrix;
mod display;

/// Ax = b
#[derive(Debug, Clone)]
pub struct LinearSystem {
    height: usize,
    width: usize,

    // A and b
    augmented_matrix: AugmentedMatrix,

    /// The variables
    // x.len() == width
    x: Vec<String>,
}

pub enum Solution {
    Unique(HashMap<String, BigRational>),
    Infinite {
        pivots: HashMap<String, Equation>,
        free_variables: HashSet<String>,
    },
    None,
}

impl LinearSystem {
    pub fn from_equation_vec(vec: Vec<Equation>) -> Self {
        let mut variables = Vec::new();

        let mut augmented_matrix = AugmentedMatrix::default();

        for equation in vec {
            let Equation {
                variables: mut eq_variables,
                sum,
            } = equation;

            let mut row = Vec::new();

            for name in &variables {
                row.push(eq_variables.remove(name).unwrap_or(BigRational::ZERO));
            }

            for (name, value) in eq_variables.into_iter() {
                variables.push(name);

                augmented_matrix.add_column();
                row.push(value);
            }

            augmented_matrix.add_row(row, -sum);
        }

        Self {
            height: augmented_matrix.height(),
            width: augmented_matrix.width(),
            augmented_matrix,
            x: variables,
        }
    }
}

impl LinearSystem {
    pub fn augmented_matrix(&self) -> &AugmentedMatrix {
        &self.augmented_matrix
    }

    pub fn row_echelon_form(&mut self) {
        self.augmented_matrix.row_echelon_form();
    }
    
    pub fn reduced_row_echelon_form(&mut self) {
        self.augmented_matrix.reduced_row_echelon_form();
    }

    /// https://en.wikipedia.org/wiki/Gaussian_elimination
    pub fn solve(&mut self) -> Solution {
        self.augmented_matrix.reduced_row_echelon_form();

        todo!()
    }
}
