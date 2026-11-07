//! The [`Results`] contract: summary, evaluation, export, filter.
//!
//! Pattern: Repository (interface) — every method is a read, so all are required.

use async_trait::async_trait;
use fs_objects::ObjectId;
use fs_surveys_methods::Method;

use crate::error::Result;
use crate::model::{Evaluation, Summary};

/// Reading results. `stage` 0xFFFF = all stages. Interface status: draft (ADR 0045).
#[async_trait]
pub trait Results: Send + Sync {
    /// `result.summary`: totals per question.
    async fn summary(&self, id: ObjectId, stage: u16) -> Result<Summary>;
    /// `survey.method`: decide one question by one method; the same ballots suit several methods.
    async fn evaluate(
        &self,
        id: ObjectId,
        question: ObjectId,
        method: Method,
        stage: u16,
    ) -> Result<Evaluation>;
    /// `result.export`: `text/csv` only; `named` has one row per response, anonymous totals only.
    async fn export(&self, id: ObjectId, media_type: &str) -> Result<Vec<u8>>;
    /// `result.filter`: `named` only; totals of the responses that chose `choice` in `question`.
    async fn filter(
        &self,
        id: ObjectId,
        question: ObjectId,
        choice: &str,
        stage: u16,
    ) -> Result<Summary>;
}
