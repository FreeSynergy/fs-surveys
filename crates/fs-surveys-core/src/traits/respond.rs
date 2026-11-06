//! The [`Respond`] contract: open, save, resume, submit.
//!
//! Pattern: Repository (interface) + Null Object — reads required, writes default to `Unsupported`.

use async_trait::async_trait;
use fs_store::ObjectId;

use crate::error::Result;
use crate::model::{Answer, Credential, NamedResponse, Survey};
use crate::unsupported::not_possible;

/// Answering surveys. Interface status: draft (ADR 0045).
#[async_trait]
pub trait Respond: Send + Sync {
    /// `survey.open`: the survey, if it exists and is open.
    async fn open(&self, id: ObjectId) -> Result<Option<Survey>>;
    /// `response.save`: the saved draft of a person (`named` only).
    async fn resume(&self, id: ObjectId, person: &str) -> Result<Option<NamedResponse>>;

    /// `response.save`: keep a draft (`named` only; anonymous drafts stay on the device).
    async fn save(
        &self,
        _id: ObjectId,
        _person: &str,
        _answers: Vec<Answer>,
    ) -> Result<NamedResponse> {
        not_possible("response.save")
    }
    /// `response.submit`: the credential must fit the anonymity level.
    async fn submit(
        &self,
        _id: ObjectId,
        _credential: Credential,
        _answers: Vec<Answer>,
    ) -> Result<()> {
        not_possible("response.submit")
    }
    /// `survey.publicLink`: work to do before a `public` answer.
    async fn challenge(&self, _id: ObjectId) -> Result<(Vec<u8>, u8)> {
        not_possible("survey.publicLink")
    }
}
