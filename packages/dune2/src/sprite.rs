use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SpriteFrameTransform {
    FlipX,
    FlipY,
    FlipXY,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteFrame {
    pub index: usize,
    pub transform: Option<SpriteFrameTransform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub frames: Vec<SpriteFrame>,
}
