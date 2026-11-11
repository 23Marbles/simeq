use std::ops::Neg;

use num::{BigRational, Signed, Zero};

use crate::solver::augmented_matrix::column_iter::ColumnIter;

mod column_iter {
    pub struct ColumnIter<'a, T> {
        current_column: usize,

        width: usize,
        rows: &'a Vec<Vec<T>>,
    }

    impl<'a, T> ColumnIter<'a, T> {
        pub fn new(rows: &'a Vec<Vec<T>>) -> Self {
            let width = rows.get(0).map_or_default(|row| row.len());

            for row in rows.iter().skip(1) {
                assert_eq!(
                    row.len(),
                    width,
                    "Rows in iter should all have the same length"
                )
            }

            Self {
                current_column: 0,
                width,
                rows,
            }
        }
    }

    impl<'a, T: Clone> Iterator for ColumnIter<'a, T> {
        type Item = Vec<&'a T>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current_column == self.width {
                return None;
            };

            let res = self
                .rows
                .iter()
                .map(|row| &row[self.current_column])
                .collect();

            self.current_column += 1;

            Some(res)
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AugmentedMatrix {
    height: usize,
    width: usize,

    rows: Vec<Vec<BigRational>>,
    /// height
    augment: Vec<BigRational>,
}

impl AugmentedMatrix {
    pub fn add_column(&mut self) {
        self.width += 1;

        for row in &mut self.rows {
            row.push(BigRational::ZERO);
        }
    }

    pub fn add_row(&mut self, row: Vec<BigRational>, sum: BigRational) {
        self.height += 1;

        self.rows.push(row);
        self.augment.push(sum);
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn rows(&self) -> &Vec<Vec<BigRational>> {
        &self.rows
    }

    pub fn augment(&self) -> &Vec<BigRational> {
        &self.augment
    }
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

    pub fn add_rows(&mut self, pivot: usize, mul: BigRational, target: usize) {
        assert!(pivot < self.height);
        assert!(target < self.height);
        assert_ne!(pivot, target, "Use multiply row instead");

        let [src, dst] = self.rows.get_disjoint_mut([pivot, target]).unwrap();
        for (src, dst) in src.iter().zip(dst) {
            *dst += mul.clone() * &*src;
        }

        let [src, dst] = self.augment.get_disjoint_mut([pivot, target]).unwrap();
        *dst += mul * &*src;
    }

    pub fn reorder_triangle(&mut self) {
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
        let mut current_pivot_y = 0;

        for x in 0..self.width {
            let column = ColumnIter::new(&self.rows)
                .nth(x)
                .expect("Should have count() of width");

            let Some((pivot_row, pivot_val)) = column
                .into_iter()
                .enumerate()
                .skip(current_pivot_y)
                .max_by(|(_, v1), (_, v2)| v1.abs().cmp(&v2.abs()))
            else {
                break;
            };

            let pivot_val = pivot_val.clone();

            if pivot_val.is_zero() {
                continue;
            }

            self.swap_rows(current_pivot_y, pivot_row);

            let Some(lower) = current_pivot_y
                .checked_add(1)
                .and_then(|n| (n < self.height).then_some(n))
            else {
                break;
            };

            for y in lower..self.height {
                let numer = &self.rows[y][x];

                let mul = numer / &pivot_val;

                self.add_rows(current_pivot_y, mul.neg(), y);
            }

            current_pivot_y += 1;
        }
    }
}
