use std::str::FromStr;

use js_sys::JsString;
use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate web_sys;

use crate::{
    Bitmap,
    BitmapPutPixel,
    Color,
    Faction,
    Point,
    Resources,
    Size,
    point_to_index,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub const BLACK: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
};

#[wasm_bindgen]
pub struct Dune2Resources {
    resources: Resources,
}

#[wasm_bindgen]
impl Dune2Resources {
    pub fn load(
        data: &[u8],
    ) -> Result<Dune2Resources, JsError> {
        let mut reader = std::io::Cursor::new(data);
        Resources::read_from(&mut reader)
            .map(|resources| Self { resources })
            .map_err(|_| JsError::new("Failed to load resources"))
    }

    #[wasm_bindgen(js_name = getTilesets)]
    pub fn get_tilesets(
        &self,
    ) -> Vec<JsString> {
        self.resources
            .tilesets
            .keys()
            .map(|tileset| JsString::from(tileset.as_str()))
            .collect()
    }

    #[wasm_bindgen(js_name = getTileSize)]
    pub fn get_tile_size(
        &self,
        tileset: &str,
    ) -> Result<Size, JsValue> {
        self.resources
            .tilesets
            .get(tileset)
            .map(|tileset| tileset.tile_size)
            .ok_or_else(|| JsValue::from_str("Invalid tileset"))
    }

    #[wasm_bindgen(js_name = getTileCount)]
    pub fn get_tile_count(
        &self,
        tileset: &str,
    ) -> Result<usize, JsValue> {
        self.resources
            .tilesets
            .get(tileset)
            .map(|tileset| tileset.tiles.len())
            .ok_or_else(|| JsValue::from_str("Invalid tileset"))
    }

    #[wasm_bindgen(js_name = getTile)]
    pub fn get_tile(
        &self,
        tileset: &str,
        tile: usize,
        faction: &str,
        scale: u32,
    ) -> Result<web_sys::ImageData, JsValue> {
        let faction =
            Faction::from_str(faction)
                .map_err(|_| JsValue::from_str("Invalid faction"))?;

        let src_bitmap =
            self.resources
                .tile_bitmap(tileset, tile, Some(faction))
                .map_err(|_| JsValue::from_str("Failed to get tile bitmap"))?;
        let src_rect = src_bitmap.rect();

        let mut dst_bitmap = RGBABitmap::new(src_bitmap.size()*scale, Some(BLACK));
        let dst_rect = dst_bitmap.rect();

        crate::bitmap::blit(
            &src_bitmap,
            &src_rect,
            &mut dst_bitmap,
            &dst_rect
        );

        web_sys::ImageData::new_with_u8_clamped_array(
            wasm_bindgen::Clamped(dst_bitmap.data.as_slice()),
            dst_bitmap.width(),
        )
    }

    #[wasm_bindgen(js_name = getTilemapCount)]
    pub fn get_tilemap_count(
        &self,
    ) -> usize {
        self.resources
            .tilemaps
            .len()
    }

    #[wasm_bindgen(js_name = getTilemapSize)]
    pub fn get_tilemap_size(
        &self,
        tilemap: usize,
    ) -> Result<Size, JsValue> {
        let tilemap = self.resources
            .tilemaps
            .get(tilemap)
            .ok_or_else(|| JsValue::from_str("Invalid tilemap"))?;
        let tile_size = self.get_tile_size(&tilemap.tileset)?;
        Ok(tile_size*tilemap.shape)
    }

    #[wasm_bindgen(js_name = getTilemap)]
    pub fn get_tilemap(
        &self,
        tilemap: usize,
        faction: &str,
        scale: u32,
    ) -> Result<web_sys::ImageData, JsValue> {
        let faction =
            Faction::from_str(faction)
                .map_err(|_| JsValue::from_str("Invalid faction"))?;

        let src_bitmap =
            self.resources
                .tilemap_bitmap(tilemap, Some(faction))
                .map_err(|_| JsValue::from_str("Failed to get tile bitmap"))?;
        let src_rect = src_bitmap.rect();

        let mut dst_bitmap = RGBABitmap::new(src_bitmap.size()*scale, Some(BLACK));
        let dst_rect = dst_bitmap.rect();

        crate::bitmap::blit(
            &src_bitmap,
            &src_rect,
            &mut dst_bitmap,
            &dst_rect
        );

        web_sys::ImageData::new_with_u8_clamped_array(
            wasm_bindgen::Clamped(dst_bitmap.data.as_slice()),
            dst_bitmap.width(),
        )
    }

    #[wasm_bindgen(js_name = getSprites)]
    pub fn get_sprites(
        &self,
    ) -> Vec<JsString> {
        self.resources
            .sprites
            .keys()
            .map(|sprite| JsString::from(sprite.as_str()))
            .collect()
    }

    #[wasm_bindgen(js_name = getSpriteFrameCount)]
    pub fn get_sprite_frame_count(
        &self,
        sprite: &str,
    ) -> Result<usize, JsValue> {
        self.resources.sprites
            .get(sprite)
            .map(|sprite| sprite.frame_count())
            .ok_or_else(|| JsValue::from_str("Invalid sprite"))
    }

    #[wasm_bindgen(js_name = getSpriteFrame)]
    pub fn get_sprite_frame(
        &self,
        sprite: &str,
        frame: usize,
        faction: &str,
        scale: u32,
    ) -> Result<web_sys::ImageData, JsValue> {
        let faction =
            Faction::from_str(faction)
                .map_err(|_| JsValue::from_str("Invalid faction"))?;

        let src_bitmap =
            self.resources
                .sprite_frame_bitmap(sprite, frame, Some(faction))
                .map_err(|_| JsValue::from_str("Failed to get sprite bitmap"))?;
        let src_rect = src_bitmap.rect();

        let mut dst_bitmap = RGBABitmap::new(src_bitmap.size()*scale, Some(BLACK));
        let dst_rect = dst_bitmap.rect();

        crate::bitmap::blit(
            &src_bitmap,
            &src_rect,
            &mut dst_bitmap,
            &dst_rect
        );

        web_sys::ImageData::new_with_u8_clamped_array(
            wasm_bindgen::Clamped(dst_bitmap.data.as_slice()),
            dst_bitmap.width(),
        )
    }
}

struct RGBABitmap {
    color_key: Option<Color>,
    data: Vec<u8>,
    size: Size,
}

impl RGBABitmap {
    fn new(
        size: Size,
        color_key: Option<Color>,
    ) -> Self {
        Self {
            color_key,
            data: vec![0; 4*(size.width*size.height) as usize],
            size,
        }
    }
}

impl Bitmap for RGBABitmap {
    fn width(&self) -> u32 {
        self.size.width
    }

    fn height(&self) -> u32 {
        self.size.height
    }
}

impl BitmapPutPixel for RGBABitmap {
    fn put_pixel(
        &mut self,
        p: Point,
        color: Color,
    ) -> &mut Self {
        let size = self.size();
        if let Some(offset) = point_to_index(p, size).map(|offset| 4*offset) {
            let color = match self.color_key {
                Some(color_key) if color == color_key => (0, 0, 0, 0),
                _ => (color.red, color.green, color.blue, 255),
            };
            self.data[offset + 0] = color.0;
            self.data[offset + 1] = color.1;
            self.data[offset + 2] = color.2;
            self.data[offset + 3] = color.3;
        }
        self
    }
}
