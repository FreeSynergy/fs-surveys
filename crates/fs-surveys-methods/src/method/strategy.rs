//! The strategy trait every method implements.
//!
//! Pattern: Strategy (one implementation per method, chosen by the registry).

use crate::{Ballot, Result, Tally};

/// One way of deciding. Pure: same ballots, same [`Tally`], in any order.
pub trait DecisionMethod: Send + Sync {
    /// Whether this ballot form fits the method.
    fn accepts(&self, ballot: &Ballot) -> bool;

    /// Evaluates `ballots` over `choices`.
    ///
    /// # Errors
    /// [`Error::NoChoices`](crate::Error::NoChoices), `WrongBallot`, `UnknownChoice`,
    /// `InvalidBallot`, or `InvalidThreshold`.
    fn evaluate(&self, choices: &[String], ballots: &[Ballot]) -> Result<Tally>;

    /// Whether `held_back` answers, kept out of `ballots` (which has `ballots` entries), could
    /// still change the decided `tally`. The default is conservative: any held-back answer can.
    /// Methods override it only where the bound is provable with integer arithmetic.
    fn held_back_can_change(&self, _tally: &Tally, _ballots: usize, held_back: usize) -> bool {
        held_back > 0
    }
}
