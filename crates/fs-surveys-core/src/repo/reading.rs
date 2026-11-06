//! Which answers count: released batches on anonymous surveys, everything submitted on `named`.
//!
//! Pattern: Strategy by anonymity level — reading never releases anything and never writes.

use fs_store::{ObjectId, Query};

use super::consts::RESPONSE_TYPE;
use super::handle::SurveyRepo;
use crate::error::Result;
use crate::model::{ALL_STAGES, Answer, AnswerValue, Survey, SurveyState};

/// One respondent's answers; the person is known on `named` only.
pub(super) struct Row {
    pub person: Option<String>,
    pub answers: Vec<Answer>,
    /// The released batch it came in; named answers are all batch 0.
    pub batch: usize,
}

/// The answers that count, and how many are held back.
pub(super) struct Collected {
    pub rows: Vec<Row>,
    pub held_back: u32,
}

impl SurveyRepo {
    /// The counting answers of `s` for `stage` (`0xFFFF` = all).
    pub(super) async fn collect(&self, s: &Survey, stage: u16) -> Result<Collected> {
        if s.is_anonymous() {
            let b = self.read_box(s.id()).await?;
            let rows = b
                .released(stage)
                .into_iter()
                .map(|(batch, answers)| Row {
                    person: None,
                    answers: answers.clone(),
                    batch,
                })
                .collect();
            // While the survey is open the number would tick with every submit and show when
            // people answered; it is real once the survey is closed.
            let held = if s.state == SurveyState::Open {
                0
            } else {
                b.held_back()
            };
            return Ok(Collected {
                rows,
                held_back: u32::try_from(held).unwrap_or(u32::MAX),
            });
        }
        let q = Query::new()
            .of_type(RESPONSE_TYPE)
            .with_index("survey", s.id().to_string())
            .with_index("submitted", "true");
        let mut rows = vec![];
        for o in self.reads.query(&q).await? {
            let at = o
                .index
                .get("stage")
                .and_then(|v| v.first())
                .and_then(|v| v.parse::<u16>().ok());
            if stage != ALL_STAGES && at != Some(stage) {
                continue;
            }
            let person = o
                .content
                .get("fs:person")
                .and_then(|p| p.as_str())
                .map(String::from);
            let answers =
                serde_json::from_value(o.content.get("fs:answers").cloned().unwrap_or_default())?;
            rows.push(Row {
                person,
                answers,
                batch: 0,
            });
        }
        rows.sort_by(|a, b| a.person.cmp(&b.person));
        Ok(Collected { rows, held_back: 0 })
    }
}

/// The values given to `question`. On an anonymous survey a batch counts only if it holds at
/// least `minGroup` answers to this question, or a conditional question would show a single
/// answer through the difference between two readings.
pub(super) fn values_for<'a>(
    s: &Survey,
    rows: &'a [Row],
    question: ObjectId,
) -> Vec<&'a AnswerValue> {
    let mut by_batch: std::collections::BTreeMap<usize, Vec<&AnswerValue>> = Default::default();
    for r in rows {
        if let Some(a) = r.answers.iter().find(|a| a.question == question) {
            by_batch.entry(r.batch).or_default().push(&a.value);
        }
    }
    let enough = |n: usize| !s.is_anonymous() || n >= usize::from(s.min_group);
    by_batch
        .into_values()
        .filter(|v| enough(v.len()))
        .flatten()
        .collect()
}
