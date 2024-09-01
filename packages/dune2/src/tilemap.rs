use anyhow::Result;

use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::{
    Bitmap,
    BitmapGetPixel,
    Color,
    Faction,
    Point,
    Resources,
    Shape,
    Size,
    TileBitmap,
};

// Q: Why there is two declartion for struct Tilemap ?
// A: I cannot use [cfg_attr(feature = "wasm", wasm_bindgen(skip))] because of
// the following compilation error:
//  - error: expected non-macro attribute, found attribute macro `wasm_bindgen`
//
// I found this solution [https://github.com/rustwasm/wasm-bindgen/issues/2703]
// but it uses nightly Rust channel and I don't want to use it.
//
// So I decided to duplicate the struct declaration.

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg(not(feature = "wasm"))]
pub struct Tilemap {
    pub class: Box<str>,
    pub shape: Shape,
    pub tiles: Box<[usize]>,
    pub tileset: Box<str>,
}

#[cfg(feature = "wasm")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Tilemap {
    #[wasm_bindgen(skip)]
    pub class: Box<str>,
    #[wasm_bindgen(skip)]
    pub shape: Shape,
    #[wasm_bindgen(skip)]
    pub tiles: Box<[usize]>,
    #[wasm_bindgen(skip)]
    pub tileset: Box<str>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Tilemap {
    #[wasm_bindgen(js_name = getClass)]
    pub fn get_class(&self) -> String {
        String::from(self.class.as_ref())
    }

    #[wasm_bindgen(js_name = getShape)]
    pub fn get_shape(&self) -> Shape {
        self.shape
    }

    #[wasm_bindgen(js_name = getTiles)]
    pub fn get_tiles(&self) -> Vec<usize> {
        Vec::from(self.tiles.as_ref())
    }

    #[wasm_bindgen(js_name = getTileset)]
    pub fn get_tileset(&self) -> String {
        String::from(self.tileset.as_ref())
    }
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
                let tile = tileset.tile_at(*tile_index)?;
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
