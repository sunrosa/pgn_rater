use std::collections::HashMap;

use chrono::NaiveDate;
use pgn::OutcomeResult;
use pgn_reader::Color;
use skillratings::glicko2::{self, Glicko2Config, Glicko2Rating};

mod pgn;

fn main() {
    // Raw PGN buffer
    let mut pgn = pgn::from_file("all.pgn").unwrap();

    // Glicko-2 data
    let mut g2_players: HashMap<String, Glicko2Rating> = HashMap::new();
    let g2_config = Glicko2Config::new();

    // Rating graph per player
    let mut rating_graph: HashMap<String, HashMap<NaiveDate, f64>> = HashMap::new();

    // Actual data captured from PGN
    let mut games: Vec<OutcomeResult> = Vec::new();

    // Pull relevant information from PGN with visitor and error checking
    loop {
        let mut visitor = pgn::Outcome::default();
        let result = pgn.read_game(&mut visitor);

        match result {
            Ok(o) => match o {
                Some(s) => match s {
                    Ok(outcome) => games.push(outcome),
                    Err(e) => match e {
                        pgn::OutcomeError::HeaderUtf8(e) => panic!("{}", e),
                        pgn::OutcomeError::DateFormatting(e) => panic!("{}", e),
                        _ => {}
                    },
                },
                None => break,
            },
            Err(e) => eprintln!("IO error: {}", e),
        }
    }

    // Sort games by date
    games.sort_by(|a, b| a.date.cmp(&b.date));

    // Rate all games
    for game in games {
        rate(game.clone(), &mut g2_players, g2_config);

        // Insert ratings into rating graph
        rating_graph
            .entry(game.white.clone())
            .or_default()
            .insert(game.date, g2_players.entry(game.white).or_default().rating);
        rating_graph
            .entry(game.black.clone())
            .or_default()
            .insert(game.date, g2_players.entry(game.black).or_default().rating);
    }

    // Sort players by rating descending and filter by RD
    let mut ratings_sorted = g2_players
        .into_iter()
        .filter(|r| r.1.deviation < 180.)
        .collect::<Vec<_>>();
    ratings_sorted.sort_by(|a, b| b.1.rating.total_cmp(&a.1.rating));

    // Print all filtered player ratings
    for rating in ratings_sorted {
        println!("{}: {} ({})", rating.0, rating.1.rating, rating.1.deviation);
    }

    // Print my rating graph
    let mut rating_graph_sorted: Vec<_> =
        rating_graph.get("Sunrosa").unwrap().into_iter().collect();
    rating_graph_sorted.sort_by(|a, b| a.0.cmp(&b.0));
    println!(
        "{}",
        rating_graph_sorted
            .into_iter()
            .fold(String::new(), |a, b| format!("{a}\n{:?}: {}", b.0, b.1))
    );
}

fn rate(
    outcome: OutcomeResult,
    g2_players: &mut HashMap<String, Glicko2Rating>,
    g2_config: Glicko2Config,
) {
    // Win-loss-draw outcome (Fix this shit name lol)
    let wl = match outcome.outcome {
        pgn_reader::Outcome::Decisive { winner } => match winner {
            Color::White => skillratings::Outcomes::WIN,
            Color::Black => skillratings::Outcomes::LOSS,
        },
        pgn_reader::Outcome::Draw => skillratings::Outcomes::DRAW,
    };

    // Calculate the ratings
    let (w, b) = glicko2::glicko2(
        g2_players
            .get(&outcome.white)
            .unwrap_or(&Glicko2Rating::default()),
        g2_players
            .get(&outcome.black)
            .unwrap_or(&Glicko2Rating::default()),
        &wl,
        &g2_config,
    );

    // Write the ratings
    g2_players.insert(outcome.white, w);
    g2_players.insert(outcome.black, b);
}
