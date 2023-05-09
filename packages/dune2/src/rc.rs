use serde::{Deserialize, Serialize};

use crate::color::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dune2RC {
    pub palette: Palette,
}