//! Approval: each ballot approves any number of choices, most approvals wins.
//!
//! Pattern: Strategy.

use super::read::{lists, need_choices};
use crate::method::DecisionMethod;
use crate::rank::{from_totals, lead_can_change, no_ballots};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::Approval`](crate::Method::Approval).
pub(crate) struct Approval;

impl DecisionMethod for Approval {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Multiple(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let sets = lists("approval", choices, ballots, false)?;
        if sets.is_empty() {
            return Ok(no_ballots(choices));
        }
        let mut votes = vec![0_i64; choices.len()];
        for i in sets.into_iter().flatten() {
            votes[i] += 1;
        }
        Ok(from_totals(choices, &votes, true))
    }

    fn held_back_can_change(&self, tally: &Tally, _ballots: usize, held_back: usize) -> bool {
        lead_can_change(tally, held_back)
    }
}
