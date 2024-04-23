use aba2sat::{
    aba::{Aba, Num},
    parser,
};
use graph_cycles::Cycles;
use iter_tools::Itertools;
use std::{collections::BTreeMap, path::PathBuf};

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
    let mut graph = petgraph::graph::DiGraph::<Num, ()>::new();
    let universe = aba
        .universe()
        .unique()
        .scan(&mut graph, |graph, element| {
            let idx = graph.add_node(*element);
            Some((*element, idx))
        })
        .collect::<BTreeMap<_, _>>();
    aba.rules
        .iter()
        .flat_map(|(head, body)| body.iter().map(|body_element| (*body_element, *head)))
        .for_each(|(from, to)| {
            let from = universe.get(&from).unwrap();
            let to = universe.get(&to).unwrap();
            graph.update_edge(*from, *to, ());
        });
    let mut loops = 0;
    const LOOP_SIZE_IN_MULT_UNIVERSE_SIZE: f32 = 1.0;
    let max_loops = if let Some(max) = max_loops {
        max
    } else {
        (universe.len() as f32 * LOOP_SIZE_IN_MULT_UNIVERSE_SIZE) as usize
    };
    let mut output_printed = false;
    graph.visit_cycles(|_graph, _cycle| {
        loops += 1;
        if loops >= max_loops {
            if !output_printed {
                eprintln!(
                    "Too... many... cycles... Aborting cycle detection with {} cycles.",
                    loops
                );
                output_printed = true;
            }
            std::ops::ControlFlow::Break(())
        } else {
            std::ops::ControlFlow::Continue(())
        }
    });
    loops
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
            .with_rule('b', ['c'])
            .with_rule('c', ['b']);
        assert_eq!(count_loops(aba.aba(), None), 1);
    }
}
