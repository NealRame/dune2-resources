use anyhow::{anyhow, Result};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub const COLOR_HARKONNEN: usize = 144;
pub const COLOR_ATREIDES: usize = 160;
pub const COLOR_ORDOS: usize = 176;
pub const COLOR_FREMEN: usize = 192;
pub const COLOR_SARDAUKAR: usize = 208;
pub const COLOR_MERCENARY: usize = 224;


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Faction {
    Harkonnen,
    Atreides,
    Ordos,
    Fremen,
    Sardaukar,
    Mercenary,
}

impl Faction {
    pub fn try_from_str(faction: &str) -> Result<Self> {
        match faction.to_lowercase().as_str() {
            "harkonnen" => {
                Ok(Self::Harkonnen)
            },
            "atreides" => {
                Ok(Self::Atreides)
            },
            "ordos" => {
                Ok(Self::Ordos)
            },
            "fremen" => {
                Ok(Self::Fremen)
            },
            "sardaukar" => {
                Ok(Self::Sardaukar)
            },
            "mercenary" => {
                Ok(Self::Mercenary)
            },
            _ => {
                Err(anyhow!("'{faction}' Invalid faction"))
            }
        }
    }
}

#[cfg(feature = "wasm")]
impl Faction {
    pub fn try_from_js_value(value: &JsValue) -> Result<Faction, JsError> {
        match value.as_string() {
            Some(value) => {
                Faction::try_from_str(value.as_str()).map_err(|err| {
                    JsError::new(err.to_string().as_str())
                })
            },
            _ => {
                Err(JsError::new("Invalid value"))
            }
        }
    }
}
