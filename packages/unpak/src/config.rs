use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, about, version)]
pub struct Cli {
    /// Input file path
    pub input_filepath: Option<PathBuf>,

    /// Output folder path
    #[arg(short = 'd', long, default_value = "pak_output")]
    pub output_dir: PathBuf,

    /// Overwrite existing files
    #[arg(short = 'f', long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub overwrite: bool,

    /// Verbose mode
    #[arg(short = 'v', long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub verbose: bool,

    /// List files
    #[arg(short = 'l', long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub list: bool,
}
