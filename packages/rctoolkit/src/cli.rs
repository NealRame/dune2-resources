use std::path::PathBuf;

use clap::{Parser, Subcommand, Args};

#[derive(Args)]
pub struct CreateArgs {
    /// Input file path
    pub config_filepath: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,

    /// Output folder path
    #[arg(long, short, default_value = "dune2.rc")]
    pub output_file: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
}

#[derive(Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
