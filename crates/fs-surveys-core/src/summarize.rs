//! Totals for one question from a list of answers.
//!
//! Pattern: Strategy by question kind (a `match`), pure — no storage. Anonymous results show choice
//! and scale counts and means; number and date only as a mean; free text shuffled.

use std::collections::BTreeMap;

use crate::error::Result;
use crate::model::{AnswerValue, ChoiceCount, Question, QuestionKind, QuestionSummary};
use crate::random::shuffle;

fn count(choice: String, count: u32) -> ChoiceCount {
    ChoiceCount { choice, count }
}

fn choice_counts(q: &Question, values: &[&AnswerValue]) -> Vec<ChoiceCount> {
    let mut by: BTreeMap<&str, u32> = BTreeMap::new();
    for v in values {
        let picked: Vec<&String> = match v {
            AnswerValue::Choice(c) => vec![c],
            AnswerValue::Choices(l) => l.iter().collect(),
            AnswerValue::Ranking(l) => l.first().into_iter().collect(),
            // Number of answers that scored the choice, not the points.
            AnswerValue::Scores(l) => l.iter().map(|s| &s.choice).collect(),
            _ => vec![],
        };
        for c in picked {
            *by.entry(c.as_str()).or_default() += 1;
        }
    }
    q.choices
        .iter()
        .map(|c| count(c.id.clone(), by.get(c.id.as_str()).copied().unwrap_or(0)))
        .collect()
}

fn text_counts<T: Ord + ToString>(items: Vec<T>) -> Vec<ChoiceCount> {
    let mut by: BTreeMap<T, u32> = BTreeMap::new();
    for i in items {
        *by.entry(i).or_default() += 1;
    }
    by.into_iter()
        .map(|(k, n)| count(k.to_string(), n))
        .collect()
}

fn days(d: &str) -> Option<f64> {
    use chrono::Datelike;
    let d = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()?;
    Some(f64::from(d.num_days_from_ce() - 719_163))
}

fn mean(nums: &[f64]) -> Option<f64> {
    #[allow(clippy::cast_precision_loss)]
    (!nums.is_empty()).then(|| nums.iter().sum::<f64>() / nums.len() as f64)
}

/// Summarise `values` (one per respondent who answered). `show_values` is true on `named` surveys,
/// where number and date values may be listed; `min_group` hides small anonymous groups.
pub fn summarize(
    q: &Question,
    values: &[&AnswerValue],
    anonymous: bool,
    min_group: u16,
) -> Result<QuestionSummary> {
    let answers = u32::try_from(values.len()).unwrap_or(u32::MAX);
    let mut s = QuestionSummary {
        question: q.id,
        answers,
        hidden: anonymous && values.len() < usize::from(min_group),
        counts: vec![],
        mean: None,
        texts: vec![],
    };
    if s.hidden {
        return Ok(s);
    }
    match q.kind {
        QuestionKind::Text => {
            s.texts = values
                .iter()
                .filter_map(|v| {
                    if let AnswerValue::Text(t) = v {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
                .collect();
            shuffle(&mut s.texts)?;
        }
        QuestionKind::Scale => {
            let n: Vec<i32> = values
                .iter()
                .filter_map(|v| {
                    if let AnswerValue::Scale(x) = v {
                        Some(*x)
                    } else {
                        None
                    }
                })
                .collect();
            s.mean = mean(&n.iter().map(|x| f64::from(*x)).collect::<Vec<_>>());
            s.counts = text_counts(n);
        }
        QuestionKind::Number => {
            let n: Vec<f64> = values
                .iter()
                .filter_map(|v| {
                    if let AnswerValue::Number(x) = v {
                        Some(*x)
                    } else {
                        None
                    }
                })
                .collect();
            s.mean = mean(&n);
            if !anonymous {
                s.counts = text_counts(n.iter().map(ToString::to_string).collect());
            }
        }
        QuestionKind::Date => {
            let d: Vec<&String> = values
                .iter()
                .filter_map(|v| {
                    if let AnswerValue::Date(x) = v {
                        Some(x)
                    } else {
                        None
                    }
                })
                .collect();
            s.mean = mean(&d.iter().filter_map(|x| days(x)).collect::<Vec<_>>());
            if !anonymous {
                s.counts = text_counts(d.into_iter().cloned().collect());
            }
        }
        _ => s.counts = choice_counts(q, values),
    }
    Ok(s)
}
