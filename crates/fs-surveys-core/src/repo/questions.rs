//! Questions of a draft: add, edit, move, remove, set conditions.
//!
//! Pattern: Repository — each change rewrites the one survey object after the design rules pass, so
//! a refused change leaves the survey as it was.

use fs_objects::ObjectId;

use super::handle::SurveyRepo;
use crate::check::{check_conditions, check_question};
use crate::error::{Error, Result};
use crate::model::{Condition, Question, Survey};

impl SurveyRepo {
    /// Load a draft, change its questions, check the result, save.
    async fn change(
        &self,
        id: ObjectId,
        f: impl FnOnce(&mut Vec<Question>) -> Result<()>,
    ) -> Result<Survey> {
        let _guard = self.lock_for(id).await;
        let mut s = self.require(id).await?;
        Self::need_draft(&s)?;
        f(&mut s.questions)?;
        s.questions.iter().try_for_each(check_question)?;
        check_conditions(&s.questions)?;
        self.save(&s).await
    }

    fn at(qs: &mut [Question], id: ObjectId) -> Result<&mut Question> {
        qs.iter_mut().find(|q| q.id == id).ok_or(Error::NotFound {
            what: "question",
            id,
        })
    }

    /// `question.add`: a new id is assigned; the given one is ignored.
    pub async fn add_question(
        &self,
        id: ObjectId,
        mut question: Question,
    ) -> Result<(Survey, ObjectId)> {
        question.id = ObjectId::new();
        let qid = question.id;
        let s = self
            .change(id, |qs| {
                qs.push(question);
                Ok(())
            })
            .await?;
        Ok((s, qid))
    }

    /// `question.edit`: replace the question that has the same id.
    pub async fn edit_question(&self, id: ObjectId, question: Question) -> Result<Survey> {
        self.change(id, |qs| {
            let slot = Self::at(qs, question.id)?;
            *slot = question;
            Ok(())
        })
        .await
    }

    /// `question.move`: to `position`; a position past the end means last.
    pub async fn move_question(
        &self,
        id: ObjectId,
        question: ObjectId,
        position: u16,
    ) -> Result<Survey> {
        self.change(id, |qs| {
            let from = qs
                .iter()
                .position(|q| q.id == question)
                .ok_or(Error::NotFound {
                    what: "question",
                    id: question,
                })?;
            let q = qs.remove(from);
            qs.insert(usize::from(position).min(qs.len()), q);
            Ok(())
        })
        .await
    }

    /// `question.edit`: remove a question; refused while a condition refers to it.
    pub async fn remove_question(&self, id: ObjectId, question: ObjectId) -> Result<Survey> {
        self.change(id, |qs| {
            Self::at(qs, question)?;
            qs.retain(|q| q.id != question);
            Ok(())
        })
        .await
    }

    /// `logic.set`: conditions may refer to earlier questions only.
    pub async fn set_conditions(
        &self,
        id: ObjectId,
        question: ObjectId,
        conditions: Vec<Condition>,
    ) -> Result<Survey> {
        self.change(id, |qs| {
            Self::at(qs, question)?.conditions = conditions;
            Ok(())
        })
        .await
    }
}
