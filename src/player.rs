use std::{
    fs::File,
    io::{BufRead, BufReader},
};

// Only consider up to this many MCCs in the selected season.
// if this is set to usize::MAX, this means consider all MCCs.
const STOP_AT_MCC: usize = usize::MAX;

#[derive(Clone)]
pub struct Player {
    pub skill: f64,
    pub username: String,
    pub coin_history: Vec<u32>,
}

impl Player {
    /// Finds the proportion of coins less than or equal to c.
    pub fn ecdf(&self, c: u32) -> f64 {
        let coin_history = &self.coin_history;
        let n = coin_history.iter().filter(|&&x| x > 0).count();
        let count = coin_history.iter().filter(|&&x| x > 0 && x <= c).count();
        (count as f64) / (n as f64)
    }

    /// Finds the proportion of coins exactly equal to c.
    pub fn epmf(&self, c: u32) -> f64 {
        let coin_history = &self.coin_history;
        let n = coin_history.iter().filter(|&&x| x > 0).count();
        let count = coin_history.iter().filter(|&&x| x == c).count();
        (count as f64) / (n as f64)
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}

impl Eq for Player {}

pub enum Season {
    All,
    Season1,
    Season2,
    Season3,
}

impl Season {
    fn as_str(&self) -> &'static str {
        match self {
            Season::All => "season_all.csv",
            Season::Season1 => "season_1.csv",
            Season::Season2 => "season_2.csv",
            Season::Season3 => "season_3.csv",
        }
    }
}

pub fn get_players(season: Season) -> Vec<Player> {
    let mut players = Vec::new();
    let file = File::open(season.as_str()).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        let split_line: Vec<&str> = line.split(',').collect();
        let username = String::from(split_line[0]);
        let mut coin_history = Vec::<u32>::new();
        let mut has_played = false;
        for coin_str in split_line.iter().skip(1).take(STOP_AT_MCC) {
            let coins = coin_str.trim().parse().unwrap();
            coin_history.push(coins);
            if !has_played && coins > 0 {
                has_played = true;
            }
        }
        if !has_played {
            continue;
        }
        let player = Player {
            coin_history,
            skill: 0.0,
            username,
        };
        players.push(player);
    }
    players
}
