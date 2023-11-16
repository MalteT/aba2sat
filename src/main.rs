#![feature(iter_collect_into)]
#![feature(iter_intersperse)]

use std::fs::read_to_string;

use aba::problems::VerifyAdmissibility;
use clap::Parser;

use crate::error::{Error, Result};

pub mod aba;
pub mod args;
pub mod clauses;
pub mod error;
pub mod literal;
pub mod mapper;
pub mod parser;
#[cfg(test)]
mod tests;

fn main() -> Result {
    let args = args::Args::parse();
    println!("{args:?}");

    match args.problem {
        args::Problems::VeAd { set } => {
            let content = read_to_string(&args.file).map_err(Error::OpeningAbaFile)?;
            let aba = parser::aba_file(&content)?;
            let result = aba::problems::solve(
                VerifyAdmissibility {
                    assumptions: set.into_iter().collect(),
                },
                &aba,
            );
            print_bool_result(result);
        }
    }
    Ok(())
}

fn print_bool_result(result: bool) {
    match result {
        true => println!("YES"),
        false => println!("NO"),
    }
}
