//! Guarantee 6: conditions, validation and state transitions; each refusal.

mod common;

use common::*;
use fs_store::ObjectId;
use fs_surveys_core::{
    Anonymity, Answer, AnswerValue as V, Error, OptionScore, Question, QuestionKind as K, Survey,
};

fn ans(q: ObjectId, value: V) -> Answer {
    Answer { question: q, value }
}

async fn survey(e: &Env, qs: Vec<Question>) -> (Survey, Vec<ObjectId>) {
    let s = e
        .repo
        .create("S", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let mut ids = vec![];
    for q in qs {
        ids.push(e.repo.add_question(s.id(), q).await.unwrap().1);
    }
    (e.repo.publish(s.id()).await.unwrap(), ids)
}

fn scale() -> Question {
    let mut q = Question::new(K::Scale, "Mood");
    (q.min, q.max) = (1, 5);
    q
}

async fn refused(e: &Env, s: &Survey, a: Vec<Answer>) {
    let r = e.repo.submit(s.id(), human(), a).await;
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
}

#[tokio::test]
async fn answers_are_validated() {
    let e = env().await;
    let mut num = Question::new(K::Number, "Age");
    (num.min_number, num.max_number) = (Some(0.0), Some(150.0));
    let mut scores = Question::new(K::OptionScores, "Rate").with_choices(&[("a", "A"), ("b", "B")]);
    (scores.min, scores.max) = (0, 10);
    let rank = Question::new(K::Ranking, "Rank").with_choices(&[("a", "A"), ("b", "B")]);
    let multi = Question::new(K::MultipleChoice, "Many").with_choices(&[("a", "A"), ("b", "B")]);
    let (s, id) = survey(
        &e,
        vec![
            pick().required(),
            scale(),
            num,
            scores,
            rank,
            Question::new(K::Date, "When"),
            multi,
        ],
    )
    .await;
    let (p, sc, n, so, r, d, m) = (id[0], id[1], id[2], id[3], id[4], id[5], id[6]);
    let ok = || choice(p, "a");
    refused(&e, &s, vec![]).await; // required missing
    refused(&e, &s, vec![ans(p, V::Choice("zzz".into()))]).await; // unknown choice
    refused(&e, &s, vec![ans(p, V::Text("a".into()))]).await; // wrong arm
    refused(&e, &s, vec![ans(ObjectId::new(), V::Scale(1))]).await; // unknown question
    refused(&e, &s, [ok(), ok()].concat()).await; // answered twice
    for bad in [V::Scale(0), V::Scale(6)] {
        refused(&e, &s, [ok(), vec![ans(sc, bad)]].concat()).await;
    }
    for bad in [-1.0, 151.0, f64::NAN] {
        refused(&e, &s, [ok(), vec![ans(n, V::Number(bad))]].concat()).await;
    }
    let sc2 = |c: &str, v| {
        V::Scores(vec![OptionScore {
            choice: c.into(),
            score: v,
        }])
    };
    refused(&e, &s, [ok(), vec![ans(so, sc2("a", 11))]].concat()).await;
    refused(&e, &s, [ok(), vec![ans(so, sc2("x", 1))]].concat()).await;
    refused(
        &e,
        &s,
        [ok(), vec![ans(r, V::Ranking(vec!["a".into(), "a".into()]))]].concat(),
    )
    .await;
    refused(
        &e,
        &s,
        [ok(), vec![ans(r, V::Ranking(vec!["x".into()]))]].concat(),
    )
    .await;
    refused(
        &e,
        &s,
        [ok(), vec![ans(m, V::Choices(vec!["a".into(), "a".into()]))]].concat(),
    )
    .await;
    for bad in [
        "2026-13-01",
        "2026-1-5",
        "2026-02-30",
        "26-01-01",
        "2026-01-01T00:00:00Z",
        "today",
    ] {
        refused(&e, &s, [ok(), vec![ans(d, V::Date(bad.into()))]].concat()).await;
    }
    let good = vec![
        ans(p, V::Choice("b".into())),
        ans(sc, V::Scale(5)),
        ans(n, V::Number(42.0)),
        ans(d, V::Date("2026-02-28".into())),
        ans(r, V::Ranking(vec!["b".into(), "a".into()])),
        ans(so, sc2("a", 10)),
        ans(m, V::Choices(vec!["a".into()])),
    ];
    e.repo.submit(s.id(), human(), good).await.unwrap();
}
