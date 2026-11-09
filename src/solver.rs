use std::{
    fmt::{Display, Write},
    ops::Neg,
};

use num::{BigRational, Zero};

struct AugmentedMatrix {
    height: usize,
    width: usize,

    rows: Vec<Vec<BigRational>>,
    /// height
    augment: Vec<BigRational>,
}

impl AugmentedMatrix {
    pub fn swap_rows(&mut self, a: usize, b: usize) {
        assert!(a < self.height);
        assert!(b < self.height);

        self.augment.swap(a, b);
        self.rows.swap(a, b);
    }

    pub fn multiply_row(&mut self, row: usize, mul: BigRational) {
        assert!(row < self.height);

        for n in self.rows.get_mut(row).unwrap() {
            *n *= mul.clone()
        }

        *self.augment.get_mut(row).unwrap() *= mul;
    }

    pub fn add_rows(&mut self, to_add: usize, mul: BigRational, target: usize) {
        assert!(to_add < self.height);
        assert!(target < self.height);
        assert_ne!(to_add, target, "Use multiply row instead");

        let [src, dst] = self.rows.get_disjoint_mut([to_add, target]).unwrap();
        for (src, dst) in src.iter_mut().zip(dst) {
            *dst += mul.clone() * &*src;
        }

        let [src, dst] = self.augment.get_disjoint_mut([to_add, target]).unwrap();
        *dst += mul * &*src;
    }

    pub fn reorder(&mut self) {
        let mut leading_zeros = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let leading_zeros = row
                    .iter()
                    .enumerate()
                    .find(|(_, n)| !n.is_zero())
                    .map_or(row.len(), |(i, _)| i);

                (i, leading_zeros)
            })
            .collect::<Vec<_>>();

        leading_zeros.sort_by(|(_, zeros1), (_, zeros2)| zeros1.cmp(zeros2));

        for (dst, (src, _)) in leading_zeros.into_iter().enumerate() {
            self.swap_rows(src, dst);
        }
    }

    pub fn row_echelon_form(&mut self) {
        for x in 0..self.width {
            let Some(lower) = 0usize
                .checked_add(1)
                .and_then(|n| (n < self.height).then_some(n))
            else {
                break;
            };

            for y in lower..self.height {
                let mul = {
                    let numer = &self.rows[y][x];
                    let denom = &self.rows[x][x];

                    (numer / denom).neg()
                };

                self.add_rows(x, mul, y);
            }
        }
    }
}

pub struct EquationMatrix {
    height: usize,
    width: usize,

    augmented_matrix: AugmentedMatrix,

    /// width
    x: Vec<String>,
}

impl EquationMatrix {
    /// https://en.wikipedia.org/wiki/Gaussian_elimination
    pub fn solve() {}
}
