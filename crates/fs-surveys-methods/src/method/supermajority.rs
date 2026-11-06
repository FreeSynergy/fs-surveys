//! Supermajority: a choice needs at least `threshold_percent` of the ballots (51..=100).
//!
//! Pattern: Strategy (shares its counting with majority through `share`).

use super::read::{need_choices, singles};
use super::share::decide_by_share;
use crate::method::DecisionMethod;
use crate::rank::top_and_lead;
use crate::{Ballot, Error, Result, Tally};

/// Strategy for [`Method::Supermajority`](crate::Method::Supermajority).
pub(crate) struct Supermajority {
    pub(crate) threshold_percent: u8,
}

impl DecisionMethod for Supermajority {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Single(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        if !(51..=100).contains(&self.threshold_percent) {
            return Err(Error::InvalidThreshold(self.threshold_percent));
        }
        let picks = singles("supermajority", choices, ballots)?;
        let t = u64::from(self.threshold_percent);
        // Integer arithmetic, no rounding: votes * 100 >= threshold * ballots.
        let need = format!("At least {t}% of the ballots");
        Ok(decide_by_share(
            choices,
            &picks,
            |v, n| v * 100 >= t * n,
            &need,
        ))
    }

    fn held_back_can_change(&self, tally: &Tally, ballots: usize, held_back: usize) -> bool {
        if tally.winners.is_empty() {
            return held_back > 0;
        }
        let votes = top_and_lead(tally).0;
        votes * 100 < u64::from(self.threshold_percent) * (ballots + held_back) as u64
    }
}
