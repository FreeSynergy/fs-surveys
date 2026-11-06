//! Systemic consensing: resistance 0..=10 per choice, least total resistance wins.
//!
//! Provisional: a choice missing from a ballot counts as the maximum, 10 (no statement is taken as
//! full resistance). The owner may change this once real surveys show what people expect.
//!
//! Pattern: Strategy.

use super::read::{need_choices, scores};
use crate::method::DecisionMethod;
use crate::rank::{from_totals, no_ballots};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::SystemicConsensing`](crate::Method::SystemicConsensing).
pub(crate) struct SystemicConsensing;

impl DecisionMethod for SystemicConsensing {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Scores(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let all = scores("systemic consensing", choices, ballots, Some((0, 10)))?;
        if all.is_empty() {
            return Ok(no_ballots(choices));
        }
        let mut sums = vec![10_i64 * i64::try_from(all.len()).unwrap_or(0); choices.len()];
        for (i, s) in all.into_iter().flatten() {
            sums[i] += i64::from(s) - 10;
        }
        Ok(from_totals(choices, &sums, false))
    }
}
