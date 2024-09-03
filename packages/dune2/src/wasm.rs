use js_sys::JsString;
use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate web_sys;

use crate::prelude::*;
use crate::utils::point_to_index;


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
    ) -> core::result::Result<Dune2Resources, JsError> {
        let mut reader = std::io::Cursor::new(data);
        let resources = Resources::read_from(&mut reader)?;

        Ok(Self { resources })
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

    #[wasm_bindgen(js_name = getTilesetTileSize)]
    pub fn get_tileset_tile_size(
        &self,
        tileset_id: &str,
    ) -> core::result::Result<Size, JsValue> {
        let tile_size = self.resources
            .get_tileset(tileset_id)
            .map(|tileset| tileset.tile_size())?;

        Ok(tile_size)
    }

    #[wasm_bindgen(js_name = getTilesetTileCount)]
    pub fn get_tileset_tile_count(
        &self,
        tileset_id: &str,
    ) -> core::result::Result<usize, JsValue> {
        let tile_count = self.resources
            .get_tileset(tileset_id)
            .map(|tileset| tileset.tile_count())?;

        Ok(tile_count)
    }

    #[wasm_bindgen(js_name = getTilesetImageData)]
    pub fn get_tileset_image_data(
        &self,
        tileset_id: &str,
        columns: u32,
        faction: Option<String>,
    ) -> core::result::Result<web_sys::ImageData, JsValue> {
        let palette = &self.resources.palette;
        let faction =
            if let Some(str) = faction {
                Some(Faction::try_from_str(str.as_ref())?)
            } else {
                None
            };

        let tileset = self.resources.get_tileset(tileset_id)?;

        let tile_size = tileset.tile_size();
        let tile_count = tileset.tile_count() as u32;
        let rows = if tile_count%columns != 0 {
            tile_count/columns + 1
        } else {
            tile_count/columns
        };

        let texture_size = tile_size*Shape{ rows, columns };
        let mut dst = RGBABitmap::new(texture_size, Some(BLACK));

        for tile_index in 0..tile_count {
            let tile = tileset
                .tile_at(tile_index as usize)
                .map_err(|err| JsError::new(err.to_string().as_str()))?;

            let src = TileBitmap::with_palette(tile, faction, palette);
            let src_rect = src.rect();

            let dst_top_left = Point {
                x: (tile_size.width*tile_index%columns) as i32,
                y: (tile_size.height*tile_index/columns) as i32,
            };
            let dst_rect = Rect::from_point_and_size(dst_top_left, tile_size);

            bitmap_blit(&src, &src_rect, &mut dst, &dst_rect);
        }

        web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(dst.data.as_slice()),
            dst.width(),
            dst.height(),
        )
    }

    #[wasm_bindgen(js_name = getTilesetTileImageData)]
    pub fn get_tileset_tile_image_data(
        &self,
        tileset: &str,
        tile: usize,
        faction: JsValue,
        scale: JsValue,
    ) -> core::result::Result<web_sys::ImageData, JsValue> {
        let faction =
            if faction.is_truthy() {
                Some(Faction::try_from_js_value(&faction)?)
            } else {
                None
            };

        let scale = u32::max(1, scale.as_f64().unwrap_or(1.) as u32);

        let src_bitmap = self.resources.get_tile_bitmap(
            tileset,
            tile,
            faction
        )?;
        let src_rect = src_bitmap.rect();

        let mut dst_bitmap = RGBABitmap::new(
            src_bitmap.size()*scale,
            Some(BLACK)
        );
        let dst_rect = dst_bitmap.rect();

        bitmap_blit(
            &src_bitmap,
            &src_rect,
            &mut dst_bitmap,
            &dst_rect
        );

        web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(dst_bitmap.data.as_slice()),
            dst_bitmap.width(),
            dst_bitmap.height(),
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
    ) -> Option<Tilemap> {
        self.resources.tilemaps
            .get(tilemap_index)
            .and_then(|tilemap| Some(tilemap.clone()))
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
