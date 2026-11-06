//! Consent: score 1 is an objection, 0 none. A choice without objections passes.
//!
//! Pattern: Strategy.

use super::read::{need_choices, scores};
use crate::method::DecisionMethod;
use crate::rank::{no_ballots, options, to_f64};
use crate::{Ballot, Result, Tally};

/// Strategy for [`Method::Consent`](crate::Method::Consent).
pub(crate) struct Consent;

impl DecisionMethod for Consent {
    fn accepts(&self, ballot: &Ballot) -> bool {
        matches!(ballot, Ballot::Scores(_))
    }

    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
        need_choices(choices)?;
        let all = scores("consent", choices, ballots, Some((0, 1)))?;
        if all.is_empty() {
            return Ok(no_ballots(choices));
        }
        let mut objections = vec![0_i64; choices.len()];
        for (i, s) in all.into_iter().flatten() {
            objections[i] += i64::from(s);
        }
        let keys: Vec<i64> = objections.iter().map(|&o| -o).collect();
        let values: Vec<f64> = objections.iter().map(|&o| to_f64(o)).collect();
        let winners: Vec<String> = choices
            .iter()
            .zip(&objections)
            .filter(|&(_, &o)| o == 0)
            .map(|(c, _)| c.clone())
            .collect();
        let note = if winners.is_empty() {
            "Every choice has at least one objection, so nothing passes."
        } else {
            ""
        };
        Ok(Tally {
            winners,
            options: options(choices, &keys, &values),
            note: note.into(),
        })
    }
}
