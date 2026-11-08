//! AI participants in anonymous surveys: the class is fixed when a token is issued (or by the
//! marking of the person the host names), ballots go to their own lane and batches, and the hand-over
//! of a token is a stated limit.

mod common;

use common::ai::*;
use common::*;
use fs_surveys_core::Designer as _;
use fs_surveys_core::Results as _;
use fs_surveys_core::{ALL_STAGES, Anonymity, Credential, Error};

#[tokio::test]
async fn an_ai_gets_no_token_unless_allowed() {
    let e = env().await;
    let (s, _) = survey(&e, Anonymity::AnonymousOnce, false).await;
    let r = e
        .repo
        .invite(
            s.id(),
            &[
                person("ada"),
                bot("1"),
                "https://elsewhere.example/x".into(),
            ],
            &[],
        )
        .await
        .unwrap();
    assert_eq!(r.invited, [person("ada")]);
    assert_eq!(
        r.refused,
        [bot("1"), "https://elsewhere.example/x".to_string()]
    );
    assert_eq!(e.delivery.all().len(), 1, "nothing was sent to the AI");
    e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    let r = e.repo.invite(s.id(), &[bot("1")], &[]).await.unwrap();
    assert_eq!((r.invited, r.refused), (vec![bot("1")], vec![]));
}

#[tokio::test]
async fn token_ballots_are_released_and_shown_apart() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOnce, true).await;
    let ppl: Vec<String> = (0..3).map(|i| person(&format!("h{i}"))).collect();
    let bots: Vec<String> = (0..3).map(|i| bot(&i.to_string())).collect();
    e.repo.invite(s.id(), &ppl, &[]).await.unwrap();
    e.repo.invite(s.id(), &bots, &[]).await.unwrap();
    for p in &ppl {
        let t = e.delivery.token_of(p);
        e.repo
            .submit(s.id(), Credential::Token(t), choice(q, "a"))
            .await
            .unwrap();
    }
    for b in &bots {
        let t = e.delivery.token_of(b);
        e.repo
            .submit(s.id(), Credential::Token(t), choice(q, "c"))
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(
        (sum.ballots, count(&sum, q, "a"), count(&sum, q, "c")),
        (3, 3, 0)
    );
    let ai = sum.ai.unwrap();
    assert_eq!(
        (ai.ballots, count(&ai, q, "c"), count(&ai, q, "a")),
        (3, 3, 0)
    );
    let csv = String::from_utf8(e.repo.export(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(!csv.contains("participants"), "{csv}");
    let csv_ai = String::from_utf8(e.repo.export_ai(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(
        csv_ai.lines().skip(1).all(|l| l.starts_with("ai,")),
        "{csv_ai}"
    );
}

#[tokio::test]
async fn the_batches_of_the_two_classes_do_not_fill_each_other() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOpen, true).await;
    for i in 0..2 {
        let h = Credential::Person(person(&format!("h{i}")));
        e.repo.submit(s.id(), h, choice(q, "a")).await.unwrap();
    }
    for i in 0..3 {
        let b = Credential::Person(bot(&i.to_string()));
        e.repo.submit(s.id(), b, choice(q, "c")).await.unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert!(
        sum.hidden,
        "2 people are fewer than minGroup, whatever the AIs add"
    );
    assert_eq!(sum.held_back, 2);
    let ai = sum.ai.unwrap();
    assert!(!ai.hidden && ai.ballots == 3 && ai.held_back == 0);
    assert_eq!(count(&ai, q, "c"), 3);
}

#[tokio::test]
async fn anonymous_open_asks_the_marking_and_stores_nobody() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOpen, false).await;
    let ada = person("ada");
    e.repo
        .submit(s.id(), Credential::Person(ada.clone()), choice(q, "a"))
        .await
        .unwrap();
    let r = e
        .repo
        .submit(s.id(), Credential::Person(bot("1")), choice(q, "a"))
        .await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "marked AI: {r:?}");
    // No identity is no longer doubt: an open survey asks "Are you an AI?" (see `open.rs`).
    let r = e
        .repo
        .submit(s.id(), Credential::None, choice(q, "a"))
        .await;
    assert!(matches!(r, Err(Error::Refused(_))), "{r:?}");
    assert!(
        !raw_box(&e, &s).await.contains(&ada),
        "the person is not kept"
    );
    assert!(!raw(&e.main).await.contains(&ada), "nor in the main store");
}

#[tokio::test]
async fn a_withdrawn_permission_blocks_ai_tokens_without_spending_them() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOnce, true).await;
    e.repo.invite(s.id(), &[bot("1")], &[]).await.unwrap();
    let t = e.delivery.token_of(&bot("1"));
    e.repo
        .set_ai_allowed(s.id(), &owner(), false)
        .await
        .unwrap();
    let r = e
        .repo
        .submit(s.id(), Credential::Token(t.clone()), choice(q, "c"))
        .await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    e.repo
        .submit(s.id(), Credential::Token(t), choice(q, "c"))
        .await
        .unwrap();
}

/// The stated limit: the core sees a token, not who holds it. A person can hand theirs to an AI, and
/// the ballot then counts as the person who was invited.
#[tokio::test]
async fn a_token_handed_to_an_ai_counts_as_the_invited_person_limit() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOnce, false).await;
    let people: Vec<String> = (0..3).map(|i| person(&format!("h{i}"))).collect();
    e.repo.invite(s.id(), &people, &[]).await.unwrap();
    for p in &people {
        let t = e.delivery.token_of(p);
        e.repo
            .submit(s.id(), Credential::Token(t), choice(q, "c"))
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(
        count(&sum, q, "c"),
        3,
        "possibly typed by an AI; the core cannot know"
    );
}

#[tokio::test]
async fn nothing_in_the_ballot_box_names_a_person() {
    let e = env().await;
    let (s, q) = survey(&e, Anonymity::AnonymousOnce, true).await;
    e.repo
        .invite(s.id(), &[person("ada"), bot("1")], &[])
        .await
        .unwrap();
    let t = e.delivery.token_of(&bot("1"));
    e.repo
        .submit(s.id(), Credential::Token(t), choice(q, "c"))
        .await
        .unwrap();
    let boxed = raw_box(&e, &s).await;
    assert!(
        !boxed.contains("bot-1") && !boxed.contains("ada"),
        "{boxed}"
    );
}
