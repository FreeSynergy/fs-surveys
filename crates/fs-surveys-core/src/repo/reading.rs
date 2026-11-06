//! Which answers count: released batches on anonymous surveys, everything submitted on `named`.
//!
//! Pattern: Strategy by anonymity level — reading never releases anything and never writes.

use fs_store::Query;

use super::consts::RESPONSE_TYPE;
use super::handle::SurveyRepo;
use crate::error::Result;
use crate::model::{ALL_STAGES, Answer, Survey};

/// One respondent's answers; the person is known on `named` only.
pub(super) struct Row {
    pub person: Option<String>,
    pub answers: Vec<Answer>,
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
                .map(|answers| Row {
                    person: None,
                    answers: answers.clone(),
                })
                .collect();
            return Ok(Collected {
                rows,
                held_back: u32::try_from(b.held_back()).unwrap_or(u32::MAX),
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
            rows.push(Row { person, answers });
        }
        rows.sort_by(|a, b| a.person.cmp(&b.person));
        Ok(Collected { rows, held_back: 0 })
    }
}
