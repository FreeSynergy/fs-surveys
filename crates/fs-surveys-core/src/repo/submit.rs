//! Submitting an answer: the credential must fit the anonymity level.
//!
//! Pattern: Strategy by anonymity level (a `match`) — named goes to the main store, anonymous ballots
//! to the lane of their class in the box, and a token is spent in the same write as its ballot. The
//! rule for AIs is the Specification `admits`, asked on every path.

use fs_interfaces::Unsupported;
use fs_store::ObjectId;

use super::handle::SurveyRepo;
use crate::check::{admits, drop_empty, validate_answers};
use crate::error::{Error, Result};
use crate::model::{Anonymity, Answer, Class, Credential, Survey, SurveyState};
use crate::tokens::hash_token;

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
            Credential::None => Some(Class::Ai),
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
            (Anonymity::AnonymousOpen, Credential::Person(_) | Credential::None, Some(class)) => {
                // The person is only asked about, never kept.
                admits(&s, class)?;
                validate_answers(&s, &answers, true)?;
                self.edit_box(&s, |b| b.add_ballot(class, ballot(&s, answers)))
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
            (Anonymity::AnonymousOpen, _, _) => Err(wrong("none or person")),
            _ => Err(wrong("token")),
        }
    }
}

/// The ballot: answers in question order, which says nothing about the respondent.
fn ballot(s: &Survey, mut answers: Vec<Answer>) -> Vec<Answer> {
    answers.sort_by_key(|a| s.position(a.question));
    answers
}
