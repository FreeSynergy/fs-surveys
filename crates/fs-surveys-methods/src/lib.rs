//! `fs-surveys-methods`: decision methods of FreeSynergy surveys. Pure code, no storage.
//!
//! Ballots go in, a [`Tally`] comes out. Each method is one strategy behind [`DecisionMethod`];
//! [`method_for`] maps a [`Method`] to its strategy. A ballot form that does not fit a method is
//! an error, never a guess.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]

mod ballot;
mod error;
mod method;
mod rank;
mod registry;
mod tally;

pub use ballot::Ballot;
pub use error::{Error, Result};
pub use method::{DecisionMethod, Method};
pub use registry::{evaluate, method_for};
pub use tally::{OptionResult, Tally};
