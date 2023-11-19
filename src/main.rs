#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(result_option_inspect)]

use std::fs::read_to_string;

use aba::problems::VerifyAdmissibility;
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
    println!("{args:?}");

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
