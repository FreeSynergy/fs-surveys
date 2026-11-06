//! Survey items: create, edit, classify, get, list, delete.
//!
//! Pattern: Repository — every write is one object put, so every edit is atomic. `delete` is a
//! cascade: named answers, invited people, decisions, the survey, then the box.

use fs_store::{ObjectId, Order, Query};

use super::consts::{CONTEXT, DECISION_TYPE, INVITEE_TYPE, RESPONSE_TYPE, SURVEY_TYPE};
use super::handle::SurveyRepo;
use crate::check::check_min_group;
use crate::error::{Error, Result};
use crate::model::{Anonymity, Cursor, DEFAULT_MIN_GROUP, Page, Survey, SurveyState};
use crate::traits::SurveyChange;

fn nonblank(name: &str) -> Result<&str> {
    let n = name.trim();
    if n.is_empty() {
        return Err(Error::Invalid("a survey needs a name".into()));
    }
    Ok(n)
}

impl SurveyRepo {
    /// `survey.create`: a new draft.
    pub async fn create(
        &self,
        name: &str,
        anonymity: Anonymity,
        scope: &[ObjectId],
    ) -> Result<Survey> {
        let s = Survey {
            envelope: self.fresh_envelope(SURVEY_TYPE, nonblank(name)?),
            state: SurveyState::Draft,
            anonymity,
            questions: vec![],
            scope: scope.to_vec(),
            stage: 0,
            min_group: DEFAULT_MIN_GROUP,
        };
        check_min_group(anonymity, s.min_group)?;
        self.objects.put(s.to_object(), &self.actor).await?;
        self.feed.notify(&SurveyChange::Created(s.id()));
        self.require(s.id()).await
    }

    /// `survey.read`: one survey, if it exists.
    pub async fn get(&self, id: ObjectId) -> Result<Option<Survey>> {
        let Some(o) = self.objects.get(id).await? else {
            return Ok(None);
        };
        if o.envelope.object_type != SURVEY_TYPE || o.envelope.context.as_deref() != Some(CONTEXT) {
            return Ok(None);
        }
        Ok(Survey::from_object(&o))
    }

    pub(super) async fn require(&self, id: ObjectId) -> Result<Survey> {
        self.get(id)
            .await?
            .ok_or(Error::NotFound { what: "survey", id })
    }

    /// Write a changed survey back and return it as stored.
    pub(super) async fn save(&self, s: &Survey) -> Result<Survey> {
        self.objects.put(s.to_object(), &self.actor).await?;
        self.feed.notify(&SurveyChange::Updated(s.id()));
        self.require(s.id()).await
    }

    /// `survey.read`: surveys in name order, one page.
    pub async fn list(&self, cursor: Option<&Cursor>) -> Result<Page<Survey>> {
        let offset = cursor.map_or(0, Cursor::offset);
        let q = Query::new()
            .of_type(SURVEY_TYPE)
            .in_context(CONTEXT)
            .ordered(Order::NameAsc)
            .offset(offset)
            .limit(self.page_size + 1);
        let mut items: Vec<Survey> = self
            .reads
            .query(&q)
            .await?
            .iter()
            .filter_map(Survey::from_object)
            .collect();
        let next = (items.len() > self.page_size).then(|| Cursor::at(offset + self.page_size));
        items.truncate(self.page_size);
        Ok(Page { items, next })
    }

    /// `survey.edit`, `survey.anonymity`: name, anonymity, `minGroup`; draft only.
    pub async fn edit(
        &self,
        id: ObjectId,
        name: &str,
        anonymity: Anonymity,
        min_group: u16,
    ) -> Result<Survey> {
        let _guard = self.lock_for(id).await;
        let mut s = self.require(id).await?;
        Self::need_draft(&s)?;
        check_min_group(anonymity, min_group)?;
        s.envelope.name = nonblank(name)?.into();
        s.anonymity = anonymity;
        s.min_group = min_group;
        self.save(&s).await
    }

    /// `survey.classify`: DPV categories. An unencrypted store refuses them; the error is passed on.
    pub async fn classify(&self, id: ObjectId, categories: &[String]) -> Result<Survey> {
        let _guard = self.lock_for(id).await;
        let mut s = self.require(id).await?;
        s.envelope.sensitive = categories.to_vec();
        let saved = self.save(&s).await?;
        self.reclassify_box(&saved).await?;
        Ok(saved)
    }

    pub(super) fn need_draft(s: &Survey) -> Result<()> {
        if s.state == SurveyState::Draft {
            return Ok(());
        }
        Err(Error::Refused(format!(
            "only a draft can be changed, this one is {:?}",
            s.state
        )))
    }

    /// Remove the survey, its named answers, invited people, decisions and its box. The box goes
    /// first and the survey last, so a failed delete can be repeated.
    pub async fn delete(&self, id: ObjectId) -> Result<()> {
        let _guard = self.lock_for(id).await;
        self.require(id).await?;
        self.delete_box(id).await?;
        for kind in [RESPONSE_TYPE, INVITEE_TYPE, DECISION_TYPE] {
            let q = Query::new()
                .of_type(kind)
                .with_index("survey", id.to_string());
            for o in self.reads.query(&q).await? {
                self.objects.delete(o.envelope.id, &self.actor).await?;
            }
        }
        self.objects.delete(id, &self.actor).await?;
        self.feed.notify(&SurveyChange::Deleted(id));
        Ok(())
    }
}
