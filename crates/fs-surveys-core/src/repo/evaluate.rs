//! Evaluating one question under a decision method.
//!
//! Pattern: Adapter — answers become `fs_surveys_methods::Ballot`s; the strategies live in that crate.
//! Held-back answers are handed on, so a result they could tip is not called decided.

use fs_objects::ObjectId;
use fs_surveys_methods::{Ballot, Method, evaluate_with_held_back};

use super::handle::SurveyRepo;
use super::reading::values_for;
use crate::error::{Error, Result};
use crate::model::{AnswerValue, Evaluation, Question, QuestionKind, SurveyState};

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

/// Consent counts 0 or 1 and systemic consensing 0 to 10; a question with another range is
/// refused here, not after the ballots were read.
fn check_range(q: &Question, method: Method) -> Result<()> {
    let (lo, hi) = match method {
        Method::Consent => (0, 1),
        Method::SystemicConsensing => (0, 10),
        _ => return Ok(()),
    };
    if q.kind == QuestionKind::OptionScores && (q.min < lo || q.max > hi) {
        return Err(Error::Invalid(format!(
            "this method needs scores from {lo} to {hi}"
        )));
    }
    Ok(())
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
        check_range(q, method)?;
        let counted = self.collect(&survey, stage).await?;
        let (values, left_out) = values_for(&survey, &counted.rows, question);
        let ballots: Vec<Ballot> = values
            .into_iter()
            .filter_map(|v| ballot_of(q.kind, v))
            .collect();
        // Answers the per-question rule left out are held back for this question too.
        let held_back = counted
            .held_back
            .saturating_add(u32::try_from(left_out).unwrap_or(u32::MAX));
        let mut out = Evaluation {
            method,
            winners: vec![],
            options: vec![],
            note: String::new(),
            hidden: survey.is_anonymous() && ballots.len() < usize::from(survey.min_group),
            held_back,
        };
        if out.hidden {
            "too few answers to show".clone_into(&mut out.note);
            return Ok(out);
        }
        let tally =
            evaluate_with_held_back(&method, &q.choice_ids(), &ballots, held_back as usize)?;
        (out.winners, out.options, out.note) = (tally.winners, tally.options, tally.note);
        if survey.is_anonymous() && survey.state == SurveyState::Open {
            // A winner (or its absence) would change with each submit and show arrival times.
            out.winners.clear();
            "the survey is still open; a result is given after it is closed"
                .clone_into(&mut out.note);
        }
        Ok(out)
    }
}
