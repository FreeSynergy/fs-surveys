//! The backend of the `Question` type.
//!
//! Pattern: Repository over the core (`Typed`) — a question lives in its survey; its record id is
//! `<survey IRI>/questions/<id>`. `change` is an `Edit` (a field not given stays, an empty one is cleared).

use std::sync::Arc;

use fs_admin_frontend::{Outcome, Record, Values};
use fs_objects::ObjectId;
use fs_surveys_core::commands::{Command, KIND, OPS};
use fs_surveys_core::describe::{QUESTION, camel, kebab};
use fs_surveys_core::{Choice, Condition, ConditionOp, Question, QuestionKind, Survey};

use crate::ctx::Ctx;
use crate::gate::{Gated, Typed};
use crate::ids::{question_iri, split_question, tail};
use crate::input::{given, id_of, list, need, number, pick, text, yes_no};
use crate::methods::{key_of, method_of};
use crate::refuse::RunError;
use crate::survey::{all, survey_iri};

/// Questions over the core.
pub struct QuestionBackend {
    ctx: Arc<Ctx>,
}

impl QuestionBackend {
    /// The backend, behind the gate for unsupported commands.
    pub fn new(ctx: Arc<Ctx>) -> Gated<Self> {
        Gated(Self { ctx })
    }
}

fn condition_line(survey_iri: &str, c: &Condition) -> String {
    format!(
        "{} {} {}",
        question_iri(survey_iri, c.question),
        kebab(c.op.as_str()),
        c.value
    )
}

fn record(ctx: &Ctx, s: &Survey, q: &Question) -> Record {
    let at = survey_iri(ctx, s);
    let mut v = Values::new();
    v.insert("survey".into(), vec![at.clone()]);
    v.insert("text".into(), vec![q.text.clone()]);
    v.insert("kind".into(), vec![kebab(q.kind.as_str())]);
    v.insert(
        "required".into(),
        vec![if q.required { "yes" } else { "no" }.into()],
    );
    v.insert(
        "choices".into(),
        q.choices
            .iter()
            .map(|c| format!("{}={}", c.id, c.label))
            .collect(),
    );
    v.insert("min".into(), vec![q.min.to_string()]);
    v.insert("max".into(), vec![q.max.to_string()]);
    for (key, n) in [("minNumber", q.min_number), ("maxNumber", q.max_number)] {
        v.insert(key.into(), n.map(|n| n.to_string()).into_iter().collect());
    }
    let (method, threshold) = key_of(q.method);
    v.insert("method".into(), vec![method.into()]);
    v.insert(
        "thresholdPercent".into(),
        threshold.map(|t| t.to_string()).into_iter().collect(),
    );
    v.insert(
        "conditions".into(),
        q.conditions
            .iter()
            .map(|c| condition_line(&at, c))
            .collect(),
    );
    Record {
        id: question_iri(&at, q.id),
        values: v,
    }
}

/// `id=label` or just `label` (then the id is the label).
fn choice(line: &str) -> Choice {
    let (id, label) = line.split_once('=').unwrap_or((line, line));
    Choice {
        id: id.trim().to_owned(),
        label: label.trim().to_owned(),
    }
}

fn conditions(v: &Values) -> Result<Vec<Condition>, RunError> {
    list(v, "conditions")
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let bad = || RunError::Bad {
                field: "conditions".into(),
                index,
            };
            let mut parts = line.splitn(3, ' ');
            let (at, op, value) = (
                parts.next().ok_or_else(bad)?,
                parts.next().ok_or_else(bad)?,
                parts.next().unwrap_or(""),
            );
            let op = OPS
                .iter()
                .find(|k| **k == op)
                .and_then(|k| ConditionOp::parse(&camel(k)))
                .ok_or_else(bad)?;
            Ok(Condition {
                question: tail(at).map_err(|_| bad())?,
                op,
                value: value.trim().to_owned(),
            })
        })
        .collect()
}

