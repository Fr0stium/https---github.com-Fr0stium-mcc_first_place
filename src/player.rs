#[derive(Clone)]
pub struct Player {
    pub username: String,
    pub coin_history: Vec<Option<i32>>,
    pub playcount: usize,
}

impl Player {
    /// Finds the proportion of coins less than or equal to `c`.
    pub fn ecdf(&self, c: i32) -> f64 {
        let coin_history = &self.coin_history;
        let n = coin_history.iter().flatten().count();
        let count = coin_history.iter().flatten().filter(|&&x| x <= c).count();

        (count as f64) / (n as f64)
    }

    /// Finds the proportion of coins exactly equal to `c`.
    pub fn epmf(&self, c: i32) -> f64 {
        let coin_history = &self.coin_history;
        let n = coin_history.iter().flatten().count();
        let count = coin_history.iter().flatten().filter(|&&x| x == c).count();

        (count as f64) / (n as f64)
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}

impl Eq for Player {}
