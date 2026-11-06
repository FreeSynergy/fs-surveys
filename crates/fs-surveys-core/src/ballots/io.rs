//! Reading and writing the box in the ballot store.
//!
//! Pattern: Repository — one box object per survey, id = survey id, written unlogged (no activity at
//! any moment), with a fixed envelope time (Unix epoch) so no last-write time exists.

use fs_store::{Envelope, ObjectId, StoredObject};

use super::model::BallotBox;
use crate::error::Result;
use crate::model::Survey;
use crate::repo::{BOX_TYPE, SurveyRepo};

fn epoch() -> fs_store::Timestamp {
    fs_store::Timestamp::UNIX_EPOCH
}

fn object(id: ObjectId, data: &BallotBox, sensitive: &[String]) -> Result<StoredObject> {
    let mut e = Envelope::new(BOX_TYPE, "ballot box");
    e.id = id;
    e.published = epoch();
    e.updated = epoch();
    e.sensitive = sensitive.to_vec();
    Ok(StoredObject::new(
        e,
        "application/json",
        serde_json::to_value(data)?,
    ))
}

impl SurveyRepo {
    /// The box of a survey; empty if none was written yet. Reading never changes it.
    pub(crate) async fn read_box(&self, survey: ObjectId) -> Result<BallotBox> {
        match self.ballots.read_unlogged(survey).await? {
            Some(o) => Ok(serde_json::from_value(o.content)?),
            None => Ok(BallotBox::default()),
        }
    }

    /// Change the box; the caller holds the survey lock; it is written only if `f` succeeds. The box copies the
    /// sensitive categories of the survey.
    pub(crate) async fn edit_box<T>(
        &self,
        survey: &Survey,
        f: impl FnOnce(&mut BallotBox) -> Result<T>,
    ) -> Result<T> {
        let mut data = self.read_box(survey.id()).await?;
        let out = f(&mut data)?;
        self.ballots
            .put_unlogged(object(survey.id(), &data, &survey.envelope.sensitive)?)
            .await?;
        Ok(out)
    }

    /// Release pending ballots at the current stage, if enough (close and widen only).
    pub(crate) async fn release_box(&self, survey: &Survey) -> Result<()> {
        if !survey.is_anonymous() {
            return Ok(());
        }
        let (stage, min) = (survey.stage, survey.min_group);
        self.edit_box(survey, |b| b.release(stage, min).map(|_| ()))
            .await
    }

    /// Copy new sensitive categories to an existing box.
    /// Caller holds the survey lock.
    pub(crate) async fn reclassify_box(&self, survey: &Survey) -> Result<()> {
        if let Some(o) = self.ballots.read_unlogged(survey.id()).await? {
            let data: BallotBox = serde_json::from_value(o.content)?;
            let fresh = object(survey.id(), &data, &survey.envelope.sensitive)?;
            self.ballots.put_unlogged(fresh).await?;
        }
        Ok(())
    }

    /// Remove the box of a survey. Caller holds the survey lock.
    pub(crate) async fn delete_box(&self, survey: ObjectId) -> Result<()> {
        Ok(self.ballots.delete_unlogged(survey).await?)
    }
}
