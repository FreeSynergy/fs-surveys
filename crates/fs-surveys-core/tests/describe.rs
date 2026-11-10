//! The type descriptions against the traits and the requirements (ADR 0066).

mod common;

use std::collections::BTreeSet;

use common::env;
use fs_surveys_core::commands::{self, Command};
use fs_surveys_core::describe::{command_description, registry};
use fs_surveys_core::{Surveys, describe};
use fs_types::{BasicRules, DescribesType, TypeDescription};

/// The copy of `docs/en/programs/apps/surveys/requirements.md` this was written against.
const SNAPSHOT: &str = include_str!("data/requirements.md");

/// Command ids named in backticks: `survey.create`, `poll.embed`, and the bare `subscribe`.
fn requirement_ids(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for part in text.split('`').skip(1).step_by(2) {
        let dotted = part.split_once('.').is_some_and(|(a, b)| {
            !a.is_empty()
                && !b.is_empty()
                && a.chars().all(|c| c.is_ascii_lowercase())
                && b.chars().all(|c| c.is_ascii_alphabetic())
        });
        if dotted || part == "subscribe" {
            ids.insert(part.to_owned());
        }
    }
    ids
}

/// Ids of requirements.md or of the contract calls that the capability report of the core does not list.
const NOT_IN_REPORT: [&str; 2] = ["response.voluntary", "question.types"];

fn all_described() -> Vec<TypeDescription> {
    vec![
        fs_surveys_core::model::Survey::describe(),
        fs_surveys_core::model::Question::describe(),
        fs_surveys_core::model::NamedResponse::describe(),
        fs_surveys_core::model::Decision::describe(),
    ]
}

#[test]
fn every_requirement_command_is_in_the_table() {
    let wanted = requirement_ids(SNAPSHOT);
    // A guard against a regex that reads nothing: the file names at least these.
    assert!(wanted.len() >= 35, "read only {wanted:?}");
    for id in [
        "survey.create",
        "result.summary",
        "subscribe",
        "vote.delegate",
    ] {
        assert!(wanted.contains(id), "{id} not read from the file");
    }
    let table: BTreeSet<String> = commands::requirement_ids()
        .into_iter()
        .map(String::from)
        .collect();
    let missing: Vec<_> = wanted.difference(&table).collect();
    assert!(
        missing.is_empty(),
        "requirement commands not in the table: {missing:?}"
    );
}

#[test]
fn the_snapshot_is_the_current_document_when_it_can_be_compared() {
    // Set `FS_SURVEYS_REQUIREMENTS` to the file in fs-documentation to see drift; absent, the snapshot stands.
    if let Ok(path) = std::env::var("FS_SURVEYS_REQUIREMENTS") {
        let current = std::fs::read_to_string(path).unwrap();
        assert_eq!(requirement_ids(&current), requirement_ids(SNAPSHOT));
    }
}

#[test]
fn the_description_holds_every_command_of_the_table_supported_or_not() {
    for t in all_described() {
        let rows = commands::for_type(&t.name);
        assert!(!rows.is_empty(), "{} has no commands", t.name);
        let names: Vec<&str> = t.commands.iter().map(|c| c.name.as_str()).collect();
        let wanted: Vec<&str> = rows.iter().map(|c| c.name).collect();
        assert_eq!(names, wanted, "{}", t.name);
    }
    let described: usize = all_described().iter().map(|t| t.commands.len()).sum();
    assert_eq!(described, commands::all().len());
    // The unsupported ones are in it, as asked by the coordinator.
    let survey = fs_surveys_core::model::Survey::describe();
    for name in ["publicLink", "embed", "delegate", "modes"] {
        assert!(survey.find_command(name).is_some(), "{name}");
    }
    assert!(!commands::unsupported_commands().is_empty());
}

