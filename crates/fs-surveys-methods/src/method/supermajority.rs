//! Stub, replaced next.
//!
//! Pattern: Strategy.

use crate::method::DecisionMethod;
use crate::{Ballot, Result, Tally};

/// Strategy.
pub(crate) struct Supermajority {
    pub(crate) threshold_percent: u8,
}

impl DecisionMethod for Supermajority {
    fn accepts(&self, _ballot: &Ballot) -> bool {
        false
    }
    fn evaluate(&self, _choices: &[String], _ballots: &[Ballot]) -> Result<Tally> {
        Err(crate::Error::InvalidThreshold(self.threshold_percent))
    }
}
