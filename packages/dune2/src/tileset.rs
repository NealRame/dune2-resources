use serde::{Deserialize, Serialize};

use crate::prelude::{
    Error,
    Result,
    Size,
    Tile,
};


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
            Err(Error::TilesetInvalidTileSize(
                self.id.clone(),
                tile_size,
            ))
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
            .ok_or(Error::TilesetInvalidTileIndex(
                self.id.clone(),
                tile_index,
            ))
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
