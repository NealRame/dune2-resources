use std::collections::HashMap;
use std::io::{
    Read,
    Write,
};

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

use serde::{Deserialize, Serialize};

use rmp_serde;

use crate::prelude::{
    Error,
    Faction,
    Palette,
    Result,
    TileBitmap,
    Tilemap,
    Tileset,
};


#[derive(Debug, Serialize, Deserialize)]
pub struct Resources {
    pub palette: Palette,
    pub tilesets: HashMap<String, Tileset>,
    pub tilemaps: Vec<Tilemap>,
}

impl Resources {
    pub fn get_tileset(
        &self,
        tileset_id: &str,
    ) -> Result<&Tileset> {
        self.tilesets
            .get(tileset_id)
            .ok_or(Error::TilesetInvalidId(tileset_id.into()))
    }

    pub fn get_tile_bitmap(
        &self,
        tileset_id: &str,
        tile_index: usize,
        faction: Option<Faction>,
    ) -> Result<TileBitmap> {
        let tileset = self.get_tileset(tileset_id)?;
        let tile = tileset.tile_at(tile_index)?;

        Ok(TileBitmap::with_resources(tile, faction, self))
    }
}

impl Resources {
    pub fn read_from<R: Read>(
        reader: &mut R,
    ) -> core::result::Result<Resources, rmp_serde::decode::Error> {
        let mut inflate_reader = DeflateDecoder::new(reader);
        rmp_serde::decode::from_read(&mut inflate_reader)
    }
}

impl Resources {
    pub fn write_to<W: Write>(
        &self,
        writer: &mut W,
    ) -> core::result::Result<(), rmp_serde::encode::Error> {
        let mut output = DeflateEncoder::new(writer, Compression::best());
        rmp_serde::encode::write(&mut output, self)
    }
}
