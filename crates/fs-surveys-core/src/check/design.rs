//! Checks on the design of a survey: questions, conditions, `minGroup`, publishing.
//!
//! Pattern: Specification — small predicates that return the reason for a refusal.

use std::collections::HashSet;

use crate::error::{Error, Result};
use crate::model::{Anonymity, ConditionOp, MIN_GROUP_FLOOR, Question, QuestionKind, Survey};

fn bad<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Invalid(msg.into()))
}

/// Anonymous surveys need a `minGroup` of at least 3.
pub fn check_min_group(anonymity: Anonymity, min_group: u16) -> Result<()> {
    if anonymity != Anonymity::Named && min_group < MIN_GROUP_FLOOR {
        return Err(Error::Refused(format!(
            "minGroup {min_group} is below {MIN_GROUP_FLOOR}, the least an anonymous survey may have"
        )));
    }
    Ok(())
}

/// The shape of one question: text, unique choice ids, bounds.
pub fn check_question(q: &Question) -> Result<()> {
    if q.text.trim().is_empty() {
        return bad("a question needs a text");
    }
    let mut seen = HashSet::new();
    for c in &q.choices {
        if c.id.is_empty() || !seen.insert(c.id.as_str()) {
            return bad(format!("choice id {:?} is empty or used twice", c.id));
        }
    }
    if !q.kind.has_choices() && !q.choices.is_empty() {
        return bad("only choice questions have choices");
    }
    let scored = matches!(q.kind, QuestionKind::Scale | QuestionKind::OptionScores);
    if scored && q.min > q.max {
        return bad("min is above max");
    }
    if let (Some(a), Some(b)) = (q.min_number, q.max_number) {
        if a > b {
            return bad("minNumber is above maxNumber");
        }
    }
    Ok(())
}

/// Every condition names an earlier question and a value that fits it; so there are no cycles.
pub fn check_conditions(questions: &[Question]) -> Result<()> {
    for (i, q) in questions.iter().enumerate() {
        for c in &q.conditions {
            let Some(target) = questions[..i].iter().find(|t| t.id == c.question) else {
                return Err(Error::Refused(format!(
                    "a condition of {:?} must refer to an earlier question",
                    q.text
                )));
            };
            match c.op {
                ConditionOp::Answered => {}
                ConditionOp::AtLeast | ConditionOp::AtMost => {
                    if c.value.parse::<f64>().is_err() {
                        return bad(format!("{:?} is not a number", c.value));
                    }
                }
                ConditionOp::Equals | ConditionOp::Includes => {
                    let known = target.choices.iter().any(|ch| ch.id == c.value);
                    if target.kind.has_choices() && !known {
                        return bad(format!(
                            "{:?} is not a choice of {:?}",
                            c.value, target.text
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// A draft may be published: one question at least, two choices for choice questions.
pub fn check_publishable(s: &Survey) -> Result<()> {
    if s.anonymity == Anonymity::Public {
        return Err(fs_interfaces::Unsupported::new(
            "survey.publicLink",
            "a public survey could not be answered yet; open links are not built",
        )
        .into());
    }
    if s.questions.is_empty() {
        return Err(Error::Refused(
            "a survey needs at least one question".into(),
        ));
    }
    check_min_group(s.anonymity, s.min_group)?;
    for q in &s.questions {
        check_question(q)?;
        if q.kind.has_choices() && q.choices.len() < 2 {
            return Err(Error::Refused(format!(
                "question {:?} needs at least 2 choices",
                q.text
            )));
        }
    }
    check_conditions(&s.questions)
}
