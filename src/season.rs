use std::{fmt, fmt::Display};

pub enum Season {
    All,
    Season1,
    Season2,
    Season3,
}

impl Season {
    pub fn get_file(&self) -> &'static str {
        match self {
            Season::All => "season_all.csv",
            Season::Season1 => "season_1.csv",
            Season::Season2 => "season_2.csv",
            Season::Season3 => "season_3.csv",
        }
    }
}

impl Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::All => write!(f, "All"),
            Self::Season1 => write!(f, "1"),
            Self::Season2 => write!(f, "2"),
            Self::Season3 => write!(f, "3"),
        }
    }
}
