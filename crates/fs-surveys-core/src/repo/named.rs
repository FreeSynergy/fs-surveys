//! Named answers: one object per person and survey in the main store.
//!
//! Pattern: Repository — `save` keeps a draft, `submit` sets `submitted`, submitting again replaces
//! the answers while the survey is open.

use fs_objects::{ObjectId, Query, StoredObject};
use serde_json::json;

use super::consts::RESPONSE_TYPE;
use super::envelope::context;
use super::handle::SurveyRepo;
use crate::check::{drop_empty, validate_answers};
use crate::error::{Error, Result};
use crate::model::{AS2_TYPE, Anonymity, Answer, NamedResponse, Survey, SurveyState};
use crate::traits::SurveyChange;

fn read(o: &StoredObject) -> Option<NamedResponse> {
    let c = &o.content;
    Some(NamedResponse {
        envelope: o.envelope.clone(),
        survey: o.index.get("survey")?.first()?.parse().ok()?,
        person: c.get("fs:person")?.as_str()?.into(),
        answers: serde_json::from_value(c.get("fs:answers")?.clone()).ok()?,
        submitted: c.get("fs:submitted")?.as_bool()?,
    })
}

impl SurveyRepo {
    async fn response_object(
        &self,
        survey: ObjectId,
        person: &str,
    ) -> Result<Option<StoredObject>> {
        let q = Query::new()
            .of_type(RESPONSE_TYPE)
            .with_index("survey", survey.to_string())
            .with_index("person", person);
        Ok(self.reads.query(&q).await?.into_iter().next())
    }

    pub(super) fn need_open_named(s: &Survey) -> Result<()> {
        if s.anonymity != Anonymity::Named {
            return Err(Error::Refused(
                "only a named survey keeps answers with their author".into(),
            ));
        }
        if s.state != SurveyState::Open {
            return Err(Error::Refused(format!(
                "the survey is {:?}, not open",
                s.state
            )));
        }
        Ok(())
    }

    /// `response.resume`: the saved or submitted response of a person.
    pub async fn resume(&self, id: ObjectId, person: &str) -> Result<Option<NamedResponse>> {
        Ok(self
            .response_object(id, person)
            .await?
            .as_ref()
            .and_then(read))
    }

    /// Write the response of `person`, replacing an earlier one; the survey is `s`.
    pub(super) async fn put_response(
        &self,
        s: &Survey,
        person: &str,
        answers: Vec<Answer>,
        submitted: bool,
    ) -> Result<NamedResponse> {
        let mut env = match self.response_object(s.id(), person).await? {
            Some(o) => o.envelope,
            None => self.fresh_envelope(RESPONSE_TYPE, &format!("Response to {}", s.envelope.name)),
        };
        env.attributed_to = Some(person.into());
        env.in_reply_to = Some(self.survey_ref(s.id()));
        let content = json!({
            "@context": context(), "type": RESPONSE_TYPE, "inReplyTo": env.in_reply_to,
            "fs:person": person, "fs:submitted": submitted, "fs:answers": answers,
        });
        let o = StoredObject::new(env, AS2_TYPE, content)
            .with_index("survey", s.id().to_string())
            .with_index("person", person)
            .with_index("submitted", submitted.to_string())
            .with_index("stage", s.stage.to_string());
        self.objects.put(o, &self.actor).await?;
        let stored = self
            .response_object(s.id(), person)
            .await?
            .as_ref()
            .and_then(read);
        stored.ok_or_else(|| Error::Invalid("response could not be read back".into()))
    }

    /// `response.save`: keep a draft; a submitted response is replaced by submitting again.
    pub async fn save_response(
        &self,
        id: ObjectId,
        person: &str,
        answers: Vec<Answer>,
    ) -> Result<NamedResponse> {
        let answers = drop_empty(answers);
        let _guard = self.lock_for(id).await;
        let s = self.require(id).await?;
        Self::need_open_named(&s)?;
        if person.is_empty() {
            return Err(Error::Invalid("a named answer needs a person".into()));
        }
        if self.resume(id, person).await?.is_some_and(|r| r.submitted) {
            return Err(Error::Refused(
                "already submitted; submit again to replace it".into(),
            ));
        }
        validate_answers(&s, &answers, false)?;
        self.put_response(&s, person, answers, false).await
    }

    pub(super) async fn submit_named(
        &self,
        s: &Survey,
        person: &str,
        answers: Vec<Answer>,
    ) -> Result<()> {
        if person.is_empty() {
            return Err(Error::Invalid("a named answer needs a person".into()));
        }
        validate_answers(s, &answers, true)?;
        let r = self.put_response(s, person, answers, true).await?;
        self.feed
            .notify(&SurveyChange::Answered(s.id(), r.envelope.id));
        Ok(())
    }
}
