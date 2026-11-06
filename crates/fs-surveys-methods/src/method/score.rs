//! Score: highest sum of scores wins; a choice missing from a ballot counts as 0.
//!
//! Pattern: Strategy.

use super::read::{need_choices, scores};
use crate::method::DecisionMethod;
use crate::rank::{from_totals, no_ballots};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::Score`](crate::Method::Score).
pub(crate) struct Score;

impl DecisionMethod for Score {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Scores(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let all = scores("score", choices, ballots, None)?;
        if all.is_empty() {
            return Ok(no_ballots(choices));
        }
        let mut sums = vec![0_i64; choices.len()];
        for (i, s) in all.into_iter().flatten() {
            sums[i] += i64::from(s);
        }
        Ok(from_totals(choices, &sums, true))
    }
}
