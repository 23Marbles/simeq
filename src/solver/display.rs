use std::{
    fmt::{Display, Write},
    iter,
};

use crate::solver::augmented_matrix::AugmentedMatrix;

impl Display for AugmentedMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_widths = Vec::with_capacity(self.width() + 1);

        let rows_string: Vec<Vec<_>> = self
            .rows()
            .iter()
            .map(|row| row.iter().map(|n| n.to_string()).collect())
            .collect();
        let augment_string: Vec<_> = self.augment().iter().map(|n| n.to_string()).collect();

        for x in 0..self.width() {
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
