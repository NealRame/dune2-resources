#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::prelude::{
    Error,
    Result,
};


pub const COLOR_HARKONNEN: usize = 144;
pub const COLOR_ATREIDES: usize = 160;
pub const COLOR_ORDOS: usize = 176;
pub const COLOR_FREMEN: usize = 192;
pub const COLOR_SARDAUKAR: usize = 208;
pub const COLOR_MERCENARY: usize = 224;


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Faction {
    Atreides,
    Harkonnen,
    Ordos,
    Fremen,
    Mercenary,
    Sardaukar,
}

impl Faction {
    pub fn try_from_str(faction: &str) -> Result<Self> {
        match faction.to_lowercase().as_str() {
            "atreides" => {
                Ok(Self::Atreides)
            },
            "harkonnen" => {
                Ok(Self::Harkonnen)
            },
            "ordos" => {
                Ok(Self::Ordos)
            },
            "fremen" => {
                Ok(Self::Fremen)
            },
            "mercenary" => {
                Ok(Self::Mercenary)
            },
            "sardaukar" => {
                Ok(Self::Sardaukar)
            },
            _ => {
                Err(Error::FactionInvalidString(faction.into()))
            },
        }
    }
}

#[cfg(feature = "wasm")]
impl Faction {
    pub fn try_from_js_value(
        value: &JsValue,
    ) -> core::result::Result<Faction, JsError> {
        match value.as_string() {
            Some(value) => {
                let faction = Faction::try_from_str(value.as_str())?;
                Ok(faction)
            },
            _ => {
                Err(JsError::from(Error::FactionInvalidValueType))
            }
        }
    }
}
