//! The commands of the `Question` type.
//!
//! Pattern: Registry rows (data only). A question lives inside a survey (AS2 `OrderedCollection`) but is
//! its own data type in the contract (`Question`), so it has its own commands.

use fs_types::BasicType::Text;
use fs_types::CommandKind as K;

use super::choices::{KIND, METHOD, YES_NO};
use super::table::{Command, Field};

const T: &str = "Question";
const ID: Field = Field::reference("id", "Question").required();
/// All rows of the type.
pub(super) const ROWS: &[Command] = &[
    Command::new(
        T,
        "add",
        K::Create,
        &["question.add"],
        &[
            Field::reference("survey", "Survey").required(),
            Field::new("text", Text).required(),
            Field::choice("kind", KIND).required(),
            Field::choice("required", YES_NO),
            Field::new("choices", Text).multiple(),
            Field::new("min", Text),
            Field::new("max", Text),
            Field::new("minNumber", Text),
            Field::new("maxNumber", Text),
            Field::choice("method", METHOD),
            Field::new("thresholdPercent", Text),
        ],
        &[Field::reference("id", "Question").required()],
    ),
    // The standard `change`: an `Edit` over the fields of the type (types.capnp).
    Command::new(T, "edit", K::Change, &["question.edit"], &[], &[]),
    Command::new(T, "remove", K::Delete, &["question.edit"], &[ID], &[]),
    Command::new(
        T,
        "move",
        K::Other,
        &["question.move"],
        &[ID, Field::new("position", Text).required()],
        &[],
    ),
    Command::new(
        T,
        "setConditions",
        K::Other,
        &["logic.set"],
        &[ID, Field::new("conditions", Text).multiple()],
        &[],
    ),
    Command::new(
        T,
        "find",
        K::Find,
        &[],
        &[
            Field::reference("survey", "Survey"),
            Field::new("term", Text),
        ],
        &[Field::reference("items", "Question").multiple()],
    ),
    Command::new(
        T,
        "types",
        K::Other,
        &["question.types"],
        &[],
        &[Field::choice("kinds", KIND).multiple()],
    ),
];
