//! Plurality: one choice per ballot, most votes wins, a tie lists all tied.
//!
//! Pattern: Strategy.

use super::read::{need_choices, singles};
use crate::method::DecisionMethod;
use crate::rank::{from_totals, lead_can_change, no_ballots};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::Plurality`](crate::Method::Plurality).
pub(crate) struct Plurality;

impl DecisionMethod for Plurality {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Single(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let picks = singles("plurality", choices, ballots)?;
        if picks.is_empty() {
            return Ok(no_ballots(choices));
        }
        let mut votes = vec![0_i64; choices.len()];
        for p in picks {
            votes[p] += 1;
        }
        Ok(from_totals(choices, &votes, true))
    }

    fn held_back_can_change(&self, tally: &Tally, _ballots: usize, held_back: usize) -> bool {
        lead_can_change(tally, held_back)
    }
}
