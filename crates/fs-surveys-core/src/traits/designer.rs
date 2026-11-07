//! The [`Designer`] contract: create, edit, publish, close, invite, widen.
//!
//! Pattern: Repository (interface) + Template method + Null Object — like a parent class (ADR 0045):
//! reads are required; every mutating method a provider does not override answers `Unsupported`.

use async_trait::async_trait;
use fs_store::ObjectId;

use crate::error::Result;
use crate::model::{Anonymity, Condition, Cursor, InviteReport, Page, Question, Survey};
use crate::unsupported::not_possible;

/// Designing and running surveys. Interface status: draft (ADR 0045).
#[async_trait]
pub trait Designer: Send + Sync {
    /// `Designer.get`: one survey, if it exists.
    async fn get(&self, id: ObjectId) -> Result<Option<Survey>>;
    /// `Designer.list`: surveys in name order, one page.
    async fn list(&self, cursor: Option<&Cursor>) -> Result<Page<Survey>>;

    /// `survey.create`: a new draft.
    async fn create(
        &self,
        _name: &str,
        _anonymity: Anonymity,
        _scope: &[ObjectId],
    ) -> Result<Survey> {
        not_possible("survey.create")
    }
    /// `survey.create`: like `create`, and records `creator`, the only person who can later allow AIs.
    async fn create_by(
        &self,
        _name: &str,
        _anonymity: Anonymity,
        _scope: &[ObjectId],
        _creator: &str,
    ) -> Result<Survey> {
        not_possible("survey.create")
    }
    /// `survey.aiAllowed`: allow or forbid AIs in this survey. Only the creator, only while the survey
    /// is a draft or open; allowing needs a creator who is a human by the marking. Forbidding leaves
    /// earlier AI answers where they are, apart.
    async fn set_ai_allowed(&self, _id: ObjectId, _by: &str, _allowed: bool) -> Result<Survey> {
        not_possible("survey.aiAllowed")
    }
    /// `survey.edit`, `survey.anonymity`: name, anonymity and `minGroup`; draft only.
    async fn edit(
        &self,
        _id: ObjectId,
        _name: &str,
        _anonymity: Anonymity,
        _min_group: u16,
    ) -> Result<Survey> {
        not_possible("survey.edit")
    }
    /// `survey.classify`: DPV categories of sensitive data.
    async fn classify(&self, _id: ObjectId, _categories: &[String]) -> Result<Survey> {
        not_possible("survey.classify")
    }
    /// `question.add`: the id of `question` is ignored; returns the survey and the new id.
    async fn add_question(&self, _id: ObjectId, _question: Question) -> Result<(Survey, ObjectId)> {
        not_possible("question.add")
    }
    /// `question.edit`: replace the question with the same id.
    async fn edit_question(&self, _id: ObjectId, _question: Question) -> Result<Survey> {
        not_possible("question.edit")
    }
    /// `question.move`: to `position` (counted from 0).
    async fn move_question(
        &self,
        _id: ObjectId,
        _question: ObjectId,
        _position: u16,
    ) -> Result<Survey> {
        not_possible("question.move")
    }
    /// `question.edit`: remove a question.
    async fn remove_question(&self, _id: ObjectId, _question: ObjectId) -> Result<Survey> {
        not_possible("question.edit")
    }
    /// `logic.set`: conditions may refer to earlier questions only.
    async fn set_conditions(
        &self,
        _id: ObjectId,
        _question: ObjectId,
        _conditions: Vec<Condition>,
    ) -> Result<Survey> {
        not_possible("logic.set")
    }
    /// `survey.publish`: draft to open.
    async fn publish(&self, _id: ObjectId) -> Result<Survey> {
        not_possible("survey.publish")
    }
    /// `survey.close`: open to closed; anonymous ballots are released.
    async fn close(&self, _id: ObjectId) -> Result<Survey> {
        not_possible("survey.close")
    }
    /// `survey.close`: closed to open.
    async fn reopen(&self, _id: ObjectId) -> Result<Survey> {
        not_possible("survey.close")
    }
    /// `survey.close`: closed to archived.
    async fn archive(&self, _id: ObjectId) -> Result<Survey> {
        not_possible("survey.close")
    }
    /// `survey.create`: remove the survey with everything that belongs to it.
    async fn delete(&self, _id: ObjectId) -> Result<()> {
        not_possible("survey.create")
    }
    /// `survey.invite`, `survey.tokens`: invite people and the members of groups.
    async fn invite(
        &self,
        _id: ObjectId,
        _people: &[String],
        _groups: &[ObjectId],
    ) -> Result<InviteReport> {
        not_possible("survey.invite")
    }
    /// `survey.widen`: ask the parent group too; results and anonymity stay.
    async fn widen(&self, _id: ObjectId, _parent: ObjectId) -> Result<Survey> {
        not_possible("survey.widen")
    }
}
