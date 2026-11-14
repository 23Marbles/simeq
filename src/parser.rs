use std::{collections::VecDeque, str::FromStr};

use num::BigRational;

use crate::Equation;

impl FromStr for Equation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _terms = str_to_terms(s)?;

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

impl FromStr for TermKind<'_> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = s;

        todo!("TermKind::FromStr impl")
    }
}

struct Term<'a> {
    constant: BigRational,
    kind: TermKind<'a>,
}

impl FromStr for Term<'_> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = s;

        todo!("Term::FromStr impl")
    }
}

fn str_to_terms(s: &str) -> Result<VecDeque<Term<'_>>, String> {
    todo!("Find all the terms for a str")
}

#[cfg(test)]
mod test {
    use std::{collections::VecDeque, str::FromStr};

    use num::{BigInt, BigRational, FromPrimitive};

    use crate::{
        Equation,
        parser::{Token, tokenize},
    };

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
