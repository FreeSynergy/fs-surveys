//! The store-backed surveys program: module map.
//!
//! Pattern: Facade (declarations) — `SurveyRepo` is the one entry point; each file is one concept.

mod backend;
mod consts;
mod decisions;
mod envelope;
mod evaluate;
mod export;
mod handle;
mod invite;
mod lifecycle;
mod named;
mod provide;
mod questions;
mod reading;
mod submit;
mod summary;
mod support;
mod surveys;
mod widen;

pub use consts::{BOX_TYPE, CONTEXT, DECISION_TYPE, INVITEE_TYPE, RESPONSE_TYPE, SURVEY_TYPE};
pub use handle::SurveyRepo;
