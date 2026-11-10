//! Why a command did not run, as a type, and how it reaches the frontend.
//!
//! Pattern: Adapter (error mapping) — `RunError` keeps the reason typed for callers of `Gated::try_run`
//! and tests; the `Backend` trait of the frontend knows only three failures, so `From<RunError>` maps down.
//! Limit, reported to thread A: `BackendError` has no variant for "refused by a rule" or "not supported";
//! the nearest fits are used and named below.

use fs_admin_frontend::BackendError;
use fs_surveys_core::Error;
use fs_types::{RecordIssue, RecordProblem, TypeProblem};

/// Why a command did not run.
#[derive(Debug)]
pub enum RunError {
    /// The command is in the description but the core does not carry it out (ADR 0066).
    Unsupported {
        /// Data type.
        data_type: &'static str,
        /// Command name.
        command: &'static str,
        /// Why (English, for the log).
        reason: &'static str,
    },
    /// No such command or record.
    NotFound,
    /// A required input is missing.
    Missing(&'static str),
    /// An input value cannot be read (a number, an id, a pair).
    Bad {
        /// The input.
        field: String,
        /// Which of its values.
        index: usize,
    },
    /// The core refused by a rule of the survey (state, anonymity, creator ...); the reason is its text.
    Refused(String),
    /// A store could not be opened or failed.
    Store(String),
}

impl From<Error> for RunError {
    fn from(e: Error) -> Self {
        match e {
            Error::NotFound { .. } => Self::NotFound,
            Error::Store(_) | Error::Json(_) | Error::Random(_) | Error::Delivery { .. } => {
                Self::Store(e.to_string())
            }
            other => Self::Refused(other.to_string()),
        }
    }
}

fn issue(field: &str, index: usize, problem: RecordProblem) -> BackendError {
    BackendError::Rejected(vec![RecordIssue {
        field: field.to_owned(),
        index,
        problem,
    }])
}

impl From<RunError> for BackendError {
    fn from(e: RunError) -> Self {
        match e {
            // The nearest fit: "there is no such command" (it is listed, locked, and not carried out).
            RunError::Unsupported { .. } | RunError::NotFound => Self::NotFound,
            RunError::Missing(f) => issue(f, 0, RecordProblem::Missing),
            // A value that does not read (a number, an id): the closest code is "character".
            RunError::Bad { field, index } => {
                issue(&field, index, RecordProblem::Value(TypeProblem::Character))
            }
            // A rule of the survey said no: shown on the record, as "not in the allowed list".
            RunError::Refused(_) => issue("id", 0, RecordProblem::Value(TypeProblem::NotInList)),
            RunError::Store(text) => Self::Unavailable(text),
        }
    }
}
