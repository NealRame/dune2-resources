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
