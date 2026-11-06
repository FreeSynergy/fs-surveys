//! Evaluating one question under a decision method.
//!
//! Pattern: Adapter — answers become `fs_surveys_methods::Ballot`s; the strategies live in that crate.
//! Held-back answers are handed on, so a result they could tip is not called decided.

use fs_store::ObjectId;
use fs_surveys_methods::{Ballot, Method, evaluate_with_held_back};

use super::handle::SurveyRepo;
use crate::error::{Error, Result};
use crate::model::{AnswerValue, Evaluation, Question, QuestionKind};

fn ballot_of(kind: QuestionKind, v: &AnswerValue) -> Option<Ballot> {
    Some(match (kind, v) {
        (QuestionKind::SingleChoice, AnswerValue::Choice(c)) => Ballot::Single(c.clone()),
        (QuestionKind::MultipleChoice, AnswerValue::Choices(l)) => Ballot::Multiple(l.clone()),
        (QuestionKind::Ranking, AnswerValue::Ranking(l)) => Ballot::Ranking(l.clone()),
        (QuestionKind::OptionScores, AnswerValue::Scores(l)) => {
            Ballot::Scores(l.iter().map(|s| (s.choice.clone(), s.score)).collect())
        }
        _ => return None,
    })
}

fn check_kind(q: &Question) -> Result<()> {
    match q.kind {
        QuestionKind::SingleChoice
        | QuestionKind::MultipleChoice
        | QuestionKind::Ranking
        | QuestionKind::OptionScores => Ok(()),
        _ => Err(Error::Invalid(format!(
            "a {} question has no ballots to evaluate",
            q.kind.as_str()
        ))),
    }
}

impl SurveyRepo {
    /// `survey.method`: the same ballots can be evaluated by every method whose ballot form fits.
    pub async fn evaluate(
        &self,
        id: ObjectId,
        question: ObjectId,
        method: Method,
        stage: u16,
    ) -> Result<Evaluation> {
        let survey = self.require(id).await?;
        let q = survey.question(question).ok_or(Error::NotFound {
            what: "question",
            id: question,
        })?;
        check_kind(q)?;
        let counted = self.collect(&survey, stage).await?;
        let ballots: Vec<Ballot> = c
            .rows
            .iter()
            .filter_map(|r| r.answers.iter().find(|a| a.question == question))
            .filter_map(|a| ballot_of(q.kind, &a.value))
            .collect();
        let mut out = Evaluation {
            method,
            winners: vec![],
            options: vec![],
            note: String::new(),
            hidden: survey.is_anonymous() && ballots.len() < usize::from(survey.min_group),
            held_back: counted.held_back,
        };
        if out.hidden {
            "too few answers to show".clone_into(&mut out.note);
            return Ok(out);
        }
        let tally = evaluate_with_held_back(
            &method,
            &q.choice_ids(),
            &ballots,
            counted.held_back as usize,
        )?;
        (out.winners, out.options, out.note) = (tally.winners, tally.options, tally.note);
        Ok(out)
    }
}
