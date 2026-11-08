//! Held-back answers: exact boundaries of every override.
mod common;
use common::*;
use fs_surveys_methods::{Method, evaluate, evaluate_with_held_back, method_for};

fn decided(m: Method, c: &[String], b: &[fs_surveys_methods::Ballot], held: usize) -> bool {
    !evaluate_with_held_back(&m, c, b, held)
        .unwrap()
        .winners
        .is_empty()
}

#[test]
fn zero_held_back_equals_evaluate() {
    let c = ids(&["a", "b"]);
    let b = singles(&[("a", 2), ("b", 2)]);
    for m in [Method::Plurality, Method::Majority, Method::Borda] {
        let ballots = if matches!(m, Method::Borda) {
            vec![ranking(&["a"])]
        } else {
            b.clone()
        };
        assert_eq!(
            evaluate_with_held_back(&m, &c, &ballots, 0),
            evaluate(&m, &c, &ballots)
        );
    }
}

#[test]
fn plurality_lead_must_exceed_held_back() {
    let c = ids(&["a", "b"]);
    let b = singles(&[("a", 5), ("b", 3)]); // lead 2
    assert!(decided(Method::Plurality, &c, &b, 1));
    assert!(!decided(Method::Plurality, &c, &b, 2));
    let t = evaluate_with_held_back(&Method::Plurality, &c, &b, 2).unwrap();
    assert!(t.note.contains("2 held-back"));
    assert_eq!(order(&t), ["a", "b"]);
    assert!((value(&t, "a") - 5.0).abs() < f64::EPSILON);
}

#[test]
fn approval_uses_the_same_bound() {
    let c = ids(&["a", "b"]);
    let mut b = vec![multi(&["a"]); 4];
    b.push(multi(&["a", "b"]));
    b.push(multi(&["b"])); // a 5, b 2, lead 3
    assert!(decided(Method::Approval, &c, &b, 2));
    assert!(!decided(Method::Approval, &c, &b, 3));
}

#[test]
fn majority_boundary() {
    let c = ids(&["a", "b"]);
    let b = singles(&[("a", 3), ("b", 1)]); // 3*2 > 4 + held
    assert!(decided(Method::Majority, &c, &b, 1));
    assert!(!decided(Method::Majority, &c, &b, 2));
}

#[test]
fn supermajority_boundary() {
    let c = ids(&["a", "b"]);
    let b = singles(&[("a", 3), ("b", 1)]);
    let m = Method::Supermajority {
        threshold_percent: 60,
    }; // 300 >= 60 * (4 + held)
    assert!(decided(m, &c, &b, 1));
    assert!(!decided(m, &c, &b, 2));
}

#[test]
fn other_methods_use_the_conservative_default() {
    let c = ids(&["a", "b"]);
    let b = vec![ranking(&["a", "b"]); 50];
    for m in [Method::Borda, Method::Schulze, Method::InstantRunoff] {
        assert!(decided(m, &c, &b, 0), "{m:?}");
        assert!(!decided(m, &c, &b, 1), "{m:?}");
    }
    let t = evaluate(&Method::Borda, &c, &b).unwrap();
    assert!(method_for(&Method::Score).held_back_can_change(&t, 50, 1));
    assert!(!method_for(&Method::Score).held_back_can_change(&t, 50, 0));
}

#[test]
fn undecided_stays_undecided() {
    let c = ids(&["a", "b"]);
    let t =
        evaluate_with_held_back(&Method::Majority, &c, &singles(&[("a", 1), ("b", 1)]), 3).unwrap();
    assert_eq!(t.winners, [] as [std::string::String; 0]);
}
