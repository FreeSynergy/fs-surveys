//! Schulze: pairwise preferences, widest paths with winning-votes strength.
//!
//! A ranked choice beats an unranked one; unranked choices are indifferent to each other.
//! Winners are all undefeated candidates. Value is the number of opponents beaten by path
//! strength; ranks follow that count.
//!
//! Pattern: Strategy.

use super::read::{lists, need_choices};
use crate::method::DecisionMethod;
use crate::rank::{count, no_ballots, options, to_f64};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::Schulze`](crate::Method::Schulze).
pub(crate) struct Schulze;

/// `d[i][j]`: ballots preferring `i` to `j`.
fn preferences(n: usize, rankings: &[Vec<usize>]) -> Vec<Vec<i64>> {
    let mut d = vec![vec![0_i64; n]; n];
    for ranking in rankings {
        let mut pos = vec![usize::MAX; n];
        for (p, &c) in ranking.iter().enumerate() {
            pos[c] = p;
        }
        for i in 0..n {
            for j in 0..n {
                if pos[i] < pos[j] {
                    d[i][j] += 1;
                }
            }
        }
    }
    d
}

/// Strongest path strengths `p[i][j]` (Floyd-Warshall variant).
fn strongest_paths(d: &[Vec<i64>]) -> Vec<Vec<i64>> {
    let n = d.len();
    let mut p = vec![vec![0_i64; n]; n];
    for i in 0..n {
        for j in 0..n {
            if i != j && d[i][j] > d[j][i] {
                p[i][j] = d[i][j];
            }
        }
    }
    for k in 0..n {
        for i in (0..n).filter(|&i| i != k) {
            for j in (0..n).filter(|&j| j != i && j != k) {
                p[i][j] = p[i][j].max(p[i][k].min(p[k][j]));
            }
        }
    }
    p
}

impl DecisionMethod for Schulze {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Ranking(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let rankings = lists("schulze", choices, ballots, true)?;
        if rankings.is_empty() {
            return Ok(no_ballots(choices));
        }
        let n = choices.len();
        let p = strongest_paths(&preferences(n, &rankings));
        let beaten: Vec<i64> = (0..n)
            .map(|i| count((0..n).filter(|&j| j != i && p[i][j] > p[j][i]).count()))
            .collect();
        let winners: Vec<String> = (0..n)
            .filter(|&i| (0..n).all(|j| j == i || p[i][j] >= p[j][i]))
            .map(|i| choices[i].clone())
            .collect();
        let values: Vec<f64> = beaten.iter().map(|&b| to_f64(b)).collect();
        let note = if winners.len() > 1 {
            "Several choices are undefeated and tied; all of them are listed as winners."
        } else {
            ""
        };
        Ok(Tally {
            winners,
            options: options(choices, &beaten, &values),
            note: note.into(),
        })
    }
}
