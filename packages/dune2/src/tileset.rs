use std::fmt;

use serde::{Deserialize, Serialize};

use crate::*;

pub struct TileBitmap<'a> {
    resource: &'a Resources,
    tileset: String,
    tile_index: usize,
    faction: Option<Faction>,
}

impl<'a> TileBitmap<'a> {
    pub fn new(
        resource: &'a Resources,
        tileset: String,
        tile_index: usize,
        faction: Option<Faction>,
    ) -> Self {
        Self {
            resource,
            tileset,
            tile_index,
            faction,
        }
    }
}

impl Bitmap for TileBitmap<'_> {
    fn width(&self) -> u32 {
        let tileset = self.resource.tilesets.get(&self.tileset).unwrap();
        tileset.tile_size.width
    }

    fn height(&self) -> u32 {
        let tileset = self.resource.tilesets.get(&self.tileset).unwrap();
        tileset.tile_size.height
    }
}

impl BitmapGetPixel for TileBitmap<'_> {
    fn get_pixel(&self, point: Point) -> Option<Color> {
        let palette = &self.resource.palette;
        let tileset = self.resource.tilesets.get(&self.tileset).unwrap();
        point_to_index(point, self.size()).map(|index| {
            let mut color_index =
                tileset.tiles[self.tile_index][index] as usize;

            if let Some(faction) = self.faction {
                let faction_palette_offset = 16*(faction as usize);

                if color_index >= COLOR_HARKONNEN
                    && color_index < COLOR_HARKONNEN + 7 {
                    color_index = color_index + faction_palette_offset
                }
            }

            palette.color_at(color_index)
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
