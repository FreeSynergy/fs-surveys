//! Export as CSV.
//!
//! Pattern: Builder (`Csv`) — `named`: one row per response with the person; anonymous: totals only.
//! Free text is neutralised against spreadsheet formulas. People and AIs are two files; the AI file
//! tags every row.

use fs_interfaces::Unsupported;
use fs_objects::ObjectId;

use super::handle::SurveyRepo;
use super::reading::{Collected, Row};
use crate::csv::{Csv, escape, text_cell};
use crate::error::Result;
use crate::model::{ALL_STAGES, AnswerValue, Class, Question, QuestionKind, Survey};

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

fn document(class: Class) -> Csv {
    if class.is_ai() {
        Csv::tagged("ai")
    } else {
        Csv::default()
    }
}

fn rows_csv(s: &Survey, rows: &[Row], class: Class) -> Vec<u8> {
    let mut out = document(class);
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

fn totals_csv(s: &Survey, summary: &crate::model::Summary, class: Class) -> Vec<u8> {
    let mut out = document(class);
    out.row(&["question".into(), "value".into(), "count".into()]);
    for (q, t) in s.questions.iter().zip(&summary.questions) {
        let name = text_cell(&q.text);
        if t.hidden {
            out.row(&[name, "hidden".into(), t.answers.to_string()]);
            continue;
        }
        for c in &t.counts {
            // Scale values are genuine numbers; every other label comes from people.
            let label = if q.kind == QuestionKind::Scale {
                escape(&c.choice)
            } else {
                text_cell(&c.choice)
            };
            out.row(&[name.clone(), label, c.count.to_string()]);
        }
        if let Some(m) = t.mean {
            out.row(&[name, "mean".into(), m.to_string()]);
        }
    }
    out.finish()
}

impl SurveyRepo {
    /// `result.export`: `text/csv` only. The answers of people; the AI answers are `export_class`.
    pub async fn export(&self, id: ObjectId, media_type: &str) -> Result<Vec<u8>> {
        self.export_class(id, media_type, Class::Human).await
    }

    /// `result.export` of one class. The AI file has its header and no rows when the AI part is not shown.
    pub(super) async fn export_class(
        &self,
        id: ObjectId,
        media_type: &str,
        class: Class,
    ) -> Result<Vec<u8>> {
        if media_type != "text/csv" {
            return Err(Unsupported::new(
                "result.export",
                format!("{media_type} is not offered; text/csv is"),
            )
            .into());
        }
        let s = self.require(id).await?;
        let lanes = self.collect(&s, ALL_STAGES).await?;
        let (found, shown) = lanes.of(class);
        let none = Collected {
            rows: vec![],
            held_back: found.held_back,
        };
        let c = if shown { found } else { &none };
        if s.is_anonymous() {
            return Ok(totals_csv(&s, &super::summary::totals(&s, c)?, class));
        }
        Ok(rows_csv(&s, &c.rows, class))
    }
}
