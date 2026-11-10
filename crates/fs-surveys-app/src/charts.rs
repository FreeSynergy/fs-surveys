//! The result of a question as a chart specification (data, not a look).
//!
//! Pattern: Adapter (core value to Vega-Lite subset) — `fs-components-charts` reads and prepares the spec;
//! drawing it is that crate's view and the frontend's business (ADR 0066: no render code here). A
//! question with fewer answers than `minGroup` is `hidden` in the summary and gives no chart: its figures
//! would let a person be recognised.

use fs_components_charts::Chart;
use fs_surveys_core::{Question, Summary};
use serde_json::json;

/// Why a chart could not be made.
#[derive(Debug)]
pub enum ChartError {
    /// The summary has no figures for this question.
    NoFigures,
    /// Too few answers to show (`minGroup`).
    Hidden,
    /// The charts crate refused the spec.
    Spec(String),
}

/// A bar chart of how often each choice (or scale or number value) was given.
///
/// # Errors
/// [`ChartError::NoFigures`], [`ChartError::Hidden`], or the charts crate's refusal.
pub fn question_chart(question: &Question, summary: &Summary) -> Result<Chart, ChartError> {
    let q = summary
        .questions
        .iter()
        .find(|q| q.question == question.id)
        .ok_or(ChartError::NoFigures)?;
    if summary.hidden || q.hidden {
        return Err(ChartError::Hidden);
    }
    let label = |id: &str| {
        question
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or_else(|| id.to_owned(), |c| c.label.clone())
    };
    let rows: Vec<_> = q
        .counts
        .iter()
        .map(|c| json!({"choice": label(&c.choice), "count": c.count}))
        .collect();
    let spec = json!({
        "title": question.text,
        "data": {"values": rows},
        "mark": "bar",
        "encoding": {
            "x": {"field": "choice", "type": "nominal", "sort": null},
            "y": {"field": "count", "type": "quantitative"}
        }
    });
    Chart::from_json(&spec.to_string()).map_err(|e| ChartError::Spec(format!("{e:?}")))
}
