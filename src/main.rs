#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(doc_notable_trait)]

use std::{collections::HashSet, fmt::Write, fs::read_to_string};

use aba::problems::admissibility::{
    EnumerateAdmissibleExtensions, SampleAdmissibleExtension, VerifyAdmissibleExtension,
};
use clap::Parser;

use crate::error::{Error, Result};

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

mod aba;
mod args;
mod clauses;
mod error;
mod literal;
mod mapper;
mod parser;
#[cfg(test)]
mod tests;

fn __main() -> Result {
    let args = args::Args::parse();

    let content = read_to_string(&args.file).map_err(Error::OpeningAbaFile)?;
    let aba = parser::aba_file(&content)?;
    match args.problem {
        args::Problems::VerifyAdmissibility { set } => {
            let result = aba::problems::solve(
                VerifyAdmissibleExtension {
                    assumptions: set.into_iter().collect(),
                },
                &aba,
            )?;
            print_bool_result(result);
        }
        args::Problems::EnumerateAdmissibility => {
            let result =
                aba::problems::multishot_solve(EnumerateAdmissibleExtensions::default(), &aba)?;
            print_witnesses_result(result)?;
        }
        args::Problems::SampleAdmissibility => {
            let result = aba::problems::solve(SampleAdmissibleExtension, &aba)?;
            print_witness_result(result)?;
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
    result.into_iter().try_for_each(print_witness_result)
}

fn print_witness_result(result: HashSet<u32>) -> Result {
    let set = result
        .into_iter()
        .try_fold(String::new(), |mut list, num| -> Result<_, Error> {
            write!(list, " {num}")?;
            Result::Ok(list)
        })?;
    println!("w{set}");
    Ok(())
}
