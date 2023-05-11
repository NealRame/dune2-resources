use serde::{Deserialize, Serialize};

use crate::{Palette, SpriteFrame, Tileset };

#[derive(Debug, Serialize, Deserialize)]
pub struct Dune2RC {
    pub palette: Palette,
    pub tileset: Tileset,
    pub sprites: Vec<SpriteFrame>,
}
