use std::fs;

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::str;

use clap::Args;

use flate2::Compression;
use flate2::write::DeflateEncoder;

use rmp_serde::Serializer;

use serde::Deserialize;
use serde::Serialize;

use toml;

#[derive(Args)]
pub struct Cli {
    /// Input file path
    pub config_filepath: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,

    /// Output folder path
    #[arg(long, short, default_value = "dune2.rc")]
    pub output_file: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct PaletteConfig {
    pub source: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct TilesetConfig {
    pub source: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct TilemapsConfig {
    pub source: PathBuf,
    pub shapes: HashMap<String, dune2::Shape>,
}

#[derive(Debug, Deserialize)]
pub struct SpriteConfig {
    pub source: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub palette: PaletteConfig,
    pub tileset: TilesetConfig,
    pub tilemaps: TilemapsConfig,
    pub sprites: Vec<SpriteConfig>,
}

impl Config {
    fn try_read_from_file(
        config_filepath: &PathBuf,
    ) -> Result<Self, Box<dyn Error>> {
        let config_str = fs::read_to_string(config_filepath)?;
        let mut config = toml::from_str::<Config>(&config_str)?;
    
        let config_dir = config_filepath.parent().unwrap();
        
        // if palette path is relative, make it absolute by joining it with the
        // config file's directory
        if !config.palette.source.is_absolute() {
            config.palette.source = config_dir.join(&config.palette.source);
        }
    
        // if tileset path is relative, make it absolute by joining it with the
        // config file's directory
        if !config.tileset.source.is_absolute() {
            config.tileset.source = config_dir.join(&config.tileset.source);
        }

        // if tilemaps path is relative, make it absolute by joining it with
        // the config file's directory
        if !config.tilemaps.source.is_absolute() {
            config.tilemaps.source = config_dir.join(&config.tilemaps.source);
        }

        // if sprite paths are relative, make them absolute by joining them
        // with the config file's directory
        config.sprites.iter_mut().for_each(|sprite| {
            if !sprite.source.is_absolute() {
                sprite.source = config_dir.join(&sprite.source);
            }
        });
    
        Ok(config)
    }
}

pub fn run(args: &Cli) -> Result<(), Box<dyn Error>> {
    let config = Config::try_read_from_file(&args.config_filepath)?;

    let palette = dune2::Palette::from_pal_file(&config.palette.source)?;
    let tileset = dune2::Tileset::from_icn_file(&config.tileset.source)?;
    let tilemaps = dune2::Tilemap::from_map_file(
            &config.tilemaps.source,
            &config.tilemaps.shapes,
    )?;
    let mut sprites = Vec::new();

    for sprite in &config.sprites {
        let sprite_frames = dune2::SpriteFrame::from_shp_file(&sprite.source)?;
        sprites.extend(sprite_frames);
    }

    let rc = dune2::RC {
        palette,
        tileset,
        tilemaps,
        sprites,
    };

    if args.output_file.exists() && !args.force_overwrite {
        return Err(
            "Output file already exists. Use --force-overwrite to overwrite.".into());
    }

    let mut output = DeflateEncoder::new(
        fs::File::create(&args.output_file)?,
        Compression::best(),
    );

    rc.serialize(&mut Serializer::new(&mut output))?;
    Ok(())
}
