//! Stub, replaced next.
//!
//! Pattern: Strategy.

use crate::method::DecisionMethod;
use crate::{Ballot, Result, Tally};

/// Strategy.
pub(crate) struct InstantRunoff;

impl DecisionMethod for InstantRunoff {
    fn accepts(&self, _ballot: &Ballot) -> bool {
        false
    }
    fn evaluate(&self, _choices: &[String], _ballots: &[Ballot]) -> Result<Tally> {
        Err(crate::Error::NoChoices)
    }
}
