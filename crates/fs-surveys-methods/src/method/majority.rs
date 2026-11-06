//! Majority: a choice wins only with strictly more than half of the ballots.
//!
//! Pattern: Strategy (shares its counting with supermajority through `share`).

use super::read::{need_choices, singles};
use super::share::decide_by_share;
use crate::method::DecisionMethod;
use crate::rank::top_and_lead;
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::Majority`](crate::Method::Majority).
pub(crate) struct Majority;

impl DecisionMethod for Majority {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Single(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let picks = singles("majority", choices, ballots)?;
        // votes * 2 > ballots  <=>  votes * 100 > 50 * ballots
        Ok(decide_by_share(
            choices,
            &picks,
            |v, n| v * 2 > n,
            "More than half of the ballots",
        ))
    }

    fn held_back_can_change(&self, tally: &Tally, ballots: usize, held_back: usize) -> bool {
        if tally.winners.is_empty() {
            return held_back > 0;
        }
        let votes = top_and_lead(tally).0;
        votes * 2 <= (ballots + held_back) as u64
    }
}
