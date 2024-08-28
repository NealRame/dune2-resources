use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::Bitmap;
use crate::BitmapGetPixel;
use crate::Color;
use crate::Faction;
use crate::Point;
use crate::Resources;
use crate::Shape;
use crate::Size;
use crate::TileBitmap;


#[derive(Debug, Serialize, Deserialize)]
pub struct Tilemap {
    pub class: String,
    pub shape: Shape,
    pub tiles: Box<[usize]>,
    pub tileset: Box<str>,
}


pub struct TilemapBitmap<'a> {
    bitmaps: Vec<TileBitmap<'a>>,
    tilemap_shape: Shape,
    tile_size: Size,
}

impl<'a> TilemapBitmap<'a> {
    pub fn try_with_resources(
        tilemap: &Tilemap,
        faction: Option<Faction>,
        resources: &'a Resources,
    ) -> Result<Self> {
        let tileset = resources.get_tileset(&tilemap.tileset)?;
        let bitmaps = tilemap.tiles
            .iter()
            .map(|tile_index| -> Result<TileBitmap> {
                let tile = tileset.get_tile(*tile_index)?;
                Ok(TileBitmap::with_resources(tile, faction, resources))
            })
            .collect::<Result<Vec<TileBitmap>, _>>()?;

        Ok(Self {
            bitmaps,
            tilemap_shape: tilemap.shape,
            tile_size: tileset.tile_size(),
        })
    }
}

impl Bitmap for TilemapBitmap<'_> {
    fn width(&self) -> u32 {
        self.tilemap_shape.columns*self.bitmaps[0].width()
    }

    fn height(&self) -> u32 {
        self.tilemap_shape.rows*self.bitmaps[0].height()
    }
}

impl BitmapGetPixel for TilemapBitmap<'_> {
    fn get_pixel(&self, point: Point) -> Option<Color> {
        if point.x < 0 || point.y < 0 {
            return None;
        }

        let x = point.x.abs() as u32;
        let y = point.y.abs() as u32;

        let tilemap_col = x/self.tile_size.width;
        let tilemap_row = y/self.tile_size.height;
        let index = (tilemap_row*self.tilemap_shape.columns + tilemap_col) as usize;

        let tile_bitmap_x = (x%self.tile_size.width) as i32;
        let tile_bitmap_y = (y%self.tile_size.height) as i32;

        self.bitmaps[index].get_pixel(Point {
            x: tile_bitmap_x,
            y: tile_bitmap_y,
        })
    }
}
