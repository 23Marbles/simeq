use std::str::FromStr;

use num::{BigInt, BigRational, FromPrimitive, Zero};

use crate::Equation;

impl FromStr for Equation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (before, after) = s.split_once('=').ok_or("Expected `=` in equation")?;

        let left_terms = find_terms(before)?;
        let right_terms = find_terms(after)?;

        let mut me = Equation::default();

        fn handle_term_kind(
            constant: Option<BigRational>,
            kind: TermKind<'_>,
            is_sign_flipped: bool,
            this: &mut Equation,
        ) {
            let standardize_const = |constant: Option<BigRational>| {
                let constant = constant.unwrap_or(BigRational::ONE);

                if is_sign_flipped { -constant } else { constant }
            };

            match (constant, kind) {
                (Some(constant), TermKind::None) => {
                    if is_sign_flipped {
                        this.sum += constant
                    } else {
                        this.sum -= constant
                    }
                }
                (None, TermKind::None) => panic!("term should not be completely empty"),
                (coefficient, TermKind::Var(name)) => {
                    this.add_term(name.to_string(), standardize_const(coefficient));
                }
                (
                    multiplier,
                    TermKind::Div {
                        numerator,
                        denominator,
                    },
                ) => {
                    let constant = standardize_const(multiplier) / denominator;

                    handle_term_kind(Some(constant), *numerator, is_sign_flipped, this);
                }
                (multiplier, TermKind::Brackets(terms)) => {
                    let standardized_multiplier = standardize_const(multiplier);

                    for term in terms {
                        if matches!(term.kind, TermKind::None) {
                            if let Some(constant) = term.constant
                                && !constant.is_zero()
                            {
                                this.sum -= standardized_multiplier.clone() * constant
                            }
                        } else {
                            handle_term_kind(
                                Some(
                                    standardized_multiplier.clone()
                                        * standardize_const(term.constant),
                                ),
                                term.kind,
                                is_sign_flipped,
                                this,
                            );
                        }
                    }
                }
            };
        }

        for term in left_terms {
            handle_term_kind(term.constant, term.kind, false, &mut me);
        }

        for term in right_terms {
            handle_term_kind(term.constant, term.kind, true, &mut me);
        }

        Ok(me)
    }
}

enum TermKind<'a> {
    None,
    Var(&'a str),
    Div {
        numerator: Box<TermKind<'a>>,
        denominator: BigRational,
    },
    Brackets(Vec<Term<'a>>),
}

impl<'a> TermKind<'a> {
    fn greedy_parse(s: &mut &'a str) -> Result<Self, String> {
        let Some((ch, new_s)) = take_next_char(s) else {
            return Ok(Self::None);
        };

        let kind = match ch {
            '/' => {
                return Err("Expected item before `/`".to_string());
            }
            d if d.is_ascii_digit() => {
                panic!("All digits should be handled before reaching this function")
            }
            '(' => {
                let mut bracket_depth: usize = 1;

                let Some(end_idx) = new_s.char_indices().find_map(|(idx, c)| {
                    if c == '(' {
                        bracket_depth += 1;
                    } else if c == ')' {
                        bracket_depth -= 1;
                    };

                    (bracket_depth == 0).then_some(idx)
                }) else {
                    return Err("Unterminated `(`".to_string());
                };

                *s = new_s;

                let bracket_terms = find_terms(&s[..end_idx])?;

                *s = &s[end_idx + 1..];

                Self::Brackets(bracket_terms)
            }
            a if a.is_alphabetic() => {
                let end_idx = s.find(|ch: char| !ch.is_alphabetic()).unwrap_or(s.len());

                let (name, residual) = s.split_at(end_idx);

                *s = residual;

                Self::Var(name)
            }
            _ => return Ok(Self::None),
        };

        if let Some(('/', new_s)) = take_next_char(s) {
            *s = new_s;

            Ok(Self::Div {
                numerator: Box::new(kind),
                denominator: parse_big_rational(s)?,
            })
        } else {
            Ok(kind)
        }
    }
}

fn find_terms<'a>(mut s: &'a str) -> Result<Vec<Term<'a>>, String> {
    let mut terms = Vec::new();

    while !s.is_empty() {
        let term = Term::greedy_parse(&mut s)?;

        terms.push(term);
    }

    Ok(terms)
}

struct Term<'a> {
    constant: Option<BigRational>,
    kind: TermKind<'a>,
}

impl<'a> Term<'a> {
    fn greedy_parse(s: &mut &'a str) -> Result<Self, String> {
        let Some((mut ch, new_s)) = take_next_char(s) else {
            return Err("Did not expect a fully whitespace / empty string".to_string());
        };

        *s = new_s;

        if ch == '+' {
            let Some((new_ch, new_s)) = take_next_char(s) else {
                return Err("Expected a character after `+`".to_string());
            };

            *s = new_s;

            ch = new_ch;
        };

        let mut is_neg = false;

        while ch == '-' {
            is_neg = !is_neg;

            let Some((new_ch, new_s)) = take_next_char(s) else {
                return Err("Expected a character after `-`".to_string());
            };

            *s = new_s;

            ch = new_ch;
        }

        let constant = ch.is_ascii_digit().then_some(parse_big_rational(s)?);

        let kind = TermKind::greedy_parse(s)?;

        Ok(Self { constant, kind })
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
        let denom =
            BigInt::from_usize(dec_str.len()).expect("Bigint can alway hold usize values") * 10;

        num += BigRational::new(numer, denom);
    };

    let mut chars_iter = s.chars();

    match (chars_iter.next(), chars_iter.next()) {
        (None, Some(_)) => unreachable!("An iterator cannot return Some after returning None"),
        (Some('/'), None) => return Err("Expect a character after `/`".to_string()),
        (Some('/'), Some(d)) if d.is_ascii_digit() => {
            *s = &s[1..];

            num /= parse_big_rational(s)?;
        }
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
