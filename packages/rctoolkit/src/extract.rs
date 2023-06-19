use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Subcommand};

use flate2::read::DeflateDecoder;

use rmp_serde;

use dune2::{Bitmap};

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

#[derive(Args)]
pub struct Dune2SpriteCommandArgs {
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
    Sprites(Dune2SpriteCommandArgs),
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
 * Extract Tiles
 *****************************************************************************/
fn extract_tiles(
    rc: &dune2::Resources,
    args: &Dune2TilesCommandArgs,
) -> Result<(), Box<dyn Error>> {
    for tileset in rc.tilesets.keys() {
        let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tilesets")?).join(tileset);

        fs::create_dir_all(&output_dir)?;

        for i in 0..rc.tilesets.get(tileset).unwrap().tiles.len() {
            let bitmap = rc.tile_bitmap(tileset, i, None);
            let src_rect = bitmap.rect();

            let mut image = BMPImage::new(args.scale*bitmap.size());
            let dst_rect = image.rect();

            dune2::bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
            image.save(output_dir.join(format!("{:02}.bmp", i)))?;
        }
    }

    return Ok(());
}

/******************************************************************************
 * Extract Tilemaps
 *****************************************************************************/
fn extract_tilemaps(
    rc: &dune2::Resources,
    args: &Dune2TilemapCommandArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tilemaps")?);

    fs::create_dir_all(&output_dir)?;

    for i in 0..rc.tilemaps.len() {
        let output_filepath = output_dir.join(format!("{:02}.bmp", i));

        let bitmap = rc.tilemap_bitmap(i, Some(faction));
        let src_rect = bitmap.rect();

        let mut image = BMPImage::new(args.scale*bitmap.size());
        let dst_rect = image.rect();

        dune2::bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
        image.save(output_filepath)?;
    }

    return Ok(());
}

/******************************************************************************
 * Extract Sprites
 *****************************************************************************/
fn extract_sprites(
    rc: &dune2::Resources,
    args: &Dune2SpriteCommandArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;

    for sprite in rc.sprites.keys() {
        let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tilesets")?).join(sprite);

        fs::create_dir_all(&output_dir)?;

        for i in 0..rc.sprites.get(sprite).unwrap().frames.len() {
            let bitmap = rc.sprite_frame_bitmap(sprite, i, Some(faction));
            let src_rect = bitmap.rect();

            let mut image = BMPImage::new(args.scale*bitmap.size());
            let dst_rect = image.rect();

            dune2::bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
            image.save(output_dir.join(format!("{:02}.bmp", i)))?;
        }
    }

    Ok(())
}

/******************************************************************************
 * RUN
 *****************************************************************************/
pub fn run(args: &Cli) -> Result<(), Box<dyn Error>> {
    let mut inflate_reader = DeflateDecoder::new(fs::File::open(&args.input_rc_file)?);
    let rc: dune2::Resources = rmp_serde::from_read(&mut inflate_reader)?;

    match &args.command {
        Commands::Palette(args) => extract_palette(&rc, args),
        Commands::Tiles(args) => extract_tiles(&rc, args),
        Commands::Tilemaps(args) => extract_tilemaps(&rc, args),
        Commands::Sprites(args) => extract_sprites(&rc, args),
    }
}
