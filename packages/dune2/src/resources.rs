use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resources {
    pub palette: Palette,
    pub tilesets: HashMap<String, Tileset>,
    pub tilemaps: Vec<Tilemap>,
    pub sprites: HashMap<String, Sprite>,
}

impl Resources {
    pub fn tile_bitmap(
        &self,
        tileset_id: &str,
        tile_index: usize,
        faction: Option<Faction>,
    ) -> TileBitmap {
        TileBitmap::new(self, tileset_id.into(), tile_index, faction)
    }

    pub fn tilemap_bitmap(
        &self,
        index: usize,
        faction: Option<Faction>,
    ) -> TilemapBitmap {
        TilemapBitmap::new(self, index, faction)
    }

    pub fn sprite_frame_bitmap(
        &self,
        sprite_id: &str,
        sprite_frame_index: usize,
        faction: Option<Faction>,
    ) -> SpriteFrameBitmap {
        SpriteFrameBitmap::new(self, sprite_id.into(), sprite_frame_index, faction)
    }
}
