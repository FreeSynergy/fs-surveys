//! `fs-surveys-app`: the window of the surveys program (ADR 0065, 0066).
//!
//! No render code and no look is here: the program gives the type descriptions of `fs-surveys-core`
//! and a [`Backend`](fs_admin_frontend::Backend) per data type over the core; the generated Admin
//! frontend of `fs-components` draws them in the standard window shell. See `DESIGN.md`.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc
)]

mod answers;
mod charts;
mod ctx;
mod decision;
mod gate;
pub mod gui;
mod ids;

mod input;
mod methods;
mod operator;
mod question;
mod refuse;
mod response;
mod results;
mod survey;
mod survey_run;
mod words;

pub use charts::{ChartError, question_chart};
pub use ctx::Ctx;
pub use decision::DecisionBackend;
pub use gate::{Gated, Typed};
pub use operator::OperatorIsHuman;
pub use question::QuestionBackend;
pub use refuse::RunError;
pub use response::ResponseBackend;
pub use survey::SurveyBackend;
pub use words::{TITLE, Words};
