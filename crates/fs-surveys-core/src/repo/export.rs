//! Export as CSV.
//!
//! Pattern: Builder (`Csv`) — `named`: one row per response with the person; anonymous: totals only.
//! Free text is neutralised against spreadsheet formulas.

use fs_interfaces::Unsupported;
use fs_store::ObjectId;

use super::handle::SurveyRepo;
use super::reading::Row;
use crate::csv::{Csv, escape, text_cell};
use crate::error::Result;
use crate::model::{ALL_STAGES, AnswerValue, Question, Survey};

fn cell(q: &Question, row: &Row) -> String {
    let Some(a) = row.answers.iter().find(|a| a.question == q.id) else {
        return String::new();
    };
    let list = |l: &[String]| text_cell(&l.join(";"));
    match &a.value {
        AnswerValue::Choice(s) | AnswerValue::Text(s) | AnswerValue::Date(s) => text_cell(s),
        AnswerValue::Choices(l) | AnswerValue::Ranking(l) => list(l),
        AnswerValue::Number(n) => n.to_string(),
        AnswerValue::Scale(n) => n.to_string(),
        AnswerValue::Scores(l) => text_cell(
            &l.iter()
                .map(|s| format!("{}={}", s.choice, s.score))
                .collect::<Vec<_>>()
                .join(";"),
        ),
    }
}

fn rows_csv(s: &Survey, rows: &[Row]) -> Vec<u8> {
    let mut out = Csv::default();
    let mut head = vec!["person".to_string()];
    head.extend(s.questions.iter().map(|q| text_cell(&q.text)));
    out.row(&head);
    for r in rows {
        let mut line = vec![text_cell(r.person.as_deref().unwrap_or_default())];
        line.extend(s.questions.iter().map(|q| cell(q, r)));
        out.row(&line);
    }
    out.finish()
}

fn totals_csv(s: &Survey, summary: &crate::model::Summary) -> Vec<u8> {
    let mut out = Csv::default();
    out.row(&["question".into(), "value".into(), "count".into()]);
    for (q, t) in s.questions.iter().zip(&summary.questions) {
        let name = text_cell(&q.text);
        if t.hidden {
            out.row(&[name, "hidden".into(), t.answers.to_string()]);
            continue;
        }
        for c in &t.counts {
            out.row(&[name.clone(), escape(&c.choice), c.count.to_string()]);
        }
        if let Some(m) = t.mean {
            out.row(&[name, "mean".into(), m.to_string()]);
        }
    }
    out.finish()
}

impl SurveyRepo {
    /// `result.export`: `text/csv` only.
    pub async fn export(&self, id: ObjectId, media_type: &str) -> Result<Vec<u8>> {
        if media_type != "text/csv" {
            return Err(Unsupported::new(
                "result.export",
                format!("{media_type} is not offered; text/csv is"),
            )
            .into());
        }
        let s = self.require(id).await?;
        let c = self.collect(&s, ALL_STAGES).await?;
        if s.is_anonymous() {
            return Ok(totals_csv(
                &s,
                &super::summary::totals(&s, &c.rows, c.held_back)?,
            ));
        }
        Ok(rows_csv(&s, &c.rows))
    }
}
