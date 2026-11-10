//! `DescribesType` for the four data types of the program.
//!
//! Pattern: Registry entry (trait first) — each type implements the `fs-types` trait, so
//! `TypeRegistry::register::<T>()` puts it in the catalog the program serves. Fields come from
//! `fields.rs`, commands from the one table in `commands`, filtered by type.

use fs_types::{DescribesType, TypeDescription};

use super::build::{PROGRAM, command_description, field, type_key};
use super::fields;
use crate::commands::{self, Field};
use crate::model::{Decision, NamedResponse, Question, Survey};

/// Name of the survey type in the catalog.
pub const SURVEY: &str = "Survey";
/// Name of the question type.
pub const QUESTION: &str = "Question";
/// Name of the answer type (the contract's `NamedResponse`).
pub const RESPONSE: &str = "SurveyResponse";
/// Name of the decision type.
pub const DECISION: &str = "Decision";

fn describe(name: &str, standard: &str, rows: &[Field]) -> TypeDescription {
    let base = type_key(name);
    let mut t = TypeDescription::new(name, PROGRAM, standard, &base);
    for f in rows {
        t = t.field(field(f, &format!("{base}.{}", super::kebab(f.name))));
    }
    for c in commands::for_type(name) {
        t = t.command(command_description(c));
    }
    t
}

/// A survey: AS2 `OrderedCollection` of questions, the profile in `fs:` members.
///
/// Not described: the AS2 JSON itself (`as2`), the envelope (it belongs to `fs-objects`).
impl DescribesType for Survey {
    fn describe() -> TypeDescription {
        describe(SURVEY, "AS2 OrderedCollection + fs profile", fields::SURVEY)
    }
}

/// A question: AS2 `Question` (`oneOf`, `anyOf`, `closed`) plus the profile for scales, bounds and conditions.
impl DescribesType for Question {
    fn describe() -> TypeDescription {
        describe(QUESTION, "AS2 Question + fs profile", fields::QUESTION)
    }
}

/// A named answer. AS2 has no answer type, so this follows the contract's own profile.
impl DescribesType for NamedResponse {
    fn describe() -> TypeDescription {
        describe(RESPONSE, "fs profile (AS2 has no answer)", fields::RESPONSE)
    }
}

/// A decision. AS2 has none, so this follows the contract's own profile.
impl DescribesType for Decision {
    fn describe() -> TypeDescription {
        describe(
            DECISION,
            "fs profile (AS2 has no decision)",
            fields::DECISION,
        )
    }
}
