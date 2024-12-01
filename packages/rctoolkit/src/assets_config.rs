use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use serde::Deserialize;

use dune2_assets::prelude::{
    Palette,
    Size,
    Tile,
    TileAnchor,
    Tilemap,
    TileTransformation,
};

use crate::{
    icn, pal, shp
};


#[derive(Debug, Deserialize)]
pub struct PaletteConfig {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub enum SourceType {
    SHP,
    ICN,
}

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub path: PathBuf,
    pub kind: SourceType,
}

#[derive(Debug, Deserialize)]
pub struct TileRef {
    pub index: usize,
    pub transform: Option<TileTransformation>,
    pub anchor: Option<TileAnchor>,
}

#[derive(Debug, Deserialize)]
pub struct TilesetConfig {
    pub id: String,
    pub size: Size,
    pub tile_refs: Vec<TileRef>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub palette: PaletteConfig,
    pub sources: Vec<SourceConfig>,
    pub tilesets: Vec<TilesetConfig>,
    pub tilemaps: Vec<Tilemap>,
}

impl Config {
    pub fn try_read_from_file(
        config_filepath: &PathBuf,
    ) -> Result<Self> {
        let config_str = fs::read_to_string(config_filepath)?;
        let mut config = toml::from_str::<Config>(&config_str)?;

        let data_dir =
            std::env::var("DUNE2_DATA_DIR")
                .map(|s| PathBuf::from(s))
                .unwrap_or_else(|_| {
                    config_filepath.parent().unwrap().to_path_buf()
                });

        // if palette source path is relative, make it absolute by joining it
        // with the config file's directory
        if !config.palette.path.is_absolute() {
            config.palette.path = data_dir.join(&config.palette.path);
        }

        // if source paths are relative, make them absolute by joining them
        // with the config file's directory
        for source in config.sources.iter_mut() {
            if !source.path.is_absolute() {
                source.path = data_dir.join(&source.path);
            }
        }

        Ok(config)
    }

    pub fn load_palette(
        &self,
    ) -> Result<Palette> {
        pal::read_palette_from_file(&self.palette.path)
    }

    pub fn load_sources(
        &self,
    ) -> Result<Vec<Tile>> {
        let mut tiles = Vec::new();

        for source in self.sources.iter() {
            tiles.append(&mut match source.kind {
                SourceType::ICN => icn::read_tiles_from_file(&source.path)?,
                SourceType::SHP => shp::read_tiles_from_file(&source.path)?,
            });
        }
        Ok(tiles)
    }
}
