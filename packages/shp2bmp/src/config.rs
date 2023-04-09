use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, about, version)]
pub struct Cli {
    /// SHP Input file path
    pub shp_input_filepath: PathBuf,

    /// SHP Input file path
    pub pal_input_filepath: PathBuf,

    /// Output folder path
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Faction to export
    #[arg(short = 'F', long, default_value = "harkonnen")]
    pub faction: String,

    /// Overwrite existing files
    #[arg(short = 'f', long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub overwrite: bool,

    /// Export Remap Table
    #[arg(short = 'r', long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub export_remap_table: bool,
}
