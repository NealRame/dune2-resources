use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Subcommand};

use flate2::read::DeflateDecoder;

use rmp_serde;

use dune2::Bitmap;

#[derive(Args)]
pub struct PaletteArgs {
    /// Output folder path
    #[arg(short, long, default_value = "palette.bmp")]
    pub output_filepath: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Args)]
pub struct SpritesArgs {
    /// Output folder path
    #[arg(short = 'd', long, default_value = "sprites")]
    pub output_dir: PathBuf,

    /// Faction to export
    #[arg(short = 'F', long, default_value = "harkonnen")]
    pub faction: String,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Args)]
pub struct Tileset {
    /// Output folder path
    #[arg(short = 'd', long, default_value = "tileset")]
    pub output_dir: PathBuf,

    /// Faction to export
    #[arg(short = 'F', long, default_value = "harkonnen")]
    pub faction: String,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Args)]
pub struct Tilemaps {
    /// Output folder path
    #[arg(short = 'd', long, default_value = "tilemaps")]
    pub output_dir: PathBuf,

    /// Faction to export
    #[arg(short = 'F', long, default_value = "harkonnen")]
    pub faction: String,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Palette(PaletteArgs),
    Sprites(SpritesArgs),
    Tileset(Tileset),
    Tilemaps(Tilemaps),
}

#[derive(Args)]
pub struct Cli {
    /// Input file path
    pub input_rc_file: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

/******************************************************************************
 * Extract Palette
 *****************************************************************************/
fn extract_palette(
    rc: &dune2::RC,
    args: &PaletteArgs,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = args.output_filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    let palette_watch_size = dune2::Size { width: 32, height: 32 };
    let mut palette_surface = dune2::Surface::new(dune2::Size {
        width: 32*16,
        height: 32*((rc.palette.len() as f32)/16.).ceil() as u32,
    });

    for (i, color) in rc.palette.iter() {
        let rect = dune2::Rect::from_point_and_size(
            32*dune2::Point {
                x: (i as i32)%16,
                y: (i as i32)/16,
            },
            palette_watch_size,
        );
        palette_surface.fill_rect(&rect, color);
    }

    if args.output_filepath.exists() && !args.force_overwrite {
        return Err("Output file already exists. Use --force to overwrite.".into());
    }

    let mut output = fs::File::create(&args.output_filepath)?;
    dune2::write_bmp_with_palette(&palette_surface, &rc.palette, &mut output)?;

    Ok(())
}

/******************************************************************************
 * Extract Sprites
 *****************************************************************************/
fn export_frame_to_bmp(
    frame: &dune2::SpriteFrame,
    palette: &dune2::Palette,
    faction: dune2::Faction,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let surface = frame.surface(&palette, faction);
    let mut output = fs::File::create(output_filepath)?;
    dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;
    return Ok(());
}

fn extract_sprites(
    rc: &dune2::RC,
    args: &SpritesArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;

    fs::create_dir_all(&args.output_dir)?;

    for (i, frame) in rc.sprites.iter().enumerate() {
        let output_filepath = args.output_dir.join(format!("{:02}.bmp", i));
        export_frame_to_bmp(
            frame,
            &rc.palette,
            faction,
            &output_filepath
        )?;
    }

    return Ok(());
}

/******************************************************************************
 * Extract Tileset
 *****************************************************************************/
fn export_tilemap_to_bmp(
    tilemap: &dune2::Tilemap,
    tileset: &dune2::Tileset,
    palette: &dune2::Palette,
    faction: dune2::Faction,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let surface = tilemap.surface(&palette, &tileset, faction);
    let mut output = fs::File::create(output_filepath)?;
    dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;
    return Ok(());
}

fn extract_tileset(
    rc: &dune2::RC,
    args: &Tileset,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;

    fs::create_dir_all(&args.output_dir)?;

    let tilemaps = Vec::from_iter((0..rc.tileset.tiles.len()).map(|i| {
        dune2::Tilemap {
            shape: dune2::Shape { rows: 1, columns: 1, },
            tiles: vec![i],
        }
    }));

    for (i, tilemap) in tilemaps.iter().enumerate() {
        let output_filepath = args.output_dir.join(format!("{:02}.bmp", i));
        export_tilemap_to_bmp(
            &tilemap,
            &rc.tileset,
            &rc.palette,
            faction,
            &output_filepath
        )?;
    }

    return Ok(());
}

fn extract_tilemaps(
    rc: &dune2::RC,
    args: &Tilemaps,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;

    fs::create_dir_all(&args.output_dir)?;

    for (i, tilemap) in rc.tilemaps.iter().enumerate() {
        let output_filepath = args.output_dir.join(format!("{:02}.bmp", i));
        export_tilemap_to_bmp(
            &tilemap,
            &rc.tileset,
            &rc.palette,
            faction,
            &output_filepath
        )?;
    }

    return Ok(());
}

/******************************************************************************
 * RUN
 *****************************************************************************/
pub fn run(args: &Cli) -> Result<(), Box<dyn Error>> {
    let mut inflate_reader = DeflateDecoder::new(fs::File::open(&args.input_rc_file)?);
    let rc: dune2::RC = rmp_serde::from_read(&mut inflate_reader)?;

    match &args.command {
        Commands::Palette(args) => extract_palette(&rc, args),
        Commands::Sprites(args) => extract_sprites(&rc, args),
        Commands::Tileset(args) => extract_tileset(&rc, args),
        Commands::Tilemaps(args) => extract_tilemaps(&rc, args),
    }
}
