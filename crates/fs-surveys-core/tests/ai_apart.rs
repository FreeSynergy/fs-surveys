//! AI participants: counted and shown apart. No total, summary, evaluation (all ten methods), filter,
//! CSV or event of the people holds an AI answer; the AI part has its own.

mod common;

use std::sync::{Arc, Mutex};

use common::ai::*;
use common::*;
use fs_objects::ObjectId;
use fs_surveys_core::Results as _;
use fs_surveys_core::{ALL_STAGES, Evaluation, Survey, SurveyChange, SurveyFeed, SurveyListener};

async fn eval(e: &Env, s: &Survey, q: ObjectId, m: fs_surveys_core::Method) -> Evaluation {
    e.repo.evaluate(s.id(), q, m, ALL_STAGES).await.unwrap()
}

fn same(a: &Evaluation, b: &Evaluation) -> bool {
    (&a.winners, &a.options, &a.note, a.held_back, a.hidden)
        == (&b.winners, &b.options, &b.note, b.held_back, b.hidden)
}

#[tokio::test]
async fn all_ten_methods_count_people_and_ais_apart() {
    let e = env().await;
    let (mixed, qm) = fixture(&e, true).await;
    fill(&e, &mixed, &qm, 5, 4).await;
    let (people_only, qp) = fixture(&e, false).await;
    fill(&e, &people_only, &qp, 5, 0).await;
    let (ais_only, qa) = fixture(&e, true).await;
    fill(&e, &ais_only, &qa, 0, 4).await;
    for (m, i) in TEN {
        let got = eval(&e, &mixed, qm[i], m).await;
        let people = eval(&e, &people_only, qp[i], m).await;
        let ais = eval(&e, &ais_only, qa[i], m).await.ai.expect("AI part");
        assert!(
            same(&got, &people),
            "{m:?}: the people's part holds an AI answer"
        );
        assert!(
            people.ai.is_none(),
            "{m:?}: no AI part when none is allowed"
        );
        let part = got.ai.as_deref().expect("AI part");
        assert!(
            same(part, &ais),
            "{m:?}: the AI part is the AI answers alone"
        );
        assert!(part.ai.is_none(), "{m:?}: never nested");
        assert!(!same(part, &got), "{m:?}: the fixture tells the two apart");
    }
}

#[tokio::test]
async fn the_summary_totals_are_apart() {
    let e = env().await;
    let (s, q) = fixture(&e, true).await;
    fill(&e, &s, &q, 5, 4).await;
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 5);
    assert_eq!(count(&sum, q[0], "a"), 5);
    assert_eq!(count(&sum, q[0], "c"), 0);
    assert_eq!(sum.questions[0].answers, 5);
    let ai = sum.ai.expect("AI part");
    assert_eq!(
        (ai.ballots, count(&ai, q[0], "c"), count(&ai, q[0], "a")),
        (4, 4, 0)
    );
    assert_eq!(ai.questions[0].answers, 4);
    assert!(ai.ai.is_none());
}

#[tokio::test]
async fn an_allowed_survey_shows_an_empty_ai_part() {
    let e = env().await;
    let (s, q) = survey(&e, fs_surveys_core::Anonymity::Named, true).await;
    many(&e, &s, q, "a", 2, false).await;
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(sum.ai.map(|a| a.ballots), Some(0));
}

#[tokio::test]
async fn csv_export_is_apart() {
    let e = env().await;
    let (s, q) = survey(&e, fs_surveys_core::Anonymity::Named, true).await;
    many(&e, &s, q, "a", 2, false).await;
    many(&e, &s, q, "c", 2, true).await;
    let people = String::from_utf8(e.repo.export(s.id(), "text/csv").await.unwrap()).unwrap();
    let ai = String::from_utf8(e.repo.export_ai(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(
        people.contains("/people/h0") && !people.contains("bot-"),
        "{people}"
    );
    assert!(!people.lines().any(|l| l.ends_with(",c")), "{people}");
    assert!(ai.contains("bot-0") && !ai.contains("/people/h"), "{ai}");
    assert_eq!(
        ai.lines().next().unwrap().split(',').next(),
        Some("participants")
    );
    assert!(ai.lines().skip(1).all(|l| l.starts_with("ai,")), "{ai}");
    assert_eq!(people.lines().count(), 3, "header and two people");
    assert_eq!(ai.lines().count(), 3, "header and two AIs");
}

#[tokio::test]
async fn filter_is_apart() {
    let e = env().await;
    let (s, q) = fixture(&e, true).await;
    fill(&e, &s, &q, 3, 2).await;
    let f = e.repo.filter(s.id(), q[0], "a", ALL_STAGES).await.unwrap();
    assert_eq!(f.ballots, 3);
    assert_eq!(f.ai.unwrap().ballots, 0, "no AI chose a");
    let f = e.repo.filter(s.id(), q[0], "c", ALL_STAGES).await.unwrap();
    assert_eq!(f.ballots, 0);
    assert_eq!(f.ai.unwrap().ballots, 2);
}

#[derive(Default)]
struct Seen(Mutex<Vec<SurveyChange>>);

impl SurveyListener for Seen {
    fn changed(&self, c: &SurveyChange) {
        self.0.lock().unwrap().push(c.clone());
    }
}

#[tokio::test]
async fn events_are_apart() {
    let e = env().await;
    let (s, q) = survey(&e, fs_surveys_core::Anonymity::Named, true).await;
    let seen = Arc::new(Seen::default());
    let _sub = e.repo.subscribe(seen.clone());
    say(&e, &s, &person("ada"), choice(q, "a")).await.unwrap();
    say(&e, &s, &bot("1"), choice(q, "c")).await.unwrap();
    let got = seen.0.lock().unwrap().clone();
    let answered = |f: fn(&SurveyChange) -> bool| got.iter().filter(|c| f(c)).count();
    assert_eq!(answered(|c| matches!(c, SurveyChange::Answered(..))), 1);
    assert_eq!(answered(|c| matches!(c, SurveyChange::AiAnswered(..))), 1);
}
