//! Rules checked before anything is stored: design, conditions, answers, participation.
//!
//! Pattern: Facade (declarations) — `design` for the survey, `interpret` for conditions, `answers` for submits.

mod answers;
mod design;
mod interpret;
mod participation;
mod patterns;

pub use answers::{drop_empty, validate_answers};
pub use design::{check_conditions, check_min_group, check_publishable, check_question};
pub use interpret::{given, holds};
pub use participation::{admits, class_of};
pub use patterns::{BURST_LIMIT, same_text, signals_of};
