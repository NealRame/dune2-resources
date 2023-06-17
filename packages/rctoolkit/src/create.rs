use std::fs;

use std::error::Error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str;

use clap::Args;

use flate2::Compression;
use flate2::write::DeflateEncoder;

use rmp_serde::Serializer;

use serde::Deserialize;
use serde::Serialize;

use toml;

use dune2::{Tilemap, Tileset};

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
pub struct ConfigPalette {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SourceType {
    SHP,
    ICN,
}

#[derive(Debug, Deserialize)]
struct TilesetConfig {
    path: PathBuf,
    kind: SourceType,
}

// #[derive(Debug, Deserialize)]
// struct ConfigSprite {
//     frames: PathBuf,
// }

#[derive(Debug, Deserialize)]
struct Config {
    palette: ConfigPalette,
    tilesets: Vec<TilesetConfig>,
    tilemaps: Vec<Tilemap>,
}

impl Config {
    fn try_read_from_file(
        config_filepath: &PathBuf,
    ) -> Result<Self, Box<dyn Error>> {
        let config_dir = config_filepath.parent().unwrap();
        let config_str = fs::read_to_string(config_filepath)?;
        let mut config = toml::from_str::<Config>(&config_str)?;

        // if palette source path is relative, make it absolute by joining it
        // with the config file's directory
        if !config.palette.path.is_absolute() {
            config.palette.path = config_dir.join(&config.palette.path);
        }

        // if source paths are relative, make them absolute by joining them
        // with the config file's directory
        for tilset in config.tilesets.iter_mut() {
            if !tilset.path.is_absolute() {
                tilset.path = config_dir.join(&tilset.path);
            }
        }

        Ok(config)
    }
}

fn load_sources(
    config_tilesets: &Vec<TilesetConfig>,
) -> Result<HashMap<String, Tileset>, Box<dyn Error>> {
    let mut sources = HashMap::<String, Tileset>::new();

    for source_config in config_tilesets.iter() {
        let mut tilesets = match source_config.kind {
            SourceType::SHP => {
                dune2::Tileset::from_shp_file(&source_config.path)?
            },
            SourceType::ICN => {
                vec![dune2::Tileset::from_icn_file(&source_config.path)?]
            },
        };

        while let Some(mut tileset) = tilesets.pop() {
            let name = format!("tiles_{}", tileset.tile_size);

            if let Some(existing_tileset) = sources.get_mut(&name) {
                existing_tileset.append(&mut tileset)?;
            } else {
                sources.insert(name.clone(), tileset);
            }
        }
    }

    Ok(sources)
}

pub fn run(args: &Cli) -> Result<(), Box<dyn Error>> {
    let config = Config::try_read_from_file(&args.config_filepath)?;

    let palette = dune2::Palette::from_pal_file(&config.palette.path)?;
    let tilesets = load_sources(&config.tilesets)?;

    let rc = dune2::RC {
        palette,
        tilesets,
        tilemaps: config.tilemaps,
    };

    if args.output_file.exists() && !args.force_overwrite {
        return Err(
            "Output file already exists. Use --force-overwrite to overwrite.".into()
        );
    }

    let mut output = DeflateEncoder::new(
        fs::File::create(&args.output_file)?,
        Compression::best(),
    );

    rc.serialize(&mut Serializer::new(&mut output))?;

    Ok(())
}
