//! Validation of the answers of one respondent against the survey.
//!
//! Pattern: Specification + Interpreter — conditions decide which questions are asked; every answer
//! must fit its question's kind, choices and bounds.

use std::collections::HashSet;

use super::interpret::{asked, given};
use crate::error::{Error, Result};
use crate::model::{Answer, AnswerValue, Question, Survey};

fn bad<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Invalid(msg.into()))
}

/// RFC 3339 full-date: `YYYY-MM-DD`, a real day.
fn full_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

fn known_unique(q: &Question, ids: impl Iterator<Item = String>) -> Result<()> {
    let mut seen = HashSet::new();
    for id in ids {
        if !q.choices.iter().any(|c| c.id == id) {
            return bad(format!("{id:?} is not a choice of {:?}", q.text));
        }
        if !seen.insert(id.clone()) {
            return bad(format!("{id:?} is given twice"));
        }
    }
    Ok(())
}

fn within(q: &Question, v: i32) -> Result<()> {
    if v < q.min || v > q.max {
        return bad(format!(
            "{v} is outside {}..{} for {:?}",
            q.min, q.max, q.text
        ));
    }
    Ok(())
}

fn check_value(q: &Question, v: &AnswerValue) -> Result<()> {
    match v {
        AnswerValue::Choice(c) => known_unique(q, std::iter::once(c.clone())),
        AnswerValue::Choices(l) | AnswerValue::Ranking(l) => known_unique(q, l.iter().cloned()),
        AnswerValue::Scores(l) => {
            known_unique(q, l.iter().map(|s| s.choice.clone()))?;
            l.iter().try_for_each(|s| within(q, s.score))
        }
        AnswerValue::Scale(s) => within(q, *s),
        AnswerValue::Number(n) => {
            let low = q.min_number.is_some_and(|m| *n < m);
            let high = q.max_number.is_some_and(|m| *n > m);
            if !n.is_finite() || low || high {
                return bad(format!("{n} is outside the bounds of {:?}", q.text));
            }
            Ok(())
        }
        AnswerValue::Date(d) if !full_date(d) => bad(format!("{d:?} is not an RFC 3339 full-date")),
        AnswerValue::Date(_) | AnswerValue::Text(_) => Ok(()),
    }
}

fn fits(q: &Question, v: &AnswerValue) -> bool {
    use crate::model::QuestionKind as K;
    matches!(
        (q.kind, v),
        (K::SingleChoice, AnswerValue::Choice(_))
            | (K::MultipleChoice, AnswerValue::Choices(_))
            | (K::Text, AnswerValue::Text(_))
            | (K::Number, AnswerValue::Number(_))
            | (K::Date, AnswerValue::Date(_))
            | (K::Scale, AnswerValue::Scale(_))
            | (K::Ranking, AnswerValue::Ranking(_))
            | (K::OptionScores, AnswerValue::Scores(_))
    )
}

/// An answer that says nothing: blank text or an empty list.
fn is_empty(v: &AnswerValue) -> bool {
    match v {
        AnswerValue::Text(t) | AnswerValue::Choice(t) | AnswerValue::Date(t) => t.trim().is_empty(),
        AnswerValue::Choices(l) | AnswerValue::Ranking(l) => l.is_empty(),
        AnswerValue::Scores(l) => l.is_empty(),
        AnswerValue::Number(_) | AnswerValue::Scale(_) => false,
    }
}

/// Check `answers`. With `complete` (a submit) required questions must be answered; a saved draft
/// may be partial. A question whose conditions fail must be unanswered and is not required.
pub fn validate_answers(survey: &Survey, answers: &[Answer], complete: bool) -> Result<()> {
    let mut ids = HashSet::new();
    for a in answers {
        if !ids.insert(a.question) {
            return bad("a question is answered twice");
        }
        let Some(q) = survey.question(a.question) else {
            return bad(format!("unknown question {}", a.question));
        };
        if !fits(q, &a.value) {
            return bad(format!("the answer does not fit the kind of {:?}", q.text));
        }
        check_value(q, &a.value)?;
    }
    let g = given(answers);
    for q in &survey.questions {
        let answered = g.contains_key(&q.id);
        if !asked(q, &g) {
            if answered {
                return Err(Error::Invalid(format!("{:?} is not asked here", q.text)));
            }
        } else if complete && q.required && g.get(&q.id).is_none_or(|v| is_empty(v)) {
            return Err(Error::Invalid(format!("{:?} is required", q.text)));
        }
    }
    Ok(())
}
