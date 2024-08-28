use dune2_rc::constants;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ArgExtractFaction {
    Harkonnen,
    Atreides,
    Ordos,
    Fremen,
    Sardaukar,
    Mercenary,
}

impl Into<constants::Faction> for ArgExtractFaction {
    fn into(self) -> constants::Faction {
        match self {
            Self::Harkonnen => constants::Faction::Harkonnen,
            Self::Atreides => constants::Faction::Atreides,
            Self::Ordos => constants::Faction::Ordos,
            Self::Fremen => constants::Faction::Fremen,
            Self::Sardaukar => constants::Faction::Sardaukar,
            Self::Mercenary => constants::Faction::Mercenary,
        }
    }
}
