use std::fs;

use std::error::Error;
use std::path::PathBuf;

use deflate::Compression;
use deflate::write::ZlibEncoder;

use rmp_serde::Serializer;

use serde::Serialize;
use serde::Deserialize;

use toml;

use dune2::Dune2RC;

use crate::cli;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub palette: PathBuf,
    pub tileset: PathBuf,
    pub sprites: Vec<PathBuf>,
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
        if !config.palette.is_absolute() {
            config.palette = config_dir.join(&config.palette);
        }
    
        // if tileset path is relative, make it absolute by joining it with the
        // config file's directory
        if !config.tileset.is_absolute() {
            config.tileset = config_dir.join(&config.tileset);
        }
    
        // if sprite paths are relative, make them absolute by joining them with
        // the config file's directory
        config.sprites =
            config.sprites.iter().map(|sprite| {
                if !sprite.is_absolute() {
                    config_dir.join(sprite)
                } else {
                    sprite.clone()
                }
            }).collect::<_>();
    
        Ok(config)
    }
}


pub fn run(args: &cli::CreateArgs) -> Result<(), Box<dyn Error>> {
    let config = Config::try_read_from_file(&args.config_filepath)?;

    let palette = dune2::Palette::from_pal_file(&config.palette)?;
    let tileset = dune2::Tileset::from_icn_file(&config.tileset)?;
    let mut sprites = Vec::new();

    for sprite in &config.sprites {
        let sprite_frames = dune2::SpriteFrame::from_shp_file(sprite)?;
        sprites.extend(sprite_frames);
    }

    let rc = Dune2RC {
        palette,
        tileset,
        sprites,
    };

    if args.output_file.exists() && !args.force_overwrite {
        return Err(
            "Output file already exists. Use --force-overwrite to overwrite.".into());
    }

    let mut output = ZlibEncoder::new(
        fs::File::create(&args.output_file)?,
        Compression::Best
    );

    rc.serialize(&mut Serializer::new(&mut output))?;
    Ok(())
}
