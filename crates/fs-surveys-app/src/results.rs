//! Summaries and evaluations as typed result values.
//!
//! Pattern: Adapter (core value to `Values`) — the description's result fields are lines of text; a
//! counts line is `<question IRI> <choice>=<count>` and an option line `<choice>=<value> rank <n>`.
//! The charts (`charts.rs`) read the core value, not these lines.

use fs_admin_frontend::{Outcome, Values};
use fs_surveys_core::{Evaluation, Summary};

use crate::ids::question_iri;

fn yes(b: bool) -> String {
    if b { "yes" } else { "no" }.to_owned()
}

fn lines(s: &Summary, survey_iri: &str) -> Vec<String> {
    s.questions
        .iter()
        .flat_map(|q| {
            let at = question_iri(survey_iri, q.question);
            q.counts
                .iter()
                .map(move |c| format!("{at} {}={}", c.choice, c.count))
        })
        .collect()
}

/// The result of `summary` and `filter`.
pub fn summary(s: &Summary, survey_iri: &str) -> Outcome {
    let mut values = Values::new();
    values.insert("ballots".into(), vec![s.ballots.to_string()]);
    values.insert("hidden".into(), vec![yes(s.hidden)]);
    values.insert("heldBack".into(), vec![s.held_back.to_string()]);
    values.insert("counts".into(), lines(s, survey_iri));
    values.insert(
        "ai".into(),
        s.ai.as_ref()
            .map(|a| lines(a, survey_iri))
            .unwrap_or_default(),
    );
    Outcome { id: None, values }
}

/// The result of `evaluate`.
pub fn evaluation(e: &Evaluation) -> Outcome {
    let mut values = Values::new();
    values.insert("winners".into(), e.winners.clone());
    values.insert(
        "options".into(),
        e.options
            .iter()
            .map(|o| format!("{}={} rank {}", o.choice, o.value, o.rank))
            .collect(),
    );
    values.insert("note".into(), vec![e.note.clone()]);
    values.insert("hidden".into(), vec![yes(e.hidden)]);
    Outcome { id: None, values }
}
