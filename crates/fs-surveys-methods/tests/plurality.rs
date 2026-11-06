//! Plurality and the errors shared by all methods.
mod common;
use common::*;
use fs_surveys_methods::{Error, Method, evaluate};

#[test]
fn most_votes_wins() {
    let t = evaluate(
        &Method::Plurality,
        &ids(&["a", "b", "c"]),
        &singles(&[("a", 2), ("b", 3), ("c", 1)]),
    )
    .unwrap();
    assert_eq!(winners(&t), ["b"]);
    assert_eq!(order(&t), ["b", "a", "c"]);
    assert!((value(&t, "b") - 3.0).abs() < f64::EPSILON);
}

#[test]
fn tie_lists_all_and_ranks_share() {
    let t = evaluate(
        &Method::Plurality,
        &ids(&["a", "b", "c", "d"]),
        &singles(&[("b", 2), ("c", 2), ("d", 1)]),
    )
    .unwrap();
    assert_eq!(winners(&t), ["b", "c"]);
    assert_eq!(order(&t), ["b", "c", "d", "a"]);
    assert_eq!(
        [rank(&t, "b"), rank(&t, "c"), rank(&t, "d"), rank(&t, "a")],
        [1, 1, 3, 4]
    );
    assert!(!t.note.is_empty());
}

#[test]
fn equal_values_sort_by_choice_order() {
    let t = evaluate(
        &Method::Plurality,
        &ids(&["x", "y", "z"]),
        &singles(&[("z", 1)]),
    )
    .unwrap();
    assert_eq!(order(&t), ["z", "x", "y"]);
    assert_eq!([rank(&t, "z"), rank(&t, "x"), rank(&t, "y")], [1, 2, 2]);
}

#[test]
fn no_ballots_is_undecided_for_every_method() {
    let methods = [
        Method::Plurality,
        Method::Majority,
        Method::Supermajority {
            threshold_percent: 60,
        },
        Method::Approval,
        Method::Score,
        Method::Schulze,
        Method::InstantRunoff,
        Method::Borda,
        Method::Consent,
        Method::SystemicConsensing,
    ];
    for m in methods {
        let t = evaluate(&m, &ids(&["a", "b"]), &[]).unwrap();
        assert!(t.winners.is_empty(), "{m:?}");
        assert!(!t.note.is_empty(), "{m:?}");
        assert_eq!(t.options.len(), 2, "{m:?}");
    }
}

#[test]
fn no_choices_is_an_error_for_every_method() {
    assert_eq!(
        evaluate(&Method::Plurality, &[], &[]),
        Err(Error::NoChoices)
    );
    assert_eq!(
        evaluate(&Method::Schulze, &[], &[ranking(&[])]),
        Err(Error::NoChoices)
    );
}

#[test]
fn wrong_ballot_form_is_refused() {
    let r = evaluate(&Method::Plurality, &ids(&["a"]), &[ranking(&["a"])]);
    assert!(matches!(r, Err(Error::WrongBallot { .. })));
    let r = evaluate(&Method::Borda, &ids(&["a"]), &[single("a")]);
    assert!(matches!(r, Err(Error::WrongBallot { .. })));
}

#[test]
fn unknown_choice_is_refused() {
    let r = evaluate(&Method::Plurality, &ids(&["a"]), &[single("zz")]);
    assert_eq!(r, Err(Error::UnknownChoice));
    let r = evaluate(&Method::Schulze, &ids(&["a"]), &[ranking(&["a", "q"])]);
    assert_eq!(r, Err(Error::UnknownChoice));
}

#[test]
fn duplicates_in_one_ballot_are_invalid() {
    for (m, b) in [
        (Method::Approval, multi(&["a", "a"])),
        (Method::Borda, ranking(&["a", "a"])),
        (Method::Score, scores(&[("a", 1), ("a", 2)])),
    ] {
        assert!(
            matches!(
                evaluate(&m, &ids(&["a", "b"]), &[b]),
                Err(Error::InvalidBallot(_))
            ),
            "{m:?}"
        );
    }
}
