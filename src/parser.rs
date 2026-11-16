use std::str::FromStr;

use num::{BigInt, BigRational, FromPrimitive, Zero, pow::Pow};

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
        ) -> Result<(), String> {
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
                    if denominator.is_zero() {
                        return Err("Cannot divide by zero".to_string());
                    }

                    let constant = multiplier.unwrap_or(BigRational::ONE) / denominator;

                    handle_term_kind(Some(constant), *numerator, is_sign_flipped, this)?;
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
                            )?;
                        }
                    }
                }
            };

            Ok(())
        }

        for term in left_terms {
            handle_term_kind(term.constant, term.kind, false, &mut me)?;
        }

        for term in right_terms {
            handle_term_kind(term.constant, term.kind, true, &mut me)?;
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
            '/' => Self::None,
            d if d.is_ascii_digit() => {
                panic!("All digits should be handled before reaching this function")
            }
            '(' => {
                *s = new_s;

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

                let bracket_terms = find_terms(&s[..end_idx])?;

                *s = &s[end_idx + ')'.len_utf8()..];

                Self::Brackets(bracket_terms)
            }
            a if a.is_alphabetic() => {
                let name_start = s.len() - new_s.len() - a.len_utf8();
                let name_tail_len = new_s
                    .char_indices()
                    .find_map(|(i, ch)| (!ch.is_alphabetic()).then_some(i))
                    .unwrap_or(new_s.len());
                let name_end = s.len() - new_s.len() + name_tail_len;

                let (prefix, residual) = s.split_at(name_end);

                *s = residual;

                Self::Var(&prefix[name_start..])
            }
            '+' | '-' | ')' => return Ok(Self::None),
            ch => return Err(format!("Unexpected char {ch}")),
        };

        if let Some(('/', new_s)) = take_next_char(s) {
            *s = new_s;

            // Skip whitespace
            let _ = take_next_char(s);

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

    loop {
        let previous_len = s.len();
        let term = match Term::greedy_parse(&mut s) {
            Ok(t) => t,
            Err(s) if s == "Did not expect a fully whitespace / empty string" => break,
            Err(e) => return Err(e),
        };

        if s.len() == previous_len {
            return Err("Parser did not consume input".to_string());
        }

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
        let Some((mut ch, mut new_s)) = take_next_char(s) else {
            return Err("Did not expect a fully whitespace / empty string".to_string());
        };

        if ch == '+' {
            *s = new_s;

            let Some((new_ch, after_new_ch)) = take_next_char(s) else {
                return Err("Expected a character after `+`".to_string());
            };

            new_s = after_new_ch;
            ch = new_ch;
        };

        let is_neg = {
            let mut is_neg = false;
            let mut s_after_current_minus_sign = new_s;

            while ch == '-' {
                *s = s_after_current_minus_sign;

                is_neg = !is_neg;

                let Some((new_ch, after_new_ch)) = take_next_char(s) else {
                    return Err("Expected a character after `-`".to_string());
                };

                s_after_current_minus_sign = after_new_ch;

                ch = new_ch;
            }

            is_neg
        };

        let constant = if ch.is_ascii_digit() {
            Some({
                let num = parse_big_rational(s)?;
                if is_neg { -num } else { num }
            })
        } else {
            None
        };

        let kind = TermKind::greedy_parse(s)?;

        Ok(Self {
            constant: if !matches!(kind, TermKind::None) && is_neg && constant.is_none() {
                Some(-BigRational::ONE)
            } else {
                constant
            },
            kind,
        })
    }
}

