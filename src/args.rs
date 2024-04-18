use std::path::PathBuf;

use clap::{Parser, Subcommand};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ARGS: Option<Args> = Args::try_parse().ok();
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
)]
pub struct Args {
    #[command(subcommand)]
    pub problem: Problems,
    /// File to load the aba from
    #[arg(long, short, value_name = "PATH")]
    pub file: PathBuf,
    /// Maximum number of loops to break before starting the solving process.
    /// Will use the number of atoms by default.
    #[arg(long, short = 'l', value_name = "COUNT")]
    pub max_loops: Option<usize>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Subcommand)]
pub enum Problems {
    #[clap(visible_alias = "ve-ad")]
    VerifyAdmissibility {
        #[arg(long, short = 's', required = true)]
        set: Vec<u32>,
    },
    #[clap(visible_alias = "dc-ad")]
    DecideCredulousAdmissibility {
        #[arg(long, short = 'a', required = true)]
        query: u32,
    },
    #[clap(visible_alias = "ee-ad")]
    EnumerateAdmissibility,
    /// Will only return the empty extension if no other is found
    #[clap(visible_alias = "se-ad")]
    SampleAdmissibility,
    #[clap(visible_alias = "ee-co")]
    EnumerateComplete,
    #[clap(visible_alias = "dc-co")]
    DecideCredulousComplete {
        #[arg(long, short = 'a', required = true)]
        query: u32,
    },
}
