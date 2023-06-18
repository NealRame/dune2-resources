use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Palette, Sprite, Tilemap, Tileset};

#[derive(Debug, Serialize, Deserialize)]
pub struct RC {
    pub palette: Palette,
    pub tilesets: HashMap<String, Tileset>,
    pub tilemaps: Vec<Tilemap>,
    pub sprites: HashMap<String, Sprite>,
}
