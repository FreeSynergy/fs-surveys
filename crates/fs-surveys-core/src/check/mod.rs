//! Rules checked before anything is stored: design, conditions, answers.
//!
//! Pattern: Facade (declarations) — `design` for the survey, `interpret` for conditions, `answers` for submits.

mod answers;
mod design;
mod interpret;

pub use answers::{drop_empty, validate_answers};
pub use design::{check_conditions, check_min_group, check_publishable, check_question};
pub use interpret::{given, holds};
