use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub enum SubCommand {
    Install,
}
#[derive(Debug, Parser)]
pub struct Arguments {
    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>,
    pub yaml_file: Option<PathBuf>,
}
