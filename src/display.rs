use std::{
    fmt::{Display, Write},
    iter,
    ops::Range,
};

use num::{BigRational, One};

use crate::{
    Equation,
    solver::{LinearSystem, Solution, augmented_matrix::AugmentedMatrix},
};

pub struct DisplayMatrix {
    brackets_range: Range<usize>,

    width: usize,
    height: usize,

    has_augment: bool,
    rows_string: Vec<Vec<String>>,
}

impl Display for DisplayMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.brackets_range.end >= self.brackets_range.start);
        assert!(self.brackets_range.end <= self.height);

        let max_widths = Vec::from_iter((0..self.width).map(|x| {
            let mut max = 0;

            // Should only be unicode characters
            for len in self.rows_string.iter().map(|row| row[x].len()) {
                max = std::cmp::max(max, len)
            }

            max
        }));

        fn whitespace_string(len: usize) -> String {
            String::from_iter(iter::repeat_n(' ', len))
        }
        fn write_padded(width: usize, s: &str) -> String {
            let mut res = whitespace_string(width - s.len());
            res.push_str(s);

            debug_assert_eq!(res.len(), width);

            res
        }
        let mut write_row =
            |left: char, mid: char, right: char, newline: bool, row: &Vec<String>| {
                write!(
                    f,
                    "{left} {} {}{right}",
                    {
                        iter::zip(
                            {
                                if self.has_augment && !row.is_empty() {
                                    &row[..row.len() - 1]
                                } else {
                                    &row
                                }
                            },
                            max_widths.iter(),
                        )
                        .map(|(s, width)| write_padded(*width, s))
                        .collect::<Vec<_>>()
                        .join(", ")
                    },
                    if self.has_augment {
                        format!(
                            "{} {} ",
                            mid,
                            write_padded(
                                *max_widths.last().unwrap_or(&0),
                                row.last().unwrap_or(&String::new())
                            )
                        )
                    } else {
                        String::new()
                    }
                )?;

                if newline {
                    f.write_char('\n')?;
                }

                Ok(())
            };

        if self.rows_string.len() == 0 {
            return if self.has_augment {
                f.write_str("[  ]")
            } else {
                f.write_str("[  |  ]")
            };
        };

        let mut rows_iter = self.rows_string.iter();

        let last_row = rows_iter.next_back().expect("Already checked len != 0");

        for _ in 0..self.brackets_range.start {
            let Some(row) = rows_iter.next() else {
                write_row(' ', ' ', ' ', true, last_row)?;

                return Ok(());
            };

            write_row(' ', ' ', ' ', true, row)?;
        }

        if self.brackets_range.len() == 1 {
            let row = rows_iter.next();

            write_row('[', '|', ']', row.is_some(), row.unwrap_or(last_row))?;

            if row.is_none() {
                return Ok(());
            }
        }

        if !self.brackets_range.is_empty() {
            write_row(
                '⎡',
                '⎢',
                '⎤',
                true,
                rows_iter.next().expect("Just tRUST me bro 😉"),
            )?;

            if let Some(middle_len) = self.brackets_range.len().checked_sub(2) {
                for _ in 0..middle_len {
                    write_row(
                        '⎢',
                        '⎢',
                        '⎥',
                        true,
                        rows_iter.next().expect("Just tRUST me bro 😉"),
                    )?;
                }
            }

            let row = rows_iter.next();

            write_row('⎣', '⎢', '⎦', row.is_some(), row.unwrap_or(last_row))?;

            if row.is_none() {
                return Ok(());
            }
        }

        for row in rows_iter {
            write_row(' ', ' ', ' ', true, row)?;
        }

        write_row(' ', ' ', ' ', false, last_row)?;

        Ok(())
    }
}

impl From<AugmentedMatrix> for DisplayMatrix {
    fn from(value: AugmentedMatrix) -> Self {
        let rows_string: Vec<Vec<_>> = value
            .rows()
            .iter()
            .zip(value.augment())
            .map(|(row, augment)| {
                let mut row: Vec<String> = row.iter().map(|n| n.to_string()).collect();
                row.push(augment.to_string());

                row
            })
            .collect();

        Self {
            brackets_range: 0..rows_string.len(),
            width: value.width() + 1,
            height: value.height(),
            has_augment: true,
            rows_string,
        }
    }
}

impl From<LinearSystem> for DisplayMatrix {
    fn from(value: LinearSystem) -> Self {
        let mut x = value.variables().clone();
        let augmented_matrix = value.augmented_matrix().clone();

        let mut display_matrix: DisplayMatrix = augmented_matrix.into();

        x.push(String::new());

        display_matrix.rows_string.insert(0, x);
        display_matrix.height += 1;

        display_matrix.brackets_range.start += 1;
        display_matrix.brackets_range.end += 1;

        display_matrix
    }
}

impl Display for AugmentedMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayMatrix::from(self.clone()).fmt(f)
    }
}

impl Display for LinearSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayMatrix::from(self.clone()).fmt(f)
    }
}

impl Display for Equation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            {
                let s = self
                    .variables
                    .iter()
                    .map(|(c, v)| format!("{c}{v}"))
                    .collect::<Vec<_>>()
                    .join(" + ");
                (!s.is_empty())
                    .then_some(format!("{s} = "))
                    .unwrap_or_default()
            },
            self.sum
        )
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Solution::Unique(unique_variables) => {
                write!(f, "Unique Solution")?;

                for (name, value) in unique_variables {
                    write!(f, "\n{name} = {value}")?;
                }

                Ok(())
            },
            Solution::Infinite {
                unique_variables,
                pivots,
                free_variables,
            } => {
                let mut has_started = false;

                if !unique_variables.is_empty() {
                    write!(f, "Unique Variables")?;

                    for (name, value) in unique_variables {
                        write!(f, "\n{name} = {value}")?;
                    }

                    has_started = true;
                }

                if !pivots.is_empty() {
                    if has_started {
                        f.write_char('\n')?;
                    }

                    write!(f, "Expressions")?;

                    for (name, equation) in pivots {
                        let Equation { variables, sum } = equation;

                        let variable_iter: Vec<_> = variables
                            .iter()
                            .filter(|(var_name, _)| *var_name != name)
                            .map(|(var_name, coefficient)| {
                                format!(
                                    "{}{}",
                                    match coefficient {
                                        one if one.is_one() => String::new(),
                                        neg_one if neg_one == &-BigRational::ONE => "-".to_string(),
                                        c => c.to_string(),
                                    },
                                    var_name
                                )
                            })
                            .collect();

                        write!(f, "\n{name} = {} + {sum}", variable_iter.join(" + "))?;
                    }

                    has_started = true;
                }

                if !free_variables.is_empty() {
                    if has_started {
                        f.write_char('\n')?;
                    }

                    write!(f, "Free Variables")?;

                    for name in free_variables {
                        write!(f, "\nname")?;
                    }

                    has_started = true;
                }

                if !has_started {
                    write!(f, "Empty infinte solution, this is a bug. Oops.")?;
                }

                Ok(())
            }
            Solution::None => write!(f, "No solution"),
        }
    }
}
