use aba2sat::{aba::Aba, parser};
use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    name = "count-loops",
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
)]
pub struct Args {
    /// File to load the aba from
    #[arg(long, short, value_name = "PATH")]
    pub file: PathBuf,
    /// Maximum number of loops to break before starting the solving process.
    /// Will use the number of atoms by default.
    #[arg(long, short = 'l', value_name = "COUNT")]
    pub max_loops: Option<usize>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("opening aba file: {_0}")]
    OpeningAbaFile(#[from] std::io::Error),
    #[error("parsing aba file: {_0}")]
    ParsingAbaFile(#[from] aba2sat::Error),
}

fn count_loops(aba: &Aba, max_loops: Option<usize>) -> usize {
    aba2sat::aba::loops_of(aba)
        .take(max_loops.unwrap_or(usize::MAX))
        .count()
}

fn __main() -> Result<(), Error> {
    let args = Args::parse();
    let content = std::fs::read_to_string(args.file).map_err(Error::OpeningAbaFile)?;
    let aba = parser::aba_file(&content)?;
    println!("{}", count_loops(&aba, args.max_loops));
    Ok(())
}

fn main() -> Result<(), Error> {
    __main().inspect_err(|why| eprintln!("Error: {why}"))
}

#[cfg(test)]
mod tests {
    use aba2sat::aba::{debug::DebugAba, Aba};

    use crate::count_loops;

    #[test]
    pub fn empty_aba() {
        let aba = Aba::default();
        assert_eq!(count_loops(&aba, None), 0);
    }

    #[test]
    pub fn one_loop() {
        let aba = DebugAba::default()
            .with_assumption('a', 'c')
            .with_rule('b', ['a'])
            .with_rule('b', ['c'])
            .with_rule('c', ['b']);
        assert_eq!(count_loops(aba.aba(), None), 1);
    }
}
