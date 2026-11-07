//! The class of a participant, and who may allow AIs.
//!
//! Pattern: Strategy (the injected `AiMarking`) + Specification (`check::class_of`) — this file asks
//! the strategy and applies the specification; it never reads what a participant says
//! about itself. Doubt counts as AI.

use fs_objects::ObjectId;

use super::handle::SurveyRepo;
use crate::check::class_of;
use crate::error::{Error, Result};
use crate::model::{Anonymity, Class, Survey, SurveyState};

impl SurveyRepo {
    /// The class of `person`, as the marking tells. No identity, a blank one, no marking configured or
    /// no answer: an AI. An error of the marking is passed on, never taken for a human.
    pub(super) async fn class_for(&self, person: Option<&str>) -> Result<Class> {
        let (Some(who), Some(marking)) = (
            person.map(str::trim).filter(|p| !p.is_empty()),
            &self.marking,
        ) else {
            return Ok(Class::Ai);
        };
        Ok(class_of(marking.is_ai(who).await?))
    }

    /// `survey.create` with a creator: the only person who can later allow AIs.
    pub(super) async fn make_survey(
        &self,
        name: &str,
        anonymity: Anonymity,
        scope: &[ObjectId],
        creator: &str,
    ) -> Result<Survey> {
        let creator = creator.trim();
        if creator.is_empty() {
            return Err(Error::Invalid("a survey needs a creator".into()));
        }
        self.create_with(name, anonymity, scope, Some(creator.into()))
            .await
    }

    /// `survey.aiAllowed`: only the creator, only in `draft` or `open`; allowing needs a creator who
    /// is a human by the marking, so an AI cannot let AIs in. Earlier AI answers are not touched.
    pub(super) async fn change_ai_permission(
        &self,
        id: ObjectId,
        by: &str,
        allowed: bool,
    ) -> Result<Survey> {
        let class = if allowed {
            Some(self.class_for(Some(by)).await?)
        } else {
            None
        };
        let _guard = self.lock_for(id).await;
        let mut s = self.require(id).await?;
        if s.creator
            .as_deref()
            .is_none_or(|c| c != by.trim() || c.is_empty())
        {
            return Err(Error::NotCreator);
        }
        if !matches!(s.state, SurveyState::Draft | SurveyState::Open) {
            return Err(Error::Refused(format!(
                "the permission cannot change while the survey is {:?}",
                s.state
            )));
        }
        if class == Some(Class::Ai) {
            // The creator must be a person before they can let AIs in.
            return Err(Error::AiNotAllowed);
        }
        s.ai_allowed = allowed;
        self.save(&s).await
    }
}
