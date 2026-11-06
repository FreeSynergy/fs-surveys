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

/// The ballots that say something. A ballot with no content is not a ballot, whatever the method;
/// one of a form the method does not accept is kept, so that the method can refuse it.
fn with_content(strategy: &dyn DecisionMethod, ballots: &[Ballot]) -> Vec<Ballot> {
    ballots
        .iter()
        .filter(|b| !(b.is_empty() && strategy.accepts(b)))
        .cloned()
        .collect()
}

/// Evaluates `ballots` under `method`; ballots with no content are left out.
///
/// # Errors
/// See [`DecisionMethod::evaluate`].
pub fn evaluate(method: &Method, choices: &[String], ballots: &[Ballot]) -> Result<Tally> {
    let strategy = method_for(method);
    strategy.evaluate(choices, &with_content(&*strategy, ballots))
}

/// Like [`evaluate`], but `held_back` answers are known to exist and are not in `ballots`.
/// When they could change a decided result, `winners` is emptied and `note` says so; options
/// and ranks stay.
///
/// # Errors
/// See [`DecisionMethod::evaluate`].
pub fn evaluate_with_held_back(
    method: &Method,
    choices: &[String],
    ballots: &[Ballot],
    held_back: usize,
) -> Result<Tally> {
    let strategy = method_for(method);
    let ballots = with_content(&*strategy, ballots);
    let mut tally = strategy.evaluate(choices, &ballots)?;
    if held_back > 0
        && !tally.winners.is_empty()
        && strategy.held_back_can_change(&tally, ballots.len(), held_back)
    {
        tally.winners.clear();
        tally.note = format!(
            "{held_back} held-back answers could change the result, so it is not decided yet."
        );
    }
    Ok(tally)
}
