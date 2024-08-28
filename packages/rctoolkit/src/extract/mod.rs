mod cli_config;
mod palette;
mod tilemaps;
mod tilesets;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use dune2_rc;


#[derive(clap::Subcommand)]
pub enum Commands {
    /// Extract palette
    Palette(palette::Args),
    /// Extract tilesets
    Tilesets(tilesets::Args),
    /// Extract tilemaps
    Tilemaps(tilemaps::Args),
}

#[derive(clap::Args)]
pub struct Args {
    /// Input file path
    pub input_rc_file: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

pub fn run(args: &Args) -> Result<()> {
    let mut reader = fs::File::open(&args.input_rc_file)?;
    let rc = dune2_rc::Resources::read_from(&mut reader)?;

    match &args.command {
        Commands::Palette(args) => palette::extract(&rc, args),
        Commands::Tilemaps(args) => tilemaps::extract(&rc, args),
        Commands::Tilesets(args) => tilesets::extract(&rc, args),
    }
}
