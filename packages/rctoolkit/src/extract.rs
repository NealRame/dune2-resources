use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Subcommand, ValueEnum};

use dune2_rc::{self as dune2, Bitmap};

use crate::image::BMPImage;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CliExtractFaction {
    Harkonnen,
    Atreides,
    Ordos,
    Fremen,
    Sardaukar,
    Mercenary,
}

impl Into<dune2::constants::Faction> for CliExtractFaction {
    fn into(self) -> dune2::constants::Faction {
        match self {
            Self::Harkonnen => dune2::constants::Faction::Harkonnen,
            Self::Atreides => dune2::constants::Faction::Atreides,
            Self::Ordos => dune2::constants::Faction::Ordos,
            Self::Fremen => dune2::constants::Faction::Fremen,
            Self::Sardaukar => dune2::constants::Faction::Sardaukar,
            Self::Mercenary => dune2::constants::Faction::Mercenary,
        }
    }
}

// ============================================================================
// Extract Palette
// ============================================================================
#[derive(Args)]
pub struct CliExtractPaletteCommandArgs {
    /// Output folder path
    #[arg(short, long, default_value = "palette.bmp")]
    pub output_filepath: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

fn extract_palette(
    rc: &dune2::Resources,
    args: &CliExtractPaletteCommandArgs,
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

// ============================================================================
// Extract Tiles
// ============================================================================
#[derive(Args)]
pub struct CliExtractTilesCommandArgs {
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

fn extract_tiles(
    rc: &dune2::Resources,
    args: &CliExtractTilesCommandArgs,
) -> Result<(), Box<dyn Error>> {
    for tileset in rc.tilesets.keys() {
        let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tilesets")?).join(tileset);

        fs::create_dir_all(&output_dir)?;

        for i in 0..rc.tilesets.get(tileset).unwrap().tiles.len() {
            let bitmap = rc.tile_bitmap(tileset, i, None)?;
            let src_rect = bitmap.rect();

            let mut image = BMPImage::new(args.scale*bitmap.size());
            let dst_rect = image.rect();

            dune2::bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
            image.save(output_dir.join(format!("{:02}.bmp", i)))?;
        }
    }

    return Ok(());
}

// ============================================================================
// Extract Tilemaps
// ============================================================================
#[derive(Args)]
pub struct CliExtractTilemapCommandArgs {
    /// Output folder path
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Faction to export
    #[arg(short = 'F', long, default_value = "harkonnen")]
    pub faction: CliExtractFaction,

    /// Scale factor
    #[arg(short = 's', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..=4))]
    pub scale: u32,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

fn extract_tilemaps(
    rc: &dune2::Resources,
    args: &CliExtractTilemapCommandArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = args.faction.into();
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tilemaps")?);

    fs::create_dir_all(&output_dir)?;

    for i in 0..rc.tilemaps.len() {
        let output_filepath = output_dir.join(format!("{:02}.bmp", i));

        let bitmap = rc.tilemap_bitmap(i, Some(faction))?;
        let src_rect = bitmap.rect();

        let mut image = BMPImage::new(args.scale*bitmap.size());
        let dst_rect = image.rect();

        dune2::bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
        image.save(output_filepath)?;
    }

    return Ok(());
}

// ============================================================================
// Extract Sprites
// ============================================================================
#[derive(Args)]
pub struct CliExtractSpriteCommandArgs {
    /// Output folder path
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Faction to export
    #[arg(short = 'F', long, default_value = "harkonnen")]
    pub faction: CliExtractFaction,

    /// Scale factor
    #[arg(short = 's', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..=4))]
    pub scale: u32,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

fn extract_sprites(
    rc: &dune2::Resources,
    args: &CliExtractSpriteCommandArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = args.faction.into();

    for sprite in rc.sprites.keys() {
        let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("sprites")?).join(sprite);

        fs::create_dir_all(&output_dir)?;

        for i in 0..rc.sprites.get(sprite).unwrap().frame_count() {
            let bitmap = rc.sprite_frame_bitmap(sprite, i, Some(faction))?;
            let src_rect = bitmap.rect();

            let mut image = BMPImage::new(args.scale*bitmap.size());
            let dst_rect = image.rect();

            dune2::bitmap::blit(&bitmap, &src_rect, &mut image, &dst_rect);
            image.save(output_dir.join(format!("{:02}.bmp", i)))?;
        }
    }

    Ok(())
}

// ============================================================================
// Extract run
// ============================================================================
#[derive(Subcommand)]
pub enum Commands {
    /// Extract palette
    Palette(CliExtractPaletteCommandArgs),
    /// Extract tiles
    Tiles(CliExtractTilesCommandArgs),
    /// Extract tilemaps
    Tilemaps(CliExtractTilemapCommandArgs),
    /// Extract sprites
    Sprites(CliExtractSpriteCommandArgs),
}

#[derive(Args)]
pub struct Cli {
    /// Input file path
    pub input_rc_file: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

pub fn run(args: &Cli) -> Result<(), Box<dyn Error>> {
    let mut reader = fs::File::open(&args.input_rc_file)?;
    let rc: dune2::Resources = dune2::Resources::read_from(&mut reader)?;

    match &args.command {
        Commands::Palette(args) => extract_palette(&rc, args),
        Commands::Tiles(args) => extract_tiles(&rc, args),
        Commands::Tilemaps(args) => extract_tilemaps(&rc, args),
        Commands::Sprites(args) => extract_sprites(&rc, args),
    }
}
