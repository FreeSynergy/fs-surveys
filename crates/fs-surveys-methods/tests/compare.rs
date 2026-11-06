//! One ballot set under several methods; permutation independence.
mod common;
use common::*;
use fs_surveys_methods::{Ballot, Method, evaluate};

/// 6x A>B>C, 1x B>A>C, 3x B>C>A, 5x C>B>A (15 ballots).
fn ballots() -> Vec<Ballot> {
    let mut b = rankings(6, &["A", "B", "C"]);
    b.extend(rankings(1, &["B", "A", "C"]));
    b.extend(rankings(3, &["B", "C", "A"]));
    b.extend(rankings(5, &["C", "B", "A"]));
    b
}

#[test]
fn plurality_irv_and_schulze_disagree() {
    let c = ids(&["A", "B", "C"]);
    let b = ballots();
    // Plurality (first choices): A6 B4 C5 -> A.
    let firsts: Vec<Ballot> = b
        .iter()
        .map(|x| match x {
            Ballot::Ranking(r) => single(&r[0]),
            _ => unreachable!(),
        })
        .collect();
    let plurality = evaluate(&Method::Plurality, &c, &firsts).unwrap();
    assert_eq!(winners(&plurality), ["A"]);
    // IRV: R1 A6 B4 C5, B out; B's [B,A,C] x1 to A, [B,C,A] x3 to C: A7 C8 of 15 -> C.
    let irv = evaluate(&Method::InstantRunoff, &c, &b).unwrap();
    assert_eq!(winners(&irv), ["C"]);
    // Pairwise: B>A 9:6, B>C 10:5, so B is the Condorcet winner -> Schulze B.
    let schulze = evaluate(&Method::Schulze, &c, &b).unwrap();
    assert_eq!(winners(&schulze), ["B"]);
    // Borda: A 6*2+1 = 13, B 6+2+6+5 = 19, C 3+10 = 13 -> B.
    let borda = evaluate(&Method::Borda, &c, &b).unwrap();
    assert_eq!(winners(&borda), ["B"]);
}

/// Deterministic shuffle (LCG), no dependencies.
fn shuffled<T: Clone>(items: &[T], seed: u64) -> Vec<T> {
    let mut v = items.to_vec();
    let mut s = seed;
    for i in (1..v.len()).rev() {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let j = usize::try_from((s >> 33) % (i as u64 + 1)).unwrap();
        v.swap(i, j);
    }
    v
}

#[test]
fn permuting_ballots_never_changes_the_tally() {
    let c = ids(&["A", "B", "C"]);
    let sets: Vec<(Method, Vec<Ballot>)> = vec![
        (Method::Plurality, singles(&[("A", 3), ("B", 3), ("C", 2)])),
        (Method::Majority, singles(&[("A", 5), ("B", 3)])),
        (
            Method::Supermajority {
                threshold_percent: 60,
            },
            singles(&[("A", 5), ("B", 3)]),
        ),
        (
            Method::Approval,
            vec![multi(&["A", "B"]), multi(&["C"]), multi(&[]), multi(&["B"])],
        ),
        (
            Method::Score,
            vec![
                scores(&[("A", 3)]),
                scores(&[("B", 3), ("A", -1)]),
                scores(&[("C", 7)]),
            ],
        ),
        (Method::Borda, ballots()),
        (Method::Schulze, ballots()),
        (Method::InstantRunoff, ballots()),
        (
            Method::Consent,
            vec![
                scores(&[("A", 1)]),
                scores(&[("B", 0)]),
                scores(&[("A", 0), ("C", 1)]),
            ],
        ),
        (
            Method::SystemicConsensing,
            vec![
                scores(&[("A", 4)]),
                scores(&[("B", 2), ("A", 9)]),
                scores(&[("C", 7)]),
            ],
        ),
    ];
    for (m, b) in sets {
        let base = evaluate(&m, &c, &b).unwrap();
        for seed in 1..=20 {
            assert_eq!(
                evaluate(&m, &c, &shuffled(&b, seed)).unwrap(),
                base,
                "{m:?} seed {seed}"
            );
        }
    }
}
