use anyhow::{anyhow, Result};

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::prelude::{
    Size,
    Tile,
};


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TilesetError {
    InvalidTileSize(String, Size),
    InvalidTileIndex(String, usize),
}

impl fmt::Display for TilesetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTileSize(tileset_id, size) => write!(
                f,
                "Tileset '{tileset_id}': invalid tile size '{size}'",
            ),
            Self::InvalidTileIndex(tileset_id, tile_index) => write!(
                f,
                "Tileset '{tileset_id}': invalid tile index '{tile_index}'"
            )
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tileset {
    id: String,
    tile_size: Size,
    tiles: Vec<Tile>,
}

impl Tileset {
    pub fn new(
        tileset_id: &str,
        tile_size: Size,
    ) -> Self {
        let tiles = Vec::new();
        Self {
            id: tileset_id.into(),
            tile_size,
            tiles,
        }
    }

    pub fn add(
        &mut self,
        tile: Tile,
    ) -> Result<()> {
        let tile_size = tile.size();

        if self.tile_size == tile_size {
            self.tiles.push(tile);
            Ok(())
        } else {
            Err(anyhow!(TilesetError::InvalidTileSize(
                self.id.clone(),
                tile_size,
            )))
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn tile_at(
        &self,
        tile_index: usize,
    ) -> Result<&Tile> {
        self.tiles
            .get(tile_index)
            .ok_or(anyhow!(TilesetError::InvalidTileIndex(
                self.id.clone(),
                tile_index,
            )))
    }

    pub fn tile_iter(&self) -> std::slice::Iter<'_, Tile> {
        self.tiles.iter()
    }

    pub fn tile_size(&self) -> Size {
        self.tile_size
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}
