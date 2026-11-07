//! Error type of the surveys core.
//!
//! Pattern: none (plain error enum via `thiserror`) — one `Error` for every public operation.

use fs_objects::ObjectId;

use crate::model::SurveyState;

/// Result of every fallible operation of this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong in the surveys core.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The object store failed (an unencrypted store refusing sensitive data shows up here).
    #[error("object store: {0}")]
    Store(#[from] fs_objects::Error),
    /// Stored JSON could not be read or written.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// No object of this kind with this id.
    #[error("{what} {id} not found")]
    NotFound {
        /// What was looked for.
        what: &'static str,
        /// The id.
        id: ObjectId,
    },
    /// The input is wrong (bad answer, unknown id, bad bound, ...).
    #[error("invalid: {0}")]
    Invalid(String),
    /// The rules of the survey forbid it (wrong anonymity level, wrong state, ...).
    #[error("refused: {0}")]
    Refused(String),
    /// The transition is not part of the life cycle.
    #[error("survey is {from:?} and cannot be {step}")]
    InvalidTransition {
        /// Current state.
        from: SurveyState,
        /// The step tried.
        step: &'static str,
    },
    /// The participant is an AI, or not known as a human, and the survey does not allow AIs.
    #[error("this survey does not allow AIs")]
    AiNotAllowed,
    /// Only the creator of a survey can change who may take part.
    #[error("only the creator of the survey can do this")]
    NotCreator,
    /// The token is spent or unknown; the two cannot be told apart on purpose.
    #[error("token is not valid")]
    InvalidToken,
    /// A token could not be delivered; the person is not counted as invited.
    #[error("token for {person} could not be delivered: {reason}")]
    Delivery {
        /// The person.
        person: String,
        /// The delivery's reason.
        reason: String,
    },
    /// The system's random source failed.
    #[error("random source: {0}")]
    Random(String),
    /// Decision methods refused the ballots.
    #[error(transparent)]
    Method(#[from] fs_surveys_methods::Error),
    /// The explicit "not possible" of a provider (ADR 0045).
    #[error(transparent)]
    Unsupported(#[from] fs_interfaces::Unsupported),
}
