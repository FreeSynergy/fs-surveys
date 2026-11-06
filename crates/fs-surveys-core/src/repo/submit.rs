//! Submitting an answer: the credential must fit the anonymity level.
//!
//! Pattern: Strategy by anonymity level (a `match`) — named goes to the main store, anonymous ballots
//! to the box, and a token is spent in the same write as its ballot.

use fs_interfaces::Unsupported;
use fs_store::ObjectId;

use super::handle::SurveyRepo;
use crate::check::{drop_empty, validate_answers};
use crate::error::{Error, Result};
use crate::model::{Anonymity, Answer, Credential, Survey, SurveyState};
use crate::tokens::hash_token;

fn wrong(needs: &str) -> Error {
    Error::Refused(format!("this survey needs a {needs} credential"))
}

impl SurveyRepo {
    /// `response.submit`.
    pub async fn submit(
        &self,
        id: ObjectId,
        credential: Credential,
        answers: Vec<Answer>,
    ) -> Result<()> {
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
        match (s.anonymity, credential) {
            (Anonymity::Named, Credential::Person(p)) => self.submit_named(&s, &p, answers).await,
            (Anonymity::AnonymousOpen, Credential::None) => {
                validate_answers(&s, &answers, true)?;
                self.edit_box(&s, |b| b.add_ballot(ballot(&s, answers)))
                    .await
            }
            (Anonymity::AnonymousOnce, Credential::Token(t)) => {
                validate_answers(&s, &answers, true)?;
                let hash = hash_token(&t);
                self.edit_box(&s, |b| {
                    if !b.spend(&hash) {
                        return Err(Error::InvalidToken);
                    }
                    b.add_ballot(ballot(&s, answers))
                })
                .await
            }
            (Anonymity::Named, _) => Err(wrong("person")),
            (Anonymity::AnonymousOpen, _) => Err(wrong("none")),
            _ => Err(wrong("token")),
        }
    }
}

/// The ballot: answers in question order, which says nothing about the respondent.
fn ballot(s: &Survey, mut answers: Vec<Answer>) -> Vec<Answer> {
    answers.sort_by_key(|a| s.position(a.question));
    answers
}
