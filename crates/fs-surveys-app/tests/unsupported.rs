//! The gate: commands that are described but not carried out are refused with a typed problem.

mod common;

use common::World;
use fs_admin_frontend::{Backend, Values};
use fs_surveys_app::{DecisionBackend, QuestionBackend, ResponseBackend, RunError, SurveyBackend};
use fs_surveys_core::commands;

#[test]
fn every_unsupported_command_is_refused_with_a_typed_problem_and_reported_locked() {
    let w = World::new();
    type Probe = Box<dyn Fn(&str) -> (bool, Result<(), RunError>)>;
    let backends: [(&str, Probe); 4] = {
        let (s, q, r, d) = (
            SurveyBackend::new(w.ctx.clone()),
            QuestionBackend::new(w.ctx.clone()),
            ResponseBackend::new(w.ctx.clone()),
            DecisionBackend::new(w.ctx.clone()),
        );
        [
            (
                "Survey",
                Box::new(move |c| {
                    (
                        s.supported(c),
                        s.try_run(c, None, &Values::new()).map(|_| ()),
                    )
                }),
            ),
            (
                "Question",
                Box::new(move |c| {
                    (
                        q.supported(c),
                        q.try_run(c, None, &Values::new()).map(|_| ()),
                    )
                }),
            ),
            (
                "SurveyResponse",
                Box::new(move |c| {
                    (
                        r.supported(c),
                        r.try_run(c, None, &Values::new()).map(|_| ()),
                    )
                }),
            ),
            (
                "Decision",
                Box::new(move |c| {
                    (
                        d.supported(c),
                        d.try_run(c, None, &Values::new()).map(|_| ()),
                    )
                }),
            ),
        ]
    };
    let mut seen = 0;
    for (ty, name) in commands::unsupported_commands() {
        let (_, probe) = backends.iter().find(|(t, _)| *t == ty).unwrap();
        let (supported, outcome) = probe(name);
        assert!(!supported, "{ty}.{name} says it is supported");
        match outcome {
            Err(RunError::Unsupported {
                data_type,
                command,
                reason,
            }) => {
                assert_eq!((data_type, command), (ty, name));
                assert!(!reason.is_empty());
            }
            other => panic!("{ty}.{name}: {other:?}"),
        }
        seen += 1;
    }
    assert!(seen >= 10, "the table has many missing commands");
    // And the supported ones are reported as free.
    let (supported, _) = (backends[0].1)("create");
    assert!(supported);
}
