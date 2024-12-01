use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;

use dune2_assets::prelude::{
    Color,
    Assets,
};


/******************************************************************************
 * Info Palette
 *****************************************************************************/
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ColorStringFormat {
    /// array3f
    Array3f,
    /// array4f
    Array4f,
    /// css Hex format
    CssHex,
    /// css RGB format
    CssRGB,
}

#[derive(clap::Args)]
pub struct PaletteCommandArgs {
    #[arg(short, long, default_value = "css-hex", value_enum)]
    pub format: ColorStringFormat,
}

fn color_hex_string(color: &Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}

fn color_rgb_string(color: &Color) -> String {
    format!("rgb({}, {}, {})", color.red, color.green, color.blue)
}

fn color_array3f_string(color: &Color) -> String {
    format!("[{:.3}, {:.3}, {:.3}]",
        (color.red as f32)/255.,
        (color.green as f32)/255.,
        (color.blue as f32)/255.,
    )
}

fn color_array4f_string(color: &Color) -> String {
    format!("[{:.3}, {:.3}, {:.3}, 1]",
        (color.red as f32)/255.,
        (color.green as f32)/255.,
        (color.blue as f32)/255.,
    )
}

fn info_palette(
    rc: &Assets,
    args: &PaletteCommandArgs,
) -> Result<()> {
    let formatter = match args.format {
        ColorStringFormat::Array3f => color_array3f_string,
        ColorStringFormat::Array4f => color_array4f_string,

        ColorStringFormat::CssHex => color_hex_string,
        ColorStringFormat::CssRGB => color_rgb_string,
    };

    rc.palette.iter().for_each(|(index, color)| {
        println!("{:03}: {}", index, formatter(&color));
    });
    Ok(())
}


/******************************************************************************
 * Info Tiles
 *****************************************************************************/
fn info_tilesets(
    rc: &Assets,
) -> Result<()> {
    rc.tilesets.iter().for_each(|(name, tileset)| {
        println!("{}:", name);
        println!("  size: {}", tileset.tile_size(),);
        println!("  count: {}", tileset.tile_count());
    });
    Ok(())
}


/******************************************************************************
 * Info Tilemaps
 *****************************************************************************/
fn info_tilemaps(
    rc: &Assets,
) -> Result<()> {
    let mut classes = HashMap::<String, usize>::from_iter(
        rc.tilemaps.iter().map(|tilemap| {
            (String::from(tilemap.class.as_ref()), 0 as usize)
        })
    );

    rc.tilemaps.iter().enumerate().for_each(move |(index, tilemap)| {
        let class = String::from(tilemap.class.as_ref());
        let class_count = classes.get_mut(&class).unwrap();

        *class_count += 1;

        let tiles: String = tilemap.tiles.iter().map(|i| format!("{i}, ")).collect();

        println!("- [{}] {}#{}", index, class, class_count);
        println!("    shape: {}", tilemap.shape);
        println!("  tileset: {}", tilemap.tileset);
        println!("    tiles: [{}]", tiles);
    });

    Ok(())
}


/******************************************************************************
 * Info run
 *****************************************************************************/
#[derive(clap::Subcommand)]
pub enum Commands {
    Palette(PaletteCommandArgs),
    Tilesets,
    Tilemaps,
}

#[derive(clap::Args)]
pub struct Args {
    /// Input file path
    pub input_rc_file: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

pub fn run(args: &Args) -> Result<()> {
    let mut reader = File::open(&args.input_rc_file)?;
    let rc: Assets = Assets::read_from(&mut reader)?;

    match &args.command {
        Commands::Palette(args) => info_palette(&rc, &args),
        Commands::Tilesets => info_tilesets(&rc),
        Commands::Tilemaps => info_tilemaps(&rc),
    }
}
