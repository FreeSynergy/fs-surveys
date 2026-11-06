//! The life cycle of a survey.
//!
//! Pattern: State — the enum `SurveyState` plus one transition function; anything else is an error.

use crate::error::{Error, Result};
use crate::model::SurveyState;

/// A step of the life cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// draft -> open.
    Publish,
    /// open -> closed.
    Close,
    /// closed -> open.
    Reopen,
    /// closed -> archived.
    Archive,
}

impl Step {
    /// Name used in messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Publish => "published",
            Self::Close => "closed",
            Self::Reopen => "reopened",
            Self::Archive => "archived",
        }
    }
}

/// The one transition function.
pub fn transition(from: SurveyState, step: Step) -> Result<SurveyState> {
    use SurveyState::{Archived, Closed, Draft, Open};
    match (from, step) {
        (Draft, Step::Publish) | (Closed, Step::Reopen) => Ok(Open),
        (Open, Step::Close) => Ok(Closed),
        (Closed, Step::Archive) => Ok(Archived),
        _ => Err(Error::InvalidTransition {
            from,
            step: step.name(),
        }),
    }
}
