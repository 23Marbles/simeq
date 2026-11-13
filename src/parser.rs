use std::{
    collections::{HashMap, VecDeque},
    ops::Neg,
    str::FromStr,
};

use num::{BigRational, FromPrimitive};

use crate::Equation;

impl FromStr for Equation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left_s, right_s) = s
            .split_once('=')
            .ok_or("No equals sign found".to_string())?;

        let mut left_tokens = tokenize(left_s)?;
        let mut right_tokens = tokenize(right_s)?;

        let mut me = Equation::default();

        left_tokens.push_front(Token::Add);
        right_tokens.push_front(Token::Add);

        fn handle_term(
            is_neg: bool,
            me: &mut Equation,
            var: Option<&str>,
            coefficient: BigRational,
        ) {
            fn handle_var(
                variables: &mut HashMap<String, BigRational>,
                coefficient: BigRational,
                name: &str,
            ) {
                if let Some(var_coefficient) = variables.get_mut(name) {
                    *var_coefficient += coefficient;
                } else {
                    variables.insert(name.to_string(), coefficient);
                }
            }

            let coefficient = if is_neg {
                coefficient.neg()
            } else {
                coefficient
            };

            if let Some(name) = var {
                handle_var(&mut me.variables, coefficient, name);
            } else {
                me.sum -= coefficient
            }
        }

        let mut token_parser = |is_neg: bool, tokens: &mut VecDeque<Token>| {
            while let Some(token) = tokens.pop_front() {
                let mut is_neg = match token {
                    Token::Sub => !is_neg,
                    Token::Add => is_neg,
                    _ => return Err("Expected `-` or `+` before term".to_string()),
                };

                while tokens
                    .pop_front_if(|tok| matches!(tok, Token::Sub))
                    .is_some()
                {
                    is_neg = !is_neg
                }

                let coefficient =
                    tokens
                        .pop_front_if(|t| matches!(t, Token::Num(_)))
                        .map(|coefficient| {
                            let Token::Num(c) = coefficient else {
                                unreachable!("Already checked enum variant")
                            };

                            c
                        });

                let mut current_token;

                loop {
                    current_token = tokens.pop_front_if(|t| !matches!(t, Token::Add | Token::Sub));

                    let Some(token) = current_token else {
                        break;
                    };

                    match token {
                        Token::Num(ratio) => todo!(),
                        Token::Var(_) => todo!(),
                        Token::Div { numer, denom } => todo!(),
                        Token::Brackets(tokens) => todo!(),
                        Token::Sub | Token::Add => unreachable!("Already checked enum variant"),
                    }
                }

                if current_token.is_none() && coefficient.is_none() {
                    return Err("Expected either variable or constant after `+`".to_string());
                }
            }

            Ok(())
        };

        token_parser(false, &mut left_tokens)?;
        token_parser(true, &mut right_tokens)?;

        Ok(me)
    }
}

