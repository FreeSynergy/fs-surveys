//! Instant runoff: count first remaining preferences, eliminate the lowest, repeat.
//!
//! Exhausted ballots (no remaining choice ranked) leave the count. A winner needs more than half
//! of the non-exhausted ballots. Several tied for lowest go out together only if their votes
//! together are below the next count, otherwise nothing is decided. If every remaining choice is lowest, it is a tie and all are
//! winners. Value is the votes in the round a choice left or won; ranks follow the round of
//! elimination (later is better).
//!
//! Pattern: Strategy.

use super::read::{lists, need_choices};
use crate::method::DecisionMethod;
use crate::rank::{count, no_ballots, options, to_f64, undecided};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::InstantRunoff`](crate::Method::InstantRunoff).
pub(crate) struct InstantRunoff;

/// First remaining choice of each ballot, counted; the number of non-exhausted ballots.
fn count_round(n: usize, rankings: &[Vec<usize>], alive: &[bool]) -> (Vec<i64>, i64) {
    let mut votes = vec![0_i64; n];
    let mut active = 0;
    for r in rankings {
        if let Some(&c) = r.iter().find(|&&c| alive[c]) {
            votes[c] += 1;
            active += 1;
        }
    }
    (votes, active)
}

impl DecisionMethod for InstantRunoff {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Ranking(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let rankings = lists("instant runoff", choices, ballots, true)?;
        if rankings.is_empty() {
            return Ok(no_ballots(choices));
        }
        let n = choices.len();
        let mut alive = vec![true; n];
        let mut key = vec![0_i64; n];
        let mut value = vec![0.0_f64; n];
        for round in 1_i64.. {
            let (votes, active) = count_round(n, &rankings, &alive);
            let left: Vec<usize> = (0..n).filter(|&i| alive[i]).collect();
            for &i in &left {
                key[i] = round;
                value[i] = to_f64(votes[i]);
            }
            if active == 0 {
                return Ok(undecided(
                    choices,
                    &key,
                    &value,
                    "Every ballot is empty, so nothing is decided.",
                ));
            }
            let lowest = left.iter().map(|&i| votes[i]).min().unwrap_or(0);
            let out: Vec<usize> = left
                .iter()
                .copied()
                .filter(|&i| votes[i] == lowest)
                .collect();
            let top = left.iter().copied().find(|&i| votes[i] * 2 > active);
            if let Some(w) = top {
                key[w] = round + 1;
                let note = String::new();
                return Ok(Tally {
                    winners: vec![choices[w].clone()],
                    options: options(choices, &key, &value),
                    note,
                });
            }
            if out.len() == left.len() {
                for &i in &left {
                    key[i] = round + 1;
                }
                let winners = left.iter().map(|&i| choices[i].clone()).collect();
                let note = format!(
                    "All {} remaining choices are tied in round {round}; all are listed as winners.",
                    left.len()
                );
                return Ok(Tally {
                    winners,
                    options: options(choices, &key, &value),
                    note,
                });
            }
            // Several tied for last go out together only if all their votes together are still
            // below the next count; otherwise the order of elimination would decide.
            let next = left.iter().map(|&i| votes[i]).filter(|&v| v > lowest).min();
            if out.len() > 1 && next.is_some_and(|n| lowest * count(out.len()) >= n) {
                let note = format!(
                    "{} choices are tied for last place and together have at least as many votes as the next one, so the order of elimination would decide; nothing is decided.",
                    out.len()
                );
                return Ok(undecided(choices, &key, &value, &note));
            }
            for i in out {
                alive[i] = false;
            }
        }
        unreachable!("each round removes a choice")
    }
}
