//! Majority and supermajority.
mod common;
use common::*;
use fs_surveys_methods::{Error, Method, evaluate};

#[test]
fn exactly_half_is_not_a_majority() {
    let c = ids(&["a", "b"]);
    let t = evaluate(&Method::Majority, &c, &singles(&[("a", 2), ("b", 2)])).unwrap();
    assert!(t.winners.is_empty());
    assert!(t.note.contains("half"));
    let t = evaluate(&Method::Majority, &c, &singles(&[("a", 3), ("b", 2)])).unwrap();
    assert_eq!(winners(&t), ["a"]);
}

#[test]
fn majority_blank_options_keep_ranks() {
    let t = evaluate(
        &Method::Majority,
        &ids(&["a", "b", "c"]),
        &singles(&[("a", 1), ("b", 1), ("c", 1)]),
    )
    .unwrap();
    assert!(t.winners.is_empty());
    assert_eq!([rank(&t, "a"), rank(&t, "b"), rank(&t, "c")], [1, 1, 1]);
}

#[test]
fn threshold_boundary_with_three_ballots() {
    let c = ids(&["a", "b"]);
    let b = singles(&[("a", 2), ("b", 1)]);
    // 2 of 3 is 66.67 percent: passes 66, fails 67.
    let t = evaluate(
        &Method::Supermajority {
            threshold_percent: 66,
        },
        &c,
        &b,
    )
    .unwrap();
    assert_eq!(winners(&t), ["a"]);
    let t = evaluate(
        &Method::Supermajority {
            threshold_percent: 67,
        },
        &c,
        &b,
    )
    .unwrap();
    assert!(t.winners.is_empty());
    assert!(t.note.contains("67%"));
}

#[test]
fn threshold_must_be_51_to_100() {
    let c = ids(&["a"]);
    let b = singles(&[("a", 1)]);
    for bad in [0_u8, 50, 101, 255] {
        assert_eq!(
            evaluate(
                &Method::Supermajority {
                    threshold_percent: bad
                },
                &c,
                &b
            ),
            Err(Error::InvalidThreshold(bad))
        );
    }
    for ok in [51_u8, 100] {
        assert!(
            evaluate(
                &Method::Supermajority {
                    threshold_percent: ok
                },
                &c,
                &b
            )
            .is_ok()
        );
    }
    let t = evaluate(
        &Method::Supermajority {
            threshold_percent: 100,
        },
        &c,
        &b,
    )
    .unwrap();
    assert_eq!(winners(&t), ["a"]);
}
