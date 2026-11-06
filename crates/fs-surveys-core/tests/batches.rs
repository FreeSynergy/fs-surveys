//! Review: a batch counts for a question only if it has `minGroup` answers to that question.

mod common;
use common::*;
use fs_store::ObjectId;
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Answer, AnswerValue, Condition, ConditionOp, Credential, Method,
    Question, QuestionKind, Survey,
};

/// `yn` (a / b) and a conditional scale `band` shown only after "a".
async fn conditional(e: &Env, scope: &[ObjectId]) -> (Survey, ObjectId, ObjectId) {
    let s = e
        .repo
        .create("S", Anonymity::AnonymousOpen, scope)
        .await
        .unwrap();
    e.repo
        .edit(s.id(), "S", Anonymity::AnonymousOpen, 5)
        .await
        .unwrap();
    let (_, a) = e
        .repo
        .add_question(s.id(), pick().required())
        .await
        .unwrap();
    let mut band = Question::new(QuestionKind::Scale, "band");
    (band.min, band.max) = (0, 10);
    band.conditions = vec![Condition {
        question: a,
        op: ConditionOp::Equals,
        value: "a".into(),
    }];
    let (_, b) = e.repo.add_question(s.id(), band).await.unwrap();
    (e.repo.publish(s.id()).await.unwrap(), a, b)
}

fn with_band(a: ObjectId, b: ObjectId, v: i32) -> Vec<Answer> {
    vec![
        Answer {
            question: a,
            value: AnswerValue::Choice("a".into()),
        },
        Answer {
            question: b,
            value: AnswerValue::Scale(v),
        },
    ]
}

fn band(sum: &fs_surveys_core::Summary, b: ObjectId) -> (u32, bool, Vec<(String, u32)>) {
    let t = sum.questions.iter().find(|t| t.question == b).unwrap();
    (
        t.answers,
        t.hidden,
        t.counts
            .iter()
            .map(|c| (c.choice.clone(), c.count))
            .collect(),
    )
}

#[tokio::test]
async fn a_lone_answer_in_a_later_batch_stays_hidden_across_stages() {
    let e = env().await;
    let team = e.groups.create("Team", "t").await.unwrap();
    let root = e.groups.create("Root", "o").await.unwrap();
    e.groups.add_parent(team.id, root.id).await.unwrap();
    let (team, root): (ObjectId, ObjectId) = (
        team.id.to_string().parse().unwrap(),
        root.id.to_string().parse().unwrap(),
    );
    let (s, a, b) = conditional(&e, &[team]).await;
    for _ in 0..5 {
        e.repo
            .submit(s.id(), Credential::None, with_band(a, b, 1))
            .await
            .unwrap();
    }
    e.repo.widen(s.id(), root).await.unwrap();
    for _ in 0..4 {
        e.repo
            .submit(s.id(), Credential::None, choice(a, "b"))
            .await
            .unwrap();
    }
    e.repo
        .submit(s.id(), Credential::None, with_band(a, b, 9))
        .await
        .unwrap();
    e.repo.close(s.id()).await.unwrap();
    let all = band(&e.repo.summary(s.id(), ALL_STAGES).await.unwrap(), b);
    assert_eq!(
        all,
        (5, false, vec![("1".into(), 5)]),
        "the lone 9 is not in the all-stages view"
    );
    let one = band(&e.repo.summary(s.id(), 1).await.unwrap(), b);
    assert!(one.1 && one.2.is_empty(), "{one:?}");
}

#[tokio::test]
async fn closing_and_reopening_shows_no_lone_answer() {
    let e = env().await;
    let (s, a, b) = conditional(&e, &[]).await;
    for _ in 0..5 {
        e.repo
            .submit(s.id(), Credential::None, with_band(a, b, 1))
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let t1 = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    e.repo.reopen(s.id()).await.unwrap();
    for _ in 0..4 {
        e.repo
            .submit(s.id(), Credential::None, choice(a, "b"))
            .await
            .unwrap();
    }
    e.repo
        .submit(s.id(), Credential::None, with_band(a, b, 9))
        .await
        .unwrap();
    e.repo.close(s.id()).await.unwrap();
    let t2 = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(
        band(&t1, b),
        band(&t2, b),
        "the 9 stays out of the question's view"
    );
    assert_eq!(t2.ballots, 10, "the ballots themselves are released");
}

#[tokio::test]
async fn evaluation_follows_the_same_rule() {
    let e = env().await;
    let s = e
        .repo
        .create("S", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    e.repo
        .edit(s.id(), "S", Anonymity::AnonymousOpen, 3)
        .await
        .unwrap();
    let (_, a) = e.repo.add_question(s.id(), pick()).await.unwrap();
    let mut q2 = pick();
    q2.conditions = vec![Condition {
        question: a,
        op: ConditionOp::Equals,
        value: "a".into(),
    }];
    q2.text = "Then?".into();
    let (_, q2) = e.repo.add_question(s.id(), q2).await.unwrap();
    e.repo.publish(s.id()).await.unwrap();
    let two = |c: &str| {
        vec![
            Answer {
                question: a,
                value: AnswerValue::Choice("a".into()),
            },
            Answer {
                question: q2,
                value: AnswerValue::Choice(c.into()),
            },
        ]
    };
    for _ in 0..3 {
        e.repo
            .submit(s.id(), Credential::None, two("a"))
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    e.repo.reopen(s.id()).await.unwrap();
    for _ in 0..2 {
        e.repo
            .submit(s.id(), Credential::None, choice(a, "b"))
            .await
            .unwrap();
    }
    e.repo
        .submit(s.id(), Credential::None, two("c"))
        .await
        .unwrap();
    e.repo.close(s.id()).await.unwrap();
    let ev = e
        .repo
        .evaluate(s.id(), q2, Method::Plurality, ALL_STAGES)
        .await
        .unwrap();
    assert_eq!(ev.winners, ["a"]);
    let c = ev.options.iter().find(|o| o.choice == "c").unwrap();
    assert!(
        (c.value - 0.0).abs() < f64::EPSILON,
        "the lone c is not counted: {:?}",
        ev.options
    );
}
