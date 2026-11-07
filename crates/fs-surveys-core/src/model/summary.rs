//! Results: summaries and evaluations.
//!
//! Pattern: Value objects (plain data) — what `Results` hands out.

use fs_objects::ObjectId;
use fs_surveys_methods::{Method, OptionResult};

/// A choice (or scale value) and how often it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceCount {
    /// Choice id, or the value as text.
    pub choice: String,
    /// How often.
    pub count: u32,
}

/// The answers to one question.
#[derive(Clone, Debug, PartialEq)]
pub struct QuestionSummary {
    /// The question.
    pub question: ObjectId,
    /// How many answered.
    pub answers: u32,
    /// Too few answers to show (`minGroup`); everything else is empty.
    pub hidden: bool,
    /// Choices; scale values (and, on `named`, number and date values) as text.
    pub counts: Vec<ChoiceCount>,
    /// Number, scale and date (days since 1970-01-01); none when there is no value.
    pub mean: Option<f64>,
    /// Free text, shuffled.
    pub texts: Vec<String>,
}

/// The released (or, on `named`, submitted) answers of a survey.
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    /// Released ballots (named: submitted responses).
    pub ballots: u32,
    /// Fewer than `minGroup` ballots on an anonymous survey.
    pub hidden: bool,
    /// One entry per question.
    pub questions: Vec<QuestionSummary>,
    /// Answers not released yet.
    pub held_back: u32,
}

/// One question decided by one method.
#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    /// The method used.
    pub method: Method,
    /// Empty = not decided.
    pub winners: Vec<String>,
    /// Every choice with its number and rank.
    pub options: Vec<OptionResult>,
    /// Why not decided.
    pub note: String,
    /// Too few answers to show.
    pub hidden: bool,
    /// Answers not counted because they are not released yet.
    pub held_back: u32,
}
