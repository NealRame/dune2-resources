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

    #[wasm_bindgen(js_name = getColorCount)]
    pub fn get_color_count(
        &self,
    ) -> usize {
        self.resources.palette.len()
    }

    #[wasm_bindgen(js_name = getColor)]
    pub fn get_color(
        &self,
        color_index: usize,
    ) -> wasm_bindgen::Clamped<Vec<u8>> {
        self.resources.palette
            .color_at(color_index)
            .or(Some(BLACK))
            .map(|color| wasm_bindgen::Clamped(vec![
                color.red,
                color.green,
                color.blue,
                255
            ]))
            .unwrap()
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
        faction: Option<Dune2Faction>,
    ) -> core::result::Result<web_sys::ImageData, JsValue> {
        let palette = &self.resources.palette;
        let tileset = self.resources.get_tileset(tileset_id)?;

        let tile_count = tileset.tile_count() as u32;
        let tile_size = tileset.tile_size();
        let rows = if tile_count%columns == 0 {
            tile_count/columns
        } else {
            tile_count/columns + 1
        };

        let mut dst = RGBABitmap::new(
            Size {
                width: columns*tile_size.width,
                height: rows*tile_size.height,
            },
            Some(BLACK)
        );

        for (tile_index, tile) in tileset.tile_iter().enumerate() {
            let col = (tile_index as u32)%columns;
            let row = (tile_index as u32)/columns;

            let src = TileBitmap::with_palette(tile, faction, palette);
            let src_rect = src.rect();

            let dst_rect = Rect::from_point_and_size(
                Point {
                    x: (col*tile_size.width) as i32,
                    y: (row*tile_size.height) as i32,
                },
                tile_size,
            );

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
        faction: Option<Dune2Faction>,
        scale: Option<u32>,
    ) -> core::result::Result<web_sys::ImageData, JsValue> {
        let scale = u32::max(1, scale.unwrap_or(1));

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
