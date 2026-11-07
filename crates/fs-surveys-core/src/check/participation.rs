//! Who may take part: the rule for AI participants.
//!
//! Pattern: Specification — two pure predicates over values (a marking answer, a survey and a class),
//! asked by submit, save, invite, token spending and the permission change alike, so no path has its
//! own copy of the rule. Doubt counts as AI (see `DESIGN.md`).

use crate::error::{Error, Result};
use crate::model::{Class, Survey};

/// The class the marking's answer stands for. **Only `Some(false)` is a human**: `Some(true)` and
/// no answer are AIs.
pub fn class_of(answer: Option<bool>) -> Class {
    if answer == Some(false) {
        Class::Human
    } else {
        Class::Ai
    }
}

/// Whether `class` may take part in `survey`: people always, AIs only if the creator allowed it.
pub fn admits(survey: &Survey, class: Class) -> Result<()> {
    if class.is_ai() && !survey.ai_allowed {
        return Err(Error::AiNotAllowed);
    }
    Ok(())
}
