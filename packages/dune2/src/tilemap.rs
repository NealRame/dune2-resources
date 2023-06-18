use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tilemap {
    pub remapable: bool,
    pub shape: Shape,
    pub tiles: Vec<usize>,
    pub tileset: String,
}

pub struct TilemapBitmap<'a> {
    resources: &'a Resources,
    index: usize,
    faction: Option<Faction>,
}

impl<'a> TilemapBitmap<'a> {
    pub fn new(
        resources: &'a Resources,
        index: usize,
        faction: Option<Faction>,
    ) -> Self {
        Self {
            resources,
            index,
            faction,
        }
    }
}

impl Bitmap for TilemapBitmap<'_> {
    fn width(&self) -> u32 {
        let tilemap = &self.resources.tilemaps[self.index];
        let tileset = &self.resources.tilesets[&tilemap.tileset];
        tilemap.shape.columns*tileset.tile_size.width
    }

    fn height(&self) -> u32 {
        let tilemap = &self.resources.tilemaps[self.index];
        let tileset = &self.resources.tilesets[&tilemap.tileset];
        tilemap.shape.rows*tileset.tile_size.height
    }
}

impl BitmapGetPixel for TilemapBitmap<'_> {
    fn get_pixel(&self, point: Point) -> Option<Color> {
        let tilemap = &self.resources.tilemaps[self.index];
        let tileset = &self.resources.tilesets[&tilemap.tileset];

        let col = (point.x/tileset.tile_size.width) as usize;
        let row = (point.y/tileset.tile_size.height) as usize;
        let index = row*tilemap.shape.columns as usize + col;

        let bitmap = self.resources.tile_bitmap(
            &tilemap.tileset,
            tilemap.tiles[index],
            if tilemap.remapable { self.faction } else { None },
        );

        let bitmap_x = point.x%tileset.tile_size.width;
        let bitmap_y = point.y%tileset.tile_size.height;

        bitmap.get_pixel(Point {
            x: bitmap_x,
            y: bitmap_y,
        })
    }
}
