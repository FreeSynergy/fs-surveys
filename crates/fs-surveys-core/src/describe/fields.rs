//! The fields of the four data types, as rows.
//!
//! Pattern: Registry rows (data only), the same `Field` rows the command table uses, so the backend
//! reads fields and command inputs alike.

use fs_types::BasicType::{MultilineText, Text};

use crate::commands::{ANONYMITY, Field, KIND, METHOD, STATE, YES_NO};

/// `Survey`: the AS2 `OrderedCollection` and its `fs:` profile.
pub(super) const SURVEY: &[Field] = &[
    Field::new("name", Text).required(),
    Field::choice("state", STATE),
    Field::choice("anonymity", ANONYMITY),
    Field::reference("scope", "Group").multiple(),
    Field::new("stage", Text),
    Field::new("minGroup", Text),
    Field::choice("aiAllowed", YES_NO),
    Field::reference("creator", "Person"),
    Field::new("categories", Text).multiple(),
    Field::reference("questions", "Question").multiple(),
];

/// `Question`: the AS2 `Question` and the profile for what AS2 lacks.
pub(super) const QUESTION: &[Field] = &[
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
    Field::new("conditions", Text).multiple(),
];

/// `SurveyResponse`: a named answer (anonymous ones are never records).
pub(super) const RESPONSE: &[Field] = &[
    Field::reference("survey", "Survey").required(),
    Field::reference("person", "Person").required(),
    Field::new("answers", Text).multiple(),
    Field::choice("submitted", YES_NO),
    Field::choice("ai", YES_NO),
];

/// `Decision`: what people decided, linked to tasks.
pub(super) const DECISION: &[Field] = &[
    Field::reference("survey", "Survey").required(),
    Field::reference("question", "Question").required(),
    Field::choice("method", METHOD).required(),
    Field::new("thresholdPercent", Text),
    Field::new("outcome", Text).multiple(),
    Field::new("note", MultilineText),
    Field::reference("tasks", "Task").multiple(),
    Field::new("stage", Text),
];
