//! Shared counting of majority and supermajority.
//!
//! Pattern: Template method as a free function (the caller supplies the pass test).

use crate::Tally;
use crate::rank::{from_totals, no_ballots, undecided};

/// Counts one vote per pick; the choice for which `passes(votes, ballots)` holds wins.
/// `rule` names the requirement in the note.
pub(crate) fn decide_by_share(
    choices: &[String],
    picks: &[usize],
    passes: impl Fn(u64, u64) -> bool,
    rule: &str,
) -> Tally {
    if picks.is_empty() {
        return no_ballots(choices);
    }
    let mut votes = vec![0_i64; choices.len()];
    for &p in picks {
        votes[p] += 1;
    }
    let n = picks.len() as u64;
    let tally = from_totals(choices, &votes, true);
    if let Some(i) = votes.iter().position(|&v| passes(v.unsigned_abs(), n)) {
        return Tally {
            winners: vec![choices[i].clone()],
            note: String::new(),
            ..tally
        };
    }
    let values: Vec<f64> = votes.iter().map(|&v| crate::rank::to_f64(v)).collect();
    undecided(
        choices,
        &votes,
        &values,
        &format!("{rule} is required and no choice reached it."),
    )
}
