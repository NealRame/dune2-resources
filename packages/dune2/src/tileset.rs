use std::fmt;

use serde::{Deserialize, Serialize};

use crate::*;

pub struct TileBitmap<'a, 'b> {
    tile_index: usize,
    tileset: &'a Tileset,
    palette: &'b Palette,
    faction: Option<Faction>,
}

impl<'a, 'b> TileBitmap<'a, 'b> {
    pub fn new(
        tile_index: usize,
        tileset: &'a Tileset,
        palette: &'b Palette,
        faction: Option<Faction>,
    ) -> Self {

        Self {
            tile_index,
            palette,
            tileset,
            faction,
        }
    }
}

impl Bitmap for TileBitmap<'_, '_> {
    fn width(&self) -> u32 {
        self.tileset.tile_size.width
    }

    fn height(&self) -> u32 {
        self.tileset.tile_size.height
    }
}

impl BitmapGetPixel for TileBitmap<'_, '_> {
    fn get_pixel(&self, point: Point) -> Option<Color> {
        point_to_index(point, self.size()).map(|index| {
            let mut color_index =
                self.tileset.tiles[self.tile_index][index] as usize;

            if let Some(faction) = self.faction {
                let faction_palette_offset = 16*(faction as usize);

                if color_index >= COLOR_HARKONNEN
                    && color_index < COLOR_HARKONNEN + 7 {
                    color_index = color_index + faction_palette_offset
                }
            }

            self.palette.color_at(color_index)
        })
    }
}

#[derive(Debug, Clone)]
pub struct TilesetSizeError {
    pub expected: Size,
    pub got: Size,
}

impl fmt::Display for TilesetSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expected tile size {:?}, got {:?}", self.expected, self.got)
    }
}

impl std::error::Error for TilesetSizeError {}


#[derive(Debug, Serialize, Deserialize)]
pub struct Tileset {
    pub tile_size: Size,
    pub tiles: Vec<Vec<u8>>,
}

impl Tileset {
    pub fn new(
        tile_size: Size,
    ) -> Self {
        let tiles = Vec::new();
        Self {
            tile_size,
            tiles,
        }
    }

    pub fn append(
        &mut self,
        tilset: &mut Tileset,
    ) -> Result<(), TilesetSizeError> {
        if self.tile_size != tilset.tile_size {
            return Err(TilesetSizeError {
                expected: self.tile_size,
                got: tilset.tile_size,
            });
        }
        self.tiles.append(tilset.tiles.as_mut());
        Ok(())
    }
}

impl Tileset {
    pub fn bitmap<'a, 'b>(
        &'a self,
        index: usize,
        palette: &'b Palette,
        faction: Option<Faction>,
    ) -> TileBitmap<'a, 'b> {
        TileBitmap::new(index, self, palette, faction)
    }
}
