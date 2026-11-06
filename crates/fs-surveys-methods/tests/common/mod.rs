//! Shared helpers for the integration tests.
#![allow(dead_code)]

use fs_surveys_methods::{Ballot, Tally};

pub fn ids(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

pub fn single(id: &str) -> Ballot {
    Ballot::Single(id.to_owned())
}

pub fn singles(spec: &[(&str, usize)]) -> Vec<Ballot> {
    spec.iter()
        .flat_map(|&(id, n)| (0..n).map(move |_| single(id)))
        .collect()
}

pub fn multi(v: &[&str]) -> Ballot {
    Ballot::Multiple(ids(v))
}

pub fn ranking(v: &[&str]) -> Ballot {
    Ballot::Ranking(ids(v))
}

/// `n` copies of one ranking.
pub fn rankings(n: usize, v: &[&str]) -> Vec<Ballot> {
    (0..n).map(|_| ranking(v)).collect()
}

pub fn scores(v: &[(&str, i32)]) -> Ballot {
    Ballot::Scores(v.iter().map(|&(c, s)| (c.to_owned(), s)).collect())
}

pub fn winners(t: &Tally) -> Vec<&str> {
    t.winners.iter().map(String::as_str).collect()
}

pub fn value(t: &Tally, id: &str) -> f64 {
    t.options
        .iter()
        .find(|o| o.choice == id)
        .expect("choice listed")
        .value
}

pub fn rank(t: &Tally, id: &str) -> u16 {
    t.options
        .iter()
        .find(|o| o.choice == id)
        .expect("choice listed")
        .rank
}

pub fn order(t: &Tally) -> Vec<&str> {
    t.options.iter().map(|o| o.choice.as_str()).collect()
}
