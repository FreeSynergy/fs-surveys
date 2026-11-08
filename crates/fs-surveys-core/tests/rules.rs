//! Guarantee 4: what cannot change or be seen on anonymous surveys.

mod common;

use common::*;
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Credential, Error, Question, QuestionKind, Survey, SurveyState,
};

async fn with_number(
    e: &Env,
    a: Anonymity,
) -> (Survey, fs_objects::ObjectId, fs_objects::ObjectId) {
    let s = e.repo.create("Pay", a, &[]).await.unwrap();
    e.repo.edit(s.id(), "Pay", a, 3).await.unwrap();
    let (_, q) = e.repo.add_question(s.id(), pick()).await.unwrap();
    let (_, n) = e
        .repo
        .add_question(s.id(), Question::new(QuestionKind::Number, "Salary"))
        .await
        .unwrap();
    (e.repo.publish(s.id()).await.unwrap(), q, n)
}

#[tokio::test]
async fn anonymity_cannot_change_after_publish() {
    let e = env().await;
    let (s, _) = open_survey(&e, Anonymity::AnonymousOnce).await;
    for to in [Anonymity::Named, Anonymity::AnonymousOpen] {
        let err = e.repo.edit(s.id(), "Lunch", to, 5).await.unwrap_err();
        assert!(matches!(err, Error::Refused(_)), "{err}");
    }
    assert_eq!(
        e.repo.get(s.id()).await.unwrap().unwrap().anonymity,
        Anonymity::AnonymousOnce
    );
}

#[tokio::test]
async fn min_group_floor_holds_everywhere() {
    let e = env().await;
    let s = e
        .repo
        .create("Lunch", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    assert_eq!(s.min_group, 5, "default");
    for n in [0, 1, 2] {
        let err = e
            .repo
            .edit(s.id(), "Lunch", Anonymity::AnonymousOpen, n)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Refused(_)));
    }
    e.repo
        .edit(s.id(), "Lunch", Anonymity::AnonymousOpen, 3)
        .await
        .unwrap();
    // a named survey may have 1, but then it cannot become anonymous without a floor
    let n = e.repo.create("Open", Anonymity::Named, &[]).await.unwrap();
    e.repo
        .edit(n.id(), "Open", Anonymity::Named, 1)
        .await
        .unwrap();
    let err = e
        .repo
        .edit(n.id(), "Open", Anonymity::AnonymousOnce, 1)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Refused(_)));
}

#[tokio::test]
async fn filter_and_rows_are_refused_and_numbers_are_hidden() {
    let e = env().await;
    let (s, q, n) = with_number(&e, Anonymity::AnonymousOpen).await;
    for (c, v) in [("a", 100.0), ("a", 200.0), ("b", 300.0)] {
        let mut ans = choice(q, c);
        ans.push(fs_surveys_core::Answer {
            question: n,
            value: fs_surveys_core::AnswerValue::Number(v),
        });
        e.repo.submit(s.id(), Credential::None, ans).await.unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let err = e.repo.filter(s.id(), q, "a", ALL_STAGES).await.unwrap_err();
    assert!(matches!(err, Error::Refused(_)));
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    let num = &sum.questions[1];
    assert!(num.counts.is_empty(), "no number values or counts");
    assert_eq!(num.mean, Some(200.0));
    let csv = String::from_utf8(e.repo.export(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(!csv.contains("300") && !csv.contains("person"));
    assert!(raw(&e.main).await.contains("\"fs:state\":\"closed\""));
}

#[tokio::test]
async fn named_shows_values_and_filters() {
    let e = env().await;
    let (s, q, n) = with_number(&e, Anonymity::Named).await;
    for (p, c, v) in [("ada", "a", 100.0), ("bob", "b", 300.0)] {
        let mut ans = choice(q, c);
        ans.push(fs_surveys_core::Answer {
            question: n,
            value: fs_surveys_core::AnswerValue::Number(v),
        });
        e.repo
            .submit(s.id(), Credential::Person(person(p)), ans)
            .await
            .unwrap();
    }
    let sum = e.repo.filter(s.id(), q, "b", ALL_STAGES).await.unwrap();
    assert_eq!(sum.ballots, 1);
    assert!(!sum.hidden, "minGroup does not apply to named");
    assert_eq!(sum.questions[1].counts[0].choice, "300");
    assert_eq!(
        e.repo.get(s.id()).await.unwrap().unwrap().state,
        SurveyState::Open
    );
}

#[tokio::test]
async fn csv_is_escaped_and_formulas_are_neutralised() {
    let e = env().await;
    let s = e.repo.create("Notes", Anonymity::Named, &[]).await.unwrap();
    let (_, q) = e
        .repo
        .add_question(s.id(), Question::new(QuestionKind::Text, "Say, \"it\""))
        .await
        .unwrap();
    e.repo.publish(s.id()).await.unwrap();
    for (p, t) in [
        ("ada", "=SUM(A1)"),
        ("bob", "line\nbreak, \"q\""),
        ("cy", "-1+2"),
        ("di", "@x"),
        ("ed", "+1"),
    ] {
        let a = vec![fs_surveys_core::Answer {
            question: q,
            value: fs_surveys_core::AnswerValue::Text(t.into()),
        }];
        e.repo
            .submit(s.id(), Credential::Person(person(p)), a)
            .await
            .unwrap();
    }
    let csv = String::from_utf8(e.repo.export(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(csv.starts_with("person,\"Say, \"\"it\"\"\"\r\n"), "{csv}");
    for bad in [
        "'=SUM(A1)",
        "'-1+2",
        "'@x",
        "'+1",
        "\"line\nbreak, \"\"q\"\"\"",
    ] {
        assert!(csv.contains(bad), "{bad} in {csv}");
    }
    assert!(csv.contains("people/ada,"), "named export has the person");
    let err = e.repo.export(s.id(), "application/json").await.unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
}
