//! Method declarations and the strategy trait.

mod approval;
mod borda;
mod consent;
mod kind;
mod majority;
mod plurality;
mod read;
mod runoff;
mod schulze;
mod score;
mod share;
mod strategy;
mod supermajority;
mod systemic;

pub(crate) use approval::Approval;
pub(crate) use borda::Borda;
pub(crate) use consent::Consent;
pub use kind::Method;
pub(crate) use majority::Majority;
pub(crate) use plurality::Plurality;
pub(crate) use runoff::InstantRunoff;
pub(crate) use schulze::Schulze;
pub(crate) use score::Score;
pub use strategy::DecisionMethod;
pub(crate) use supermajority::Supermajority;
pub(crate) use systemic::SystemicConsensing;
