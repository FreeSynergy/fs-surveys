//! Which answers count: released batches on anonymous surveys, everything submitted on `named`; people
//! and AIs apart.
//!
//! Pattern: Template method — `Lanes::run` is the one flow every reading goes through (summary, filter,
//! evaluation of all ten methods, export): the same function runs once over the rows of people and once
//! over the rows of AIs, and no function ever sees both. Reading never releases or writes anything.

use fs_store::{ObjectId, Query};

use super::consts::RESPONSE_TYPE;
use super::handle::SurveyRepo;
use crate::error::Result;
use crate::model::{ALL_STAGES, Answer, AnswerValue, Class, Survey, SurveyState};

/// One respondent's answers; the person is known on `named` only.
pub(super) struct Row {
    pub person: Option<String>,
    pub answers: Vec<Answer>,
    /// The released batch it came in; named answers are all batch 0.
    pub batch: usize,
}

/// The answers of one class that count, and how many are held back.
pub(super) struct Collected {
    pub rows: Vec<Row>,
    pub held_back: u32,
}

/// The counting answers, split by class. Built only by `collect`, which puts a row in the lane of
/// its class and nowhere else.
pub(super) struct Lanes {
    people: Collected,
    ai: Collected,
    /// The AI part is shown: the survey allows AIs, or holds AI answers.
    show_ai: bool,
}

impl Lanes {
    /// Run `f` over the people, and over the AIs if their part is shown. `f` sees one class only.
    pub fn run<T>(&self, f: impl Fn(&Collected) -> Result<T>) -> Result<(T, Option<T>)> {
        let people = f(&self.people)?;
        let ai = if self.show_ai {
            Some(f(&self.ai)?)
        } else {
            None
        };
        Ok((people, ai))
    }

    /// Keep only the rows `keep` accepts, in both lanes.
    pub fn retain(&mut self, keep: impl Fn(&Row) -> bool) {
        self.people.rows.retain(&keep);
        self.ai.rows.retain(&keep);
    }

    /// The rows of one class, and whether to show them.
    pub fn of(&self, class: Class) -> (&Collected, bool) {
        match class {
            Class::Human => (&self.people, true),
            Class::Ai => (&self.ai, self.show_ai),
        }
    }
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl SurveyRepo {
    /// The counting answers of `s` for `stage` (`0xFFFF` = all), by class.
    pub(super) async fn collect(&self, s: &Survey, stage: u16) -> Result<Lanes> {
        if s.is_anonymous() {
            let b = self.read_box(s.id()).await?;
            // While the survey is open the number would tick with every submit and show when
            // people answered; it is real once the survey is closed.
            let open = s.state == SurveyState::Open;
            let lane = |class: Class| Collected {
                rows: b
                    .released(class, stage)
                    .into_iter()
                    .map(|(batch, answers)| Row {
                        person: None,
                        answers: answers.clone(),
                        batch,
                    })
                    .collect(),
                held_back: if open { 0 } else { count(b.held_back(class)) },
            };
            return Ok(Lanes {
                people: lane(Class::Human),
                ai: lane(Class::Ai),
                show_ai: s.ai_allowed || b.has_ai_ballots(),
            });
        }
        let q = Query::new()
            .of_type(RESPONSE_TYPE)
            .with_index("survey", s.id().to_string())
            .with_index("submitted", "true");
        let (mut people, mut ai) = (vec![], vec![]);
        for o in self.reads.query(&q).await? {
            let at = o
                .index
                .get("stage")
                .and_then(|v| v.first())
                .and_then(|v| v.parse::<u16>().ok());
            if stage != ALL_STAGES && at != Some(stage) {
                continue;
            }
            let person = o
                .content
                .get("fs:person")
                .and_then(|p| p.as_str())
                .map(String::from);
            let answers =
                serde_json::from_value(o.content.get("fs:answers").cloned().unwrap_or_default())?;
            let row = Row {
                person,
                answers,
                batch: 0,
            };
            // No flag means no human: only an explicit `false` puts a row among the people.
            if o.content.get("fs:ai").and_then(serde_json::Value::as_bool) == Some(false) {
                people.push(row);
            } else {
                ai.push(row);
            }
        }
        for rows in [&mut people, &mut ai] {
            rows.sort_by(|a, b| a.person.cmp(&b.person));
        }
        let show_ai = s.ai_allowed || !ai.is_empty();
        Ok(Lanes {
            people: Collected {
                rows: people,
                held_back: 0,
            },
            ai: Collected {
                rows: ai,
                held_back: 0,
            },
            show_ai,
        })
    }
}

/// The values given to `question`. On an anonymous survey a batch counts only if it holds at
/// least `minGroup` answers to this question, or a conditional question would show a single
/// answer through the difference between two readings.
///
/// The second number is how many answers that rule left out; they are held back for this question.
pub(super) fn values_for<'a>(
    s: &Survey,
    rows: &'a [Row],
    question: ObjectId,
) -> (Vec<&'a AnswerValue>, usize) {
    let mut by_batch: std::collections::BTreeMap<usize, Vec<&AnswerValue>> =
        std::collections::BTreeMap::default();
    for r in rows {
        if let Some(a) = r.answers.iter().find(|a| a.question == question) {
            by_batch.entry(r.batch).or_default().push(&a.value);
        }
    }
    let enough = |n: usize| !s.is_anonymous() || n >= usize::from(s.min_group);
    let (kept, left_out): (Vec<_>, Vec<_>) = by_batch.into_values().partition(|v| enough(v.len()));
    (
        kept.into_iter().flatten().collect(),
        left_out.iter().map(Vec::len).sum(),
    )
}
