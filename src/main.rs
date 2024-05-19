use pgn::Outcome;

mod pgn;

fn main() {
    let mut pgn = pgn::from_file("all.pgn").unwrap();
    let mut visitor = pgn::Outcome::default();
    println!("{:?}", pgn.read_game(&mut visitor));
    println!("{:?}", pgn.read_game(&mut visitor));
    println!("{:?}", pgn.read_game(&mut visitor));
    println!("{:?}", pgn.read_game(&mut visitor));
    println!("{:?}", pgn.read_game(&mut visitor));
    println!("{:?}", pgn.read_game(&mut visitor));
}
