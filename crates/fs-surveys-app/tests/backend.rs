//! The backends against the `Backend` trait of the frontend, over a real core with real stores.

mod common;

use common::{World, first, v};
use fs_admin_frontend::{Backend, BackendError, Values};
use fs_surveys_app::{QuestionBackend, ResponseBackend, RunError, SurveyBackend, question_chart};
use fs_types::BasicType::Text;
use fs_types::Edit;

#[test]
fn a_survey_is_created_edited_as_an_edit_and_listed() {
    let w = World::new();
    let surveys = SurveyBackend::new(w.ctx.clone());
    let made = surveys
        .run(
            "create",
            None,
            &v(&[("name", &["Lunch"]), ("anonymity", &["anonymous-open"])]),
        )
        .unwrap();
    let id = made.id.unwrap();
    assert!(id.starts_with("https://example.org/surveys/"), "{id}");
    let record = first(&surveys, &Values::new());
    assert_eq!(record.id, id);
    assert_eq!(record.values["anonymity"], ["anonymous-open"]);
    assert_eq!(record.values["state"], ["draft"]);
    // The rule: a field not in the edit stays; one with a value is replaced.
    let edit = Edit::new().set(&["name"], Text, &["Dinner"]);
    surveys.change(&id, &edit).unwrap();
    let after = first(&surveys, &Values::new());
    assert_eq!(after.values["name"], ["Dinner"]);
    assert_eq!(
        after.values["anonymity"],
        ["anonymous-open"],
        "untouched field stays"
    );
    // Search term of the standard find.
    assert!(
        surveys
            .list(&v(&[("term", &["nothing like it"])]))
            .unwrap()
            .is_empty()
    );
    assert_eq!(surveys.list(&v(&[("term", &["dinn"])])).unwrap().len(), 1);
}

#[test]
fn an_edit_of_a_field_the_survey_does_not_have_is_refused() {
    let w = World::new();
    let surveys = SurveyBackend::new(w.ctx.clone());
    let id = surveys
        .run("create", None, &v(&[("name", &["S"])]))
        .unwrap()
        .id
        .unwrap();
    let err = surveys
        .change(
            &id,
            &Edit::new().set(&["state"], fs_types::BasicType::Choice, &["open"]),
        )
        .unwrap_err();
    assert!(matches!(err, BackendError::Rejected(_)), "{err:?}");
}

#[test]
fn a_rule_of_the_core_is_a_refusal_not_a_crash() {
    let w = World::new();
    let surveys = SurveyBackend::new(w.ctx.clone());
    let id = surveys
        .run("create", None, &v(&[("name", &["S"])]))
        .unwrap()
        .id
        .unwrap();
    // A draft without questions cannot be closed.
    let err = surveys
        .try_run("close", Some(&id), &Values::new())
        .unwrap_err();
    assert!(matches!(err, RunError::Refused(_)), "{err:?}");
    assert!(matches!(
        surveys.run("close", Some(&id), &Values::new()),
        Err(BackendError::Rejected(_))
    ));
    // A missing record is not found.
    let ghost = "https://example.org/surveys/00000000-0000-7000-8000-000000000000";
    assert_eq!(
        surveys.run("publish", Some(ghost), &Values::new()),
        Err(BackendError::NotFound)
    );
}

#[test]
fn questions_answers_results_and_a_chart() {
    let w = World::new();
    let (surveys, questions, responses) = (
        SurveyBackend::new(w.ctx.clone()),
        QuestionBackend::new(w.ctx.clone()),
        ResponseBackend::new(w.ctx.clone()),
    );
    let sid = surveys
        .run("create", None, &v(&[("name", &["Lunch"])]))
        .unwrap()
        .id
        .unwrap();
    let qid = questions
        .run(
            "add",
            None,
            &v(&[
                ("survey", &[&sid]),
                ("text", &["Which?"]),
                ("kind", &["single-choice"]),
                ("required", &["yes"]),
                ("choices", &["a=Pizza", "b=Soup", "c=Salad"]),
            ]),
        )
        .unwrap()
        .id
        .unwrap();
    assert!(qid.starts_with(&format!("{sid}/questions/")));
    // Change one field of the question; the others stay (the rule of the Edit).
    questions
        .change(&qid, &Edit::new().set(&["text"], Text, &["Which one?"]))
        .unwrap();
    let q = first(&questions, &v(&[("survey", &[&sid])]));
    assert_eq!(q.values["text"], ["Which one?"]);
    assert_eq!(q.values["choices"].len(), 3);
    assert_eq!(q.values["required"], ["yes"]);
    surveys.run("publish", Some(&sid), &Values::new()).unwrap();
    let person = "https://example.org/users/owner";
    let answer = format!("{qid}=a");
    responses
        .run(
            "save",
            None,
            &v(&[
                ("survey", &[&sid]),
                ("person", &[person]),
                ("answers", &[&answer]),
            ]),
        )
        .unwrap();
    let saved = first(
        &responses,
        &v(&[("survey", &[&sid]), ("person", &[person])]),
    );
    assert_eq!(saved.values["submitted"], ["no"]);
    assert_eq!(saved.values["answers"], std::slice::from_ref(&answer));
    responses
        .run(
            "submit",
            None,
            &v(&[
                ("survey", &[&sid]),
                ("person", &[person]),
                ("answers", &[&answer]),
            ]),
        )
        .unwrap();
    let summary = surveys.run("summary", Some(&sid), &Values::new()).unwrap();
    assert_eq!(summary.values["ballots"], ["1"]);
    assert!(
        summary.values["counts"].iter().any(|l| l.ends_with(" a=1")),
        "{:?}",
        summary.values["counts"]
    );
    let evaluated = surveys
        .run(
            "evaluate",
            Some(&sid),
            &v(&[("question", &[&qid]), ("method", &["plurality"])]),
        )
        .unwrap();
    assert_eq!(evaluated.values["winners"], ["a"]);
    // The chart is made from the core's own summary and survey.
    let survey_id = qid
        .split("/questions/")
        .next()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let core_summary = w
        .ctx
        .block(w.ctx.repo.summary(survey_id, fs_surveys_core::ALL_STAGES))
        .unwrap();
    let survey = w.ctx.block(w.ctx.repo.get(survey_id)).unwrap().unwrap();
    // One named answer is under the minimum group of a result that may be shown, or it is shown: either way the
    // chart follows the summary and never invents figures.
    match question_chart(&survey.questions[0], &core_summary) {
        Ok(chart) => assert_eq!(
            chart.data().data.len(),
            core_summary.questions[0].counts.len()
        ),
        Err(e) => assert!(
            core_summary.hidden || core_summary.questions[0].hidden,
            "{e:?}"
        ),
    }
}
