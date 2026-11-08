//! Open surveys: anyone may answer with no proof; the respondent says whether they are an AI; the
//! patterns are counted and shown, and no answer is ever left out because of them.

mod common;

use common::ai::*;
use common::*;
use fs_objects::ObjectId;
use fs_surveys_core::Designer as _;
use fs_surveys_core::Results as _;
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Answer, AnswerValue, Credential, Error, OpenMeta, Question,
    QuestionKind, Voluntary,
};

/// An open survey with three single-choice questions and one free text, published, closed to AIs
/// unless `allow`.
async fn open(e: &Env, allow: bool) -> (fs_surveys_core::Survey, Vec<ObjectId>) {
    let s = e
        .repo
        .create_by("Open", Anonymity::AnonymousOpen, &[], &owner())
        .await
        .unwrap();
    e.repo
        .edit(s.id(), "Open", Anonymity::AnonymousOpen, 3)
        .await
        .unwrap();
    let mut qs = vec![];
    for _ in 0..3 {
        qs.push(e.repo.add_question(s.id(), pick()).await.unwrap().1);
    }
    qs.push(
        e.repo
            .add_question(s.id(), Question::new(QuestionKind::Text, "Why?"))
            .await
            .unwrap()
            .1,
    );
    if allow {
        e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    }
    (e.repo.publish(s.id()).await.unwrap(), qs)
}

fn meta(ai: bool, ms: u32) -> Credential {
    Credential::Open(OpenMeta {
        declared_ai: Some(ai),
        elapsed_ms: Some(ms),
        honeypot: None,
    })
}

fn row(qs: &[ObjectId], picks: [&str; 3], text: &str) -> Vec<Answer> {
    let mut a: Vec<Answer> = qs
        .iter()
        .zip(picks)
        .map(|(q, c)| Answer {
            question: *q,
            value: AnswerValue::Choice(c.into()),
        })
        .collect();
    a.push(Answer {
        question: qs[3],
        value: AnswerValue::Text(text.into()),
    });
    a
}

#[tokio::test]
async fn the_question_must_be_answered_and_no_identity_means_no_ai() {
    let e = env().await;
    let (s, qs) = open(&e, true).await;
    for c in [Credential::None, Credential::Open(OpenMeta::default())] {
        let r = e
            .repo
            .submit(s.id(), c, row(&qs, ["a", "a", "b"], "x"))
            .await;
        assert!(matches!(r, Err(Error::Refused(_))), "{r:?}");
    }
    // Saying "no, I am not an AI" is enough: no proof, no person; the answer is a person's.
    e.repo
        .submit(
            s.id(),
            meta(false, 60_000),
            row(&qs, ["a", "b", "c"], "fine"),
        )
        .await
        .unwrap();
    e.repo.close(s.id()).await.unwrap();
    let r = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(r.held_back, 1, "one answer, below minGroup");
    assert!(r.ai.as_ref().is_none_or(|a| a.held_back == 0));
}

#[tokio::test]
async fn a_declared_ai_counts_apart_and_only_where_allowed() {
    let e = env().await;
    let (s, qs) = open(&e, false).await;
    let r = e
        .repo
        .submit(s.id(), meta(true, 60_000), row(&qs, ["a", "b", "c"], "bot"))
        .await;
    assert!(matches!(r, Err(Error::AiNotAllowed)), "{r:?}");
    e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    for i in 0..3 {
        e.repo
            .submit(
                s.id(),
                meta(true, 60_000),
                row(&qs, ["a", "b", "c"], &format!("bot {i}")),
            )
            .await
            .unwrap();
        e.repo
            .submit(
                s.id(),
                meta(false, 60_000),
                row(&qs, ["b", "b", "b"], &format!("me {i}")),
            )
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let r = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(r.ballots, 3, "people only");
    assert_eq!(r.ai.as_ref().unwrap().ballots, 3, "AIs apart");
}

#[tokio::test]
async fn patterns_are_counted_and_nothing_is_left_out() {
    let e = env().await;
    let (s, qs) = open(&e, true).await;
    let long = "the very same long free text";
    let bot = Credential::Open(OpenMeta {
        declared_ai: Some(false),
        elapsed_ms: Some(300),
        honeypot: Some("hello".into()),
    });
    let untimed = Credential::Open(OpenMeta {
        declared_ai: Some(false),
        ..OpenMeta::default()
    });
    e.repo
        .submit(s.id(), bot, row(&qs, ["a", "a", "a"], long))
        .await
        .unwrap();
    e.repo
        .submit(s.id(), untimed, row(&qs, ["a", "b", "c"], long))
        .await
        .unwrap();
    e.repo
        .submit(
            s.id(),
            meta(false, 90_000),
            row(&qs, ["c", "b", "a"], "something else entirely"),
        )
        .await
        .unwrap();
    e.repo.close(s.id()).await.unwrap();
    let r = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(r.ballots, 3, "every answer counts");
    let p = r.patterns.unwrap();
    assert_eq!(p.signals.ballots, 3);
    assert_eq!(p.signals.too_fast, 1);
    assert_eq!(p.signals.honeypot, 1);
    assert_eq!(p.signals.straight_line, 1);
    assert_eq!(p.signals.no_timing, 1);
    assert_eq!(p.same_text, 2);
    assert_eq!(p.signals.burst, 0);
}

#[tokio::test]
async fn a_burst_is_counted() {
    let e = env().await;
    let (s, qs) = open(&e, false).await;
    for i in 0..13 {
        e.repo
            .submit(
                s.id(),
                meta(false, 90_000),
                row(&qs, ["a", "b", "c"], &format!("t{i}")),
            )
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let r = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(r.ballots, 13);
    assert_eq!(
        r.patterns.unwrap().signals.burst,
        3,
        "the 11th, 12th and 13th in a minute"
    );
}

#[tokio::test]
async fn voluntary_details_show_only_in_groups() {
    let e = env().await;
    let (s, qs) = open(&e, false).await;
    e.repo
        .submit(s.id(), meta(false, 90_000), row(&qs, ["a", "b", "c"], "x"))
        .await
        .unwrap();
    let age = |v: &str| {
        vec![Voluntary {
            key: "age".into(),
            value: v.into(),
        }]
    };
    for v in ["30-39", "30-39"] {
        e.repo.add_voluntary(s.id(), age(v)).await.unwrap();
    }
    let r = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert!(r.voluntary.is_empty(), "two are below minGroup 3");
    e.repo.add_voluntary(s.id(), age("20-29")).await.unwrap();
    let r = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    let got: Vec<_> = r
        .voluntary
        .iter()
        .map(|c| (c.choice.as_str(), c.count))
        .collect();
    assert_eq!(got, [("age=20-29", 1), ("age=30-39", 2)]);
}
