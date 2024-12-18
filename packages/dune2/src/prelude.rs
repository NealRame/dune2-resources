//! Crate prelude

pub use crate::assets::*;
pub use crate::bitmap::*;
pub use crate::constants::*;
pub use crate::error::*;
pub use crate::shape::*;
pub use crate::tile::*;
pub use crate::tilemap::*;
pub use crate::tileset::*;

#[cfg(feature = "wasm")]
pub use crate::wasm::*;

pub type Result<T> = core::result::Result<T, Error>;
