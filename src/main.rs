mod output;
mod player;
mod season;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let calculate_variance = false;
    match args.len() {
        1 => output::output_win_probabilities(&season::Season::All, usize::MAX, calculate_variance),
        2 => match args[1].parse::<u32>() {
            Ok(season) => match season {
                0 => output::output_win_probabilities(
                    &season::Season::All,
                    usize::MAX,
                    calculate_variance,
                ),
                1 => output::output_win_probabilities(
                    &season::Season::Season1,
                    usize::MAX,
                    calculate_variance,
                ),
                2 => output::output_win_probabilities(
                    &season::Season::Season2,
                    usize::MAX,
                    calculate_variance,
                ),
                3 => output::output_win_probabilities(
                    &season::Season::Season3,
                    usize::MAX,
                    calculate_variance,
                ),
                _ => println!("Season {season} not found."),
            },
            Err(err) => println!("Error: {err}. Type in an integer."),
        },
        3 => match (args[1].parse::<u32>(), args[2].parse::<usize>()) {
            (Ok(season), Ok(count)) => match season {
                0 => output::output_win_probabilities(
                    &season::Season::All,
                    count,
                    calculate_variance,
                ),
                1 => output::output_win_probabilities(
                    &season::Season::Season1,
                    count,
                    calculate_variance,
                ),
                2 => output::output_win_probabilities(
                    &season::Season::Season2,
                    count,
                    calculate_variance,
                ),
                3 => output::output_win_probabilities(
                    &season::Season::Season3,
                    count,
                    calculate_variance,
                ),
                _ => println!("Season not found."),
            },
            (Err(err), _) | (_, Err(err)) => println!("Error: {err}. Type in an integer."),
        },
        _ => println!("Error: specify at most 2 arguments."),
    }
}
