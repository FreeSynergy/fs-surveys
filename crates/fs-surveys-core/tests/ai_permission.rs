//! AI participants: the permission. Default no; only the creator changes it, only while the survey
//! is a draft or open, and an AI cannot grant it; withdrawing keeps earlier AI answers apart.

mod common;

use common::ai::*;
use common::*;
use fs_surveys_core::{ALL_STAGES, Anonymity, Designer as _, Error, Surveys};

#[tokio::test]
async fn only_the_creator_can_change_it() {
    let e = env().await;
    let (s, _) = survey(&e, Anonymity::Named, false).await;
    for who in [person("mallory"), String::new(), bot("1")] {
        let r = e.repo.set_ai_allowed(s.id(), &who, true).await;
        assert!(matches!(r, Err(Error::NotCreator)), "{who:?}: {r:?}");
    }
    assert!(!e.repo.get(s.id()).await.unwrap().unwrap().ai_allowed);
    let on = e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    assert!(on.ai_allowed);
    let off = e
        .repo
        .set_ai_allowed(s.id(), &owner(), false)
        .await
        .unwrap();
    assert!(!off.ai_allowed);
}

#[tokio::test]
async fn a_survey_without_a_creator_can_never_allow_ais() {
    let e = env().await;
    let s = e.repo.create("Plain", Anonymity::Named, &[]).await.unwrap();
    assert_eq!(s.creator, None);
    let r = e.repo.set_ai_allowed(s.id(), &owner(), true).await;
    assert!(matches!(r, Err(Error::NotCreator)), "{r:?}");
}

#[tokio::test]
async fn an_ai_cannot_allow_ais_but_can_forbid_them() {
    let e = env().await;
    let s = e
        .repo
        .create_by("By a bot", Anonymity::Named, &[], &bot("maker"))
        .await
        .unwrap();
    let r = e.repo.set_ai_allowed(s.id(), &bot("maker"), true).await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    let off = e
        .repo
        .set_ai_allowed(s.id(), &bot("maker"), false)
        .await
        .unwrap();
    assert!(!off.ai_allowed);
}

#[tokio::test]
async fn other_edits_do_not_touch_it() {
    let e = env().await;
    let s = e
        .repo
        .create_by("Draft", Anonymity::Named, &[], &owner())
        .await
        .unwrap();
    e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    let s = e
        .repo
        .edit(s.id(), "Renamed", Anonymity::Named, 5)
        .await
        .unwrap();
    let s = e.repo.classify(s.id(), &[]).await.unwrap();
    assert!(s.ai_allowed && s.creator.as_deref() == Some(owner().as_str()));
    let s = e.repo.get(s.id()).await.unwrap().unwrap();
    assert!(s.ai_allowed, "kept in the store");
}

#[tokio::test]
async fn withdrawn_later_earlier_ai_answers_stay_apart_and_new_ones_are_refused() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::Named, true).await;
    many(&e, &s, q, "c", 2, true).await;
    many(&e, &s, q, "a", 3, false).await;
    e.repo
        .set_ai_allowed(s.id(), &owner(), false)
        .await
        .unwrap();
    let r = say(&e, &s, &bot("9"), choice(q, "c")).await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    let r = e
        .repo
        .save_response(s.id(), &bot("9"), choice(q, "c"))
        .await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 3, "people only");
    assert_eq!(count(&sum, q, "c"), 0, "no AI vote among the people");
    let ai = sum.ai.expect("the AI answers are still shown, apart");
    assert_eq!((ai.ballots, count(&ai, q, "c")), (2, 2));
    assert!(e.repo.resume(s.id(), &bot("0")).await.unwrap().unwrap().ai);
}

#[tokio::test]
async fn a_closed_survey_keeps_its_permission() {
    let e = env().await;
    let (s, _) = survey(&e, Anonymity::Named, true).await;
    e.repo.close(s.id()).await.unwrap();
    for allowed in [true, false] {
        let r = e.repo.set_ai_allowed(s.id(), &owner(), allowed).await;
        assert!(matches!(r, Err(Error::Refused(_))), "{r:?}");
    }
    assert!(e.repo.get(s.id()).await.unwrap().unwrap().ai_allowed);
}

#[tokio::test]
async fn the_provider_reports_the_capability() {
    let e = env().await;
    let caps = e.repo.capabilities();
    let c = caps
        .entries()
        .iter()
        .find(|c| c.capability == "survey.aiAllowed");
    assert!(c.is_some(), "listed");
}
