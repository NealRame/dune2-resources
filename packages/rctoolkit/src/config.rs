use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub palette: PathBuf,
    pub tileset: PathBuf,
    pub sprites: Vec<PathBuf>,
}

#[derive(Parser)]
#[command(author, about, version)]
pub struct Cli {
    /// Input file path
    pub config_filepath: PathBuf,

    /// Output folder path
    #[arg(short, long, default_value = "dune2.rc")]
    pub output_file: PathBuf,

    /// Overwrite existing files
    #[arg(short = 'f', long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub overwrite: bool,
}
