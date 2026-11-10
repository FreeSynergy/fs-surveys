//! `fs-surveys-core`: program core of FreeSynergy surveys (no GUI, no capnp server yet).
//!
//! A survey is one AS2 `OrderedCollection` object of AS2 `Question`s (ADR 0014, 0020, 0050). Named
//! answers are ordinary objects in the main store. Anonymous answers are ballots in a box in a second
//! store that has no activity log, are released in batches of at least `minGroup`, and never reach
//! the feed. Decision methods live in `fs-surveys-methods`. Start with [`Designer`] and [`SurveyRepo`];
//! see `DESIGN.md` for the decisions.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// Pedantic lints that would only add noise to this crate's small, documented API: every fallible fn
// returns the crate `Result` (documented once on `Error`), and results of pure getters are used anyway.
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod ballots;
mod check;
pub mod commands;
mod csv;
pub mod describe;
mod error;
pub mod model;
mod random;
mod repo;
mod state;
mod summarize;
mod tokens;
pub mod traits;
mod unsupported;

pub use describe::{PROGRAM, registry as type_registry};
pub use error::{Error, Result};
pub use fs_surveys_methods::{Ballot, Method, OptionResult, Tally};
pub use model::{
    ALL_STAGES, Anonymity, Answer, AnswerValue, Choice, ChoiceCount, Class, Condition, ConditionOp,
    Credential, Cursor, Decision, Evaluation, InviteReport, NamedResponse, OpenMeta, OptionScore,
    Page, Patterns, Question, QuestionKind, QuestionSummary, Signals, Summary, Survey, SurveyState,
    Voluntary,
};
pub use repo::{
    BOX_TYPE, CONTEXT, DECISION_TYPE, INVITEE_TYPE, RESPONSE_TYPE, SURVEY_TYPE, SurveyRepo,
};
pub use state::{Step, transition};
pub use traits::{
    AiMarking, Decisions, Designer, GroupLookup, Registry, Respond, Results, Subscription,
    SurveyChange, SurveyFeed, SurveyListener, Surveys, TokenDelivery,
};
pub use unsupported::not_possible;
