//! Guarantees 1 and 2: anonymous answers leave no trace, and results change only in batches.

mod common;

use std::sync::{Arc, Mutex};

use common::*;
use fs_surveys_core::{ALL_STAGES, Anonymity, SurveyChange, SurveyFeed, SurveyListener};

#[derive(Default)]
struct Seen(Mutex<Vec<SurveyChange>>);

impl SurveyListener for Seen {
    fn changed(&self, c: &SurveyChange) {
        self.0.lock().unwrap().push(c.clone());
    }
}

async fn small(e: &Env, a: Anonymity) -> (fs_surveys_core::Survey, fs_objects::ObjectId) {
    let s = e.repo.create("Lunch", a, &[]).await.unwrap();
    e.repo.edit(s.id(), "Lunch", a, 3).await.unwrap();
    let (_, q) = e
        .repo
        .add_question(s.id(), pick().required())
        .await
        .unwrap();
    (e.repo.publish(s.id()).await.unwrap(), q)
}

#[tokio::test]
async fn submits_leave_nothing_in_main_store_log_or_feed() {
    let e = env().await;
    let (s, q) = small(&e, Anonymity::AnonymousOpen).await;
    let main_before = raw(&e.main).await;
    let seen = Arc::new(Seen::default());
    let _sub = e.repo.subscribe(seen.clone());
    submit_open(&e, &s, q, "a", 4).await;
    assert_eq!(
        raw(&e.main).await,
        main_before,
        "main store untouched by anonymous submits"
    );
    assert!(seen.0.lock().unwrap().is_empty(), "the feed gets nothing");
    assert_eq!(
        log_len(&e.ballots).await,
        0,
        "the ballot store has no activity at all"
    );
}

#[tokio::test]
async fn box_holds_no_person_time_or_ballot_id() {
    let e = env().await;
    let (s, q) = small(&e, Anonymity::AnonymousOpen).await;
    submit_open(&e, &s, q, "a", 2).await;
    submit_open(&e, &s, q, "b", 2).await;
    let content = box_json(&e, &s).await;
    let keys: Vec<_> = content.as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, ["batches", "pending", "tokens"]);
    for ballot in content["pending"].as_array().unwrap() {
        let answers = ballot.as_array().unwrap();
        assert_eq!(answers.len(), 1);
        let mut k: Vec<_> = answers[0].as_object().unwrap().keys().cloned().collect();
        k.sort();
        assert_eq!(
            k,
            ["choice", "question"],
            "a ballot is its answers and nothing else"
        );
    }
    let all = raw(&e.ballots).await;
    assert!(!all.contains("people/") && !all.contains(ME));
    let year = chrono::Utc::now().format("%Y-").to_string();
    assert!(
        !all.contains(&year),
        "no real time in the ballot store: {all}"
    );
    assert!(
        all.contains("1970-01-01T00:00:00"),
        "the box carries the epoch"
    );
}

#[tokio::test]
async fn reading_and_reopening_change_nothing() {
    let e = env().await;
    let (s, q) = small(&e, Anonymity::AnonymousOpen).await;
    submit_open(&e, &s, q, "a", 3).await;
    e.repo.close(s.id()).await.unwrap(); // releases a batch of 3
    e.repo.reopen(s.id()).await.unwrap();
    let first = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!((first.ballots, count(&first, q, "a")), (3, 3));
    let second = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(first, second, "reading twice shows no change");
    submit_open(&e, &s, q, "b", 2).await;
    let held = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(
        (held.ballots, held.held_back),
        (3, 0),
        "new ballots are not shown, and not counted while open"
    );
    assert_eq!(count(&held, q, "b"), 0);
    e.repo.close(s.id()).await.unwrap();
    let closed = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(
        (closed.ballots, closed.held_back),
        (3, 2),
        "after close the number is real"
    );
    assert_eq!(count(&closed, q, "b"), 0);
    e.repo.reopen(s.id()).await.unwrap();
    assert_eq!(
        e.repo.summary(s.id(), ALL_STAGES).await.unwrap(),
        held,
        "2 < minGroup: no change"
    );
}

#[tokio::test]
async fn a_release_is_never_smaller_than_min_group() {
    let e = env().await;
    let (s, q) = small(&e, Anonymity::AnonymousOpen).await;
    submit_open(&e, &s, q, "a", 2).await;
    e.repo.close(s.id()).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert!(sum.hidden && sum.ballots == 0 && sum.held_back == 2);
    assert!(sum.questions[0].hidden && sum.questions[0].counts.is_empty());
    e.repo.reopen(s.id()).await.unwrap();
    submit_open(&e, &s, q, "b", 1).await;
    e.repo.close(s.id()).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!((sum.ballots, sum.held_back, sum.hidden), (3, 0, false));
    let batches = box_json(&e, &s).await["batches"]
        .as_array()
        .unwrap()
        .clone();
    assert!(
        batches
            .iter()
            .all(|b| b["ballots"].as_array().unwrap().len() >= 3)
    );
}

#[tokio::test]
async fn export_gives_totals_only() {
    let e = env().await;
    let (s, q) = small(&e, Anonymity::AnonymousOpen).await;
    submit_open(&e, &s, q, "a", 3).await;
    e.repo.close(s.id()).await.unwrap();
    let csv = String::from_utf8(e.repo.export(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(csv.starts_with("question,value,count\r\n") && csv.contains("Which?,a,3"));
    assert!(!csv.contains("person"));
}
