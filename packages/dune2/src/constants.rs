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


macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! factions {
    ($($faction:ident),+) => {
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
        #[cfg_attr(feature = "wasm", wasm_bindgen())]
        pub enum Dune2Faction {
            $($faction,)*
        }

        pub const FACTION_COUNT: usize = count!($($faction)*);
        pub const FACTIONS: [&'static str; FACTION_COUNT] = [
            $(stringify!($faction),)*
        ];
        
        impl Dune2Faction {
            pub fn try_from_str(v: &str) -> Result<Self> {
                $(
                    if stringify!($faction).to_lowercase() == v.to_lowercase() {
                        return Ok(Self::$faction);
                    }
                )*
                Err(Error::FactionInvalidString(v.into()))
            }
        }
    };
}

factions!(Harkonnen, Atreides, Ordos, Fremen, Sardaukar, Mercenary);
