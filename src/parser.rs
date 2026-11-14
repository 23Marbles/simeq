use std::{collections::VecDeque, str::FromStr};

use num::BigRational;

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
    constant: BigRational,
    kind: TermKind<'a>,
}

impl<'a> Term<'a> {
    fn greedy_parse(s: &mut &'a str) -> Result<Self, String> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use num::{BigRational};

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
