//! Decisions over a real core.

mod common;

use common::{World, first, v};
use fs_admin_frontend::{Backend, Values};
use fs_surveys_app::{DecisionBackend, QuestionBackend, SurveyBackend};

#[test]
fn a_decision_is_recorded_linked_and_found() {
    let w = World::new();
    let (surveys, questions, decisions) = (
        SurveyBackend::new(w.ctx.clone()),
        QuestionBackend::new(w.ctx.clone()),
        DecisionBackend::new(w.ctx.clone()),
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
                ("choices", &["a=Pizza", "b=Soup"]),
            ]),
        )
        .unwrap()
        .id
        .unwrap();
    let did = decisions
        .run(
            "record",
            None,
            &v(&[
                ("survey", &[&sid]),
                ("question", &[&qid]),
                ("method", &["majority"]),
                ("outcome", &["a"]),
                ("note", &["Pizza it is"]),
            ]),
        )
        .unwrap()
        .id
        .unwrap();
    let task = "https://example.org/tasks/42";
    decisions
        .run("linkTask", Some(&did), &v(&[("task", &[task])]))
        .unwrap();
    let by_task = first(&decisions, &v(&[("task", &[task])]));
    assert_eq!(by_task.id, did);
    assert_eq!(by_task.values["tasks"], [task]);
    assert_eq!(by_task.values["outcome"], ["a"]);
    decisions
        .run("unlinkTask", Some(&did), &v(&[("task", &[task])]))
        .unwrap();
    assert!(decisions.list(&v(&[("task", &[task])])).unwrap().is_empty());
    assert_eq!(decisions.list(&Values::new()).unwrap().len(), 1);
}
