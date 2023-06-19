use serde::{Deserialize, Serialize};

use crate::bitmap::{Bitmap, BitmapGetPixel};
use crate::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SpriteFrameTransform {
    FlipX,
    FlipY,
    FlipXY,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteFrame {
    pub tilemap: usize,
    pub transform: Option<SpriteFrameTransform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub frames: Vec<SpriteFrame>,
}

pub struct SpriteFrameBitmap<'a> {
    bitmap: Box<TilemapBitmap<'a>>,
    transformation: Option<SpriteFrameTransform>,
}

impl<'a> SpriteFrameBitmap<'a> {
    pub fn new(
        resources: &'a Resources,
        sprite_id: &String,
        sprite_frame_index: usize,
        faction: Option<Faction>,
    ) -> Self {
        let sprite_frame = &resources.sprites.get(sprite_id).unwrap().frames[sprite_frame_index];
        let bitmap = Box::new(resources.tilemap_bitmap(sprite_frame.tilemap, faction));
        return Self {
            bitmap,
            transformation: sprite_frame.transform,
        };
    }
}

impl Bitmap for SpriteFrameBitmap<'_> {
    fn width(&self) -> u32 {
        self.bitmap.width()
    }

    fn height(&self) -> u32 {
        self.bitmap.height()
    }
}

impl BitmapGetPixel for SpriteFrameBitmap<'_> {
    fn get_pixel(&self, p: Point) -> Option<Color> {
        match self.transformation {
            None => self.bitmap.get_pixel(p),
            Some(SpriteFrameTransform::FlipX) => {
                self.bitmap.get_pixel(Point {
                    x: self.width() - p.x - 1,
                    y: p.y,
                })
            },
            Some(SpriteFrameTransform::FlipY) => {
                self.bitmap.get_pixel(Point {
                    x: p.x,
                    y: self.height() - p.y - 1,
                })
            },
            Some(SpriteFrameTransform::FlipXY) => {
                self.bitmap.get_pixel(Point {
                    x: self.width() - p.x - 1,
                    y: self.height() - p.y - 1,
                })
            },
        }
    }
}