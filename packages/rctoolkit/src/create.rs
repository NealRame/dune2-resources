use std::collections::HashSet;
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
    pub includes: Option<HashSet<usize>>,
}

#[derive(Debug, Deserialize)]
pub struct SpriteConfig {
    pub source: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub palette: PaletteConfig,
    pub tilesets: HashMap<String, TilesetConfig>,
}

impl Config {
    fn try_read_from_file(
        config_filepath: &PathBuf,
    ) -> Result<Self, Box<dyn Error>> {
        let config_str = fs::read_to_string(config_filepath)?;
        let mut config = toml::from_str::<Config>(&config_str)?;
    
        let config_dir = config_filepath.parent().unwrap();
        
        // if palette source path is relative, make it absolute by joining it
        // with the config file's directory
        if !config.palette.source.is_absolute() {
            config.palette.source = config_dir.join(&config.palette.source);
        }
    
        // if tilesets source paths are relative, make themabsolute by joining
        // them with the config file's directory
        for (_, tileset) in config.tilesets.iter_mut() {
            if !tileset.source.is_absolute() {
                tileset.source = config_dir.join(&tileset.source);
            }
        }

        Ok(config)
    }
}

pub fn create_tilesets(
    tilesets_config: &HashMap<String, TilesetConfig>,
) -> Result<HashMap<String, dune2::Tileset>, Box<dyn Error>> {
    let mut sources = HashMap::new();
    for (_, tileset_config) in tilesets_config.iter() {
        let source = String::from(tileset_config.source.to_str().unwrap());
        if !sources.contains_key(&source) {
            let tileset = dune2::Tileset::from_icn_file(&tileset_config.source)?;
            sources.insert(source, tileset);
        }
    }

    let mut tilesets = HashMap::new();
    for (name, tileset_config) in tilesets_config.iter() {
        let source = String::from(tileset_config.source.to_str().unwrap());
        let tileset_source = sources.get(&source).unwrap();
        let mut tileset_dest = dune2::Tileset::new(tileset_source.tile_size);

        if let Some(includes) = &tileset_config.includes {
            for index in includes.iter() {
                tileset_dest.tiles.push(tileset_source.tiles[*index].clone());
            }
        } else {
            tileset_dest.tiles = tileset_source.tiles.clone();
        }

        tilesets.insert(name.clone(), tileset_dest);
    }

    Ok(tilesets)
}

pub fn run(args: &Cli) -> Result<(), Box<dyn Error>> {
    let config = Config::try_read_from_file(&args.config_filepath)?;

    println!("{:#?}", config);

    let palette = dune2::Palette::from_pal_file(&config.palette.source)?;
    let tilesets = create_tilesets(&config.tilesets)?;

    let rc = dune2::RC {
        palette,
        tilesets,
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