enum Token<'a> {
    Sub,
    Add,
    Num(BigRational),
    Var(&'a str),
    Div {
        numer: Box<Token<'a>>,
        denom: Box<Token<'a>>,
    },
    Brackets(VecDeque<Token<'a>>),
}

fn tokenize<'a>(s: &'a str) -> Result<VecDeque<Token<'a>>, String> {
    fn find_next_char(s: &str, idx: usize) -> Option<(char, usize)> {
        fn get_char(s: &str, idx: usize) -> Option<(char, usize)> {
            let next_pos = s.ceil_char_boundary(idx + 1);

            if idx >= next_pos {
                return None;
            }

            let ch = char::from_str(s.get(idx..next_pos)?).unwrap();

            Some((ch, next_pos))
        }

        let (mut ch, mut next_pos) = get_char(s, idx)?;

        while ch.is_whitespace() {
            (ch, next_pos) = get_char(s, next_pos)?;
        }

        Some((ch, next_pos))
    }

    fn next_token<'a>(next_pos: &mut usize, s: &'a str) -> Result<Option<Token<'a>>, String> {
        let start = *next_pos;

        let Some((next_char, end_of_char)) = find_next_char(s, start) else {
            return Ok(None);
        };

        *next_pos = end_of_char;

        let token = match next_char {
            '-' => Token::Sub,
            '+' => Token::Add,
            '(' => {
                let mut brackets_depth = 1;

                let end_idx = s[*next_pos..]
                    .char_indices()
                    .find_map(|(idx, ch)| {
                        if ch == '(' {
                            brackets_depth += 1
                        } else if ch == ')' {
                            brackets_depth -= 1
                        }

                        (brackets_depth == 0).then_some(idx + *next_pos)
                    })
                    .ok_or("Missing closing bracket".to_string())?;

                let vec = tokenize(&s[*next_pos..end_idx])?;

                *next_pos = end_idx + 1;

                Token::Brackets(vec)
            }
            d if d.is_ascii_digit() => {
                let end = s[*next_pos..]
                    .find(|ch: char| !(ch.is_ascii_digit()))
                    .map_or(s.len(), |i| i + *next_pos);

                *next_pos = end;

                let mut num =
                    BigRational::from_str(s[start..*next_pos].trim()).map_err(|e| e.to_string())?;

                if let Some(('.', start)) = find_next_char(s, *next_pos) {
                    let end = s[start..]
                        .find(|ch: char| !(ch.is_ascii_digit()))
                        .map_or(s.len(), |i| i + start);

                    *next_pos = end;

                    let s = s[start..*next_pos].trim();

                    if s.is_empty() {
                        return Err("Character `.` should be followd by a decimal part".to_string());
                    }

                    let decimal = BigRational::from_str(s)
                        .map_err(|e| e.to_string())? / BigRational::from_usize(s.len()).expect("usize can fit inside a bigint");

                    num += decimal
                };

                Token::Num(num)
            }
            ch if ch.is_ascii_alphabetic() => {
                let end = s[*next_pos..]
                    .find(|ch: char| !ch.is_ascii_alphabetic())
                    .map_or(s.len(), |i| i + *next_pos);

                *next_pos = end;

                Token::Var(s[start..*next_pos].trim())
            }
            ch => return Err(format!("Unrecognized token start `{ch}`")),
        };

        Ok(Some(token))
    }

    let mut tokens = VecDeque::new();

    let mut next_pos = 0;

    loop {
        let Some(mut token) = next_token(&mut next_pos, s)? else {
            break;
        };

        while let Some(('/', ch_boundary)) = find_next_char(s, next_pos) {
            next_pos = ch_boundary;

            let denom = next_token(&mut next_pos, s)?.ok_or("Expected token after `/`")?;

            token = Token::Div {
                numer: Box::new(token),
                denom: Box::new(denom),
            }
        }

        tokens.push_back(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod test {
    use std::{collections::VecDeque, str::FromStr};

    use num::{BigInt, BigRational, FromPrimitive};

use crate::{Equation, parser::{Token, tokenize}};

    #[test]
    fn test_brackets_recursive_empty() {
        let text = "(((())))";

        let mut tokens = tokenize(text).expect("Valid token stream");

        fn next_layer(tokens: &mut VecDeque<Token<'_>>, nth: usize) {
            let Token::Brackets(inner) = tokens
                .pop_front()
                .expect(&format!("{nth} layer should be non empty"))
            else {
                panic!("inner was not brackets, layer {}", nth)
            };

            assert!(
                tokens.is_empty(),
                "{nth} layer had more than one token in inner"
            );

            *tokens = inner
        }

        next_layer(&mut tokens, 1);
        next_layer(&mut tokens, 2);
        next_layer(&mut tokens, 3);
        next_layer(&mut tokens, 4);

        assert!(tokens.is_empty(), "Last layer is non empty")
    }

    // Covers the basic tokenization contract: a single variable name should come through as one `Var` token and not be split into unrelated pieces.
    #[test]
    fn test_tokenize_single_variable() {
        let tokens = tokenize("x").expect("Variable should tokenize successfully");

        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens.front(), Some(Token::Var("x"))));
    }

    // Covers the arithmetic operator path: a simple `a + b` expression should produce a stable sequence of tokens that preserves both the variable and the plus separator.
    #[test]
    fn test_tokenize_basic_addition() {
        let tokens = tokenize("a + b").expect("Basic addition should tokenize successfully");

        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Var("a")));
        assert!(matches!(tokens[1], Token::Add));
        assert!(matches!(tokens[2], Token::Var("b")));
    }

    // Covers nested grouping: repeated parentheses should create a bracketed token tree instead of flattening all the content into one token stream.
    #[test]
    fn test_tokenize_nested_brackets() {
        let tokens = tokenize("((x + y))").expect("Nested brackets should tokenize successfully");

        assert_eq!(tokens.len(), 1);
        let Token::Brackets(inner) = tokens.front().expect("Top-level token should be brackets") else {
            panic!("Expected nested bracket token at the top level")
        };

        assert_eq!(inner.len(), 1);
        let Token::Brackets(inner_inner) = inner.front().expect("Inner token should be brackets") else {
            panic!("Expected second layer of brackets")
        };

        assert!(matches!(inner_inner.front(), Some(Token::Var("x"))));
    }

    // Covers slash-form expressions: `6/2` should be treated as a single numeric term, exercising the parser's division handling without needing the full equation logic.
    #[test]
    fn test_tokenize_fraction_literal() {
        let tokens = tokenize("6/2").expect("Fraction literal should tokenize successfully");

        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens.front(), Some(Token::Num(_))));
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
        assert!(equation.sum == BigRational::from_integer(12.into()));
    }
}
