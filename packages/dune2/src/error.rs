use std::fmt;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

use crate::prelude::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    FactionInvalidString(String),
    #[cfg(feature = "wasm")]
    FactionInvalidValueType,

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

#[cfg(feature = "wasm")]
impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        JsValue::from(format!("{value}"))
    }
}
