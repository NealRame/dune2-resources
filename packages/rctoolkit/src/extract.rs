use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Subcommand};

use flate2::read::DeflateDecoder;

use rmp_serde;

#[derive(Args)]
pub struct Dune2PaletteArgs {
    /// Output folder path
    #[arg(short, long, default_value = "palette.bmp")]
    pub output_filepath: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

#[derive(Args)]
pub struct Dune2GFXAssetsArgs {
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
    Palette(Dune2PaletteArgs),
    Sprites(Dune2GFXAssetsArgs),
    Tileset(Dune2GFXAssetsArgs),
    Tilemaps(Dune2GFXAssetsArgs),
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
    args: &Dune2PaletteArgs,
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
                x: (i as u32)%16,
                y: (i as u32)/16,
            },
            palette_watch_size,
        );

        dune2::bitmap::fill_rect(&mut palette_surface, &rect, color);
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
    scale: u32,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let surface = dune2::Surface::from_bitmap_scaled(&frame.bitmap(palette, faction), scale);
    let mut output = fs::File::create(output_filepath)?;
    dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;

    return Ok(());
}

fn extract_sprites(
    rc: &dune2::RC,
    args: &Dune2GFXAssetsArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tileset")?);

    fs::create_dir_all(&output_dir)?;

    for (i, frame) in rc.sprites.iter().enumerate() {
        let output_filepath = output_dir.join(format!("{:02}.bmp", i));
        export_frame_to_bmp(
            frame,
            &rc.palette,
            faction,
            args.scale,
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
    scale: u32,
    output_filepath: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut surface = dune2::Surface::new(
        scale*tilemap.shape*tileset.tile_size
    );

    tilemap.tiles
        .iter()
        .map(|&index| tileset.bitmap(index, palette, faction))
        .enumerate()
        .for_each(|(i, tile)| {
            let row = (i as u32)/tilemap.shape.columns;
            let col = (i as u32)%tilemap.shape.columns;

            let src_rect = dune2::Bitmap::rect(&tile);
            let dst_rect = dune2::Rect::from_point_and_size(
                scale*dune2::Point {
                    x: col*tileset.tile_size.width,
                    y: row*tileset.tile_size.height,
                },
                scale*dune2::Bitmap::size(&tile)
            );

            dune2::bitmap::blit(
                &tile,
                &src_rect,
                &mut surface,
                &dst_rect
            );
        });

    let mut output = fs::File::create(output_filepath)?;
    dune2::write_bmp_with_palette(&surface, &palette, &mut output)?;

    return Ok(());
}

fn extract_tileset(
    rc: &dune2::RC,
    args: &Dune2GFXAssetsArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tileset")?);

    fs::create_dir_all(&output_dir)?;

    let tilemaps = Vec::from_iter((0..rc.tileset.tiles.len()).map(|i| {
        dune2::Tilemap {
            shape: dune2::Shape { rows: 1, columns: 1, },
            tiles: vec![i],
        }
    }));

    for (i, tilemap) in tilemaps.iter().enumerate() {
        let output_filepath = output_dir.join(format!("{:02}.bmp", i));
        export_tilemap_to_bmp(
            &tilemap,
            &rc.tileset,
            &rc.palette,
            faction,
            args.scale,
            &output_filepath
        )?;
    }

    return Ok(());
}

fn extract_tilemaps(
    rc: &dune2::RC,
    args: &Dune2GFXAssetsArgs,
) -> Result<(), Box<dyn Error>> {
    let faction = dune2::Faction::from_str(&args.faction)?;
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from_str("tileset")?);

    fs::create_dir_all(&output_dir)?;

    for (i, tilemap) in rc.tilemaps.iter().enumerate() {
        let output_filepath = output_dir.join(format!("{:02}.bmp", i));
        export_tilemap_to_bmp(
            &tilemap,
            &rc.tileset,
            &rc.palette,
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
    let rc: dune2::RC = rmp_serde::from_read(&mut inflate_reader)?;

    match &args.command {
        Commands::Palette(args) => extract_palette(&rc, args),
        Commands::Sprites(args) => extract_sprites(&rc, args),
        Commands::Tileset(args) => extract_tileset(&rc, args),
        Commands::Tilemaps(args) => extract_tilemaps(&rc, args),
    }
}
