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

    /// Deliver a fresh token, then keep its hash. Returns the hash, to undo it if need be.
    async fn issue_token(&self, s: &Survey, person: &str) -> Result<String> {
        let delivery = self
            .delivery
            .as_ref()
            .ok_or_else(|| Unsupported::new("survey.tokens", "no token delivery is configured"))?;
        let t = token()?;
        delivery
            .send(person, s.id(), &t)
            .await
            .map_err(|e| match e {
                Error::Delivery { .. } => e,
                other => Error::Delivery {
                    person: person.into(),
                    reason: other.to_string(),
                },
            })?;
        let hash = hash_token(&t);
        self.edit_box(s, |b| {
            b.add_token(hash.clone());
            Ok(())
        })
        .await?;
        Ok(hash)
    }

    /// `survey.invite`, `survey.tokens`: stops at the first person whose delivery fails; people
    /// before that stay invited (a second call lists them under `already`).
    pub async fn invite(
        &self,
        id: ObjectId,
        people: &[String],
        groups: &[ObjectId],
    ) -> Result<InviteReport> {
        let s = self.require(id).await?;
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
        let mut report = InviteReport::default();
        for person in self.people_of(people, groups).await? {
            if self.is_invited(id, &person).await? {
                report.already.push(person);
                continue;
            }
            let hash = if s.anonymity == Anonymity::AnonymousOnce {
                Some(self.issue_token(&s, &person).await?)
            } else {
                None
            };
            if let Err(e) = self.record_invitee(id, &person).await {
                if let Some(h) = hash {
                    self.edit_box(&s, |b| {
                        b.spend(&h);
                        Ok(())
                    })
                    .await?;
                }
                return Err(e);
            }
            report.invited.push(person);
        }
        Ok(report)
    }
}
