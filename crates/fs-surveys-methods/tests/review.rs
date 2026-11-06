//! Regressions found in review: runoff ties, Schulze ranks, empty ballots, errors without values.

use fs_surveys_methods::{Ballot, Error, Method, evaluate};

fn ss(v: &[&str]) -> Vec<String> {
    v.iter().map(ToString::to_string).collect()
}
fn rk(l: &[&str]) -> Ballot {
    Ballot::Ranking(ss(l))
}

#[test]
fn runoff_does_not_eliminate_tied_choices_that_could_overtake() {
    let mut b = vec![rk(&["A"]); 4];
    b.extend(vec![rk(&["B", "C"]); 3]);
    b.extend(vec![rk(&["C", "B"]); 3]);
    let t = evaluate(&Method::InstantRunoff, &ss(&["A", "B", "C"]), &b).unwrap();
    assert!(t.winners.is_empty(), "{:?}", t.winners);
    assert!(t.note.contains("tied for last place"));
}

#[test]
fn runoff_still_eliminates_tied_choices_that_are_far_behind() {
    let mut b = vec![rk(&["A"]); 5];
    b.push(rk(&["B"]));
    b.push(rk(&["C"]));
    let t = evaluate(&Method::InstantRunoff, &ss(&["A", "B", "C"]), &b).unwrap();
    assert_eq!(t.winners, ["A"]);
}

#[test]
fn schulze_winners_rank_first() {
    let ch = ss(&["W", "A", "B", "C"]);
    let b = [rk(&["W", "A", "B", "C"]), rk(&["A", "B", "C", "W"])];
    let t = evaluate(&Method::Schulze, &ch, &b).unwrap();
    assert!(!t.winners.is_empty());
    for o in &t.options {
        assert_eq!(
            o.rank == 1,
            t.winners.contains(&o.choice),
            "{:?}",
            t.options
        );
    }
}

#[test]
fn all_empty_ballots_decide_nothing() {
    let ch = ss(&["A", "B", "C"]);
    let t = evaluate(&Method::Approval, &ch, &vec![Ballot::Multiple(vec![]); 5]).unwrap();
    assert!(t.winners.is_empty() && !t.note.is_empty());
    let t = evaluate(&Method::Borda, &ch, &vec![rk(&[]); 5]).unwrap();
    assert!(t.winners.is_empty() && !t.note.is_empty());
}

#[test]
fn errors_name_no_ballot_value() {
    let ch = ss(&["A", "B"]);
    let secret = "secret-choice-id";
    let errs = [
        evaluate(&Method::Plurality, &ch, &[Ballot::Single(secret.into())]).unwrap_err(),
        evaluate(&Method::Borda, &ch, &[rk(&["A", "A"])]).unwrap_err(),
        evaluate(
            &Method::Consent,
            &ch,
            &[Ballot::Scores(vec![("A".into(), 77)])],
        )
        .unwrap_err(),
    ];
    for e in errs {
        let t = format!("{e} {e:?}");
        assert!(
            !t.contains(secret) && !t.contains("77") && !t.contains("\"A\""),
            "{t}"
        );
        assert!(matches!(e, Error::UnknownChoice | Error::InvalidBallot(_)));
    }
}
