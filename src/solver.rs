use std::{collections::{HashMap, HashSet}, str::FromStr};

use num::BigRational;

use crate::{Equation, solver::augmented_matrix::AugmentedMatrix};

pub mod augmented_matrix;
mod display;

/// Ax = b
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl FromStr for LinearSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec: Vec<_> = s.lines().map(|s| Equation::from_str(s)).collect::<Result<_, _>>()?;

        Ok(Self::from_equation_vec(vec))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::solver::LinearSystem;
    
    #[test]
    fn rerunning_ref_causes_no_change() {
        let mut system = LinearSystem::from_str("2x - 3y +  z + 4w - 2v       = 17
-x + 2y + 3z -  w + 5u - v  = -9
3x +  y - 2z + 2w + u + 3v  = 20
4x - 2y +  z - 3w + 2u - v = 2
-x -  y + 4z +  w - 3u + 2v = -12
2x - 3y +  z + 4w - 2v      = 17").unwrap();

        system.row_echelon_form();

        let v1 = system.clone();

        system.row_echelon_form();

        assert_eq!(v1, system)
    }

    #[test]
    fn rerunning_rref_causes_no_change() {
        let mut system = LinearSystem::from_str("2x - 3y +  z + 4w - 2v       = 17
-x + 2y + 3z -  w + 5u - v  = -9
3x +  y - 2z + 2w + u + 3v  = 20
4x - 2y +  z - 3w + 2u - v = 2
-x -  y + 4z +  w - 3u + 2v = -12
2x - 3y +  z + 4w - 2v      = 17").unwrap();

        system.reduced_row_echelon_form();

        let v1 = system.clone();

        system.reduced_row_echelon_form();

        assert_eq!(v1, system)
    }

    #[test]
    fn rerunning_ref_after_rref_causes_no_change() {
        let mut system = LinearSystem::from_str("2x - 3y +  z + 4w - 2v       = 17
-x + 2y + 3z -  w + 5u - v  = -9
3x +  y - 2z + 2w + u + 3v  = 20
4x - 2y +  z - 3w + 2u - v = 2
-x -  y + 4z +  w - 3u + 2v = -12
2x - 3y +  z + 4w - 2v      = 17").unwrap();

        system.reduced_row_echelon_form();

        let v1 = system.clone();

        system.row_echelon_form();

        assert_eq!(v1, system)
    }
}
