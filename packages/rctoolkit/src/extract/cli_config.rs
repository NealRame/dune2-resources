use dune2_rc::constants;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ArgExtractDune2Faction {
    Harkonnen,
    Atreides,
    Ordos,
    Fremen,
    Sardaukar,
    Mercenary,
}

impl Into<constants::Dune2Faction> for ArgExtractDune2Faction {
    fn into(self) -> constants::Dune2Faction {
        match self {
            Self::Harkonnen => constants::Dune2Faction::Harkonnen,
            Self::Atreides => constants::Dune2Faction::Atreides,
            Self::Ordos => constants::Dune2Faction::Ordos,
            Self::Fremen => constants::Dune2Faction::Fremen,
            Self::Sardaukar => constants::Dune2Faction::Sardaukar,
            Self::Mercenary => constants::Dune2Faction::Mercenary,
        }
    }
}
