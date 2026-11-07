//! `SurveyRepo` as a provider of `Respond`, `Results` and `Decisions`.
//!
//! Pattern: Adapter — each trait method calls the inherent method of the same concern.

use async_trait::async_trait;
use fs_objects::ObjectId;
use fs_surveys_methods::Method;

use super::handle::SurveyRepo;
use crate::error::Result;
use crate::model::{
    Answer, Credential, Cursor, Decision, Evaluation, NamedResponse, Page, Summary, Survey,
};
use crate::traits::{Decisions, Respond, Results};
use crate::unsupported::not_possible;

#[async_trait]
impl Respond for SurveyRepo {
    async fn open(&self, id: ObjectId) -> Result<Option<Survey>> {
        Ok(SurveyRepo::get(self, id)
            .await?
            .filter(|s| s.state == crate::model::SurveyState::Open))
    }
    async fn resume(&self, id: ObjectId, person: &str) -> Result<Option<NamedResponse>> {
        SurveyRepo::resume(self, id, person).await
    }
    async fn save(
        &self,
        id: ObjectId,
        person: &str,
        answers: Vec<Answer>,
    ) -> Result<NamedResponse> {
        self.save_response(id, person, answers).await
    }
    async fn submit(
        &self,
        id: ObjectId,
        credential: Credential,
        answers: Vec<Answer>,
    ) -> Result<()> {
        SurveyRepo::submit(self, id, credential, answers).await
    }
    async fn challenge(&self, _id: ObjectId) -> Result<(Vec<u8>, u8)> {
        not_possible("survey.publicLink")
    }
}

#[async_trait]
impl Results for SurveyRepo {
    async fn summary(&self, id: ObjectId, stage: u16) -> Result<Summary> {
        SurveyRepo::summary(self, id, stage).await
    }
    async fn evaluate(
        &self,
        id: ObjectId,
        question: ObjectId,
        method: Method,
        stage: u16,
    ) -> Result<Evaluation> {
        SurveyRepo::evaluate(self, id, question, method, stage).await
    }
    async fn export(&self, id: ObjectId, media_type: &str) -> Result<Vec<u8>> {
        SurveyRepo::export(self, id, media_type).await
    }
    async fn filter(
        &self,
        id: ObjectId,
        question: ObjectId,
        choice: &str,
        stage: u16,
    ) -> Result<Summary> {
        SurveyRepo::filter(self, id, question, choice, stage).await
    }
}

#[async_trait]
impl Decisions for SurveyRepo {
    async fn get_decision(&self, id: ObjectId) -> Result<Option<Decision>> {
        SurveyRepo::get_decision(self, id).await
    }
    async fn list_decisions(
        &self,
        survey: ObjectId,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Decision>> {
        SurveyRepo::list_decisions(self, survey, cursor).await
    }
    async fn of_task(&self, task: &str) -> Result<Vec<Decision>> {
        SurveyRepo::of_task(self, task).await
    }
    async fn record(
        &self,
        survey: ObjectId,
        question: ObjectId,
        method: Method,
        stage: u16,
        outcome: Vec<String>,
        note: &str,
    ) -> Result<Decision> {
        SurveyRepo::record(self, survey, question, method, stage, outcome, note).await
    }
    async fn link_task(&self, id: ObjectId, task: &str) -> Result<Decision> {
        SurveyRepo::link_task(self, id, task).await
    }
    async fn unlink_task(&self, id: ObjectId, task: &str) -> Result<Decision> {
        SurveyRepo::unlink_task(self, id, task).await
    }
}
