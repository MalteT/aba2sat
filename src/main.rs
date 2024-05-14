#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(doc_notable_trait)]

use std::{collections::HashSet, fmt::Write as WriteFmt, fs::read_to_string, io::Write as WriteIo};

use aba2sat::{
    aba::{
        self,
        problems::{
            admissibility::{
                DecideCredulousAdmissibility, EnumerateAdmissibleExtensions,
                SampleAdmissibleExtension, VerifyAdmissibleExtension,
            },
            complete::{DecideCredulousComplete, EnumerateCompleteExtensions},
        },
        Num,
    },
    Error,
};
use aba2sat::{parser, Result};
use args::ARGS;
use clap::Parser;

mod args;

trait IccmaFormattable {
    fn fmt_iccma(&self) -> Result<String>;
}

fn __main() -> Result {
    pretty_env_logger::init();
    let args = match &*ARGS {
        Some(args) => args,
        None => {
            args::Args::parse();
            unreachable!()
        }
    };
    let content = read_to_string(&args.file).map_err(Error::OpeningAbaFile)?;
    let aba = parser::aba_file(&content)?;
    let result = match &args.problem {
        args::Problems::VerifyAdmissibility { set } => aba::problems::solve(
            VerifyAdmissibleExtension {
                assumptions: set.iter().cloned().collect(),
            },
            aba,
            args.max_loops,
        )?
        .fmt_iccma(),
        args::Problems::EnumerateAdmissibility => aba::problems::multishot_solve(
            EnumerateAdmissibleExtensions::default(),
            aba,
            args.max_loops,
        )?
        .fmt_iccma(),
        args::Problems::SampleAdmissibility => {
            aba::problems::solve(SampleAdmissibleExtension, aba, args.max_loops)?.fmt_iccma()
        }
        args::Problems::DecideCredulousAdmissibility { query } => aba::problems::solve(
            DecideCredulousAdmissibility { element: *query },
            aba,
            args.max_loops,
        )?
        .fmt_iccma(),
        args::Problems::EnumerateComplete => aba::problems::multishot_solve(
            EnumerateCompleteExtensions::default(),
            aba,
            args.max_loops,
        )?
        .fmt_iccma(),
        args::Problems::DecideCredulousComplete { query } => aba::problems::solve(
            DecideCredulousComplete { element: *query },
            aba,
            args.max_loops,
        )?
        .fmt_iccma(),
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

impl IccmaFormattable for Vec<HashSet<Num>> {
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

impl IccmaFormattable for HashSet<Num> {
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
