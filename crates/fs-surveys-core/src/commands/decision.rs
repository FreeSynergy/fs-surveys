//! The commands of the `Decision` type.
//!
//! Pattern: Registry rows (data only). A decision is an object of its own (AS2 has none), linked to
//! tasks of `fs-tasks` by IRI.

use fs_types::BasicType::{MultilineText, Text};
use fs_types::CommandKind as K;

use super::choices::METHOD;
use super::table::{Command, Field};

const T: &str = "Decision";
const ID: Field = Field::reference("id", "Decision").required();
const TASK: Field = Field::reference("task", "Task").required();

/// All rows of the type.
pub(super) const ROWS: &[Command] = &[
    Command::new(
        T,
        "record",
        K::Create,
        &["decision.record"],
        &[
            Field::reference("survey", "Survey").required(),
            Field::reference("question", "Question").required(),
            Field::choice("method", METHOD).required(),
            Field::new("thresholdPercent", Text),
            Field::new("stage", Text),
            Field::new("outcome", Text).multiple(),
            Field::new("note", MultilineText),
        ],
        &[Field::reference("id", "Decision").required()],
    ),
    Command::new(
        T,
        "find",
        K::Find,
        &["decision.read"],
        &[
            Field::reference("survey", "Survey"),
            Field::reference("task", "Task"),
        ],
        &[Field::reference("items", "Decision").multiple()],
    ),
    Command::new(
        T,
        "linkTask",
        K::Other,
        &["decision.link"],
        &[ID, TASK],
        &[],
    ),
    Command::new(
        T,
        "unlinkTask",
        K::Other,
        &["decision.link"],
        &[ID, TASK],
        &[],
    ),
];
