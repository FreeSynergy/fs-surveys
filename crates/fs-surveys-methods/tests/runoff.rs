//! Instant runoff, hand-worked.
mod common;
use common::*;
use fs_surveys_methods::{Method, evaluate};

#[test]
fn exhausted_ballots_leave_the_count() {
    // Choices A B C D. 4x [A], 2x [B,C], 3x [C], 1x [D].
    // R1: A4 B2 C3 D1, 10 active, need 6: none. Lowest D out.
    // R2: the [D] ballot is exhausted: A4 B2 C3, 9 active, need 5: none. Lowest B out.
    // R3: [B,C] ballots move to C: A4 C5, 9 active, 5*2 > 9: C wins.
    let mut b = rankings(4, &["A"]);
    b.extend(rankings(2, &["B", "C"]));
    b.extend(rankings(3, &["C"]));
    b.extend(rankings(1, &["D"]));
    let t = evaluate(&Method::InstantRunoff, &ids(&["A", "B", "C", "D"]), &b).unwrap();
    assert_eq!(winners(&t), ["C"]);
    assert_eq!(order(&t), ["C", "A", "B", "D"]);
    assert!((value(&t, "C") - 5.0).abs() < f64::EPSILON);
    assert!((value(&t, "A") - 4.0).abs() < f64::EPSILON);
    assert!((value(&t, "B") - 2.0).abs() < f64::EPSILON);
    assert!((value(&t, "D") - 1.0).abs() < f64::EPSILON);
    assert_eq!(
        [rank(&t, "C"), rank(&t, "A"), rank(&t, "B"), rank(&t, "D")],
        [1, 2, 3, 4]
    );
}

#[test]
fn final_tie_lists_all_remaining() {
    // R1: A1 B1 C0, lowest C out (leaves two). R2: A1 B1 of 2, no majority, all lowest: tie.
    let b = [ranking(&["A", "B"]), ranking(&["B", "A"])];
    let t = evaluate(&Method::InstantRunoff, &ids(&["A", "B", "C"]), &b).unwrap();
    assert_eq!(winners(&t), ["A", "B"]);
    assert!(t.note.contains("tied"));
    assert_eq!([rank(&t, "A"), rank(&t, "B"), rank(&t, "C")], [1, 1, 3]);
}

#[test]
fn several_lowest_leave_together() {
    // R1: A3 B1 C1 D1 (6 active, need 4). B, C, D all lowest; A remains, so all three go.
    // R2: A alone with 3 of 3 active: wins.
    let mut b = rankings(3, &["A"]);
    b.extend([ranking(&["B"]), ranking(&["C"]), ranking(&["D"])]);
    let t = evaluate(&Method::InstantRunoff, &ids(&["A", "B", "C", "D"]), &b).unwrap();
    assert_eq!(winners(&t), ["A"]);
    assert_eq!([rank(&t, "B"), rank(&t, "C"), rank(&t, "D")], [2, 2, 2]);
}

#[test]
fn all_empty_ballots_decide_nothing() {
    let t = evaluate(
        &Method::InstantRunoff,
        &ids(&["A", "B"]),
        &[ranking(&[]), ranking(&[])],
    )
    .unwrap();
    assert!(t.winners.is_empty());
    assert!(!t.note.is_empty());
}

#[test]
fn first_round_majority() {
    let mut b = rankings(3, &["A", "B"]);
    b.extend(rankings(2, &["B", "A"]));
    let t = evaluate(&Method::InstantRunoff, &ids(&["A", "B"]), &b).unwrap();
    assert_eq!(winners(&t), ["A"]);
}
