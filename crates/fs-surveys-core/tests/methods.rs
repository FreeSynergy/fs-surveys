//! Guarantee 7: one set of ballots under several methods, held-back answers, decisions and tasks.

mod common;

use common::*;
use fs_objects::ObjectId;
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Answer, AnswerValue, Credential, Error, Method, Question, QuestionKind,
};

async fn ranked(e: &Env) -> (fs_surveys_core::Survey, ObjectId) {
    let s = e.repo.create("Venue", Anonymity::Named, &[]).await.unwrap();
    let q = Question::new(QuestionKind::Ranking, "Rank").with_choices(&[
        ("a", "A"),
        ("b", "B"),
        ("c", "C"),
    ]);
    let (_, q) = e.repo.add_question(s.id(), q).await.unwrap();
    let s = e.repo.publish(s.id()).await.unwrap();
    for (i, order) in [
        ["a", "b", "c"],
        ["a", "c", "b"],
        ["b", "c", "a"],
        ["b", "a", "c"],
        ["c", "b", "a"],
    ]
    .iter()
    .enumerate()
    {
        let r = AnswerValue::Ranking(order.iter().map(|c| (*c).into()).collect());
        let a = vec![Answer {
            question: q,
            value: r,
        }];
        e.repo
            .submit(s.id(), Credential::Person(person(&i.to_string())), a)
            .await
            .unwrap();
    }
    (s, q)
}

#[tokio::test]
async fn the_same_ballots_run_under_several_methods() {
    let e = env().await;
    let (s, q) = ranked(&e).await;
    for m in [Method::Borda, Method::Schulze, Method::InstantRunoff] {
        let ev = e.repo.evaluate(s.id(), q, m, ALL_STAGES).await.unwrap();
        assert_eq!(ev.method, m);
        assert_eq!(ev.options.len(), 3, "{m:?}");
        assert!(!ev.hidden && ev.held_back == 0);
    }
    let borda = e
        .repo
        .evaluate(s.id(), q, Method::Borda, ALL_STAGES)
        .await
        .unwrap();
    assert!(
        borda.options.iter().any(|o| o.choice == "b" && o.rank == 1),
        "{:?}",
        borda.options
    );
    let r = e
        .repo
        .evaluate(s.id(), q, Method::Plurality, ALL_STAGES)
        .await;
    assert!(
        matches!(r, Err(Error::Method(_))),
        "a ballot form that does not fit is an error"
    );
    let text = Question::new(QuestionKind::Text, "Why");
    let d = e.repo.create("Draft", Anonymity::Named, &[]).await.unwrap();
    let (_, t) = e.repo.add_question(d.id(), text).await.unwrap();
    assert!(matches!(
        e.repo.evaluate(d.id(), t, Method::Borda, ALL_STAGES).await,
        Err(Error::Invalid(_))
    ));
}

async fn win(e: &Env, s: ObjectId, q: ObjectId, m: Method) -> Vec<String> {
    e.repo.evaluate(s, q, m, ALL_STAGES).await.unwrap().winners
}

#[tokio::test]
async fn single_choice_under_plurality_majority_and_supermajority() {
    let e = env().await;
    let (s, q) = open_survey(&e, Anonymity::Named).await;
    for (i, c) in ["a", "a", "a", "b", "c"].iter().enumerate() {
        e.repo
            .submit(
                s.id(),
                Credential::Person(person(&i.to_string())),
                choice(q, c),
            )
            .await
            .unwrap();
    }
    assert_eq!(win(&e, s.id(), q, Method::Plurality).await, ["a"]);
    assert_eq!(win(&e, s.id(), q, Method::Majority).await, ["a"]);
    assert!(
        win(
            &e,
            s.id(),
            q,
            Method::Supermajority {
                threshold_percent: 75
            }
        )
        .await
        .is_empty()
    );
}

#[tokio::test]
async fn held_back_answers_that_could_tip_the_result_leave_it_undecided() {
    let e = env().await;
    let s = e
        .repo
        .create("Lunch", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    e.repo
        .edit(s.id(), "Lunch", Anonymity::AnonymousOpen, 3)
        .await
        .unwrap();
    let (_, q) = e.repo.add_question(s.id(), pick()).await.unwrap();
    e.repo.publish(s.id()).await.unwrap();
    submit_open(&e, &s, q, "a", 3).await;
    submit_open(&e, &s, q, "b", 2).await;
    e.repo.close(s.id()).await.unwrap();
    let ev = e
        .repo
        .evaluate(s.id(), q, Method::Plurality, ALL_STAGES)
        .await
        .unwrap();
    assert_eq!(
        (ev.winners.clone(), ev.held_back),
        (vec!["a".to_string()], 0)
    );
    e.repo.reopen(s.id()).await.unwrap();
    submit_open(&e, &s, q, "b", 2).await;
    let open = e
        .repo
        .evaluate(s.id(), q, Method::Plurality, ALL_STAGES)
        .await
        .unwrap();
    assert!(
        open.winners.is_empty() && open.held_back == 0,
        "while open: no winner, no number"
    );
    assert!(open.note.contains("still open"), "{}", open.note);
    e.repo.close(s.id()).await.unwrap();
    let ev = e
        .repo
        .evaluate(s.id(), q, Method::Plurality, ALL_STAGES)
        .await
        .unwrap();
    assert!(
        ev.winners.is_empty(),
        "2 held-back answers could make b win"
    );
    assert_eq!(ev.held_back, 2);
    assert!(ev.note.contains("held-back"), "{}", ev.note);
    assert_eq!(
        e.repo.summary(s.id(), ALL_STAGES).await.unwrap().held_back,
        2
    );
}

#[tokio::test]
async fn too_few_released_answers_are_hidden() {
    let e = env().await;
    let (s, q) = open_survey(&e, Anonymity::AnonymousOpen).await;
    submit_open(&e, &s, q, "a", 2).await;
    e.repo.close(s.id()).await.unwrap();
    let ev = e
        .repo
        .evaluate(s.id(), q, Method::Plurality, ALL_STAGES)
        .await
        .unwrap();
    assert!(ev.hidden && ev.winners.is_empty() && ev.options.is_empty());
    assert_eq!(ev.held_back, 2);
}
