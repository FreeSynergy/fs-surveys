//! Data types of the surveys core.
//!
//! Pattern: Facade (declarations) — plain data; behaviour lives in `check`, `repo` and friends.

mod answer;
mod decision;
mod kinds;
mod method_name;
mod open;
mod participant;
mod question;
mod summary;
mod survey;

pub use answer::{Answer, AnswerValue, Credential, InviteReport, NamedResponse, OptionScore};
pub use decision::{ALL_STAGES, Cursor, Decision, Page};
pub use kinds::{Anonymity, ConditionOp, QuestionKind, SurveyState};
pub use method_name::{method_parse, method_text};
pub use open::{OpenMeta, Patterns, Signals, Voluntary};
pub use participant::Class;
pub use question::{Choice, Condition, Question};
pub use summary::{ChoiceCount, Evaluation, QuestionSummary, Summary};
pub use survey::{AS2_TYPE, DEFAULT_MIN_GROUP, MIN_GROUP_FLOOR, NS, Survey};
