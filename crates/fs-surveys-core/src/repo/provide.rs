//! `SurveyRepo` as a provider of the traits.
//!
//! Pattern: Adapter — each trait method calls the inherent method of the same concern, so callers
//! with and without the trait agree.

use std::sync::Arc;

use async_trait::async_trait;
use fs_interfaces::Capabilities;
use fs_store::ObjectId;

use super::handle::SurveyRepo;
use crate::error::Result;
use crate::model::{Anonymity, Condition, Cursor, InviteReport, Page, Question, Survey};
use crate::traits::{Designer, Subscription, SurveyFeed, SurveyListener, Surveys};

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
