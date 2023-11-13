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
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use crate::aba::AbaRaw;
use crate::clauses::Atom;
use crate::error::Result;

pub fn parse_aba_file(input: &str) -> Result<AbaRaw> {
    let (input, number_of_atoms) = parse_p_line(input)?;
    let (_, aba) = all_consuming(parse_aba)(input)?;
    #[cfg(debug_assertions)]
    {
        if aba.size() != number_of_atoms as usize {
            eprintln!("Number of atoms did not match p-line!");
        }
    }
    Ok(aba)
}

fn parse_p_line(input: &str) -> IResult<&str, u32> {
    delimited(tag("p aba "), complete::u32, newline)(input)
}

enum AbaLine {
    Comment,
    Assumption(Atom),
    Inverse(Atom, Atom),
    Rule(Atom, HashSet<Atom>),
}

fn parse_aba(input: &str) -> IResult<&str, AbaRaw> {
    let parse_aba_line = terminated(
        alt((parse_assumption, parse_comment, parse_inverse, parse_rule)),
        eol_or_eoi,
    );
    let collect_aba = fold_many0(
        parse_aba_line,
        || (AbaRaw::default(), HashSet::new()),
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

fn parse_assumption(input: &str) -> IResult<&str, AbaLine> {
    map(
        tuple((tag("a"), space1, atom, space0)),
        |(_, _, atom, _)| AbaLine::Assumption(atom),
    )(input)
}

fn parse_comment(input: &str) -> IResult<&str, AbaLine> {
    map(preceded(tag("#"), not_line_ending), |_| AbaLine::Comment)(input)
}
fn parse_inverse(input: &str) -> IResult<&str, AbaLine> {
    map(
        tuple((tag("c"), space1, atom, space1, atom, space0)),
        |(_, _, from, _, to, _)| AbaLine::Inverse(from, to),
    )(input)
}
fn parse_rule(input: &str) -> IResult<&str, AbaLine> {
    let space_atom = preceded(space1, atom);
    let body = fold_many0(space_atom, HashSet::new, |mut set, atom| {
        set.insert(atom);
        set
    });
    map(
        tuple((tag("r"), space1, atom, space1, body, space0)),
        |(_, _, head, _, body, _)| AbaLine::Rule(head, body),
    )(input)
}

fn eol_or_eoi(input: &str) -> IResult<&str, &str> {
    let newline = map(newline, |_| "\n");
    alt((newline, eof))(input)
}

fn atom(input: &str) -> IResult<&str, Atom> {
    complete::u32(input)
}
