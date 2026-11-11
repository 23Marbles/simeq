use std::{
    fmt::{Display, Write},
    iter,
    ops::Neg,
};

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

#[derive(Default, Clone)]
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
}

impl Display for AugmentedMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_widths = Vec::with_capacity(self.width + 1);

        let rows_string: Vec<Vec<_>> = self
            .rows
            .iter()
            .map(|row| row.iter().map(|n| n.to_string()).collect())
            .collect();
        let augment_string: Vec<_> = self.augment.iter().map(|n| n.to_string()).collect();

        for x in 0..self.width {
            let mut max = 0;

            // Should only be unicode characters
            for len in rows_string.iter().map(|row| row[x].len()) {
                max = std::cmp::max(max, len)
            }

            max_widths.push(max);
        }

        {
            let mut max = 0;

            // Should only be unicode characters
            for len in augment_string.iter().map(|s| s.len()) {
                max = std::cmp::max(max, len)
            }

            max_widths.push(max);
        }

        fn padding_string(len: usize) -> String {
            String::from_iter(iter::repeat_n(' ', len))
        }
        fn write_padded(width: usize, s: String) -> String {
            let mut res = padding_string(width - s.len());
            res.push_str(&s);

            debug_assert_eq!(res.len(), width);

            res
        }
        let mut write_row = |left: char,
                             mid: char,
                             right: char,
                             newline: bool,
                             row: Vec<String>,
                             augment: String| {
            write!(
                f,
                "{left} {} {mid} {} {right}",
                {
                    iter::zip(row, max_widths.iter())
                        .map(|(s, width)| write_padded(*width, s))
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                write_padded(*max_widths.last().unwrap(), augment)
            )?;

            if newline {
                f.write_char('\n')?;
            }

            Ok(())
        };

        let mut rows_iter = iter::zip(rows_string, augment_string);

        let first_row = rows_iter.next();

        let Some(last_row) = rows_iter.next_back() else {
            return if let Some((row, augment)) = first_row {
                write_row('[', '|', ']', false, row, augment)
            } else {
                f.write_str("[  |  ]")
            };
        };

        let first_row = first_row.expect("Cannot have an end without a start");

        write_row('⎡', '⎢', '⎤', true, first_row.0, first_row.1)?;

        for (row, augment) in rows_iter {
            write_row('⎢', '⎢', '⎥', true, row, augment)?;
        }

        write_row('⎣', '⎢', '⎦', false, last_row.0, last_row.1)?;

        Ok(())
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
