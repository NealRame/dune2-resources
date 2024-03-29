use std::error::Error;
use std::str::FromStr;

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

impl FromStr for Faction {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "harkonnen" => Ok(Faction::Harkonnen),
            "atreides" => Ok(Faction::Atreides),
            "ordos" => Ok(Faction::Ordos),
            "fremen" => Ok(Faction::Fremen),
            "sardaukar" => Ok(Faction::Sardaukar),
            "mercenary" => Ok(Faction::Mercenary),
            _ => Err("Invalid faction".into()),
        }
    }
}