fn parse_big_rational(s: &mut &str) -> Result<BigRational, String> {
    let range_end = s
        .char_indices()
        .find_map(|(i, ch)| (!ch.is_ascii_digit()).then_some(i))
        .unwrap_or(s.len());

    let (num_str, remainder) = s.split_at(range_end);
    *s = remainder;

    let decimal_str = if s.chars().next() == Some('.') {
        *s = &s[1..];

        let range_end = s
            .char_indices()
            .find_map(|(i, ch)| (!ch.is_ascii_digit()).then_some(i))
            .unwrap_or(s.len());

        let (num_str, remainder) = s.split_at(range_end);
        *s = remainder;

        Some(num_str)
    } else {
        None
    };

    let mut num = BigRational::from_str(num_str).map_err(|e| e.to_string())?;

    if let Some(dec_str) = decimal_str {
        let numer = BigInt::from_str(dec_str).map_err(|e| e.to_string())?;
        let denom = BigInt::from_u32(10).unwrap().pow(dec_str.len() as u32);

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

fn take_next_char<'a>(s: &mut &'a str) -> Option<(char, &'a str)> {
    if s.is_empty() {
        return None;
    }

    let boundary = s.ceil_char_boundary(1);
    let (before, mut after) = s.split_at(boundary);
    let mut ch = char::from_str(before).expect("Is a single char");

    while ch.is_whitespace() {
        *s = after;

        if s.is_empty() {
            return None;
        }

        let boundary = s.ceil_char_boundary(1);
        let (before, inner_after) = s.split_at(boundary);

        ch = char::from_str(before).expect("Is a single char");

        after = inner_after
    }

    Some((ch, after))
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use num::BigRational;

    use crate::Equation;

    fn rational(numerator: i64, denominator: i64) -> BigRational {
        BigRational::new(numerator.into(), denominator.into())
    }

    fn expected_equation(variables: &[(&str, BigRational)], sum: BigRational) -> Equation {
        Equation {
            variables: variables
                .iter()
                .map(|(name, coefficient)| (name.to_string(), coefficient.clone()))
                .collect(),
            sum,
        }
    }

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
        assert!(equation.sum == BigRational::from_integer(7.into()));
    }

    #[test]
    fn test_parse_variables_on_both_sides() {
        let parsed = Equation::from_str("x = y").expect("Variable equation should parse");

        assert_eq!(
            parsed,
            expected_equation(
                &[("x", BigRational::ONE), ("y", -BigRational::ONE)],
                BigRational::ZERO,
            )
        );
    }

    #[test]
    fn test_parse_negative_variable_without_coefficient() {
        let parsed = Equation::from_str("-x = 0")
            .expect("A negative variable without an explicit coefficient should parse");

        assert_eq!(
            parsed,
            expected_equation(&[("x", -BigRational::ONE)], BigRational::ZERO)
        );
    }

    #[test]
    fn test_parse_repeated_variables_and_constant_terms() {
        let parsed = Equation::from_str("2x + 3x - 1.25 = 4")
            .expect("Repeated variables and constants should parse");

        assert_eq!(
            parsed,
            expected_equation(&[("x", rational(5, 1))], rational(21, 4))
        );
    }

    #[test]
    fn test_parse_repeated_unary_signs() {
        let positive = Equation::from_str("--x = 0")
            .expect("Two unary minus signs should produce a positive variable");
        let negative = Equation::from_str("---x = 0")
            .expect("Three unary minus signs should produce a negative variable");

        assert_eq!(
            positive,
            expected_equation(&[("x", BigRational::ONE)], BigRational::ZERO)
        );
        assert_eq!(
            negative,
            expected_equation(&[("x", -BigRational::ONE)], BigRational::ZERO)
        );
    }

    #[test]
    fn test_parse_decimal_denominator() {
        let parsed = Equation::from_str("x/0.25 = 1").expect("Decimal denominators should parse");

        assert_eq!(
            parsed,
            expected_equation(&[("x", rational(4, 1))], BigRational::ONE)
        );
    }

    #[test]
    fn test_parse_signed_coefficients_and_decimals() {
        let parsed = Equation::from_str("-2x + 1.25y = -3.5")
            .expect("Signed coefficients and decimals should parse");

        assert_eq!(
            parsed,
            expected_equation(
                &[("x", rational(-2, 1)), ("y", rational(5, 4))],
                rational(-7, 2),
            )
        );
    }

    #[test]
    fn test_parse_division_on_both_sides() {
        let parsed = Equation::from_str("x/2 = y/4")
            .expect("Variable terms divided by constants should parse");

        assert_eq!(
            parsed,
            expected_equation(
                &[("x", rational(1, 2)), ("y", rational(-1, 4))],
                BigRational::ZERO,
            )
        );
    }

    #[test]
    fn test_parse_mixed_plus_minus_signs() {
        let parsed =
            Equation::from_str("x + -y = 0").expect("A minus sign following a plus should parse");

        assert_eq!(
            parsed,
            expected_equation(
                &[("x", BigRational::ONE), ("y", -BigRational::ONE)],
                BigRational::ZERO,
            )
        );
    }

    #[test]
    fn test_parse_variable_division_with_whitespace() {
        let parsed = Equation::from_str("x / 2 = 1")
            .expect("Whitespace around variable division should parse");

        assert_eq!(
            parsed,
            expected_equation(&[("x", rational(1, 2))], BigRational::ONE)
        );
    }

    #[test]
    fn test_parse_constant_division_with_whitespace() {
        let parsed = Equation::from_str("x = 1 / 2")
            .expect("Whitespace around constant division should parse");

        assert_eq!(
            parsed,
            expected_equation(&[("x", BigRational::ONE)], rational(1, 2))
        );
    }

    #[test]
    fn test_parse_rejects_zero_denominators() {
        for input in ["x/0 = 1", "x/0.0 = 1"] {
            assert!(
                Equation::from_str(input).is_err(),
                "Zero denominator should be rejected: {input}"
            );
        }
    }

    #[test]
    fn test_parse_parenthesized_and_nested_terms() {
        let parsed = Equation::from_str("2(x + 3(y - 1)) = (z + 4)")
            .expect("Parenthesized terms should parse");

        assert_eq!(
            parsed,
            expected_equation(
                &[
                    ("x", rational(2, 1)),
                    ("y", rational(6, 1)),
                    ("z", rational(-1, 1)),
                ],
                rational(10, 1),
            )
        );
    }

    #[test]
    fn test_parse_rejects_unterminated_parentheses() {
        assert!(Equation::from_str("x = (y + 1").is_err());
    }

    #[test]
    fn test_parse_rejects_slash_without_denominator() {
        assert!(Equation::from_str("x/ = 1").is_err());
    }

    #[test]
    fn test_parse_rejects_unexpected_and_unconsumed_tokens() {
        assert!(Equation::from_str("x = @").is_err());
        assert!(Equation::from_str("x = /y").is_err());
        assert!(Equation::from_str("x = )").is_err());
        assert!(Equation::from_str("x + = 1").is_err());
    }
}
