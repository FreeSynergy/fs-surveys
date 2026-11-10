//! The type descriptions of the surveys program (ADR 0066): what a generated frontend is told.
//!
//! Pattern: Registry entry (module map only) — see `types.rs` for the `DescribesType` impls, `build.rs`
//! for the table-to-description step and `keys.rs` for the choice key spelling.

mod build;
mod fields;
mod keys;
mod types;

pub use build::{PROGRAM, command_description, registry};
pub use keys::{camel, kebab};
pub use types::{DECISION, QUESTION, RESPONSE, SURVEY};
