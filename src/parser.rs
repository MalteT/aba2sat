//! Parser for the ABA format defined by the [ICCMA](https://argumentationcompetition.org/2023/rules.html).
//!
//! 1. atoms are positive numbers from 1..=n
//! 2. the first line is a unique p-line `p aba <n>`
//! 3. lines end with a single new-line character
//! 4. a line starting with `#` is a comment
//! 5. aba lines
//!    1. `a <x>`: x is an assumption
//!    2. `c <x> <y>`: y is the inverse of x
//!    3. `r <h> <b1> ... <bl>`: a rule with head h and body b1,...,bl
//! 6. no other lines may exist
//!
//! # Example
//!
//! ```text
//! p aba 8
//! # this is a comment
//! a 1
//! a 2
//! a 3
//! c 1 6
//! c 2 7
//! c 3 8
//! r 4 5 1
//! r 5
//! r 6 2 3
//! ```
use std::collections::HashSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{self, newline, not_line_ending, space0, space1};
use nom::combinator::{all_consuming, eof, map, verify};
use nom::multi::fold_many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

use crate::aba::Aba;
use crate::error::Result;

pub fn aba_file(input: &str) -> Result<Aba<u32>> {
    let (input, number_of_atoms) = p_line(input)?;
    let (_, aba) = all_consuming(aba)(input)?;
    #[cfg(debug_assertions)]
    {
        if aba.size() != number_of_atoms as usize {
            eprintln!("Number of atoms did not match p-line!");
        }
    }
    Ok(aba)
}

fn p_line(input: &str) -> IResult<&str, u32> {
    map(
        tuple((
            tag("p"),
            space1,
            tag("aba"),
            space1,
            complete::u32,
            space0,
            newline,
        )),
        |(_, _, _, _, num, _, _)| num,
    )(input)
}

#[derive(Debug, PartialEq)]
enum AbaLine {
    Comment,
    Assumption(u32),
    Inverse(u32, u32),
    Rule(u32, HashSet<u32>),
}

fn aba(input: &str) -> IResult<&str, Aba<u32>> {
    let parse_aba_line = terminated(alt((assumption, comment, inverse, rule)), eol_or_eoi);
    let collect_aba = fold_many0(
        parse_aba_line,
        || (Aba::default(), HashSet::new()),
        |(aba, mut assumptions), line: AbaLine| match line {
            AbaLine::Comment => (aba, assumptions),
            AbaLine::Assumption(assumption) => {
                assumptions.insert(assumption);
                (aba, assumptions)
            }
            AbaLine::Inverse(from, to) => (aba.with_assumption(from, to), assumptions),
            AbaLine::Rule(head, body) => (aba.with_rule(head, body), assumptions),
        },
    );
    map(
        verify(collect_aba, |(aba, assumptions)| {
            let with_inverses = aba.inverses.keys().copied().collect();
            let wrong = assumptions.symmetric_difference(&with_inverses);
            wrong.collect::<Vec<_>>().is_empty()
        }),
        |(aba, _)| aba,
    )(input)
}

fn assumption(input: &str) -> IResult<&str, AbaLine> {
    map(
        tuple((tag("a"), space1, atom, space0)),
        |(_, _, atom, _)| AbaLine::Assumption(atom),
    )(input)
}

fn comment(input: &str) -> IResult<&str, AbaLine> {
    map(preceded(tag("#"), not_line_ending), |_| AbaLine::Comment)(input)
}

fn inverse(input: &str) -> IResult<&str, AbaLine> {
    map(
        tuple((tag("c"), space1, atom, space1, atom, space0)),
        |(_, _, from, _, to, _)| AbaLine::Inverse(from, to),
    )(input)
}

fn rule(input: &str) -> IResult<&str, AbaLine> {
    let space_atom = preceded(space1, atom);
    let body = fold_many0(space_atom, HashSet::new, |mut set, atom| {
        set.insert(atom);
        set
    });
    map(
        tuple((tag("r"), space1, atom, body, space0)),
        |(_, _, head, body, _)| AbaLine::Rule(head, body),
    )(input)
}

fn eol_or_eoi(input: &str) -> IResult<&str, &str> {
    let newline = map(newline, |_| "\n");
    alt((newline, eof))(input)
}

fn atom(input: &str) -> IResult<&str, u32> {
    verify(complete::u32, |&num| num != 0)(input)
}

#[cfg(test)]
mod tests {
    use nom::combinator::all_consuming;

    use crate::aba::Aba;

    macro_rules! set {
        ($($elem:literal),*) => {{
            vec![$($elem),*].into_iter().collect()
        }}
    }

    macro_rules! map {
        ($($from:literal => $to:literal),*) => {{
            vec![$(($from, $to)),*].into_iter().collect()
        }}
    }

