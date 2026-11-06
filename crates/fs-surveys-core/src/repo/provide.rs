//! `SurveyRepo` as a provider of the traits.
//!
//! Pattern: Adapter — each trait method calls the inherent method of the same concern, so callers
//! with and without the trait agree.

use std::sync::Arc;

use async_trait::async_trait;
use fs_interfaces::Capabilities;
use fs_store::ObjectId;
use fs_surveys_methods::Method;

use super::handle::SurveyRepo;
use crate::error::Result;
use crate::model::{
    Anonymity, Answer, Condition, Credential, Cursor, Decision, Evaluation, InviteReport,
    NamedResponse, Page, Question, Summary, Survey,
};
use crate::traits::{
    Decisions, Designer, Respond, Results, Subscription, SurveyFeed, SurveyListener, Surveys,
};
use crate::unsupported::not_possible;

impl Surveys for SurveyRepo {
    fn capabilities(&self) -> Capabilities {
        self.support()
    }
}

impl SurveyFeed for SurveyRepo {
    fn subscribe(&self, listener: Arc<dyn SurveyListener>) -> Subscription {
        self.feed.add(listener)
    }
}

#[async_trait]
impl Designer for SurveyRepo {
    async fn get(&self, id: ObjectId) -> Result<Option<Survey>> {
        SurveyRepo::get(self, id).await
    }
    async fn list(&self, cursor: Option<&Cursor>) -> Result<Page<Survey>> {
        SurveyRepo::list(self, cursor).await
    }
    async fn create(&self, name: &str, anonymity: Anonymity, scope: &[ObjectId]) -> Result<Survey> {
        SurveyRepo::create(self, name, anonymity, scope).await
    }
    async fn edit(
        &self,
        id: ObjectId,
        name: &str,
        anonymity: Anonymity,
        min_group: u16,
    ) -> Result<Survey> {
        SurveyRepo::edit(self, id, name, anonymity, min_group).await
    }
    async fn classify(&self, id: ObjectId, categories: &[String]) -> Result<Survey> {
        SurveyRepo::classify(self, id, categories).await
    }
    async fn add_question(&self, id: ObjectId, question: Question) -> Result<(Survey, ObjectId)> {
        SurveyRepo::add_question(self, id, question).await
    }
    async fn edit_question(&self, id: ObjectId, question: Question) -> Result<Survey> {
        SurveyRepo::edit_question(self, id, question).await
    }
    async fn move_question(
        &self,
        id: ObjectId,
        question: ObjectId,
        position: u16,
    ) -> Result<Survey> {
        SurveyRepo::move_question(self, id, question, position).await
    }
    async fn remove_question(&self, id: ObjectId, question: ObjectId) -> Result<Survey> {
        SurveyRepo::remove_question(self, id, question).await
    }
    async fn set_conditions(
        &self,
        id: ObjectId,
        question: ObjectId,
        conditions: Vec<Condition>,
    ) -> Result<Survey> {
        SurveyRepo::set_conditions(self, id, question, conditions).await
    }
    async fn publish(&self, id: ObjectId) -> Result<Survey> {
        SurveyRepo::publish(self, id).await
    }
    async fn close(&self, id: ObjectId) -> Result<Survey> {
        SurveyRepo::close(self, id).await
    }
    async fn reopen(&self, id: ObjectId) -> Result<Survey> {
        SurveyRepo::reopen(self, id).await
    }
    async fn archive(&self, id: ObjectId) -> Result<Survey> {
        SurveyRepo::archive(self, id).await
    }
    async fn delete(&self, id: ObjectId) -> Result<()> {
        SurveyRepo::delete(self, id).await
    }
    async fn invite(
        &self,
        id: ObjectId,
        people: &[String],
        groups: &[ObjectId],
    ) -> Result<InviteReport> {
        SurveyRepo::invite(self, id, people, groups).await
    }
    async fn widen(&self, id: ObjectId, parent: ObjectId) -> Result<Survey> {
        SurveyRepo::widen(self, id, parent).await
    }
}

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
