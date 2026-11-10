//! The one table of every command of the surveys program (ADR 0066), supported or not.
//!
//! Pattern: Registry (module map only) — see `table.rs` for the reason and the rows in `survey.rs`,
//! `question.rs`, `response.rs`, `decision.rs`.

mod choices;
mod decision;
mod question;
mod response;
mod survey;
mod table;

pub use choices::{ANONYMITY, CLAIM, EXPORT, KIND, METHOD, OPS, STATE, YES_NO};
pub use table::{Command, Field, all, for_type, requirement_ids, unsupported_commands};
