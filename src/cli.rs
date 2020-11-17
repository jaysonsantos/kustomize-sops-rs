use std::path::PathBuf;

use clap::Clap;

#[derive(Debug, Clap)]
pub enum SubCommand {
    Install,
}
#[derive(Debug, Clap)]
pub struct Arguments {
    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>,
    pub yaml_file: Option<PathBuf>,
}
