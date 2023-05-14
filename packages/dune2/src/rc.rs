use serde::{Deserialize, Serialize};

use crate::{Palette, SpriteFrame, Tileset, Tilemap };

#[derive(Debug, Serialize, Deserialize)]
pub struct RC {
    pub palette: Palette,
    pub sprites: Vec<SpriteFrame>,
    pub tileset: Tileset,
    pub tilemaps: Vec<Tilemap>,
}
