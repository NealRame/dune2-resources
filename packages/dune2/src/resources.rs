use std::error::{Error};
use std::io::{Read};
use std::collections::HashMap;

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

use serde::{Deserialize, Serialize};

use rmp_serde;

use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resources {
    pub palette: Palette,
    pub tilesets: HashMap<String, Tileset>,
    pub tilemaps: Vec<Tilemap>,
    pub sprites: HashMap<String, Sprite>,
}

impl Resources {
    pub fn tile_bitmap(
        &self,
        tileset_id: &str,
        tile_index: usize,
        faction: Option<Faction>,
    ) -> Result<TileBitmap, Box<dyn std::error::Error>> {
        TileBitmap::create(self, tileset_id.into(), tile_index, faction)
    }

    pub fn tilemap_bitmap(
        &self,
        index: usize,
        faction: Option<Faction>,
    ) -> Result<TilemapBitmap, Box<dyn std::error::Error>> {
        TilemapBitmap::create(self, index, faction)
    }

    pub fn sprite_frame_bitmap(
        &self,
        sprite_id: &str,
        sprite_frame_index: usize,
        faction: Option<Faction>,
    ) -> Result<SpriteFrameBitmap, Box<dyn std::error::Error>> {
        SpriteFrameBitmap::create(self, sprite_id.into(), sprite_frame_index, faction)
    }
}

impl Resources {
    pub fn read_from<R>(
        reader: &mut R,
    ) -> Result<Resources, Box<dyn Error>> where R: Read {
        let mut inflate_reader = DeflateDecoder::new(reader);
        let rc = rmp_serde::decode::from_read(&mut inflate_reader)?;
        Ok(rc)
    }
}

impl Resources {
    pub fn write_to<W>(
        &self,
        writer: &mut W,
    ) -> Result<(), Box<dyn Error>> where W: std::io::Write {
        let mut output = DeflateEncoder::new(writer, Compression::best());
        rmp_serde::encode::write(&mut output, self)?;
        Ok(())
    }
}