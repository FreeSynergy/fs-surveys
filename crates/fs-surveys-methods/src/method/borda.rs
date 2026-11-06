//! Borda count: with n choices, rank i (from 0) earns n-1-i points; unranked earn 0.
//!
//! Pattern: Strategy.

use super::read::{lists, need_choices};
use crate::method::DecisionMethod;
use crate::rank::{count, from_totals, no_ballots};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::Borda`](crate::Method::Borda).
pub(crate) struct Borda;

impl DecisionMethod for Borda {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Ranking(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let rankings = lists("borda", choices, ballots, true)?;
        if rankings.is_empty() {
            return Ok(no_ballots(choices));
        }
        let n = count(choices.len());
        let mut points = vec![0_i64; choices.len()];
        for ranking in rankings {
            for (pos, i) in ranking.into_iter().enumerate() {
                points[i] += n - 1 - count(pos);
            }
        }
        Ok(from_totals(choices, &points, true))
    }
}
