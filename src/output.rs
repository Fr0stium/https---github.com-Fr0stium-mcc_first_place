use crate::{player::Player, season::Season};
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const MCC_PLAYER_COUNT: usize = 40;

/// Randomly picks `MCC_PLAYER_COUNT` players from the available ones, then computes the win probability for each player in the sample.
pub fn output_win_probabilities(season: &Season, stop_at_mcc: usize) {
    let mut players = get_players(season, stop_at_mcc);
    let mut rng = thread_rng();
    let mut win_probabilities = vec![0.0; players.len()];
    let mut simulations = 1;

    loop {
        let player_sample = players.iter().choose_multiple(&mut rng, MCC_PLAYER_COUNT);

        for player in player_sample.iter() {
            let mut win_probability = 0.0;

            for &coin in player.coin_history.iter().filter(|&&x| x > 0).unique() {
                let mass = player.epmf(coin);
                let ecdf_product = player_sample
                    .iter()
                    .filter(|&opponent| opponent != player)
                    .map(|&opponent| opponent.ecdf(coin))
                    .product::<f64>();
                win_probability += mass * ecdf_product;
            }

            let index = players.iter().position(|p| &p == player).unwrap();
            win_probabilities[index] += win_probability;
        }

        // Print progress every time the number of simulations is a power of 2.
        if simulations & (simulations - 1) == 0 {
            print!("{esc}c", esc = 27 as char); // Clears the console to print.

            for i in 0..players.len() {
                players[i].win_probability = win_probabilities[i] / (simulations as f64);
            }

            let mut sorted = players.clone();
            sorted.sort_by(|a, b| a.win_probability.partial_cmp(&b.win_probability).unwrap());

            for player in sorted.iter() {
                println!("{}, {}", player.username, player.win_probability);
            }

            println!("\nSimulations: {simulations}");
            println!("Season: {season}");
            println!("MCCs: {stop_at_mcc}");
        }

        simulations += 1;
    }
}

fn get_players(season: &Season, stop_at_mcc: usize) -> Vec<Player> {
    let mut players = Vec::new();
    let file = File::open(season.get_file()).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        let split_line: Vec<&str> = line.split(',').collect();
        let username = String::from(split_line[0]).trim().to_string();
        let mut coin_history = Vec::<u32>::new();
        let mut has_played = false;

        for coin_str in split_line.iter().skip(1).take(stop_at_mcc) {
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
            win_probability: 0.0,
            username,
        };

        players.push(player);
    }
    players
}
