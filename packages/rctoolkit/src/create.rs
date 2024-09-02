use std::fs;

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use dune2_rc::{Resources, Tile, Tilemap, Tileset};

use crate::resources_config::*;


/******************************************************************************
 * Create errors
 *****************************************************************************/

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateError {
    TilesetDuplicateId(String),
    TilesetInvalidId(String),
    TilesetInvalidTileIndex(String, usize),
}

impl fmt::Display for CreateError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::TilesetDuplicateId(id) => {
                write!(f, "Duplicate tileset '{id}'")
            },
            Self::TilesetInvalidId(id) => {
                write!(f, "Invalid tileset '{id}'")
            },
            Self::TilesetInvalidTileIndex(id, index) => {
                write!(f, "Invalid tile index '#{index}' in tileset '{id}'")
            },
        }
    }
}

/******************************************************************************
 * Tilesets creation
 *****************************************************************************/

fn load_tilesets(
    config: &Config,
    sources: &[Tile],
) -> Result<HashMap<String, Tileset>> {
    let mut tilesets = HashMap::<String, Tileset>::new();

    for tileset_config in config.tilesets.iter() {
        let tileset_id = tileset_config.id.clone();

        if tilesets.contains_key(&tileset_id) {
            return Err(anyhow!(CreateError::TilesetDuplicateId(tileset_id)));
        }

        let mut tileset = Tileset::new(&tileset_id, tileset_config.size);
        let tile_size = tileset_config.size;

        for tile_ref in tileset_config.tile_refs.iter() {
            let tile_index = tile_ref.index;
            let tile = &sources[tile_index];

            if tile_index >= sources.len() {
                return Err(anyhow!(CreateError::TilesetInvalidTileIndex(
                    tileset_id,
                    tile_index
                )));
            }

            tileset.add(tile
                .resize(tile_size, tile_ref.anchor)
                .transform(tile_ref.transform)
            )?;
        }

        tilesets.insert(tileset_id, tileset);
    }

    return Ok(tilesets)
}

/******************************************************************************
 * Check tilemaps
 *****************************************************************************/
fn check_tilemaps(
    tilemaps: &Vec<Tilemap>,
    tilesets: &HashMap<String, Tileset>,
) -> Result<()> {
    for tilemap in tilemaps.iter() {
        let tileset_id = String::from(tilemap.tileset.as_ref());
        let tileset = tilesets.get(&tileset_id).ok_or(
            anyhow!(CreateError::TilesetInvalidId(tileset_id.clone()))
        )?;

        for tile_index in tilemap.tiles.iter() {
            tileset.tile_at(*tile_index)?;
        }
    }
    Ok(())
}

/******************************************************************************
 * Create command run
 *****************************************************************************/

#[derive(clap::Args)]
pub struct Args {
    /// Input file path
    pub config_filepath: PathBuf,

    /// Overwrite existing files
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,

    /// Output folder path
    #[arg(long, short, default_value = "dune2.rc")]
    pub output_file: PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let config = Config::try_read_from_file(&args.config_filepath)?;

    let palette = config.load_palette()?;
    let sources = config.load_sources()?;

    let tilesets = load_tilesets(&config, sources.as_slice())?;
    let tilemaps = config.tilemaps;

    check_tilemaps(&tilemaps, &tilesets)?;

    let rc = Resources {
        palette,
        tilesets,
        tilemaps,
    };

    if args.output_file.exists() && !args.force_overwrite {
        return Err(anyhow!(
            "Output file already exists. Use --force-overwrite to overwrite."
        ));
    }

    let mut output = fs::File::create(&args.output_file)?;
    rc.write_to(&mut output)?;

    Ok(())
}
