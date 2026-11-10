//! Answers as text lines `<question IRI>=<value>`, and back.
//!
//! Pattern: Adapter (Strategy by question kind) — there is no basic type for an answer; a line holds the
//! question's IRI and the value in the form of its kind: a choice id; choice ids joined by `,`; text; a
//! number; a date `YYYY-MM-DD`; a scale value; a ranking as ids joined by `,` (best first); scores as
//! `<choice>:<score>` joined by `,`. Provisional (no `Answer` basic type yet), named in `DESIGN.md`.

use fs_surveys_core::{Answer, AnswerValue, OptionScore, Question, QuestionKind, Survey};

use crate::ids::{question_iri, tail};
use crate::refuse::RunError;

fn bad(index: usize) -> RunError {
    RunError::Bad {
        field: "answers".into(),
        index,
    }
}

fn split(v: &str) -> Vec<String> {
    v.split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

fn value(q: &Question, text: &str, index: usize) -> Result<AnswerValue, RunError> {
    Ok(match q.kind {
        QuestionKind::SingleChoice => AnswerValue::Choice(text.to_owned()),
        QuestionKind::MultipleChoice => AnswerValue::Choices(split(text)),
        QuestionKind::Text => AnswerValue::Text(text.to_owned()),
        QuestionKind::Number => AnswerValue::Number(text.parse().map_err(|_| bad(index))?),
        QuestionKind::Date => AnswerValue::Date(text.to_owned()),
        QuestionKind::Scale => AnswerValue::Scale(text.parse().map_err(|_| bad(index))?),
        QuestionKind::Ranking => AnswerValue::Ranking(split(text)),
        QuestionKind::OptionScores => AnswerValue::Scores(
            split(text)
                .into_iter()
                .map(|p| {
                    let (choice, score) = p.split_once(':').ok_or_else(|| bad(index))?;
                    Ok(OptionScore {
                        choice: choice.trim().to_owned(),
                        score: score.trim().parse().map_err(|_| bad(index))?,
                    })
                })
                .collect::<Result<_, RunError>>()?,
        ),
    })
}

/// Read the lines of a survey's answers.
pub fn parse(survey: &Survey, lines: &[&str]) -> Result<Vec<Answer>, RunError> {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let (at, text) = line.split_once('=').ok_or_else(|| bad(i))?;
            let id = tail(at).map_err(|_| bad(i))?;
            let q = survey.question(id).ok_or(RunError::NotFound)?;
            Ok(Answer {
                question: id,
                value: value(q, text.trim(), i)?,
            })
        })
        .collect()
}

fn text_of(v: &AnswerValue) -> String {
    match v {
        AnswerValue::Choice(s) | AnswerValue::Text(s) | AnswerValue::Date(s) => s.clone(),
        AnswerValue::Choices(l) | AnswerValue::Ranking(l) => l.join(","),
        AnswerValue::Number(n) => n.to_string(),
        AnswerValue::Scale(n) => n.to_string(),
        AnswerValue::Scores(l) => l
            .iter()
            .map(|s| format!("{}:{}", s.choice, s.score))
            .collect::<Vec<_>>()
            .join(","),
    }
}

/// Write answers as lines.
pub fn format(survey_iri: &str, answers: &[Answer]) -> Vec<String> {
    answers
        .iter()
        .map(|a| {
            format!(
                "{}={}",
                question_iri(survey_iri, a.question),
                text_of(&a.value)
            )
        })
        .collect()
}
