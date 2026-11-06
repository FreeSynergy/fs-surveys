//! A question, its choices and its conditions.
//!
//! Pattern: Value objects (plain data). `Condition` is the expression the Interpreter evaluates.

use fs_store::ObjectId;
use fs_surveys_methods::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::kinds::{ConditionOp, QuestionKind};
use super::method_name::serde_method;

/// One choice of a question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    /// Id, unique within the question.
    pub id: String,
    /// What people read.
    pub label: String,
}

/// Ask a question only if another one was answered this way.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    /// An earlier question.
    pub question: ObjectId,
    /// The comparison.
    pub op: ConditionOp,
    /// Choice id or number as text; unused for `answered`.
    #[serde(default)]
    pub value: String,
}

/// A question of a survey.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    /// Id (assigned by `add_question`).
    pub id: ObjectId,
    /// What kind of answer it takes.
    pub kind: QuestionKind,
    /// The question as people read it.
    pub text: String,
    /// Must be answered (when its conditions hold).
    pub required: bool,
    /// Choices of choice questions.
    pub choices: Vec<Choice>,
    /// Lower bound of `scale` and `optionScores`.
    pub min: i32,
    /// Upper bound of `scale` and `optionScores`.
    pub max: i32,
    /// The method shown first.
    #[serde(with = "serde_method")]
    pub method: Method,
    /// All must hold for the question to be asked.
    pub conditions: Vec<Condition>,
    /// Lower bound of `number`; none = unbounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_number: Option<f64>,
    /// Upper bound of `number`; none = unbounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_number: Option<f64>,
}

impl Question {
    /// A question of `kind` with no choices, no bounds and no conditions.
    pub fn new(kind: QuestionKind, text: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            kind,
            text: text.into(),
            required: false,
            choices: vec![],
            min: 0,
            max: 0,
            method: Method::Plurality,
            conditions: vec![],
            min_number: None,
            max_number: None,
        }
    }

    /// Builder: set the choices from `(id, label)` pairs.
    #[must_use]
    pub fn with_choices(mut self, choices: &[(&str, &str)]) -> Self {
        self.choices = choices
            .iter()
            .map(|(id, label)| Choice {
                id: (*id).into(),
                label: (*label).into(),
            })
            .collect();
        self
    }

    /// Builder: use this id (to edit an existing question).
    #[must_use]
    pub fn with_id(mut self, id: ObjectId) -> Self {
        self.id = id;
        self
    }

    /// Builder: mark as required.
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// The ids of the choices.
    pub fn choice_ids(&self) -> Vec<String> {
        self.choices.iter().map(|c| c.id.clone()).collect()
    }

    /// The AS2 `Question` form, with the full profile in `fs:question`.
    pub fn as2(&self, closed: bool) -> Value {
        let options: Vec<Value> = self
            .choices
            .iter()
            .map(|c| json!({"type": "Note", "name": c.label}))
            .collect();
        let mut v = json!({
            "type": "Question", "id": format!("urn:uuid:{}", self.id), "name": self.text,
            "closed": closed, "fs:question": self,
        });
        match self.kind {
            QuestionKind::SingleChoice => v["oneOf"] = options.into(),
            QuestionKind::MultipleChoice => v["anyOf"] = options.into(),
            _ => {}
        }
        v
    }
}
