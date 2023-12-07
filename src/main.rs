#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(doc_notable_trait)]

use std::{collections::HashSet, fmt::Write as WriteFmt, fs::read_to_string, io::Write as WriteIo};

use aba::problems::admissibility::{
    DecideCredulousAdmissibility, EnumerateAdmissibleExtensions, SampleAdmissibleExtension,
    VerifyAdmissibleExtension,
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

trait IccmaFormattable {
    fn fmt_iccma(&self) -> Result<String>;
}

fn __main() -> Result {
    let args = args::Args::parse();

    let content = read_to_string(&args.file).map_err(Error::OpeningAbaFile)?;
    let aba = parser::aba_file(&content)?;
    let result = match args.problem {
        args::Problems::VerifyAdmissibility { set } => aba::problems::solve(
            VerifyAdmissibleExtension {
                assumptions: set.into_iter().collect(),
            },
            &aba,
        )?
        .fmt_iccma(),
        args::Problems::EnumerateAdmissibility => {
            aba::problems::multishot_solve(EnumerateAdmissibleExtensions::default(), &aba)?
                .fmt_iccma()
        }
        args::Problems::SampleAdmissibility => {
            aba::problems::solve(SampleAdmissibleExtension, &aba)?.fmt_iccma()
        }
        args::Problems::DecideCredulousAdmissibility { query } => {
            aba::problems::solve(DecideCredulousAdmissibility { assumption: query }, &aba)?
                .fmt_iccma()
        }
    }?;
    let mut stdout = std::io::stdout().lock();
    match writeln!(stdout, "{}", result) {
        Ok(()) => Ok(()),
        Err(why) => match why.kind() {
            std::io::ErrorKind::BrokenPipe => Ok(()),
            _ => Err(Error::Output(why)),
        },
    }
}

fn main() -> Result {
    __main().inspect_err(|why| eprintln!("Error: {why}"))
}

impl IccmaFormattable for Vec<HashSet<u32>> {
    fn fmt_iccma(&self) -> Result<String> {
        let output = self
            .iter()
            .try_fold(String::new(), |mut output, set| -> Result<String> {
                writeln!(output, "{}", set.fmt_iccma()?)?;
                Ok(output)
            })?
            .trim_end()
            .to_owned();
        Ok(output)
    }
}

impl IccmaFormattable for HashSet<u32> {
    fn fmt_iccma(&self) -> Result<String> {
        let set = self
            .iter()
            .try_fold(String::new(), |mut list, num| -> Result<_, Error> {
                write!(list, " {num}")?;
                Result::Ok(list)
            })?;
        Ok(format!("w{set}"))
    }
}

impl IccmaFormattable for bool {
    fn fmt_iccma(&self) -> Result<String> {
        let output = match self {
            true => "YES",
            false => "NO",
        };
        Ok(String::from(output))
    }
}
