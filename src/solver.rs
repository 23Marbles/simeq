use std::{
    collections::{HashMap, HashSet},
    iter,
    str::FromStr,
};

use num::{BigRational, Zero};

use crate::{Equation, solver::augmented_matrix::AugmentedMatrix};

pub mod augmented_matrix;

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

            augmented_matrix.add_row(row, sum);
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

    pub fn variables(&self) -> &Vec<String> {
        &self.x
    }

    pub fn row_echelon_form(&mut self) {
        self.augmented_matrix.row_echelon_form();
    }

    pub fn reduced_row_echelon_form(&mut self) {
        self.augmented_matrix.reduced_row_echelon_form();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Solution {
    Unique(HashMap<String, BigRational>),
    Infinite {
        unique_variables: HashMap<String, BigRational>,
        pivots: HashMap<String, Equation>,
        free_variables: HashSet<String>,
    },
    None,
}

impl LinearSystem {
    /// https://en.wikipedia.org/wiki/Gaussian_elimination
    pub fn solve(&mut self) -> Solution {
        assert_eq!(self.height, self.augmented_matrix.height());
        assert_eq!(self.width, self.augmented_matrix.width());

        let pivot_columns = self.augmented_matrix.reduced_row_echelon_form();

        let mut unique_variables = HashMap::new();
        let mut pivots = HashMap::new();

        let mut rank = 0;

        for y in 0..self.height {
            let row = &self.augmented_matrix.rows()[y];

            let Some(pivot) = row.iter().position(|n| !n.is_zero()) else {
                break;
            };

            let mut iter = iter::zip(&self.x, row)
                .skip(pivot + 1)
                .filter(|(_, v)| !v.is_zero())
                .peekable();

            if iter.peek().is_none() {
                assert!(
                    unique_variables
                        .insert(
                            self.x[pivot].clone(),
                            self.augmented_matrix.augment()[y].clone()
                        )
                        .is_none()
                );

                rank += 1;

                continue;
            }

            let variables: HashMap<_, _> = iter::once((self.x[pivot].clone(), BigRational::ONE))
                .chain(iter.map(|(n, v)| (n.clone(), v.clone())))
                .collect();

            pivots.insert(
                self.x[pivot].clone(),
                Equation {
                    variables,
                    sum: self.augmented_matrix.augment()[y].clone(),
                },
            );

            rank += 1;
        }

        let mut pivot_columns = pivot_columns;

        let free_variables: HashSet<_> = self
            .x
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                pivot_columns
                    .pop_if(|i1| *i1 == i)
                    .is_none()
                    .then_some(s.clone())
            })
            .collect();

        if self
            .augmented_matrix
            .augment()
            .iter()
            .skip(rank)
            .any(|augment| !augment.is_zero())
        {
            Solution::None
        } else if free_variables.is_empty() {
            Solution::Unique(unique_variables)
        } else {
            Solution::Infinite {
                pivots,
                free_variables,
                unique_variables,
            }
        }
    }
}

impl FromStr for LinearSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec: Vec<_> = s
            .lines()
            .map(|s| Equation::from_str(s))
            .collect::<Result<_, _>>()?;

        Ok(Self::from_equation_vec(vec))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::solver::LinearSystem;

    #[test]
    fn rerunning_ref_causes_no_change() {
        let mut system = LinearSystem::from_str(
            "2x - 3y +  z + 4w - 2v       = 17
-x + 2y + 3z -  w + 5u - v  = -9
3x +  y - 2z + 2w + u + 3v  = 20
4x - 2y +  z - 3w + 2u - v = 2
-x -  y + 4z +  w - 3u + 2v = -12
2x - 3y +  z + 4w - 2v      = 17",
        )
        .unwrap();

        system.row_echelon_form();

        let v1 = system.clone();

        system.row_echelon_form();

        assert_eq!(v1, system)
    }

    #[test]
    fn rerunning_rref_causes_no_change() {
        let mut system = LinearSystem::from_str(
            "2x - 3y +  z + 4w - 2v       = 17
-x + 2y + 3z -  w + 5u - v  = -9
3x +  y - 2z + 2w + u + 3v  = 20
4x - 2y +  z - 3w + 2u - v = 2
-x -  y + 4z +  w - 3u + 2v = -12
2x - 3y +  z + 4w - 2v      = 17",
        )
        .unwrap();

        system.reduced_row_echelon_form();

        let v1 = system.clone();

        system.reduced_row_echelon_form();

        assert_eq!(v1, system)
    }

    #[test]
    fn rerunning_solve_produces_same_result() {
        let mut system = LinearSystem::from_str(
            "2x - 3y +  z + 4w - 2v       = 17
-x + 2y + 3z -  w + 5u - v  = -9
3x +  y - 2z + 2w + u + 3v  = 20
4x - 2y +  z - 3w + 2u - v = 2
-x -  y + 4z +  w - 3u + 2v = -12
2x - 3y +  z + 4w - 2v      = 17",
        )
        .unwrap();

        let v1 = system.solve();

        let v2 = system.solve();

        assert_eq!(v1, v2)
    }

    #[test]
    fn rerunning_ref_after_rref_causes_no_change() {
        let mut system = LinearSystem::from_str(
            "2x - 3y +  z + 4w - 2v       = 17
-x + 2y + 3z -  w + 5u - v  = -9
3x +  y - 2z + 2w + u + 3v  = 20
4x - 2y +  z - 3w + 2u - v = 2
-x -  y + 4z +  w - 3u + 2v = -12
2x - 3y +  z + 4w - 2v      = 17",
        )
        .unwrap();

        system.reduced_row_echelon_form();

        let v1 = system.clone();

        system.row_echelon_form();

        assert_eq!(v1, system)
    }
}
