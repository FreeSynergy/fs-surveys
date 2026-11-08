//! Submitting an answer: the credential must fit the anonymity level.
//!
//! Pattern: Strategy by anonymity level (a `match`) — named goes to the main store, anonymous ballots
//! to the lane of their class in the box, and a token is spent in the same write as its ballot. The
//! rule for AIs is the Specification `admits`, asked on every path.

use fs_interfaces::Unsupported;
use fs_objects::ObjectId;

use super::handle::SurveyRepo;
use super::inflight::note_arrival;
use crate::check::{BURST_LIMIT, admits, drop_empty, signals_of, validate_answers};
use crate::error::{Error, Result};
use crate::model::{Anonymity, Answer, Class, Credential, OpenMeta, Survey, SurveyState};
use crate::tokens::hash_token;

/// Limits of voluntary details: pairs per call, characters, and sets per survey.
const MAX_DETAILS: usize = 5;
const MAX_KEY: usize = 40;
const MAX_VALUE: usize = 100;
const MAX_SETS: usize = 10_000;

fn wrong(needs: &str) -> Error {
    Error::Refused(format!("this survey needs a {needs} credential"))
}

impl SurveyRepo {
    /// `response.submit`. The class of the participant comes from the marking (for a person), from the
    /// token's lane (for a token) and is an AI where there is no identity; never from the answers.
    pub async fn submit(
        &self,
        id: ObjectId,
        credential: Credential,
        answers: Vec<Answer>,
    ) -> Result<()> {
        // Asked before the lock: the marking may be slow, and the survey lock should not wait for it.
        let class = match &credential {
            Credential::Person(p) => Some(self.class_for(Some(p)).await?),
            // The respondent's own claim: the question "Are you an AI?" is answered, or there is no ballot.
            Credential::Open(m) => Some(open_class(m)?),
            _ => None,
        };
        // State is read and the ballot written under one lock, so a submit cannot land after
        // a close, or recreate the box of a deleted survey.
        let _guard = self.lock_for(id).await;
        let answers = drop_empty(answers);
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
        match (s.anonymity, credential, class) {
            (Anonymity::Named, Credential::Person(p), Some(class)) => {
                self.submit_named(&s, &p, answers, class).await
            }
            (Anonymity::AnonymousOpen, Credential::Person(_), Some(class)) => {
                // Vouched for by the host: the marking decides; the person is never kept.
                admits(&s, class)?;
                validate_answers(&s, &answers, true)?;
                self.edit_box(&s, |b| b.add_ballot(class, ballot(&s, answers)))
                    .await
            }
            (Anonymity::AnonymousOpen, Credential::Open(meta), Some(class)) => {
                // Anyone may answer, with no proof. The class is the respondent's claim; the patterns
                // are counted and shown, no answer is left out.
                admits(&s, class)?;
                validate_answers(&s, &answers, true)?;
                let burst = note_arrival(&self.arrivals, id, BURST_LIMIT);
                let signals = signals_of(&answers, &meta, burst);
                self.edit_box(&s, |b| {
                    b.add_open_ballot(class, ballot(&s, answers), &signals)
                })
                .await
            }
            (Anonymity::AnonymousOnce, Credential::Token(t), _) => {
                validate_answers(&s, &answers, true)?;
                let hash = hash_token(&t);
                self.edit_box(&s, |b| {
                    // The token was issued to a class; an AI token is refused, and kept, if AIs
                    // are no longer allowed.
                    let class = b.token_class(&hash).ok_or(Error::InvalidToken)?;
                    admits(&s, class)?;
                    b.spend(&hash);
                    b.add_ballot(class, ballot(&s, answers))
                })
                .await
            }
            (Anonymity::Named, _, _) => Err(wrong("person")),
            (Anonymity::AnonymousOpen, _, _) => Err(wrong("open or person")),
            _ => Err(wrong("token")),
        }
    }
}

/// The ballot: answers in question order, which says nothing about the respondent.
fn ballot(s: &Survey, mut answers: Vec<Answer>) -> Vec<Answer> {
    answers.sort_by_key(|a| s.position(a.question));
    answers
}

/// The class an open respondent claims. The claim is required and is not a proof.
fn open_class(meta: &OpenMeta) -> Result<Class> {
    match meta.declared_ai {
        Some(true) => Ok(Class::Ai),
        Some(false) => Ok(Class::Human),
        None => Err(Error::Refused(
            "an open survey asks \"Are you an AI?\"; answer it".into(),
        )),
    }
}

impl SurveyRepo {
    /// `response.voluntary`: details given after the answers on an anonymous survey (age group, role).
    /// Kept apart from the ballots with no link to one; shown only when at least `minGroup` were
    /// given. Allowed while the survey is open.
    pub async fn add_voluntary(
        &self,
        id: ObjectId,
        details: Vec<crate::model::Voluntary>,
    ) -> Result<()> {
        let _guard = self.lock_for(id).await;
        let s = self.require(id).await?;
        if !s.is_anonymous() || s.state != SurveyState::Open {
            return Err(Error::Refused(
                "voluntary details go with an open anonymous survey".into(),
            ));
        }
        let details: Vec<_> = details
            .into_iter()
            .take(MAX_DETAILS)
            .map(|d| crate::model::Voluntary {
                key: d.key.trim().to_string(),
                value: d.value.trim().to_string(),
            })
            .filter(|d| !d.key.is_empty() && !d.value.is_empty())
            .filter(|d| d.key.chars().count() <= MAX_KEY && d.value.chars().count() <= MAX_VALUE)
            .collect();
        if details.is_empty() {
            return Ok(());
        }
        self.edit_box(&s, |b| {
            if b.voluntary().len() >= MAX_SETS {
                return Err(Error::Refused("enough voluntary details were given".into()));
            }
            b.add_voluntary(details)
        })
        .await
    }
}
