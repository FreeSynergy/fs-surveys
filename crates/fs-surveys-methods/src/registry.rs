//! Registry from [`Method`] to its strategy.
//!
//! Pattern: Registry + Factory (a method value becomes a boxed strategy).

use crate::method::{
    Approval, Borda, Consent, DecisionMethod, InstantRunoff, Majority, Method, Plurality, Schulze,
    Score, Supermajority, SystemicConsensing,
};
use crate::{Ballot, Result, Tally};

/// The strategy for `method`.
#[must_use]
pub fn method_for(method: &Method) -> Box<dyn DecisionMethod> {
    match *method {
        Method::Plurality => Box::new(Plurality),
        Method::Majority => Box::new(Majority),
        Method::Supermajority { threshold_percent } => {
            Box::new(Supermajority { threshold_percent })
        }
        Method::Approval => Box::new(Approval),
        Method::Score => Box::new(Score),
        Method::Schulze => Box::new(Schulze),
        Method::InstantRunoff => Box::new(InstantRunoff),
        Method::Borda => Box::new(Borda),
        Method::Consent => Box::new(Consent),
        Method::SystemicConsensing => Box::new(SystemicConsensing),
    }
}

/// Evaluates `ballots` under `method`.
///
/// # Errors
/// See [`DecisionMethod::evaluate`].
pub fn evaluate(method: &Method, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
    method_for(method).evaluate(choices, ballots)
}
