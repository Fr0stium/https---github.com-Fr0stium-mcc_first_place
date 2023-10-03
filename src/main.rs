mod player;
use player::Season;
use rand::{seq::IteratorRandom, thread_rng};
use std::env;

const MCC_PLAYER_COUNT: usize = 40;

fn output_skill_levels(season: Season) {
    let mut players = player::get_players(season);
    let mut rng = thread_rng();
    let mut skills = vec![0.0; players.len()];
    let mut simulations = 1;
    loop {
        // Pick 40 players from the available ones.
        let player_sample = players.iter().choose_multiple(&mut rng, MCC_PLAYER_COUNT);
        for player in player_sample.iter() {
            let mut first_place_probability = 0.0;
            for &coin in player.coin_history.iter().filter(|&&x| x > 0) {
                let mut mass = player.epmf(coin);
                for opponent in player_sample.iter().filter(|&p| p != player) {
                    mass *= opponent.ecdf(coin);
                }
                first_place_probability += mass;
            }
            let index = players.iter().position(|p| &p == player).unwrap();
            skills[index] += first_place_probability;
        }
        // Print progress every time the number of simulations is a power of 2.
        if simulations & (simulations - 1) == 0 {
            print!("{esc}c", esc = 27 as char); // Clears the console to print.
            for i in 0..players.len() {
                players[i].skill = skills[i] / (simulations as f64);
            }
            let mut sorted = players.clone();
            sorted.sort_by(|a, b| a.skill.partial_cmp(&b.skill).unwrap());
            for player in sorted.iter() {
                println!("{}, {}", player.username, player.skill);
            }
            println!("Simulations: {simulations}");
        }
        simulations += 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => output_skill_levels(Season::All),
        2 => match args[1].parse::<u32>() {
            Ok(1) => output_skill_levels(Season::Season1),
            Ok(2) => output_skill_levels(Season::Season2),
            Ok(3) => output_skill_levels(Season::Season3),
            Err(_) => println!("Type in an integer"),
            _ => println!("Season not found"),
        },
        _ => println!("Wrong number of arguments"),
    }
}
