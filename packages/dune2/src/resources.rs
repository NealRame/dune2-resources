use std::collections::HashMap;
use std::fmt;
use std::io::Read;

use anyhow::{anyhow, Result};

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

use serde::{Deserialize, Serialize};

use rmp_serde;

use crate::*;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourcesError {
    TilesetNotFound(String),
}

impl fmt::Display for ResourcesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TilesetNotFound(tileset_id) => write!(
                f,
                "Resources: tileset '{tileset_id}' not found"
            ),
        }
    }
}

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
            .ok_or(anyhow!(ResourcesError::TilesetNotFound(
                tileset_id.into(),
            )))
    }

    pub fn get_tile_bitmap(
        &self,
        tileset_id: &str,
        tile_index: usize,
        faction: Option<Faction>,
    ) -> Result<TileBitmap> {
        let tileset = self.get_tileset(tileset_id)?;
        let tile = tileset.get_tile(tile_index)?;

        Ok(TileBitmap::with_resources(tile, faction, self))
    }
}

impl Resources {
    pub fn read_from<R>(
        reader: &mut R,
    ) -> Result<Resources> where R: Read {
        let mut inflate_reader = DeflateDecoder::new(reader);
        let rc = rmp_serde::decode::from_read(&mut inflate_reader)?;

        Ok(rc)
    }
}

impl Resources {
    pub fn write_to<W>(
        &self,
        writer: &mut W,
    ) -> Result<()> where W: std::io::Write {
        let mut output = DeflateEncoder::new(writer, Compression::best());

        rmp_serde::encode::write(&mut output, self)?;
        Ok(())
    }
}
