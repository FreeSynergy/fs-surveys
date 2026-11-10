//! From the command table to the wire description.
//!
//! Pattern: Builder over the table (single source). Every command of the table is in the description, the
//! missing ones too, with `supported = false` (`fs-types` v0.0.11); the backend refuses them with a typed
//! problem. The flag is copied in `carry_flag`, the only place that reads it. A `find` starts from the
//! standard `find` of `fs-types` (term, list of references) and adds the filters of the row.

use fs_types::{
    CommandDescription, CommandKind, DescriptionProblem, FieldDescription, TypeRegistry,
};

use super::keys::kebab;
use crate::commands::{Command, Field};
use crate::model::{Decision, NamedResponse, Question, Survey};

/// The program that owns the types (ADR 0050).
pub const PROGRAM: &str = "fs-surveys";

/// `Survey` -> `survey`, `SurveyResponse` -> `survey-response`: the type part of a label key.
pub(super) fn type_key(data_type: &str) -> String {
    let mut chars = data_type.chars();
    let first = chars.next().map(|c| c.to_ascii_lowercase());
    kebab(&first.into_iter().chain(chars).collect::<String>())
}

/// A description field from a row; `label` is the translation key.
pub(super) fn field(f: &Field, label: &str) -> FieldDescription {
    let mut d = FieldDescription::new(f.name, f.basic).label(label);
    if f.required {
        d = d.required();
    }
    if f.multiple {
        d = d.multiple();
    }
    if !f.target.is_empty() {
        d = d.target(f.target);
    }
    if !f.choices.is_empty() {
        d = d.choices(f.choices);
    }
    d
}

/// The single place where the support flag of a command meets its description.
fn carry_flag(d: CommandDescription, supported: bool) -> CommandDescription {
    if supported { d } else { d.unsupported() }
}

/// The wire description of one table row (supported or not).
#[must_use]
pub fn command_description(c: &Command) -> CommandDescription {
    let base = type_key(c.data_type);
    let key = kebab(c.name);
    let label = format!("{base}.cmd.{key}");
    // The standard list (rule of 2026-11-10): `find` with a `term`; the row adds only the filters.
    let standard = c.kind == CommandKind::Find && c.name == "find";
    let mut d = if standard {
        CommandDescription::standard_find(&label, c.data_type)
    } else {
        CommandDescription::new(c.name, c.kind, &label)
    };
    let (skip_in, skip_out) = if standard {
        ("term", "items")
    } else {
        ("", "")
    };
    for f in c.input.iter().filter(|f| f.name != skip_in) {
        d = d.input(field(f, &format!("{base}.{key}.{}", kebab(f.name))));
    }
    for f in c.result.iter().filter(|f| f.name != skip_out) {
        d = d.result(field(f, &format!("{base}.{key}.{}", kebab(f.name))));
    }
    carry_flag(d, c.supported)
}

/// The registry of the four types of this program.
///
/// # Errors
/// A [`DescriptionProblem`] if a description breaks the rules (a defect of the tables, caught by tests).
pub fn registry() -> Result<TypeRegistry, DescriptionProblem> {
    let mut r = TypeRegistry::new();
    r.register::<Survey>()?;
    r.register::<Question>()?;
    r.register::<NamedResponse>()?;
    r.register::<Decision>()?;
    Ok(r)
}
