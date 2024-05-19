use std::{borrow::Borrow, collections::HashMap};

use pgn_reader::Color;
use skillratings::glicko2::{self, Glicko2Config, Glicko2Rating};

mod pgn;

fn main() {
    let mut pgn = pgn::from_file("all.pgn").unwrap();

    let mut g2_players: HashMap<String, Glicko2Rating> = HashMap::new();
    let g2_config = Glicko2Config::new();

    loop {
        let mut visitor = pgn::Outcome::default();
        let result = pgn.read_game(&mut visitor);

        match result {
            Ok(o) => match o {
                Some(s) => match s {
                    Ok(outcome) => {
                        let wl = match outcome.2 {
                            pgn_reader::Outcome::Decisive { winner } => match winner {
                                Color::White => skillratings::Outcomes::WIN,
                                Color::Black => skillratings::Outcomes::LOSS,
                            },
                            pgn_reader::Outcome::Draw => skillratings::Outcomes::DRAW,
                        };

                        let (w, b) = glicko2::glicko2(
                            g2_players
                                .get(&outcome.0)
                                .unwrap_or(&Glicko2Rating::default()),
                            g2_players
                                .get(&outcome.1)
                                .unwrap_or(&Glicko2Rating::default()),
                            &wl,
                            &g2_config,
                        );

                        g2_players.insert(outcome.0, w);
                        g2_players.insert(outcome.1, b);
                    }
                    Err(e) => println!("PGN error: {}", e),
                },
                None => break,
            },
            Err(e) => println!("IO error: {}", e),
        }
    }

    let mut ratings_sorted = g2_players.into_iter().collect::<Vec<_>>();
    ratings_sorted.sort_by(|a, b| b.1.rating.total_cmp(&a.1.rating));

    for rating in ratings_sorted {
        println!("{}: {}", rating.0, rating.1.rating);
    }
}
