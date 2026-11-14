use std::{collections::VecDeque, str::FromStr};

use num::{BigInt, BigRational, FromPrimitive};

use crate::Equation;

impl FromStr for Equation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _terms = {
            let mut s = s;

            let mut terms = VecDeque::new();

            while !s.is_empty() {
                let term = Term::greedy_parse(&mut s)?;

                terms.push_back(term);
            }

            terms
        };

        todo!("Update Equation::FromStr")
    }
}

enum TermKind<'a> {
    None,
    Var(&'a str),
    Div {
        numerator: Box<TermKind<'a>>,
        denominator: BigRational,
    },
    Brackets(VecDeque<Term<'a>>),
}

impl<'a> TermKind<'a> {
    fn greedy_parse(s: &mut &'a str) -> Result<Self, String> {
        todo!()
    }
}

struct Term<'a> {
    constant: Option<BigRational>,
    kind: TermKind<'a>,
}

impl<'a> Term<'a> {
    fn greedy_parse(s: &mut &'a str) -> Result<Self, String> {
        todo!()
    }
}

fn parse_big_rational(s: &mut &str) -> Result<BigRational, String> {
    let range_end = s
        .char_indices()
        .find_map(|(i, ch)| (!ch.is_ascii_digit()).then_some(i))
        .unwrap_or(s.len());

    let (num_str, remainder) = s.split_at(range_end);
    *s = remainder;

    let decimal_str = (s.chars().next() == Some('.')).then_some({
        *s = &s[1..];

        let range_end = s
            .char_indices()
            .find_map(|(i, ch)| (!ch.is_ascii_digit()).then_some(i))
            .unwrap_or(s.len());

        let (num_str, remainder) = s.split_at(range_end);
        *s = remainder;
        num_str
    });

    let mut num = BigRational::from_str(num_str).map_err(|e| e.to_string())?;

    if let Some(dec_str) = decimal_str {
        let numer = BigInt::from_str(dec_str).map_err(|e| e.to_string())?;
        let denom = BigInt::from_usize(dec_str.len()).expect("Bigint can alway hold usize values") * 10;

        num += BigRational::new(numer, denom);
    };

    let mut chars_iter = s.chars();

    match (chars_iter.next(), chars_iter.next()) {
        (None, Some(_)) => unreachable!("An iterator cannot return Some after returning None"),
        (Some('/'), None) => return Err("Expect a character atfer `/`".to_string()),
        (Some('/'), Some(d)) if d.is_ascii_digit() => {
            *s = &s[1..];

            num /= parse_big_rational(s)?;
        },
        _ => (),
    }

    Ok(num)
}

fn take_next_char(mut s: &str) -> Option<(char, &str)> {
    let boundary = s.ceil_char_boundary(1);
    let (before, after) = s.split_at(boundary);
    let mut ch = char::from_str(before).expect("Is a single char");
    s = after;

    while ch.is_whitespace() {
        if s.is_empty() {
            return None;
        }

        let boundary = s.ceil_char_boundary(1);
        let (before, after) = s.split_at(boundary);

        ch = char::from_str(before).expect("Is a single char");

        s = after
    }

    Some((ch, s))
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use num::BigRational;

    use crate::Equation;

    // Covers malformed eq strings: missing equals sign should be rejected early so the parser enforces the equation format before it starts interpreting terms.
    #[test]
    fn test_parse_requires_equals_sign() {
        let result = Equation::from_str("x + 2");

        assert!(result.is_err(), "Bare expressions should fail parsing");
    }

    // Covers the lead-in path for a real equation: a simple linear equation should be parseable and should retain the variable coefficient structure in the resulting model.
    #[test]
    fn test_parse_simple_linear_equation() {
        let equation = Equation::from_str("x + 5 = 12").expect("Simple equation should parse");

        assert!(equation.variables.contains_key("x"));
        assert!(equation.sum == BigRational::from_integer(12.into()));
    }
}