#[test]
fn the_registry_accepts_the_four_types_and_their_targets_resolve_but_for_other_programs() {
    let r = registry().expect("descriptions are valid");
    assert_eq!(r.len(), 4);
    let others: BTreeSet<String> = r
        .unresolved_targets()
        .into_iter()
        .map(|(_, _, target)| target)
        .collect();
    // Group is owned by fs-groups, Person by fs-people, Task by fs-tasks (ADR 0050): not ours to describe.
    assert_eq!(others, ["Group", "Person", "Task"].map(String::from).into());
}

#[test]
fn a_table_row_and_its_description_say_the_same() {
    let c: &Command = commands::for_type("Survey")
        .into_iter()
        .find(|c| c.name == "evaluate")
        .unwrap();
    let d = command_description(c);
    assert_eq!(d.input.len(), c.input.len());
    assert_eq!(d.result.len(), c.result.len());
    assert!(
        d.input
            .iter()
            .any(|f| f.name == "method" && f.required && f.choices.len() == 10)
    );
}

#[test]
fn change_is_an_edit_and_a_delete_names_its_record() {
    // Rule of 2026-11-10 (types.capnp): `change` takes an `Edit` and no input list; a record command
    // refers to the type itself.
    use fs_types::CommandKind::{Change, Delete};
    for t in all_described() {
        for c in &t.commands {
            match c.kind {
                Change => assert!(
                    c.input.is_empty(),
                    "{}.{} is a change with input",
                    t.name,
                    c.name
                ),
                Delete => assert!(
                    c.input.iter().any(|f| f.target == t.name),
                    "{}.{}",
                    t.name,
                    c.name
                ),
                _ => {}
            }
        }
        // `record_commands` is what the frontend puts on a record: every command that takes an id of the type.
        for c in t.record_commands() {
            assert!(
                matches!(c.kind, Change | Delete) || c.input.iter().any(|f| f.target == t.name),
                "{}.{}",
                t.name,
                c.name
            );
        }
    }
}

#[test]
fn the_flag_travels_in_the_description() {
    let survey = fs_surveys_core::model::Survey::describe();
    for c in &survey.commands {
        let row = commands::for_type("Survey")
            .into_iter()
            .find(|r| r.name == c.name)
            .unwrap();
        assert_eq!(c.supported, row.supported, "{}", c.name);
    }
    assert!(!survey.find_command("publicLink").unwrap().supported);
    assert!(survey.find_command("create").unwrap().supported);
    let find = survey.find_command("find").unwrap();
    assert!(
        find.input.iter().any(|f| f.name == "term"),
        "the standard find"
    );
}

#[test]
fn a_command_marked_supported_is_one_the_core_reports_as_supported() {
    let e = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(env());
    let caps = e.repo.capabilities();
    for c in commands::all().into_iter().filter(|c| c.supported) {
        for id in c
            .requirements
            .iter()
            .filter(|id| !NOT_IN_REPORT.contains(id))
        {
            assert!(
                caps.is_supported(id),
                "{}.{} is marked supported but the core reports {id} as not",
                c.data_type,
                c.name
            );
        }
    }
}

#[test]
fn an_unsupported_command_gives_a_reason_and_a_supported_one_does_not() {
    for c in commands::all() {
        assert_eq!(
            c.reason.is_empty(),
            c.supported,
            "{}.{}",
            c.data_type,
            c.name
        );
    }
}

#[test]
fn a_record_of_typed_values_is_checked_by_the_description() {
    let survey = fs_surveys_core::model::Survey::describe();
    let rules = BasicRules::standard();
    let mut ok = std::collections::BTreeMap::new();
    ok.insert("name".to_owned(), vec!["Lunch".to_owned()]);
    ok.insert("anonymity".to_owned(), vec!["anonymous-open".to_owned()]);
    assert!(survey.check_record(&rules, &ok).is_empty());
    ok.insert("anonymity".to_owned(), vec!["anonymousOpen".to_owned()]);
    assert_eq!(
        survey.check_record(&rules, &ok).len(),
        1,
        "camel case is not a key"
    );
    let _ = describe::camel("anonymous-open");
}
