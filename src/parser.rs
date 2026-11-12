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

                if let Some(('/', start)) = find_next_char(s, *next_pos) {
                    let end = s[start..]
                        .find(|ch: char| !(ch.is_ascii_digit()))
                        .map_or(s.len(), |i| i + start);

                    *next_pos = end;

                    let denom = BigRational::from_str(s[start..*next_pos].trim())
                        .map_err(|e| e.to_string())?;

                    num /= denom
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
    use std::collections::VecDeque;

    use crate::parser::{Token, tokenize};

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
}
