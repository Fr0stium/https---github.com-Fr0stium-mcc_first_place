use crate::{player::Player, season::Season};
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const MCC_PLAYER_COUNT: usize = 40;
const SIMULATIONS_PER_PLAYER: usize = 2 << 20;

/// For each player p, this function randomly picks `MCC_PLAYER_COUNT - 1` players
/// from the remaining ones to face against p, and computes the win probability
/// of p in that chosen sample. This is repeated `SIMULATIONS_PER_PLAYER` times,
/// and then the average of all the win probabilities is taken.
///
/// This function will also calculate approximate confidence intervals
/// on the outputed win probability if `calculate_variance` is set to true.
/// However, it is very slow and uses a lot of memory! The variance is exactly
/// calculated, then using the central limit theorem, it obtains an approximate CI.
/// It is highly recommended that `SIMULATIONS_PER_PLAYER` is lowered
/// SIGNIFICANTLY from the default in order to compute confidence intervals,
/// or even that f64 is changed to f32. This is a naive implementation, and it can
/// easily use gigabytes of memory! Don't crash your PC!
pub fn output_win_probabilities(season: &Season, stop_at_mcc: usize, calculate_variance: bool) {
    let players = get_players(season, stop_at_mcc);
    let mut rng = thread_rng();

    println!("Simulations per player: {}", SIMULATIONS_PER_PLAYER);

    if !calculate_variance {
        for p in players.iter() {
            let mut win_probability_sum = 0.0;
            let max_coins = *p.coin_history.iter().flatten().max().unwrap();

            // Only make this player compete against opponents they have a chance of beating.
            let players_to_sample_from = players
                .iter()
                .filter(|q| q != &p && q.ecdf(max_coins) > 0.0)
                .collect::<Vec<&Player>>();

            let players_len = players.len();
            let players_to_sample_from_len = players_to_sample_from.len();

            // If the pool of opponents is less than 39, then there exists no MCC where the player can win.
            // Just return 0 in this case.
            if players_to_sample_from_len < 39 {
                println!("{}, {}", p.username, 0);
                continue;
            }

            // Adjust the win probability to account for the smaller pool of opponents, if necessary.
            let mut ratio = 1.0;

            for i in 0..MCC_PLAYER_COUNT - 1 {
                ratio *= ((players_to_sample_from_len - i) as f64) / ((players_len - 1 - i) as f64);
            }

            for _ in 0..SIMULATIONS_PER_PLAYER {
                let mut sample_win_probability = 0.0;

                let opponent_sample = players_to_sample_from
                    .iter()
                    .filter(|&&q| q != p)
                    .choose_multiple(&mut rng, MCC_PLAYER_COUNT - 1);

                for &c in p.coin_history.iter().flatten().unique() {
                    let p_pmf_e = p.epmf(c);

                    // To compute the sample win probability, compute the product of all the eCDFs.
                    let product_q_ecdf_e =
                        opponent_sample.iter().map(|q| q.ecdf(c)).product::<f64>();

                    sample_win_probability += p_pmf_e * product_q_ecdf_e;
                }

                win_probability_sum += sample_win_probability;
            }

            let win_probability = ratio * win_probability_sum / (SIMULATIONS_PER_PLAYER as f64);
            println!("{}, {}", p.username, win_probability);
        }
    } else {
        for p in players.iter() {
            let n_p = p.playcount as f64;

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

                for &c in p.coin_history.iter().flatten().unique() {
                    let p_pmf_e = p.epmf(c);

                    // To compute the sample win probability, compute the product of all the eCDFs.
                    let product_q_ecdf_e =
                        opponent_sample.iter().map(|q| q.ecdf(c)).product::<f64>();

                    sample_win_probability += p_pmf_e * product_q_ecdf_e;

                    // To compute the sample variance, compute this product.
                    let p_pmf_2_e = (p_pmf_e + (n_p - 1.0) * p_pmf_e.powi(2)) / n_p;
                    let mut product_q_ecdf_2_e = 1.0;
                    for q in opponent_sample.iter() {
                        let n_q = q.playcount as f64;
                        let q_ecdf = q.ecdf(c);
                        product_q_ecdf_2_e *= (q_ecdf + (n_q - 1.0) * q_ecdf.powi(2)) / n_q;
                    }

                    sample_variance +=
                        p_pmf_2_e * product_q_ecdf_2_e - (p_pmf_e * product_q_ecdf_e).powi(2);
                }

                win_probability_sum += sample_win_probability;
                variance_sum += sample_variance;

                sample_win_probabilities.push(sample_win_probability);
                samples.push(opponent_sample);
            }

            // Note: this will compute the covariance exactly. It is VERY SLOW, and the CIs will be wide.
            // I estimate this has time complexity O(n^6).
            // Reducing the number of simulations per player is highly recommended.
            // This should only be used to get a general idea of the uncertainty in the point estimate.
            for sample_pair in samples.iter().combinations(2) {
                let mut covariance = 0.0;
                let sample_1 = sample_pair[0];
                let sample_2 = sample_pair[1];

                let mut sample_win_probability_q = 0.0;
                let mut sample_win_probability_r = 0.0;

                for &c in p.coin_history.iter().flatten().unique() {
                    let p_pmf_e = p.epmf(c);
                    let product_q_ecdf_e = sample_1.iter().map(|q| q.ecdf(c)).product::<f64>();
                    let product_r_ecdf_e = sample_2.iter().map(|r| r.ecdf(c)).product::<f64>();
                    sample_win_probability_q += p_pmf_e * product_q_ecdf_e;
                    sample_win_probability_r += p_pmf_e * product_r_ecdf_e;
                }

                covariance -= sample_win_probability_q * sample_win_probability_r;
                let mut coin_pair_sum = 0.0;

                for &c_1 in p.coin_history.iter().flatten().unique() {
                    for &c_2 in p.coin_history.iter().flatten().unique() {
                        let mut k_p = 1.0;
                        if c_1 == c_2 {
                            k_p *= (p.epmf(c_1) + (n_p - 1.0) * p.epmf(c_1).powi(2)) / n_p;
                        } else {
                            k_p *= p.epmf(c_1) * p.epmf(c_2);
                        }
                        for (q, r) in sample_1.iter().zip(sample_2.iter()) {
                            let n_q = q.playcount as f64;
                            let n_r = r.playcount as f64;
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

            let win_probability = win_probability_sum / (SIMULATIONS_PER_PLAYER as f64);
            let variance =
                (variance_sum + 2.0 * covariance_sum) / (SIMULATIONS_PER_PLAYER.pow(2) as f64);
            let se = variance.sqrt();
            let ci_l = (win_probability - 1.96 * se).max(0.0);
            let ci_u = (win_probability + 1.96 * se).min(1.0);
            println!("{}, {}, ({}, {})", p.username, win_probability, ci_l, ci_u);
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
        let mut coin_history = Vec::new();
        let mut has_played = false;

        for coin_str in split_line.iter().skip(1).take(stop_at_mcc) {
            let coins = coin_str.trim().parse().unwrap();
            let coins: Option<i32> = if coins > 0 { Some(coins) } else { None };
            coin_history.push(coins);
            if !has_played && coins.is_some() {
                has_played = true;
            }
        }

        if !has_played {
            continue;
        }

        let playcount = coin_history.iter().flatten().count();

        let player = Player {
            coin_history,
            username,
            playcount,
        };
        players.push(player);
    }
    players
}
