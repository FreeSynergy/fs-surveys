//! Shows one ballot set under several methods.
use fs_surveys_methods::{Ballot, Method, evaluate};

fn ranking(ids: &[&str]) -> Ballot {
    Ballot::Ranking(ids.iter().map(|s| (*s).to_owned()).collect())
}

fn main() {
    let choices: Vec<String> = ["A", "B", "C"].iter().map(|s| (*s).to_owned()).collect();
    let mut ballots = vec![ranking(&["A", "B", "C"]); 6];
    ballots.push(ranking(&["B", "A", "C"]));
    ballots.extend(vec![ranking(&["B", "C", "A"]); 3]);
    ballots.extend(vec![ranking(&["C", "B", "A"]); 5]);
    for (name, method) in [
        ("Schulze", Method::Schulze),
        ("Instant runoff", Method::InstantRunoff),
        ("Borda", Method::Borda),
    ] {
        match evaluate(&method, &choices, &ballots) {
            Ok(t) => println!("{name}: winners {:?}, options {:?}", t.winners, t.options),
            Err(e) => println!("{name}: {e}"),
        }
    }
    // A method whose ballot form does not fit is an error, never a guess.
    println!(
        "Plurality: {}",
        evaluate(&Method::Plurality, &choices, &ballots).unwrap_err()
    );
}
