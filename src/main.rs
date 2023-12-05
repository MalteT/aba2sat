#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(result_option_inspect)]

use std::{collections::HashSet, fmt::Write, fs::read_to_string};

use aba::problems::VerifyAdmissibility;
use clap::Parser;

use crate::{
    aba::problems::Admissibility,
    error::{Error, Result},
};

#[cfg(test)]
macro_rules! set {
    ($($elem:expr),*) => {{
        vec![$($elem),*].into_iter().collect()
    }}
}

#[cfg(test)]
macro_rules! map {
    ($($from:expr => $to:expr),*) => {{
        vec![$(($from, $to)),*].into_iter().collect()
    }}
}

pub mod aba;
pub mod args;
pub mod clauses;
pub mod error;
pub mod literal;
pub mod mapper;
pub mod parser;
#[cfg(test)]
mod tests;

fn __main() -> Result {
    let args = args::Args::parse();

    match args.problem {
        args::Problems::VerifyAdmissibility { set } => {
            let content = read_to_string(&args.file).map_err(Error::OpeningAbaFile)?;
            let aba = parser::aba_file(&content)?;
            let result = aba::problems::solve(
                VerifyAdmissibility {
                    assumptions: set.into_iter().collect(),
                },
                &aba,
            )?;
            print_bool_result(result);
        }
        args::Problems::EnumerateAdmissibility => {
            let content = read_to_string(&args.file).map_err(Error::OpeningAbaFile)?;
            let aba = parser::aba_file(&content)?;
            let result = aba::problems::multishot_solve(Admissibility::default(), &aba)?;
            print_witnesses_result(result)?;
        }
    }
    Ok(())
}

fn main() -> Result {
    __main().inspect_err(|why| eprintln!("Error: {why}"))
}

fn print_bool_result(result: bool) {
    match result {
        true => println!("YES"),
        false => println!("NO"),
    }
}

fn print_witnesses_result(result: Vec<HashSet<u32>>) -> Result {
    result.into_iter().try_for_each(|set| {
        let set = set
            .into_iter()
            .try_fold(String::new(), |mut list, num| -> Result<_, Error> {
                write!(list, " {num}")?;
                Result::Ok(list)
            })?;
        println!("w{set}");
        Ok(())
    })
}
