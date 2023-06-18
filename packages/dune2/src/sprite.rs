use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SpriteTransform {
    FlipX,
    FlipY,
    FlipXY,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteFrame {
    pub index: usize,
    pub transform: Option<SpriteTransform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub frames: Vec<SpriteFrame>,
}
