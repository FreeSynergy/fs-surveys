//! Publish, close, reopen, archive: the steps of the life cycle.
//!
//! Pattern: State — one `step` runs the transition function, then the side effect that belongs to
//! it: publish checks the design, close releases the anonymous batch.

use fs_store::ObjectId;

use super::handle::SurveyRepo;
use crate::check::check_publishable;
use crate::error::Result;
use crate::model::Survey;
use crate::state::{Step, transition};
use crate::traits::SurveyChange;

impl SurveyRepo {
    async fn step(&self, id: ObjectId, step: Step) -> Result<Survey> {
        let _guard = self.lock_for(id).await;
        let mut s = self.require(id).await?;
        let next = transition(s.state, step)?;
        if step == Step::Publish {
            check_publishable(&s)?;
        }
        if step == Step::Close {
            // Release first, save after: a failed save is retried and releases again by the same rule.
            self.release_box(&s).await?;
        }
        s.state = next;
        let saved = self.save(&s).await?;
        self.feed.notify(&SurveyChange::State(id, next));
        Ok(saved)
    }

    /// `survey.publish`: draft to open; needs a question, and 2 choices per choice question.
    pub async fn publish(&self, id: ObjectId) -> Result<Survey> {
        self.step(id, Step::Publish).await
    }

    /// `survey.close`: open to closed; anonymous ballots are released when there are enough.
    pub async fn close(&self, id: ObjectId) -> Result<Survey> {
        self.step(id, Step::Close).await
    }

    /// `survey.close`: closed to open.
    pub async fn reopen(&self, id: ObjectId) -> Result<Survey> {
        self.step(id, Step::Reopen).await
    }

    /// `survey.close`: closed to archived.
    pub async fn archive(&self, id: ObjectId) -> Result<Survey> {
        self.step(id, Step::Archive).await
    }
}
