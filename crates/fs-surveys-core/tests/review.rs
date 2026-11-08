//! Review: option score totals, empty answers, CSV cells, errors without values, AS2 shape, retry.

mod common;

use common::*;
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Answer, AnswerValue, Credential, Error, Method, OptionScore, Question,
    QuestionKind as K,
};

fn scores(q: fs_objects::ObjectId, c: &str, v: i32) -> Answer {
    Answer {
        question: q,
        value: AnswerValue::Scores(vec![OptionScore {
            choice: c.into(),
            score: v,
        }]),
    }
}

#[tokio::test]
async fn option_scores_count_the_answers_that_scored_each_choice() {
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
    let mut o =
        Question::new(K::OptionScores, "o").with_choices(&[("x", "X"), ("=HYPERLINK(1)", "Y")]);
    (o.min, o.max) = (0, 10);
    let (_, q) = e.repo.add_question(s.id(), o).await.unwrap();
    e.repo.publish(s.id()).await.unwrap();
    for v in [7, 8, 9] {
        e.repo
            .submit(s.id(), human(), vec![scores(q, "x", v)])
            .await
            .unwrap();
    }
    e.repo.close(s.id()).await.unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(
        (count(&sum, q, "x"), count(&sum, q, "=HYPERLINK(1)")),
        (3, 0)
    );
    let csv = String::from_utf8(e.repo.export(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(csv.contains("o,x,3"), "{csv}");
    assert!(
        csv.contains("o,'=HYPERLINK(1),0"),
        "choice ids are neutralised too: {csv}"
    );
    let err = e
        .repo
        .evaluate(s.id(), q, Method::Consent, ALL_STAGES)
        .await
        .unwrap_err();
    assert!(
        matches!(err, Error::Invalid(_)),
        "a 0..10 question does not suit consent: {err}"
    );
    assert!(
        e.repo
            .evaluate(s.id(), q, Method::SystemicConsensing, ALL_STAGES)
            .await
            .is_ok()
    );
}

#[allow(clippy::many_single_char_names)] // short fixture names (env, survey, question)
#[tokio::test]
async fn required_questions_refuse_empty_answers() {
    let e = env().await;
    let s = e
        .repo
        .create("S", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let (_, t) = e
        .repo
        .add_question(s.id(), Question::new(K::Text, "t").required())
        .await
        .unwrap();
    let multi = Question::new(K::MultipleChoice, "m")
        .with_choices(&[("a", "A"), ("b", "B")])
        .required();
    let (_, m) = e.repo.add_question(s.id(), multi).await.unwrap();
    e.repo.publish(s.id()).await.unwrap();
    let text = |v: &str| Answer {
        question: t,
        value: AnswerValue::Text(v.into()),
    };
    let list = |v: &[&str]| Answer {
        question: m,
        value: AnswerValue::Choices(v.iter().map(ToString::to_string).collect()),
    };
    for bad in [
        vec![text(""), list(&["a"])],
        vec![text("  "), list(&["a"])],
        vec![text("x"), list(&[])],
    ] {
        let r = e.repo.submit(s.id(), human(), bad).await;
        assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
    }
    e.repo
        .submit(s.id(), human(), vec![text("x"), list(&["a"])])
        .await
        .unwrap();
}

#[tokio::test]
async fn csv_quotes_semicolons_and_tabs_and_neutralises_every_cell_from_people() {
    let e = env().await;
    let s = e.repo.create("S", Anonymity::Named, &[]).await.unwrap();
    let (_, t) = e
        .repo
        .add_question(s.id(), Question::new(K::Text, "t"))
        .await
        .unwrap();
    let pick = Question::new(K::SingleChoice, "p").with_choices(&[("=evil", "E"), ("ok", "O")]);
    let (_, p) = e.repo.add_question(s.id(), pick).await.unwrap();
    let mut sc = Question::new(K::Scale, "sc");
    (sc.min, sc.max) = (-5, 5);
    let (_, sc) = e.repo.add_question(s.id(), sc).await.unwrap();
    e.repo.publish(s.id()).await.unwrap();
    let ans = vec![
        Answer {
            question: t,
            value: AnswerValue::Text("a;b\tc".into()),
        },
        Answer {
            question: p,
            value: AnswerValue::Choice("=evil".into()),
        },
        Answer {
            question: sc,
            value: AnswerValue::Scale(-3),
        },
    ];
    e.repo
        .submit(s.id(), Credential::Person(person("ada")), ans)
        .await
        .unwrap();
    let csv = String::from_utf8(e.repo.export(s.id(), "text/csv").await.unwrap()).unwrap();
    assert!(csv.contains("\"a;b\tc\""), "{csv}");
    assert!(csv.contains("'=evil"), "{csv}");
    assert!(
        csv.trim_end().ends_with(",-3"),
        "a genuine number stays numeric: {csv}"
    );
}

#[tokio::test]
async fn evaluation_errors_carry_no_ballot_value() {
    let e = env().await;
    let s = e.repo.create("S", Anonymity::Named, &[]).await.unwrap();
    let (_, q) = e.repo.add_question(s.id(), pick()).await.unwrap();
    e.repo.publish(s.id()).await.unwrap();
    e.repo
        .submit(s.id(), Credential::Person(person("ada")), choice(q, "a"))
        .await
        .unwrap();
    let err = e
        .repo
        .evaluate(s.id(), q, Method::Borda, ALL_STAGES)
        .await
        .unwrap_err();
    let text = format!("{err} {err:?}");
    assert!(!text.contains("\"a\""), "{text}");
}

#[tokio::test]
async fn the_survey_is_an_ordered_collection_with_ordered_items() {
    let e = env().await;
    let (s, _) = open_survey(&e, Anonymity::Named).await;
    let as2 = s.as2();
    assert_eq!(as2["type"], "OrderedCollection");
    assert!(as2.get("items").is_none() && as2["orderedItems"].as_array().unwrap().len() == 1);
    assert_eq!(
        e.repo.get(s.id()).await.unwrap().unwrap().questions.len(),
        1,
        "read back"
    );
}
