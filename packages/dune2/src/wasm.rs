use js_sys::JsString;
use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate web_sys;

use crate::{
    point_to_index,
    Bitmap,
    BitmapPutPixel,
    Color,
    Faction,
    Point,
    Resources,
    Size,
    Tilemap,
};

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
    #[wasm_bindgen(js_name = load)]
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
        tileset_id: &str,
    ) -> Result<Size, JsError> {
        self.resources
            .get_tileset(tileset_id)
            .map(|tileset| tileset.tile_size())
            .map_err(|err| JsError::new(err.to_string().as_str()))
    }

    #[wasm_bindgen(js_name = getTileCount)]
    pub fn get_tile_count(
        &self,
        tileset_id: &str,
    ) -> Result<usize, JsError> {
        self.resources
            .get_tileset(tileset_id)
            .map(|tileset| tileset.tile_count())
            .map_err(|err| JsError::new(err.to_string().as_str()))
    }

    #[wasm_bindgen(js_name = getTile)]
    pub fn get_tile(
        &self,
        tileset: &str,
        tile: usize,
        faction: JsValue,
        scale: JsValue,
    ) -> Result<web_sys::ImageData, JsValue> {
        let faction = if let Some(v) = faction.as_string() {
            match Faction::try_from_str(v.as_str()) {
                Ok(faction) => Some(faction),
                Err(err) => return Err(JsValue::from_str(err.to_string().as_str()))
            }
        } else { None };

        let scale = u32::max(1, scale.as_f64().unwrap_or(1.) as u32);

        let src_bitmap =
            self.resources
                .get_tile_bitmap(tileset, tile, faction)
                .map_err(|err| JsValue::from_str(err.to_string().as_str()))?;
        let src_rect = src_bitmap.rect();

        let mut dst_bitmap = RGBABitmap::new(
            src_bitmap.size()*scale,
            Some(BLACK)
        );
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
        self.resources.tilemaps.len()
    }

    #[wasm_bindgen(js_name = getTilemap)]
    pub fn get_tilemap(
        &self,
        tilemap_index: usize,
    ) -> Result<Tilemap, JsError> {
        let tilemap = self.resources.tilemaps
            .get(tilemap_index)
            .ok_or_else(|| JsError::new("Invalid tilemap index value"))?;

        Ok(tilemap.clone())
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
