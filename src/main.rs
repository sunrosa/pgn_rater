mod pgn;

fn main() {
    let mut pgn = pgn::from_file("all.pgn").unwrap();

    loop {
        let mut visitor = pgn::Outcome::default();
        let result = pgn.read_game(&mut visitor);

        match result {
            Ok(o) => match o {
                Some(s) => match s {
                    Ok(o) => {
                        println!("{:?}", o);
                    }
                    Err(e) => println!("{}", e),
                },
                None => break,
            },
            Err(e) => println!("{}", e),
        }
    }
}
