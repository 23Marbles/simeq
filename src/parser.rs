use std::{
    collections::{HashMap, VecDeque},
    ops::Neg,
    str::FromStr,
};

use num::BigRational;

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

        fn handle_token(
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
                    _ => return Err("Expected `-` or `+`".to_string()),
                };

                while tokens
                    .pop_front_if(|tok| matches!(tok, Token::Sub))
                    .is_some()
                {
                    is_neg = !is_neg
                }

                let (coefficient, var) = match tokens.pop_front() {
                    Some(Token::Num(num)) => {
                        if let Some(Token::Var(name)) =
                            tokens.pop_front_if(|t| matches!(t, Token::Var(_)))
                        {
                            (num, Some(name))
                        } else {
                            (num, None)
                        }
                    }
                    Some(Token::Var(name)) => (BigRational::ONE, Some(name)),
                    _ => return Err("Expected either variable or constant after `+`".to_string()),
                };

                handle_token(is_neg, &mut me, var, coefficient);
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

    let mut tokens = VecDeque::new();

    let mut next_pos = 0;

    loop {
        let start = next_pos;

        let Some((next_char, end_of_char)) = find_next_char(s, start) else {
            break;
        };

        next_pos = end_of_char;

        let token = match next_char {
            '-' => Token::Sub,
            '+' => Token::Add,
            d if d.is_ascii_digit() => {
                let end = s[next_pos..]
                    .find(|ch: char| !(ch.is_ascii_digit()))
                    .map_or(s.len(), |i| i + next_pos);

                next_pos = end;

                let mut num =
                    BigRational::from_str(s[start..next_pos].trim()).map_err(|e| e.to_string())?;

                if let Some(('/', start)) = find_next_char(s, next_pos) {
                    let end = s[start..]
                        .find(|ch: char| !(ch.is_ascii_digit()))
                        .map_or(s.len(), |i| i + start);

                    next_pos = end;

                    let denom = BigRational::from_str(s[start..next_pos].trim())
                        .map_err(|e| e.to_string())?;

                    num /= denom
                };

                Token::Num(num)
            }
            ch if ch.is_ascii_alphabetic() => {
                let end = s[next_pos..]
                    .find(|ch: char| !ch.is_ascii_alphabetic())
                    .map_or(s.len(), |i| i + next_pos);

                next_pos = end;

                Token::Var(s[start..next_pos].trim())
            }
            _ => return Err(format!("Unrecognized token start {next_char}")),
        };

        tokens.push_back(token);
    }

    Ok(tokens)
}
