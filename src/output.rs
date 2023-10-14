use crate::{player::Player, season::Season};
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const MCC_PLAYER_COUNT: usize = 40;
const SIMULATIONS_PER_PLAYER: usize = 2 << 19;

/// Randomly picks `MCC_PLAYER_COUNT` players from the available ones, then computes the win probability for each player in the sample.
pub fn output_win_probabilities(season: &Season, stop_at_mcc: usize) {
    let players = get_players(season, stop_at_mcc);
    let calculate_variance = false;
    let mut rng = thread_rng();

    println!("Simulations per player: {}", SIMULATIONS_PER_PLAYER);

    for i in 0..players.len() {
        let p = &players[i];
        let n_p = p.coin_history.iter().filter(|&&x| x > 0).count() as f64;

        let mut win_probability_sum = 0.0;
        let mut variance_sum = 0.0;
        let mut covariance_sum = 0.0;

        let mut samples = Vec::new();
        let mut sample_win_probabilities = Vec::new();

        for _ in 0..SIMULATIONS_PER_PLAYER {
            let mut sample_win_probability = 0.0;
            let mut sample_variance: f64 = 0.0;

            let opponent_sample = players
                .iter()
                .filter(|&opponent| opponent != p)
                .choose_multiple(&mut rng, MCC_PLAYER_COUNT - 1);

            for &c in p.coin_history.iter().filter(|&&x| x > 0).unique() {
                let p_pmf_e = p.epmf(c);

                // To compute the sample win probability, compute the product of all the eCDFs.
                let product_q_ecdf_e = opponent_sample.iter().map(|q| q.ecdf(c)).product::<f64>();

                sample_win_probability += p_pmf_e * product_q_ecdf_e;

                // To compute the sample variance, compute this product.
                let p_pmf_2_e = (p_pmf_e + (n_p - 1.0) * p_pmf_e.powi(2)) / n_p;
                let mut product_q_ecdf_2_e = 1.0;
                for q in opponent_sample.iter() {
                    let n_q = q.coin_history.iter().filter(|&&x| x > 0).count() as f64;
                    let q_ecdf = q.ecdf(c);
                    product_q_ecdf_2_e *= (q_ecdf + (n_q - 1.0) * q_ecdf.powi(2)) / n_q;
                }

                sample_variance +=
                    p_pmf_2_e * product_q_ecdf_2_e - (p_pmf_e * product_q_ecdf_e).powi(2);
            }

            win_probability_sum += sample_win_probability;
            variance_sum += sample_variance;

            if calculate_variance {
                sample_win_probabilities.push(sample_win_probability);
                samples.push(opponent_sample);
            }
        }

        if calculate_variance {
            for sample_pair in samples.iter().combinations(2) {
                let mut covariance = 0.0;
                let sample_1 = sample_pair[0];
                let sample_2 = sample_pair[1];

                let mut sample_win_probability_q = 0.0;
                let mut sample_win_probability_r = 0.0;

                for &c in p.coin_history.iter().filter(|&&x| x > 0).unique() {
                    let p_pmf_e = p.epmf(c);
                    let product_q_ecdf_e = sample_1.iter().map(|q| q.ecdf(c)).product::<f64>();
                    let product_r_ecdf_e = sample_2.iter().map(|r| r.ecdf(c)).product::<f64>();
                    sample_win_probability_q += p_pmf_e * product_q_ecdf_e;
                    sample_win_probability_r += p_pmf_e * product_r_ecdf_e;
                }

                covariance -= sample_win_probability_q * sample_win_probability_r;
                let mut coin_pair_sum = 0.0;

                for &c_1 in p.coin_history.iter().filter(|&&x| x > 0).unique() {
                    for &c_2 in p.coin_history.iter().filter(|&&x| x > 0).unique() {
                        let mut k_p = 1.0;
                        if c_1 == c_2 {
                            k_p *= (p.epmf(c_1) + (n_p - 1.0) * p.epmf(c_1).powi(2)) / n_p;
                        } else {
                            k_p *= p.epmf(c_1) * p.epmf(c_2);
                        }
                        for (q, r) in sample_1.iter().zip(sample_2.iter()) {
                            let n_q = q.coin_history.iter().filter(|&&x| x > 0).count() as f64;
                            let n_r = r.coin_history.iter().filter(|&&x| x > 0).count() as f64;
                            if c_1 == c_2 && n_q == n_r {
                                k_p *= (q.ecdf(c_1) + (n_q - 1.0) * q.ecdf(c_1).powi(2)) / n_q;
                            } else {
                                k_p *= q.ecdf(c_1) * r.ecdf(c_2);
                            }
                            if k_p == 0.0 {
                                break;
                            }
                        }
                        coin_pair_sum += k_p;
                    }
                }

                covariance += coin_pair_sum;
                covariance_sum += covariance;
            }
        }

        let win_probability = win_probability_sum / (SIMULATIONS_PER_PLAYER as f64);

        if calculate_variance {
            let variance = (variance_sum + covariance_sum) / (SIMULATIONS_PER_PLAYER.pow(2) as f64);
            let se = variance.sqrt();
            let ci_l = (win_probability - 1.96 * se).max(0.0);
            let ci_u = (win_probability + 1.96 * se).min(1.0);
            println!("{}, {}, ({}, {})", p.username, win_probability, ci_l, ci_u);
        } else {
            println!("{}, {}", p.username, win_probability);
        }
    }
}

fn get_players(season: &Season, stop_at_mcc: usize) -> Vec<Player> {
    let mut players = Vec::new();
    let file = File::open(season.get_file()).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        let split_line: Vec<&str> = line.split(',').collect();
        let username = String::from(split_line[0]).trim().to_string();
        let mut coin_history = Vec::<i32>::new();
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
            variance: 0.0,
            username,
        };
        players.push(player);
    }
    players
}
