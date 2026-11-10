//! The backend of the `Decision` type.
//!
//! Pattern: Repository over the core (`Typed`) — a decision is an object of its own, linked to tasks by IRI.

use std::sync::Arc;

use fs_admin_frontend::{Outcome, Record, Values};
use fs_surveys_core::commands::Command;
use fs_surveys_core::describe::{DECISION, kebab};
use fs_surveys_core::{ALL_STAGES, Decision};

use crate::ctx::Ctx;
use crate::gate::{Gated, Typed};
use crate::ids::{iri, question_iri, tail};
use crate::input::{id_of, list, need, number, text};
use crate::methods::{key_of, method_of};
use crate::refuse::RunError;
use crate::survey::all;

/// Decisions over the core.
pub struct DecisionBackend {
    ctx: Arc<Ctx>,
}

impl DecisionBackend {
    /// The backend, behind the gate for unsupported commands.
    pub fn new(ctx: Arc<Ctx>) -> Gated<Self> {
        Gated(Self { ctx })
    }
}

fn decision_iri(ctx: &Ctx, d: &Decision) -> String {
    d.envelope
        .iri
        .clone()
        .unwrap_or_else(|| iri(&ctx.instance, "decisions", d.id()))
}

fn record(ctx: &Ctx, d: &Decision) -> Record {
    let survey = iri(&ctx.instance, "surveys", d.survey);
    let (method, threshold) = key_of(d.method);
    let mut v = Values::new();
    v.insert("question".into(), vec![question_iri(&survey, d.question)]);
    v.insert("survey".into(), vec![survey]);
    v.insert("method".into(), vec![kebab(method)]);
    v.insert(
        "thresholdPercent".into(),
        threshold.map(|t| t.to_string()).into_iter().collect(),
    );
    v.insert("outcome".into(), d.outcome.clone());
    v.insert("note".into(), vec![d.note.clone()]);
    v.insert("tasks".into(), d.tasks.clone());
    v.insert("stage".into(), vec![d.stage.to_string()]);
    Record {
        id: decision_iri(ctx, d),
        values: v,
    }
}

fn answer(ctx: &Ctx, d: &Decision) -> Outcome {
    Outcome {
        id: Some(decision_iri(ctx, d)),
        values: Values::new(),
    }
}

impl Typed for DecisionBackend {
    fn data_type(&self) -> &'static str {
        DECISION
    }

    fn records(&self, query: &Values) -> Result<Vec<Record>, RunError> {
        let ctx = &*self.ctx;
        let mut found = Vec::new();
        if let Some(task) = text(query, "task") {
            found = ctx.block(ctx.repo.of_task(task))?;
        } else {
            let only = text(query, "survey").map(tail).transpose()?;
            for s in all(ctx)?
                .iter()
                .filter(|s| only.is_none_or(|o| o == s.id()))
            {
                let mut cursor = None;
                loop {
                    let page = ctx.block(ctx.repo.list_decisions(s.id(), cursor.as_ref()))?;
                    found.extend(page.items);
                    match page.next {
                        Some(next) => cursor = Some(next),
                        None => break,
                    }
                }
            }
        }
        Ok(found.iter().map(|d| record(ctx, d)).collect())
    }

    fn dispatch(&self, command: &'static Command, v: &Values) -> Result<Outcome, RunError> {
        let ctx = &*self.ctx;
        match command.name {
            "record" => {
                let threshold = number(v, "thresholdPercent")?;
                let method = method_of(need(v, "method")?, threshold)?;
                let outcome = list(v, "outcome").into_iter().map(str::to_owned).collect();
                let note = text(v, "note").unwrap_or("");
                let stage = number(v, "stage")?.unwrap_or(ALL_STAGES);
                let d = ctx.block(ctx.repo.record(
                    id_of(v, "survey")?,
                    tail(need(v, "question")?)?,
                    method,
                    stage,
                    outcome,
                    note,
                ))?;
                Ok(answer(ctx, &d))
            }
            "linkTask" => Ok(answer(
                ctx,
                &ctx.block(ctx.repo.link_task(id_of(v, "id")?, need(v, "task")?))?,
            )),
            "unlinkTask" => Ok(answer(
                ctx,
                &ctx.block(ctx.repo.unlink_task(id_of(v, "id")?, need(v, "task")?))?,
            )),
            _ => Err(RunError::NotFound),
        }
    }
}
