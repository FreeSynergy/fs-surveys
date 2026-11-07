//! The contract as Rust traits and the ports the core needs.
//!
//! Pattern: Facade (declarations) — callers depend on these traits, never on `SurveyRepo`.

mod ai;
mod decisions;
mod designer;
mod feed;
mod ports;
mod respond;
mod results;
mod surveys;

pub use ai::AiMarking;
pub use decisions::Decisions;
pub use designer::Designer;
pub use feed::{Registry, Subscription, SurveyChange, SurveyFeed, SurveyListener};
pub use ports::{GroupLookup, TokenDelivery};
pub use respond::Respond;
pub use results::Results;
pub use surveys::Surveys;
