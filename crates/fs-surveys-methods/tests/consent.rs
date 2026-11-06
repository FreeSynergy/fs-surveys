//! Consent and systemic consensing.
mod common;
use common::*;
use fs_surveys_methods::{Error, Method, evaluate};

#[test]
fn consent_passes_without_objection() {
    let b = [
        scores(&[("a", 1), ("b", 0)]),
        scores(&[("a", 0)]),
        scores(&[("c", 1)]),
    ];
    let t = evaluate(&Method::Consent, &ids(&["a", "b", "c"]), &b).unwrap();
    assert_eq!(winners(&t), ["b"]);
    assert!((value(&t, "a") - 1.0).abs() < f64::EPSILON);
    assert_eq!(order(&t), ["b", "a", "c"].map(|x| x));
}

#[test]
fn consent_all_may_pass() {
    let t = evaluate(&Method::Consent, &ids(&["a", "b"]), &[scores(&[("a", 0)])]).unwrap();
    assert_eq!(winners(&t), ["a", "b"]);
}

#[test]
fn consent_all_objected_has_no_winner() {
    let b = [scores(&[("a", 1)]), scores(&[("b", 1)])];
    let t = evaluate(&Method::Consent, &ids(&["a", "b"]), &b).unwrap();
    assert!(t.winners.is_empty());
    assert!(t.note.contains("objection"));
}

#[test]
fn consent_only_accepts_zero_or_one() {
    for s in [2, -1] {
        let r = evaluate(&Method::Consent, &ids(&["a"]), &[scores(&[("a", s)])]);
        assert!(matches!(r, Err(Error::InvalidBallot(_))));
    }
}

#[test]
fn systemic_least_resistance_and_missing_is_ten() {
    // a: 2 + 3 = 5; b: 0 + missing(10) = 10.
    let b = [scores(&[("a", 2), ("b", 0)]), scores(&[("a", 3)])];
    let t = evaluate(&Method::SystemicConsensing, &ids(&["a", "b"]), &b).unwrap();
    assert_eq!(winners(&t), ["a"]);
    assert!((value(&t, "a") - 5.0).abs() < f64::EPSILON);
    assert!((value(&t, "b") - 10.0).abs() < f64::EPSILON);
}

#[test]
fn systemic_tie_and_range() {
    let b = [scores(&[("a", 1), ("b", 1)])];
    let t = evaluate(&Method::SystemicConsensing, &ids(&["a", "b"]), &b).unwrap();
    assert_eq!(winners(&t), ["a", "b"]);
    for s in [-1, 11] {
        let r = evaluate(
            &Method::SystemicConsensing,
            &ids(&["a"]),
            &[scores(&[("a", s)])],
        );
        assert!(matches!(r, Err(Error::InvalidBallot(_))));
    }
    assert!(
        evaluate(
            &Method::SystemicConsensing,
            &ids(&["a"]),
            &[scores(&[("a", 10)]), scores(&[("a", 0)])]
        )
        .is_ok()
    );
}
