use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Subcommand};

use flate2::read::DeflateDecoder;

use rmp_serde;

use dune2::Bitmap;

use crate::image::BMPImage;

#[derive(Args)]
pub struct Dune2PaletteCommandArgs {
    /// Output folder path
    #[arg(short, long, default_value = "palette.bmp")]
    pub output_filepath: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Args)]
pub struct Dune2TilesCommandArgs {
    /// Output folder path
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Scale factor
    #[arg(short = 's', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..=4))]
    pub scale: u32,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Args)]
pub struct Dune2TilemapCommandArgs {
    /// Output folder path
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Faction to export
    #[arg(short = 'F', long, default_value = "harkonnen")]
    pub faction: String,

    /// Scale factor
    #[arg(short = 's', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..=4))]
    pub scale: u32,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Palette(Dune2PaletteCommandArgs),
    Tiles(Dune2TilesCommandArgs),
    Tilemaps(Dune2TilemapCommandArgs),
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
    rc: &dune2::Resources,
    args: &Dune2PaletteCommandArgs,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = args.output_filepath.parent() {
        fs::create_dir_all(parent)?;
    }

    let palette_watch_size = dune2::Size { width: 32, height: 32 };
    let mut palette_image = BMPImage::new(dune2::Size {
        width: 32*16,
        height: 32*((rc.palette.len() as f32)/16.).ceil() as u32,
    });

    for (i, color) in rc.palette.iter() {
        let rect = dune2::Rect::from_point_and_size(
            32*dune2::Point {
                x: (i as u32)%16,
                y: (i as u32)/16,
            },
            palette_watch_size,
        );

        dune2::bitmap::fill_rect(&mut palette_image, &rect, color);
    }

    if args.output_filepath.exists() && !args.force_overwrite {
        return Err("Output file already exists. Use --force to overwrite.".into());
    }

    palette_image.save(&args.output_filepath)?;

    Ok(())
}

/******************************************************************************
 * Extract Tileset
 *****************************************************************************/
fn export_tilemap_to_bmp(
    rc: &dune2::Resources,
    tilemap: &dune2::Tilemap,
    faction: Option<dune2::Faction>,
    scale: u32,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    if let Some(tileset) = rc.tilesets.get(&tilemap.tileset) {
        let bitmap = tilemap.bitmap(tileset, &rc.palette, faction);
        let src_rect = bitmap.rect();

        let mut image = BMPImage::new(scale*bitmap.size());
        let dst_rect = image.rect();

        dune2::bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);

        image.save(output_filepath)?;

        return Ok(());
    }

    Err(format!("Tileset '{}' not found.", tilemap.tileset).into())
}

fn extract_tileset(
    rc: &dune2::Resources,
    args: &Dune2TilesCommandArgs,
) -> Result<(), Box<dyn Error>> {
    for (name, tileset) in rc.tilesets.iter() {
        let tilemaps = Vec::from_iter((0..tileset.tiles.len()).map(|i| {
            dune2::Tilemap {
                remapable: false,
                shape: dune2::Shape { rows: 1, columns: 1, },
                tiles: vec![i],
                tileset: name.clone(),
            }
        }));

        let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tileset")?).join(name);

        fs::create_dir_all(&output_dir)?;

        for (i, tilemap) in tilemaps.iter().enumerate() {
            let output_filepath = output_dir.join(format!("{:02}.bmp", i));
            export_tilemap_to_bmp(
                &rc,
                &tilemap,
                None,
                args.scale,
                &output_filepath
            )?;
        }
    }

    return Ok(());
}

fn extract_tilemaps(
    rc: &dune2::Resources,
    args: &Dune2TilemapCommandArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tileset")?);

    fs::create_dir_all(&output_dir)?;

    for (i, tilemap) in rc.tilemaps.iter().enumerate() {
        let output_filepath = output_dir.join(format!("{:02}.bmp", i));
        let faction = if tilemap.remapable { Some(faction) } else { None };
        export_tilemap_to_bmp(
            &rc,
            &tilemap,
            faction,
            args.scale,
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
    let rc: dune2::Resources = rmp_serde::from_read(&mut inflate_reader)?;

    match &args.command {
        Commands::Palette(args) => extract_palette(&rc, args),
        Commands::Tiles(args) => extract_tileset(&rc, args),
        Commands::Tilemaps(args) => extract_tilemaps(&rc, args),
    }
}
