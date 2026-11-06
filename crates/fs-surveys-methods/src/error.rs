//! Errors of the decision methods.
//!
//! Pattern: Newtype-free error enum (one variant per refusal; nothing is ever guessed).

/// Why a set of ballots could not be evaluated.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The ballot form does not fit the method.
    #[error("method {method} does not accept a {ballot} ballot")]
    WrongBallot {
        /// Name of the method.
        method: String,
        /// Name of the ballot form.
        ballot: String,
    },
    /// A ballot names a choice id that is not among the choices.
    #[error("a ballot names a choice that is not among the choices")]
    UnknownChoice,
    /// The supermajority threshold is outside 51..=100.
    #[error("invalid threshold {0}: must be 51..=100")]
    InvalidThreshold(u8),
    /// There are no choices to decide between.
    #[error("no choices")]
    NoChoices,
    /// A ballot is malformed (duplicate choice, score out of range).
    #[error("invalid ballot: {0}")]
    InvalidBallot(String),
}

/// Result type of this crate.
pub type Result<T> = std::result::Result<T, Error>;
