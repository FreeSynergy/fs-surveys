//! Inviting people and groups, and handing out tokens.
//!
//! Pattern: Strategy (ports) — groups resolve through `GroupLookup`, tokens leave through `TokenDelivery`.
//! A token is delivered first; only then is its hash kept, in the box, apart from the person who
//! is recorded in the main store.

use fs_interfaces::Unsupported;
use fs_store::{ObjectId, Query, StoredObject};
use serde_json::json;

use super::consts::INVITEE_TYPE;
use super::envelope::context;
use super::handle::SurveyRepo;
use super::inflight::Mark;
use crate::error::{Error, Result};
use crate::model::{AS2_TYPE, Anonymity, InviteReport, Survey, SurveyState};
use crate::random::token;
use crate::tokens::hash_token;

impl SurveyRepo {
    async fn is_invited(&self, survey: ObjectId, person: &str) -> Result<bool> {
        let q = Query::new()
            .of_type(INVITEE_TYPE)
            .with_index("survey", survey.to_string())
            .with_index("person", person);
        Ok(self.reads.count(&q).await? > 0)
    }

    async fn record_invitee(&self, survey: ObjectId, person: &str) -> Result<()> {
        let o = StoredObject::new(
            self.fresh_envelope(INVITEE_TYPE, person),
            AS2_TYPE,
            json!({"@context": context(), "type": INVITEE_TYPE, "fs:person": person}),
        )
        .with_index("survey", survey.to_string())
        .with_index("person", person);
        self.objects.put(o, &self.actor).await?;
        Ok(())
    }

    async fn people_of(&self, people: &[String], groups: &[ObjectId]) -> Result<Vec<String>> {
        let mut all = people.to_vec();
        if !groups.is_empty() {
            let lookup = self.groups.as_ref().ok_or_else(|| {
                Unsupported::new("survey.invite", "no group lookup is configured")
            })?;
            for g in groups {
                all.extend(lookup.members(*g).await?);
            }
        }
        let mut seen = std::collections::HashSet::new();
        all.retain(|p| !p.is_empty() && seen.insert(p.clone()));
        Ok(all)
    }

    /// Deliver a fresh token to `person` and return its hash. Runs outside the survey lock and
    /// keeps nothing: the hash is stored by the caller, after the delivery succeeded.
    async fn deliver(&self, survey: ObjectId, person: &str) -> Result<String> {
        let delivery = self
            .delivery
            .as_ref()
            .ok_or_else(|| Unsupported::new("survey.tokens", "no token delivery is configured"))?;
        let t = token()?;
        delivery
            .send(person, survey, &t)
            .await
            .map_err(|e| match e {
                Error::Delivery { .. } => e,
                other => Error::Delivery {
                    person: person.into(),
                    reason: other.to_string(),
                },
            })?;
        Ok(hash_token(&t))
    }

    fn check_invitable(&self, s: &Survey) -> Result<()> {
        if s.anonymity == Anonymity::Public {
            return Err(
                Unsupported::new("survey.publicLink", "open links are not built yet").into(),
            );
        }
        if s.state != SurveyState::Open {
            return Err(Error::Refused(format!(
                "the survey is {:?}, not open",
                s.state
            )));
        }
        if s.anonymity == Anonymity::AnonymousOnce && self.delivery.is_none() {
            return Err(
                Unsupported::new("survey.tokens", "no token delivery is configured").into(),
            );
        }
        Ok(())
    }

    /// Under the lock: is this person new, and may this caller deliver to them? `None` = already
    /// invited, or another call is delivering to them right now.
    async fn claim(&self, id: ObjectId, person: &str) -> Result<Option<Mark>> {
        let _guard = self.lock_for(id).await;
        self.check_invitable(&self.require(id).await?)?;
        if self.is_invited(id, person).await? {
            return Ok(None);
        }
        Ok(Mark::claim(&self.inflight, id, person))
    }

    /// Under the lock again: keep the hash (if any), then record the person.
    async fn settle(&self, id: ObjectId, person: &str, hash: Option<String>) -> Result<()> {
        let _guard = self.lock_for(id).await;
        let s = self.require(id).await?;
        self.check_invitable(&s)?;
        if let Some(h) = &hash {
            self.edit_box(&s, |b| {
                b.add_token(h.clone());
                Ok(())
            })
            .await?;
        }
        if let Err(e) = self.record_invitee(id, person).await {
            if let Some(h) = hash {
                self.edit_box(&s, |b| {
                    b.spend(&h);
                    Ok(())
                })
                .await?;
            }
            return Err(e);
        }
        Ok(())
    }

    /// `survey.invite`, `survey.tokens`: stops at the first person whose delivery fails; people
    /// before that stay invited (a second call lists them under `already`). The tokens are
    /// delivered outside the survey lock, so a slow messenger does not hold up the survey.
    pub async fn invite(
        &self,
        id: ObjectId,
        people: &[String],
        groups: &[ObjectId],
    ) -> Result<InviteReport> {
        {
            let _guard = self.lock_for(id).await;
            self.check_invitable(&self.require(id).await?)?;
        }
        let once = self.require(id).await?.anonymity == Anonymity::AnonymousOnce;
        let mut report = InviteReport::default();
        for person in self.people_of(people, groups).await? {
            let Some(mark) = self.claim(id, &person).await? else {
                report.already.push(person);
                continue;
            };
            let hash = if once {
                Some(self.deliver(id, &person).await?)
            } else {
                None
            };
            self.settle(id, &person, hash).await?;
            drop(mark);
            report.invited.push(person);
        }
        Ok(report)
    }
}
