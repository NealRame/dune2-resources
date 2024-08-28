use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;

use crate::resources_config::*;

pub fn check_tileset(
    config: &Config,
    tileset_config: &TilesetConfig,
) {
    let tileset_id = &tileset_config.id;
    let mut tile_indexes = HashSet::<usize>::from_iter(0..tileset_config.tile_refs.len());

    print!("tilset '{tileset_id}' unused tiles:");

    config.tilemaps.iter()
        .filter(|tilemap| tilemap.tileset.as_ref() == tileset_id)
        .for_each(|tilemap| {
            for tile_index in tilemap.tiles.iter() {
                tile_indexes.remove(&tile_index);
            }
        });

    tile_indexes.iter().for_each(|tile_index| print!(" {tile_index}"));

    print!("\n");
}

/******************************************************************************
 * Create command run
 *****************************************************************************/

#[derive(clap::Args)]
pub struct Args {
    /// Input file path
    pub config_filepath: PathBuf,

    /// Tileset id
    pub tileset_id: Option<String>,
}

pub fn run(args: &Args) -> Result<()> {
    let config = Config::try_read_from_file(&args.config_filepath)?;

    for tileset_config in config.tilesets.iter() {
        if let Some(tileset_id) = &args.tileset_id {
            if tileset_id == &tileset_config.id {
                check_tileset(&config, tileset_config);
            }
        } else {
            check_tileset(&config, tileset_config);
        }
    }

    Ok(())
}
