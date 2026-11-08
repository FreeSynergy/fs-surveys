//! Boxes written by 0.1.0 have no class. Doubt counts as AI, so what they hold is never counted among people.

mod common;

use common::ai::*;
use common::*;
use fs_objects::traits::UnloggedObjects;
use fs_surveys_core::{ALL_STAGES, Anonymity, Credential, Error};
use serde_json::{Value, json};

/// Rewrite the box of `s` into the 0.1.0 layout: the lane of people at the top, no classes.
async fn make_legacy(e: &Env, s: &fs_surveys_core::Survey) {
    let mut o = e.ballots.read_unlogged(s.id()).await.unwrap().unwrap();
    let lane = o.content["people"].clone();
    assert!(lane.is_object());
    o.content =
        json!({"pending": lane["pending"], "batches": lane["batches"], "tokens": lane["tokens"]});
    e.ballots.put_unlogged(o).await.unwrap();
}

#[tokio::test]
async fn old_ballots_without_a_class_are_never_counted_among_people() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOpen, true).await;
    submit_open(&e, &s, q, "a", 3).await;
    e.repo.close(s.id()).await.unwrap();
    make_legacy(&e, &s).await;
    assert!(box_json(&e, &s).await.get("people").is_none());
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 0, "no old ballot among the people");
    assert_eq!(count(&sum, q, "a"), 0);
    assert_eq!(sum.ai.expect("shown apart").ballots, 3);
}

#[allow(clippy::many_single_char_names)] // short fixture names (env, survey, question)
#[tokio::test]
async fn old_token_hashes_are_ai_tokens() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOnce, false).await;
    e.repo.invite(s.id(), &[person("ada")], &[]).await.unwrap();
    make_legacy(&e, &s).await;
    let t = e.delivery.token_of(&person("ada"));
    let r = e
        .repo
        .submit(s.id(), Credential::Token(t), choice(q, "a"))
        .await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    let _: Value = box_json(&e, &s).await;
}
