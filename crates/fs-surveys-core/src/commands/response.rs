//! The commands of the `SurveyResponse` type.
//!
//! Pattern: Registry rows (data only). The contract's `NamedResponse`; anonymous ballots are never a
//! record anyone can list (they are written without id and time), so they only appear as `submit`.

use fs_types::BasicType::{Keywords, Text};
use fs_types::CommandKind as K;

use super::choices::CLAIM;
use super::table::{Command, Field};

const T: &str = "SurveyResponse";
const SURVEY: Field = Field::reference("survey", "Survey").required();
const PERSON: Field = Field::reference("person", "Person");
/// `<question id>=<value>`; several values of one answer are joined with `,` (see `DESIGN.md`).
const ANSWERS: Field = Field::new("answers", Text).multiple();

/// All rows of the type.
pub(super) const ROWS: &[Command] = &[
    Command::new(
        T,
        "open",
        K::Other,
        &["survey.open"],
        &[SURVEY],
        &[Field::reference("survey", "Survey").required()],
    ),
    Command::new(
        T,
        "save",
        K::Create,
        // The rules of required questions and of the bounds are checked here (`question.validate`).
        &["response.save", "question.validate"],
        &[SURVEY, PERSON.required(), ANSWERS],
        &[Field::reference("id", "SurveyResponse").required()],
    ),
    Command::new(
        T,
        "find",
        K::Find,
        &["response.save"],
        &[Field::reference("survey", "Survey"), PERSON],
        &[Field::reference("items", "SurveyResponse").multiple()],
    ),
    Command::new(
        T,
        "submit",
        K::Create,
        &["response.submit", "question.validate"],
        &[
            SURVEY,
            PERSON,
            Field::new("token", Text),
            Field::choice("declaredAi", CLAIM),
            Field::new("elapsedMs", Text),
            Field::new("honeypot", Text),
            ANSWERS,
        ],
        &[],
    ),
    Command::new(
        T,
        "voluntary",
        K::Other,
        &["response.voluntary"],
        &[SURVEY, Field::new("details", Keywords).multiple()],
        &[],
    ),
    Command::new(
        T,
        "challenge",
        K::Other,
        &["survey.publicLink"],
        &[SURVEY],
        &[],
    )
    .unsupported("not built yet: adaptive proof of work, global limit, quality score"),
];
