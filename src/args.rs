use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    #[arg(long, short)]
    pub file: PathBuf,
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
}
