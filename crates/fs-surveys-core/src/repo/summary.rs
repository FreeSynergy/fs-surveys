//! Summary and filter.
//!
//! Pattern: Strategy by kind (in `summarize`) — this file only picks the answers and applies the rules
//! of the level: `named` counts live, anonymous hides groups smaller than `minGroup`.

use fs_objects::ObjectId;

use super::handle::SurveyRepo;
use super::reading::{Collected, Lanes, values_for};
use crate::check::{given, holds};
use crate::error::{Error, Result};
use crate::model::{Condition, ConditionOp, Summary, Survey};
use crate::summarize::summarize;

/// The figures of one class.
pub(super) fn totals(s: &Survey, c: &Collected) -> Result<Summary> {
    let (rows, held_back) = (&c.rows[..], c.held_back);
    let mut questions = vec![];
    for q in &s.questions {
        let (values, _) = values_for(s, rows, q.id);
        questions.push(summarize(q, &values, s.is_anonymous(), s.min_group)?);
    }
    let ballots = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    Ok(Summary {
        ballots,
        hidden: s.is_anonymous() && rows.len() < usize::from(s.min_group),
        questions,
        held_back,
        ai: None,
    })
}

/// People first, the AI part inside it.
fn both(s: &Survey, lanes: &Lanes) -> Result<Summary> {
    let (mut people, ai) = lanes.run(|c| totals(s, c))?;
    people.ai = ai.map(Box::new);
    Ok(people)
}

impl SurveyRepo {
    /// `result.summary`: the AI part is counted apart.
    pub async fn summary(&self, id: ObjectId, stage: u16) -> Result<Summary> {
        let s = self.require(id).await?;
        both(&s, &self.collect(&s, stage).await?)
    }

    /// `result.filter`: `named` only; the responses that chose `choice` in `question`.
    pub async fn filter(
        &self,
        id: ObjectId,
        question: ObjectId,
        choice: &str,
        stage: u16,
    ) -> Result<Summary> {
        let s = self.require(id).await?;
        if s.is_anonymous() {
            return Err(Error::Refused("filtering is for named surveys only".into()));
        }
        if s.question(question).is_none() {
            return Err(Error::NotFound {
                what: "question",
                id: question,
            });
        }
        let cond = Condition {
            question,
            op: ConditionOp::Includes,
            value: choice.into(),
        };
        let mut lanes = self.collect(&s, stage).await?;
        lanes.retain(|r| holds(&cond, &given(&r.answers)));
        both(&s, &lanes)
    }
}
