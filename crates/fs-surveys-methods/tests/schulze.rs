//! Schulze, hand-worked.
mod common;
use common::*;
use fs_surveys_methods::{Method, evaluate};

#[test]
fn clear_winner_by_paths() {
    // 4x A>B>C, 3x B>C>A, 2x C>A>B (9 ballots).
    // d[A][B]=4+2=6, d[B][A]=3; d[B][C]=4+3=7, d[C][B]=2; d[A][C]=4, d[C][A]=3+2=5.
    // Direct winners: A>B (6), B>C (7), C>A (5): a cycle.
    // Path A->B->C = min(6,7) = 6 > p[C][A] = 5, so A beats C.
    // Path B->C->A = min(7,5) = 5 < p[A][B] = 6, so A beats B.
    // p[B][C] = 7 vs p[C][B] = min(5,6) = 5, so B beats C.
    // A beats 2, B beats 1, C beats 0.
    let mut b = rankings(4, &["A", "B", "C"]);
    b.extend(rankings(3, &["B", "C", "A"]));
    b.extend(rankings(2, &["C", "A", "B"]));
    let t = evaluate(&Method::Schulze, &ids(&["A", "B", "C"]), &b).unwrap();
    assert_eq!(winners(&t), ["A"]);
    assert_eq!(order(&t), ["A", "B", "C"]);
    assert!((value(&t, "A") - 2.0).abs() < f64::EPSILON);
    assert!((value(&t, "B") - 1.0).abs() < f64::EPSILON);
    assert!(value(&t, "C").abs() < f64::EPSILON);
}

#[test]
fn perfect_condorcet_cycle_ties_everyone() {
    // A>B>C, B>C>A, C>A>B: every pair is 2:1 in a cycle, every path has strength 2.
    let b = [
        ranking(&["A", "B", "C"]),
        ranking(&["B", "C", "A"]),
        ranking(&["C", "A", "B"]),
    ];
    let t = evaluate(&Method::Schulze, &ids(&["A", "B", "C"]), &b).unwrap();
    assert_eq!(winners(&t), ["A", "B", "C"]);
    assert!(t.options.iter().all(|o| o.rank == 1 && o.value == 0.0));
    assert!(!t.note.is_empty());
}

#[test]
fn partial_ranking_ranked_beats_unranked() {
    // One ballot A only: A beats B and C; B and C are indifferent.
    let t = evaluate(&Method::Schulze, &ids(&["A", "B", "C"]), &[ranking(&["A"])]).unwrap();
    assert_eq!(winners(&t), ["A"]);
    assert!((value(&t, "A") - 2.0).abs() < f64::EPSILON);
    assert_eq!([rank(&t, "B"), rank(&t, "C")], [2, 2]);
}

#[test]
fn single_choice_wins_alone() {
    let t = evaluate(&Method::Schulze, &ids(&["A"]), &[ranking(&["A"])]).unwrap();
    assert_eq!(winners(&t), ["A"]);
}
