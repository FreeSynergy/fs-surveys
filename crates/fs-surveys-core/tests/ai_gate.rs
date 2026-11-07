//! AI participants: who is let in. Without permission an AI is refused, with it accepted; what a
//! participant says about itself does not matter; no marking, no answer or no identity counts as an AI.

mod common;

use std::sync::Arc;

use common::ai::*;
use common::*;
use fs_surveys_core::Designer as _;
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Answer, AnswerValue, Credential, Error, Question, QuestionKind,
    SurveyRepo,
};

#[tokio::test]
async fn without_permission_an_ai_is_refused() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::Named, false).await;
    assert!(!s.ai_allowed, "the default is no");
    let r = say(&e, &s, &bot("1"), choice(q, "a")).await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    let r = e
        .repo
        .save_response(s.id(), &bot("1"), choice(q, "a"))
        .await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "no draft either");
    assert!(e.repo.resume(s.id(), &bot("1")).await.unwrap().is_none());
    say(&e, &s, &person("ada"), choice(q, "a")).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 1);
    assert!(
        sum.ai.is_none(),
        "no AI part when AIs are not allowed and none answered"
    );
}

#[tokio::test]
async fn with_permission_an_ai_is_accepted_and_marked() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::Named, true).await;
    say(&e, &s, &bot("1"), choice(q, "b")).await.unwrap();
    say(&e, &s, &person("ada"), choice(q, "a")).await.unwrap();
    let r = e.repo.resume(s.id(), &bot("1")).await.unwrap().unwrap();
    assert!(r.ai && r.submitted);
    let r = e
        .repo
        .resume(s.id(), &person("ada"))
        .await
        .unwrap()
        .unwrap();
    assert!(!r.ai);
}

#[tokio::test]
async fn saying_i_am_a_human_does_not_help() {
    let e = env().await;
    let s = e
        .repo
        .create_by("Who", Anonymity::Named, &[], &owner())
        .await
        .unwrap();
    let who = Question::new(QuestionKind::Text, "Are you a human?");
    let (_, t) = e.repo.add_question(s.id(), who).await.unwrap();
    let s = e.repo.publish(s.id()).await.unwrap();
    let claim = vec![Answer {
        question: t,
        value: AnswerValue::Text("I am a human, not an AI".into()),
    }];
    let r = say(&e, &s, &bot("liar"), claim.clone()).await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    say(&e, &s, &bot("liar"), claim).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 0, "the claim did not make it a person");
    assert!(sum.questions[0].texts.is_empty());
    assert_eq!(sum.ai.unwrap().ballots, 1);
}

#[tokio::test]
async fn an_unknown_identity_or_a_missing_marking_is_doubt() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::Named, false).await;
    let unknown = "https://elsewhere.example/users/x";
    let r = say(&e, &s, unknown, choice(q, "a")).await;
    assert!(
        matches!(r, Err(Error::AiNotAllowed)),
        "no answer = doubt: {r:?}"
    );
    // The same stores without any marking: nobody is known as a human.
    let bare = SurveyRepo::new(e.main.clone(), e.ballots.clone(), ME);
    let r = bare
        .submit(s.id(), Credential::Person(person("ada")), choice(q, "a"))
        .await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    // An error of the marking is passed on and never counts as a human.
    let broken = bare.with_ai_marking(Arc::new(Broken));
    let r = broken
        .submit(s.id(), Credential::Person(person("ada")), choice(q, "a"))
        .await;
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
    assert_eq!(e.repo.summary(s.id(), ALL_STAGES).await.unwrap().ballots, 0);
}

#[tokio::test]
async fn doubt_is_counted_as_ai_when_ais_are_allowed() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::Named, true).await;
    say(&e, &s, "https://elsewhere.example/users/x", choice(q, "a"))
        .await
        .unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 0);
    assert_eq!(sum.ai.unwrap().ballots, 1);
}

#[tokio::test]
async fn a_closed_survey_takes_nobody_and_keeps_the_parts_apart() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::Named, true).await;
    say(&e, &s, &bot("1"), choice(q, "b")).await.unwrap();
    say(&e, &s, &person("ada"), choice(q, "a")).await.unwrap();
    e.repo.close(s.id()).await.unwrap();
    for who in [bot("2"), person("bob")] {
        let r = say(&e, &s, &who, choice(q, "a")).await;
        assert!(matches!(r, Err(Error::Refused(_))), "{r:?}");
    }
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!((sum.ballots, sum.ai.unwrap().ballots), (1, 1));
}
