use aba2sat::{
    aba::{Aba, Loops},
    parser, STOP_LOOP_COUNTING,
};
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
    Loops::of(aba, max_loops).count()
}

fn __main() -> Result<(), Error> {
    // Init logger
    pretty_env_logger::init();
    // Register SIGUSR1 handler
    signal_hook::flag::register(signal_hook::consts::SIGUSR1, STOP_LOOP_COUNTING.clone())?;
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

    #[test]
    pub fn k_3() {
        let aba = DebugAba::default()
            .with_assumption('a', 'c')
            .with_rule('b', ['a'])
            .with_rule('c', ['a'])
            .with_rule('d', ['a'])
            .with_rule('c', ['b'])
            .with_rule('d', ['b'])
            .with_rule('b', ['c'])
            .with_rule('d', ['c'])
            .with_rule('b', ['d'])
            .with_rule('c', ['d']);
        assert_eq!(count_loops(aba.aba(), None), 4);
    }

    #[test]
    pub fn k_3_minus_one() {
        let aba = DebugAba::default()
            .with_assumption('a', 'c')
            .with_rule('b', ['a'])
            .with_rule('c', ['a'])
            .with_rule('d', ['a'])
            .with_rule('d', ['b'])
            .with_rule('b', ['c'])
            .with_rule('d', ['c'])
            .with_rule('b', ['d'])
            .with_rule('c', ['d']);
        assert_eq!(count_loops(aba.aba(), None), 3);
    }

    #[test]
    pub fn k_3_minus_two() {
        let aba = DebugAba::default()
            .with_assumption('a', 'b')
            .with_rule('b', ['a'])
            .with_rule('c', ['a'])
            .with_rule('d', ['a'])
            .with_rule('d', ['b'])
            .with_rule('d', ['c'])
            .with_rule('b', ['d'])
            .with_rule('c', ['d']);
        assert_eq!(count_loops(aba.aba(), None), 3);
    }

    #[test]
    pub fn k_5() {
        let aba = DebugAba::default()
            .with_assumption('a', 'c')
            .with_rule('b', ['a'])
            .with_rule('c', ['a'])
            .with_rule('d', ['a'])
            .with_rule('e', ['a'])
            .with_rule('f', ['a'])
            .with_rule('c', ['b'])
            .with_rule('d', ['b'])
            .with_rule('e', ['b'])
            .with_rule('f', ['b'])
            .with_rule('b', ['c'])
            .with_rule('d', ['c'])
            .with_rule('e', ['c'])
            .with_rule('f', ['c'])
            .with_rule('b', ['d'])
            .with_rule('c', ['d'])
            .with_rule('e', ['d'])
            .with_rule('f', ['d'])
            .with_rule('b', ['e'])
            .with_rule('c', ['e'])
            .with_rule('d', ['e'])
            .with_rule('f', ['e'])
            .with_rule('b', ['f'])
            .with_rule('c', ['f'])
            .with_rule('d', ['f'])
            .with_rule('e', ['f']);
        assert_eq!(count_loops(aba.aba(), None), 26);
    }

    #[test]
    pub fn k_5_adjusted() {
        let aba = DebugAba::default()
            .with_assumption('a', 'c')
            .with_rule('b', ['a'])
            .with_rule('c', ['a'])
            .with_rule('d', ['a'])
            .with_rule('e', ['a'])
            .with_rule('f', ['a'])
            .with_rule('d', ['b'])
            .with_rule('e', ['b'])
            .with_rule('f', ['b'])
            .with_rule('b', ['c'])
            .with_rule('d', ['c'])
            .with_rule('e', ['c'])
            .with_rule('f', ['c'])
            .with_rule('b', ['d'])
            .with_rule('c', ['d'])
            .with_rule('e', ['d'])
            .with_rule('f', ['d'])
            .with_rule('b', ['e'])
            .with_rule('c', ['e'])
            .with_rule('d', ['e'])
            .with_rule('f', ['e'])
            .with_rule('b', ['f'])
            .with_rule('c', ['f'])
            .with_rule('d', ['f'])
            .with_rule('e', ['f']);
        assert_eq!(count_loops(aba.aba(), None), 25);
    }
}