    fn assert_parse<F, T>(parser: F, input: &str, expected: T)
    where
        F: FnMut(&str) -> nom::IResult<&str, T>,
        T: std::fmt::Debug + PartialEq,
    {
        let (_rest, result) =
            all_consuming(parser)(input).expect(&format!("Failed to parse {input:?}"));
        assert_eq!(result, expected);
    }

    fn assert_parse_fail<F, T>(parser: F, input: &str)
    where
        F: FnMut(&str) -> nom::IResult<&str, T>,
    {
        let result = all_consuming(parser)(input);
        assert!(result.is_err(), "Parse on {input:?} did not fail!")
    }

    #[test]
    fn atom() {
        assert_parse(super::atom, "123", 123);
        assert_parse_fail(super::atom, "-123");
        assert_parse_fail(super::atom, "a");
        assert_parse_fail(super::atom, "0");
    }

    #[test]
    fn comment() {
        assert_parse(super::comment, "# okay", super::AbaLine::Comment);
        assert_parse(super::comment, "#okay", super::AbaLine::Comment);
        assert_parse(super::comment, "# \t okay", super::AbaLine::Comment);
        assert_parse_fail(super::comment, "# fail\n");
        assert_parse_fail(super::comment, "fail");
        assert_parse_fail(super::comment, "%fail");
        assert_parse_fail(super::comment, ";fail");
    }

    #[test]
    fn p_line() {
        assert_parse(super::p_line, "p aba 2\n", 2);
        assert_parse(super::p_line, "p aba 0\n", 0);
        assert_parse(super::p_line, "p\taba 0\n", 0);
        assert_parse(super::p_line, "p aba\t0\n", 0);
        assert_parse(super::p_line, "p  aba 0\n", 0);
        assert_parse(super::p_line, "p aba  0\n", 0);
        assert_parse(super::p_line, "p aba 0 \n", 0);
        assert_parse(super::p_line, "p aba 0  \n", 0);
        assert_parse(super::p_line, "p aba 0\t\n", 0);
        assert_parse_fail(super::p_line, "p aba -1\n");
    }

    #[test]
    fn assumption() {
        use super::AbaLine::Assumption;
        assert_parse(super::assumption, "a 1", Assumption(1));
        assert_parse(super::assumption, "a 4294967295", Assumption(u32::MAX));
        assert_parse(super::assumption, "a\t1", Assumption(1));
        assert_parse(super::assumption, "a\t 1", Assumption(1));
        assert_parse(super::assumption, "a\t 1 ", Assumption(1));
        assert_parse(super::assumption, "a \t 1\t", Assumption(1));
        assert_parse(super::assumption, "a \t 1 \t", Assumption(1));
        assert_parse_fail(super::assumption, "a -1");
        assert_parse_fail(super::assumption, "a 0");
        assert_parse_fail(super::assumption, "a");
    }

    #[test]
    fn inverse() {
        use super::AbaLine::Inverse;
        assert_parse(super::inverse, "c 1 6", Inverse(1, 6));
        assert_parse(super::inverse, "c \t 1 \t 6 \t", Inverse(1, 6));
        assert_parse_fail(super::inverse, "c1  6 ");
        assert_parse_fail(super::inverse, "c 1");
        assert_parse_fail(super::inverse, "c -1 6");
        assert_parse_fail(super::inverse, "c 1 -6");
        assert_parse_fail(super::inverse, "c 0 6");
        assert_parse_fail(super::inverse, "c 1 0");
    }

    #[test]
    fn rule() {
        use super::AbaLine::Rule;
        assert_parse(super::rule, "r 4 5 1", Rule(4, set![5, 1]));
        assert_parse(super::rule, "r 5", Rule(5, set![]));
        assert_parse(super::rule, "r\t5\t ", Rule(5, set![]));
        assert_parse(super::rule, "r\t5\t 1", Rule(5, set![1]));
        assert_parse_fail(super::rule, "r 4 5 \n");
        assert_parse_fail(super::rule, "r 0 5");
        assert_parse_fail(super::rule, "r 2 0");
        assert_parse_fail(super::rule, "r -2 5");
        assert_parse_fail(super::rule, "r 2 -5");
    }

    #[test]
    fn aba() {
        assert_parse(
            super::aba,
            r#"a 2
c 2 1
r 1 2 3"#,
            Aba {
                rules: vec![(1, set!(2, 3))],
                inverses: map![2 => 1],
            },
        )
    }

    #[test]
    fn aba_file() {
        let res = super::aba_file(
            r#"p aba 2
a 2
c 2 1
r 1 2 3
"#,
        )
        .unwrap();
        assert_eq!(
            res,
            Aba {
                rules: vec![(1, set!(2, 3))],
                inverses: map![2 => 1],
            }
        )
    }
}
