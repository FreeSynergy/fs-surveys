//! Summary and filter.
//!
//! Pattern: Strategy by kind (in `summarize`) — this file only picks the answers and applies the rules
//! of the level: `named` counts live, anonymous hides groups smaller than `minGroup`.

use fs_store::ObjectId;

use super::handle::SurveyRepo;
use super::reading::{Row, values_for};
use crate::check::{given, holds};
use crate::error::{Error, Result};
use crate::model::{Condition, ConditionOp, Summary, Survey};
use crate::summarize::summarize;

pub(super) fn totals(s: &Survey, rows: &[Row], held_back: u32) -> Result<Summary> {
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

impl SurveyRepo {
    /// `result.summary`.
    pub async fn summary(&self, id: ObjectId, stage: u16) -> Result<Summary> {
        let s = self.require(id).await?;
        let c = self.collect(&s, stage).await?;
        totals(&s, &c.rows, c.held_back)
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
        let mut c = self.collect(&s, stage).await?;
        c.rows.retain(|r| holds(&cond, &given(&r.answers)));
        totals(&s, &c.rows, c.held_back)
    }
}
