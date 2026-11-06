//! Conditions evaluated over the answers given so far.
//!
//! Pattern: Interpreter — a `Condition` is a small expression, `holds` evaluates it against `Given`.

use std::collections::HashMap;

use fs_store::ObjectId;

use crate::model::{Answer, AnswerValue, Condition, ConditionOp, Question};

/// The answers of one respondent by question.
pub type Given<'a> = HashMap<ObjectId, &'a AnswerValue>;

/// Index the answers by question.
pub fn given(answers: &[Answer]) -> Given<'_> {
    answers.iter().map(|a| (a.question, &a.value)).collect()
}

fn number(v: &AnswerValue) -> Option<f64> {
    match v {
        AnswerValue::Number(n) => Some(*n),
        AnswerValue::Scale(s) => Some(f64::from(*s)),
        _ => None,
    }
}

fn equals(v: &AnswerValue, want: &str) -> bool {
    match v {
        AnswerValue::Choice(s) | AnswerValue::Text(s) | AnswerValue::Date(s) => s == want,
        AnswerValue::Choices(l) => l.len() == 1 && l[0] == want,
        other => match (number(other), want.parse::<f64>()) {
            (Some(a), Ok(b)) => (a - b).abs() < f64::EPSILON,
            _ => false,
        },
    }
}

fn includes(v: &AnswerValue, want: &str) -> bool {
    match v {
        AnswerValue::Choices(l) | AnswerValue::Ranking(l) => l.iter().any(|c| c == want),
        AnswerValue::Scores(l) => l.iter().any(|s| s.choice == want),
        other => equals(other, want),
    }
}

/// Whether one condition holds. A missing answer holds for nothing, not even `equals`.
pub fn holds(c: &Condition, given: &Given<'_>) -> bool {
    let Some(v) = given.get(&c.question) else {
        return false;
    };
    let want = c.value.as_str();
    let cmp = |f: fn(f64, f64) -> bool| match (number(v), want.parse::<f64>()) {
        (Some(a), Ok(b)) => f(a, b),
        _ => false,
    };
    match c.op {
        ConditionOp::Answered => true,
        ConditionOp::Equals => equals(v, want),
        ConditionOp::Includes => includes(v, want),
        ConditionOp::AtLeast => cmp(|a, b| a >= b),
        ConditionOp::AtMost => cmp(|a, b| a <= b),
    }
}

/// Whether the question is asked: all its conditions hold.
pub fn asked(q: &Question, given: &Given<'_>) -> bool {
    q.conditions.iter().all(|c| holds(c, given))
}
