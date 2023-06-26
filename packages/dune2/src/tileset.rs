use std::fmt;

use serde::{Deserialize, Serialize};

use crate::*;

pub struct TileBitmap<'a> {
    faction: Option<Faction>,
    palette: &'a Palette,
    tile_size: Size,
    tile_data: &'a Vec<u8>,
}

impl<'a> TileBitmap<'a> {
    pub fn create(
        resource: &'a Resources,
        tileset_id: String,
        tile_index: usize,
        faction: Option<Faction>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let palette = &resource.palette;
        let tileset = resource.tilesets.get(&tileset_id).ok_or(
            format!("Tileset {} not found", tileset_id)
        )?;

        let tile_data = tileset.tiles.get(tile_index).ok_or(
            format!("Tile #{} not found in tileset {}", tile_index, tileset_id)
        )?;

        Ok(Self {
            faction,
            palette,
            tile_data,
            tile_size: tileset.tile_size,
        })
    }
}

impl Bitmap for TileBitmap<'_> {
    fn width(&self) -> u32 {
        self.tile_size.width
    }

    fn height(&self) -> u32 {
        self.tile_size.height
    }
}

impl BitmapGetPixel for TileBitmap<'_> {
    fn get_pixel(&self, point: Point) -> Option<Color> {
        point_to_index(point, self.size()).map(|index| {
            let mut color_index = self.tile_data[index] as usize;

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
