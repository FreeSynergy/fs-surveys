//! Shared rank and tally helpers: the one place where ranks, winners and notes are built.
//!
//! Pattern: Template method as free functions (methods supply keys, this builds the `Tally`).

use crate::{OptionResult, Tally};

/// Note used by every method when there are no ballots.
pub(crate) const NO_BALLOTS: &str = "No ballots were cast, so nothing is decided.";

/// Exact conversion for the small numbers used here (vote counts and score sums).
#[allow(clippy::cast_precision_loss)]
pub(crate) fn to_f64(v: i64) -> f64 {
    v as f64
}

/// Converts a count to `i64`, saturating (counts never get near the limit).
pub(crate) fn count(v: usize) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}

/// Options sorted by `keys` (higher is better), then by position; equal keys share a rank.
pub(crate) fn options(choices: &[String], keys: &[i64], values: &[f64]) -> Vec<OptionResult> {
    let mut order: Vec<usize> = (0..choices.len()).collect();
    order.sort_by(|&a, &b| keys[b].cmp(&keys[a]).then(a.cmp(&b)));
    let mut out: Vec<OptionResult> = Vec::with_capacity(order.len());
    let mut rank = 1_u16;
    for (pos, &i) in order.iter().enumerate() {
        if pos > 0 && keys[order[pos - 1]] != keys[i] {
            rank = u16::try_from(pos + 1).unwrap_or(u16::MAX);
        }
        out.push(OptionResult {
            choice: choices[i].clone(),
            value: values[i],
            rank,
        });
    }
    out
}

/// Positions holding the highest key, in order.
pub(crate) fn top(keys: &[i64]) -> Vec<usize> {
    let best = keys.iter().copied().max();
    keys.iter()
        .enumerate()
        .filter(|&(_, &k)| Some(k) == best)
        .map(|(i, _)| i)
        .collect()
}

/// A tally from one total per choice; all choices with the best total win.
/// `higher_is_better` false means the lowest total wins.
pub(crate) fn from_totals(choices: &[String], totals: &[i64], higher_is_better: bool) -> Tally {
    let keys: Vec<i64> = totals
        .iter()
        .map(|&t| if higher_is_better { t } else { -t })
        .collect();
    let values: Vec<f64> = totals.iter().map(|&t| to_f64(t)).collect();
    let winners: Vec<String> = top(&keys).into_iter().map(|i| choices[i].clone()).collect();
    let note = if winners.len() > 1 {
        "Several choices are tied for the best result; all of them are listed as winners.".into()
    } else {
        String::new()
    };
    Tally {
        winners,
        options: options(choices, &keys, &values),
        note,
    }
}

/// A tally with no winner and the given note; `keys`/`values` are as for [`options`].
pub(crate) fn undecided(choices: &[String], keys: &[i64], values: &[f64], note: &str) -> Tally {
    Tally {
        winners: Vec::new(),
        options: options(choices, keys, values),
        note: note.into(),
    }
}

/// The tally for "no ballots".
pub(crate) fn no_ballots(choices: &[String]) -> Tally {
    let zeros = vec![0_i64; choices.len()];
    undecided(choices, &zeros, &vec![0.0; choices.len()], NO_BALLOTS)
}

/// The vote count behind a `value` (values of counting methods are whole numbers).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn as_votes(value: f64) -> u64 {
    value.max(0.0) as u64
}

/// Votes of the best choice and its lead over the next one (`None` without a runner-up).
pub(crate) fn top_and_lead(tally: &Tally) -> (u64, Option<u64>) {
    let top = tally.options.first().map_or(0, |o| as_votes(o.value));
    let lead = tally
        .options
        .get(1)
        .map(|o| top.saturating_sub(as_votes(o.value)));
    (top, lead)
}

/// Shared bound for plurality and approval: each held ballot adds at most 1 to any choice.
pub(crate) fn lead_can_change(tally: &Tally, held_back: usize) -> bool {
    if tally.winners.is_empty() {
        return held_back > 0;
    }
    match top_and_lead(tally).1 {
        Some(lead) => lead <= held_back as u64,
        None => false,
    }
}
