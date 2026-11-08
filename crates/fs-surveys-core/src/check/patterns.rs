//! Patterns of open answers: pure counting rules.
//!
//! Pattern: Specification — predicates over one ballot and its meta data, and one over a set of ballots.
//! They only count. Nothing here decides that an answer is dropped: the result shows the counts beside
//! the figures and the reader judges.

use std::collections::HashMap;

use crate::model::{Answer, AnswerValue, OpenMeta, Signals};

/// A person needs at least this long for the form, plus [`MS_PER_ANSWER`] for each answer.
pub const MS_BASE: u32 = 1500;
/// Reading and choosing one answer takes at least this long.
pub const MS_PER_ANSWER: u32 = 600;
/// More open ballots than this in one minute is a burst.
pub const BURST_LIMIT: usize = 10;
/// Shorter free text is not compared (short answers repeat by chance).
pub const MIN_SAME_TEXT: usize = 12;
/// Choice or scale answers needed in a row for the straight-line pattern.
pub const MIN_STRAIGHT: usize = 3;

/// The counts of one ballot (`ballots` is 1). `burst` says the caller saw a burst.
pub fn signals_of(answers: &[Answer], meta: &OpenMeta, burst: bool) -> Signals {
    let mut s = Signals {
        ballots: 1,
        ..Signals::default()
    };
    match meta.elapsed_ms {
        None => s.no_timing = 1,
        Some(ms) => {
            let need = MS_BASE.saturating_add(
                MS_PER_ANSWER.saturating_mul(u32::try_from(answers.len()).unwrap_or(u32::MAX)),
            );
            s.too_fast = u32::from(ms < need);
        }
    }
    s.honeypot = u32::from(
        meta.honeypot
            .as_deref()
            .is_some_and(|t| !t.trim().is_empty()),
    );
    s.straight_line = u32::from(straight_line(answers));
    s.burst = u32::from(burst);
    s
}

/// The same choice or scale value in every one of at least [`MIN_STRAIGHT`] such answers.
fn straight_line(answers: &[Answer]) -> bool {
    let picks: Vec<String> = answers
        .iter()
        .filter_map(|a| match &a.value {
            AnswerValue::Choice(c) => Some(c.clone()),
            AnswerValue::Scale(n) => Some(n.to_string()),
            _ => None,
        })
        .collect();
    picks.len() >= MIN_STRAIGHT && picks.windows(2).all(|w| w[0] == w[1])
}

fn normal(t: &str) -> String {
    t.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// How many ballots hold a free text (of at least [`MIN_SAME_TEXT`] characters) that another ballot
/// also holds.
pub fn same_text<'a>(ballots: impl IntoIterator<Item = &'a [Answer]>) -> u32 {
    let per_ballot: Vec<Vec<String>> = ballots
        .into_iter()
        .map(|b| {
            let mut texts: Vec<String> = b
                .iter()
                .filter_map(|a| match &a.value {
                    AnswerValue::Text(t) => Some(normal(t)),
                    _ => None,
                })
                .filter(|t| t.chars().count() >= MIN_SAME_TEXT)
                .collect();
            texts.sort();
            texts.dedup();
            texts
        })
        .collect();
    let mut seen: HashMap<&str, u32> = HashMap::new();
    for t in per_ballot.iter().flatten() {
        *seen.entry(t.as_str()).or_default() += 1;
    }
    let n = per_ballot
        .iter()
        .filter(|b| {
            b.iter()
                .any(|t| seen.get(t.as_str()).copied().unwrap_or(0) > 1)
        })
        .count();
    u32::try_from(n).unwrap_or(u32::MAX)
}
