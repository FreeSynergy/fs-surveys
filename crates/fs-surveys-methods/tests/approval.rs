//! Approval, score, borda.
mod common;
use common::*;
use fs_surveys_methods::{Ballot, Method, evaluate};

#[test]
fn approval_counts_and_empty_ballot_is_valid() {
    let b = [
        multi(&["a", "b"]),
        multi(&["b"]),
        multi(&[]),
        multi(&["a", "b", "c"]),
    ];
    let t = evaluate(&Method::Approval, &ids(&["a", "b", "c"]), &b).unwrap();
    assert_eq!(winners(&t), ["b"]);
    assert!((value(&t, "b") - 3.0).abs() < f64::EPSILON);
    assert!((value(&t, "c") - 1.0).abs() < f64::EPSILON);
}

#[test]
fn approval_tie_lists_all() {
    let b = [multi(&["a"]), multi(&["b"])];
    let t = evaluate(&Method::Approval, &ids(&["a", "b"]), &b).unwrap();
    assert_eq!(winners(&t), ["a", "b"]);
}

#[test]
fn score_sums_and_missing_is_zero() {
    // a: 5 + 3 + 0 = 8; b: 0 + 4 + (-2) = 2; negative scores are allowed.
    let b = [
        scores(&[("a", 5)]),
        scores(&[("a", 3), ("b", 4)]),
        scores(&[("b", -2)]),
    ];
    let t = evaluate(&Method::Score, &ids(&["a", "b"]), &b).unwrap();
    assert_eq!(winners(&t), ["a"]);
    assert!((value(&t, "a") - 8.0).abs() < f64::EPSILON);
    assert!((value(&t, "b") - 2.0).abs() < f64::EPSILON);
}

#[test]
fn borda_points_with_partial_rankings() {
    // 3 choices: first = 2, second = 1, unranked = 0.
    // a: 2 (b1) + 0 (b2) + 0 (b3) = 2; b: 1 + 2 + 2 = 5; c: 0 + 1 + 1 = 2.
    let b = [
        ranking(&["a", "b"]),
        ranking(&["b", "c"]),
        ranking(&["b", "c", "a"]),
    ];
    let t = evaluate(&Method::Borda, &ids(&["a", "b", "c"]), &b).unwrap();
    assert_eq!(winners(&t), ["b"]);
    assert!((value(&t, "b") - 5.0).abs() < f64::EPSILON);
    assert!((value(&t, "a") - 2.0).abs() < f64::EPSILON);
    assert_eq!([rank(&t, "b"), rank(&t, "a"), rank(&t, "c")], [1, 2, 2]);
}

#[test]
fn borda_rejects_scores() {
    let r = evaluate(&Method::Borda, &ids(&["a"]), &[Ballot::Scores(vec![])]);
    assert!(r.is_err());
}
