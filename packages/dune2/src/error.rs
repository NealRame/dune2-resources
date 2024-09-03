use std::fmt;

use crate::prelude::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    FactionInvalidString(String),

    TilesetInvalidTileSize(String, Size),
    TilesetInvalidTileIndex(String, usize),
    TilesetInvalidId(String),
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
