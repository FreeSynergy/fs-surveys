//! The [`Decisions`] contract: record what people decided and link it to tasks.
//!
//! Pattern: Repository (interface) + Null Object — reads required, writes default to `Unsupported`.

use async_trait::async_trait;
use fs_store::ObjectId;
use fs_surveys_methods::Method;

use crate::error::Result;
use crate::model::{Cursor, Decision, Page};
use crate::unsupported::not_possible;

/// Decisions. Interface status: draft (ADR 0045).
#[async_trait]
pub trait Decisions: Send + Sync {
    /// `decision.read`: one decision.
    async fn get_decision(&self, id: ObjectId) -> Result<Option<Decision>>;
    /// `decision.read`: the decisions of a survey, one page.
    async fn list_decisions(
        &self,
        survey: ObjectId,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Decision>>;
    /// `decision.read`: the decisions linked to a task IRI.
    async fn of_task(&self, task: &str) -> Result<Vec<Decision>>;

    /// `decision.record`: people decide; the outcome need not be the method's own winner.
    #[allow(clippy::too_many_arguments)]
    async fn record(
        &self,
        _survey: ObjectId,
        _question: ObjectId,
        _method: Method,
        _stage: u16,
        _outcome: Vec<String>,
        _note: &str,
    ) -> Result<Decision> {
        not_possible("decision.record")
    }
    /// `decision.link`: link a task IRI.
    async fn link_task(&self, _id: ObjectId, _task: &str) -> Result<Decision> {
        not_possible("decision.link")
    }
    /// `decision.link`: remove a task link.
    async fn unlink_task(&self, _id: ObjectId, _task: &str) -> Result<Decision> {
        not_possible("decision.link")
    }
}
