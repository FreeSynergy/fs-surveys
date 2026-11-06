//! Reading ballots into choice positions, with all validation in one place.
//!
//! Pattern: Adapter (turns public ballots into index vectors for the counting code).

use crate::{Ballot, Error, Result};

/// Position of `id` in `choices`.
fn index(choices: &[String], id: &str) -> Result<usize> {
    choices
        .iter()
        .position(|c| c == id)
        .ok_or(Error::UnknownChoice)
}

/// Positions of `ids`; unknown ids and duplicates are errors.
fn resolve<'a>(choices: &[String], ids: impl Iterator<Item = &'a String>) -> Result<Vec<usize>> {
    let mut seen = vec![false; choices.len()];
    let mut out = Vec::new();
    for id in ids {
        let i = index(choices, id)?;
        if std::mem::replace(&mut seen[i], true) {
            return Err(Error::InvalidBallot("a choice appears twice".into()));
        }
        out.push(i);
    }
    Ok(out)
}

fn wrong(method: &str, ballot: &Ballot) -> Error {
    Error::WrongBallot {
        method: method.into(),
        ballot: ballot.kind().into(),
    }
}

/// One position per `Single` ballot.
pub(crate) fn singles(method: &str, choices: &[String], ballots: &[Ballot]) -> Result<Vec<usize>> {
    ballots
        .iter()
        .map(|b| match b {
            Ballot::Single(id) => index(choices, id),
            other => Err(wrong(method, other)),
        })
        .collect()
}

/// Positions per `Multiple` (approval sets) or `Ranking` (best first) ballot.
pub(crate) fn lists(
    method: &str,
    choices: &[String],
    ballots: &[Ballot],
    ranking: bool,
) -> Result<Vec<Vec<usize>>> {
    ballots
        .iter()
        .map(|b| match (b, ranking) {
            (Ballot::Multiple(ids), false) | (Ballot::Ranking(ids), true) => {
                resolve(choices, ids.iter())
            }
            (other, _) => Err(wrong(method, other)),
        })
        .collect()
}

/// `(position, score)` pairs per `Scores` ballot, scores checked against `range`.
pub(crate) fn scores(
    method: &str,
    choices: &[String],
    ballots: &[Ballot],
    range: Option<(i32, i32)>,
) -> Result<Vec<Vec<(usize, i32)>>> {
    ballots
        .iter()
        .map(|b| {
            let Ballot::Scores(pairs) = b else {
                return Err(wrong(method, b));
            };
            let idx = resolve(choices, pairs.iter().map(|(id, _)| id))?;
            if let Some((lo, hi)) = range {
                if pairs.iter().any(|(_, s)| *s < lo || *s > hi) {
                    return Err(Error::InvalidBallot(format!(
                        "a score is outside {lo}..={hi}"
                    )));
                }
            }
            Ok(idx.into_iter().zip(pairs.iter().map(|(_, s)| *s)).collect())
        })
        .collect()
}

/// Fails with `NoChoices` when there is nothing to decide between.
pub(crate) fn need_choices(choices: &[String]) -> Result<()> {
    if choices.is_empty() {
        Err(Error::NoChoices)
    } else {
        Ok(())
    }
}
