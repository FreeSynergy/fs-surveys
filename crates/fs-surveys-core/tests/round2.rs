//! Second review: suppressed answers are held back, empties never count, decisions and the survey lock.

mod common;

use common::*;
use fs_store::{ObjectId, Query, Store};
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Answer, AnswerValue, Condition, ConditionOp, Credential, Error, Method,
    Question, QuestionKind as K,
};

fn pair(a: ObjectId, b: ObjectId, first: &str, second: &str) -> Vec<Answer> {
    vec![
        Answer {
            question: a,
            value: AnswerValue::Choice(first.into()),
        },
        Answer {
            question: b,
            value: AnswerValue::Choice(second.into()),
        },
    ]
}

#[tokio::test]
async fn suppressed_answers_count_as_held_back_in_evaluate() {
    let e = env().await;
    let s = e
        .repo
        .create("S", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let (_, a) = e.repo.add_question(s.id(), pick()).await.unwrap();
    let mut follow = Question::new(K::SingleChoice, "then").with_choices(&[("x", "X"), ("y", "Y")]);
    follow.conditions = vec![Condition {
        question: a,
        op: ConditionOp::Equals,
        value: "a".into(),
    }];
    let (_, b) = e.repo.add_question(s.id(), follow).await.unwrap();
    e.repo.publish(s.id()).await.unwrap();
    for second in ["x", "x", "x", "y", "y"] {
        e.repo
            .submit(s.id(), Credential::None, pair(a, b, "a", second))
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    e.repo.reopen(s.id()).await.unwrap();
    for _ in 0..4 {
        e.repo
            .submit(s.id(), Credential::None, pair(a, b, "a", "y"))
            .await
            .unwrap();
    }
    e.repo
        .submit(s.id(), Credential::None, choice(a, "b"))
        .await
        .unwrap();
    e.repo.close(s.id()).await.unwrap();
    let ev = e
        .repo
        .evaluate(s.id(), b, Method::Plurality, ALL_STAGES)
        .await
        .unwrap();
    assert_eq!(
        ev.held_back, 4,
        "the 4 suppressed answers are held back for this question"
    );
    assert!(
        ev.winners.is_empty(),
        "x leads 3:2 but 4 held-back answers could flip it: {ev:?}"
    );
}

#[tokio::test]
async fn empty_answers_are_dropped_and_never_count() {
    let e = env().await;
    let s = e
        .repo
        .create("S", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let (_, t) = e
        .repo
        .add_question(s.id(), Question::new(K::Text, "t"))
        .await
        .unwrap();
    let (_, m) = e
        .repo
        .add_question(
            s.id(),
            Question::new(K::MultipleChoice, "m").with_choices(&[("a", "A"), ("b", "B")]),
        )
        .await
        .unwrap();
    e.repo.publish(s.id()).await.unwrap();
    let text = |v: &str| Answer {
        question: t,
        value: AnswerValue::Text(v.into()),
    };
    let none = Answer {
        question: m,
        value: AnswerValue::Choices(vec![]),
    };
    for v in ["", "  ", "", "", "secret"] {
        e.repo
            .submit(s.id(), Credential::None, vec![text(v), none.clone()])
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 5);
    for q in &sum.questions {
        assert!(
            q.answers <= 1 && q.hidden && q.texts.is_empty() && q.counts.is_empty(),
            "{q:?}"
        );
    }
}

#[tokio::test]
async fn decisions_need_the_survey_and_the_lock() {
    let e = env().await;
    let (s, q) = open_survey(&e, Anonymity::Named).await;
    let d = e
        .repo
        .record(
            s.id(),
            q,
            Method::Plurality,
            ALL_STAGES,
            vec!["a".into()],
            "",
        )
        .await
        .unwrap();
    e.repo.delete(s.id()).await.unwrap();
    let late = e
        .repo
        .record(s.id(), q, Method::Plurality, ALL_STAGES, vec![], "")
        .await;
    assert!(matches!(late, Err(Error::NotFound { .. })));
    assert!(matches!(
        e.repo.link_task(d.id(), "t").await,
        Err(Error::NotFound { .. })
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn record_and_delete_in_parallel_leave_no_decision() {
    for _ in 0..30 {
        let e = env().await;
        let (s, q) = open_survey(&e, Anonymity::Named).await;
        let id = s.id();
        let (r1, r2) = (e.repo.clone(), e.repo.clone());
        let h1 = tokio::spawn(async move {
            r1.record(id, q, Method::Plurality, ALL_STAGES, vec![], "")
                .await
                .is_ok()
        });
        let h2 = tokio::spawn(async move { r2.delete(id).await.is_ok() });
        let _ = (h1.await.unwrap(), h2.await.unwrap());
        let left = e
            .main
            .query(&Query::new().of_type("Decision"))
            .await
            .unwrap();
        assert!(left.is_empty(), "an orphan decision is left");
    }
}
