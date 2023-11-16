use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub problem: Problems,
    #[arg(long, short)]
    pub file: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum Problems {
    VeAd {
        #[arg(long, short = 's', required = true)]
        set: Vec<u32>,
    },
}
