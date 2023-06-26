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
    bitmaps: Vec<TileBitmap<'a>>,
    tilemap_shape: Shape,
    tile_size: Size,
}

impl<'a> TilemapBitmap<'a> {
    pub fn create(
        resources: &'a Resources,
        index: usize,
        faction: Option<Faction>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let tilemap = resources.tilemaps.get(index).unwrap();
        let tileset = resources.tilesets.get(&tilemap.tileset).unwrap();
        let tileset_id = &tilemap.tileset;

        let bitmaps = tilemap.tiles.iter().map(|tile_index| {
            TileBitmap::create(
                resources,
                tileset_id.into(),
                *tile_index,
                faction,
            )
        }).collect::<Result<Vec<TileBitmap>, Box<dyn std::error::Error>>>()?;

        Ok(Self {
            tilemap_shape: tilemap.shape,
            tile_size: tileset.tile_size,
            bitmaps,
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
        let col = (point.x/self.tile_size.width) as usize;
        let row = (point.y/self.tile_size.height) as usize;
        let index = row*self.tilemap_shape.columns as usize + col;

        let bitmap_x = point.x%self.tile_size.width;
        let bitmap_y = point.y%self.tile_size.height;

        self.bitmaps[index].get_pixel(Point {
            x: bitmap_x,
            y: bitmap_y,
        })
    }
}