/// Apply the fields of `v` that are given to `q` (partial: the rest stays).
fn apply(q: &mut Question, v: &Values) -> Result<(), RunError> {
    if given(v, "text") {
        need(v, "text")?.clone_into(&mut q.text);
    }
    if let Some(i) = pick(v, "kind", KIND)? {
        q.kind = QuestionKind::parse(&camel(KIND[i])).ok_or(RunError::Missing("kind"))?;
    }
    if given(v, "required") {
        q.required = yes_no(v, "required")?.unwrap_or(false);
    }
    if given(v, "choices") {
        q.choices = list(v, "choices").into_iter().map(choice).collect();
    }
    if given(v, "min") {
        q.min = number(v, "min")?.unwrap_or(0);
    }
    if given(v, "max") {
        q.max = number(v, "max")?.unwrap_or(0);
    }
    if given(v, "minNumber") {
        q.min_number = number(v, "minNumber")?;
    }
    if given(v, "maxNumber") {
        q.max_number = number(v, "maxNumber")?;
    }
    if given(v, "method") || given(v, "thresholdPercent") {
        let current = key_of(q.method);
        let key = text(v, "method").unwrap_or(current.0);
        let threshold = if given(v, "thresholdPercent") {
            number(v, "thresholdPercent")?
        } else {
            current.1
        };
        q.method = method_of(key, threshold)?;
    }
    if given(v, "conditions") {
        q.conditions = conditions(v)?;
    }
    Ok(())
}

fn find(ctx: &Ctx, at: &str) -> Result<(Survey, Question), RunError> {
    let (sid, qid) = split_question(at)?;
    let s = ctx.block(ctx.repo.get(sid))?.ok_or(RunError::NotFound)?;
    let q = s.question(qid).cloned().ok_or(RunError::NotFound)?;
    Ok((s, q))
}

fn answer(ctx: &Ctx, s: &Survey, q: ObjectId) -> Outcome {
    Outcome {
        id: Some(question_iri(&survey_iri(ctx, s), q)),
        values: Values::new(),
    }
}

impl Typed for QuestionBackend {
    fn data_type(&self) -> &'static str {
        QUESTION
    }

    fn records(&self, query: &Values) -> Result<Vec<Record>, RunError> {
        let only = text(query, "survey").map(tail).transpose()?;
        let term = text(query, "term").map(str::to_lowercase);
        let mut out = Vec::new();
        for s in all(&self.ctx)?
            .iter()
            .filter(|s| only.is_none_or(|o| o == s.id()))
        {
            out.extend(
                s.questions
                    .iter()
                    .filter(|q| {
                        term.as_ref()
                            .is_none_or(|t| q.text.to_lowercase().contains(t))
                    })
                    .map(|q| record(&self.ctx, s, q)),
            );
        }
        Ok(out)
    }

    fn dispatch(&self, command: &'static Command, v: &Values) -> Result<Outcome, RunError> {
        let ctx = &*self.ctx;
        match command.name {
            "add" => {
                let sid = id_of(v, "survey")?;
                let kind = pick(v, "kind", KIND)?
                    .and_then(|i| QuestionKind::parse(&camel(KIND[i])))
                    .ok_or(RunError::Missing("kind"))?;
                let mut q = Question::new(kind, need(v, "text")?);
                apply(&mut q, v)?;
                let (s, qid) = ctx.block(ctx.repo.add_question(sid, q))?;
                Ok(answer(ctx, &s, qid))
            }
            "edit" => {
                let (s, mut q) = find(ctx, need(v, "id")?)?;
                apply(&mut q, v)?;
                let qid = q.id;
                let s = ctx.block(ctx.repo.edit_question(s.id(), q))?;
                Ok(answer(ctx, &s, qid))
            }
            "remove" => {
                let (sid, qid) = split_question(need(v, "id")?)?;
                ctx.block(ctx.repo.remove_question(sid, qid))?;
                Ok(Outcome::default())
            }
            "move" => {
                let (sid, qid) = split_question(need(v, "id")?)?;
                let s = ctx.block(ctx.repo.move_question(
                    sid,
                    qid,
                    number(v, "position")?.ok_or(RunError::Missing("position"))?,
                ))?;
                Ok(answer(ctx, &s, qid))
            }
            "setConditions" => {
                let (sid, qid) = split_question(need(v, "id")?)?;
                let s = ctx.block(ctx.repo.set_conditions(sid, qid, conditions(v)?))?;
                Ok(answer(ctx, &s, qid))
            }
            "types" => {
                let mut values = Values::new();
                values.insert(
                    "kinds".into(),
                    KIND.iter().map(|k| (*k).to_owned()).collect(),
                );
                Ok(Outcome { id: None, values })
            }
            _ => Err(RunError::NotFound),
        }
    }
}
