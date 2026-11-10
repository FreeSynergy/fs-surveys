//! The backend of the `Survey` type: the records.
//!
//! Pattern: Repository over the core (`Typed`) — reading is here, the commands are in `survey_run.rs`.

use std::sync::Arc;

use fs_admin_frontend::{Outcome, Record, Values};
use fs_surveys_core::commands::Command;
use fs_surveys_core::describe::kebab;
use fs_surveys_core::{Survey, describe::SURVEY};

use crate::ctx::Ctx;
use crate::gate::{Gated, Typed};
use crate::ids::{iri, question_iri};
use crate::input;
use crate::refuse::RunError;

/// Surveys over the core.
pub struct SurveyBackend {
    pub(crate) ctx: Arc<Ctx>,
}

impl SurveyBackend {
    /// The backend, behind the gate for unsupported commands.
    pub fn new(ctx: Arc<Ctx>) -> Gated<Self> {
        Gated(Self { ctx })
    }
}

/// The IRI of a survey.
pub fn survey_iri(ctx: &Ctx, s: &Survey) -> String {
    s.envelope
        .iri
        .clone()
        .unwrap_or_else(|| iri(&ctx.instance, "surveys", s.id()))
}

/// A survey as a record of typed values.
pub fn record(ctx: &Ctx, s: &Survey) -> Record {
    let at = survey_iri(ctx, s);
    let mut v = Values::new();
    v.insert("name".into(), vec![s.envelope.name.clone()]);
    v.insert("state".into(), vec![kebab(s.state.as_str())]);
    v.insert("anonymity".into(), vec![kebab(s.anonymity.as_str())]);
    v.insert(
        "scope".into(),
        s.scope
            .iter()
            .map(|g| iri(&ctx.instance, "groups", *g))
            .collect(),
    );
    v.insert("stage".into(), vec![s.stage.to_string()]);
    v.insert("minGroup".into(), vec![s.min_group.to_string()]);
    v.insert(
        "aiAllowed".into(),
        vec![if s.ai_allowed { "yes" } else { "no" }.into()],
    );
    if let Some(c) = &s.creator {
        v.insert("creator".into(), vec![c.clone()]);
    }
    v.insert("categories".into(), s.envelope.sensitive.clone());
    v.insert(
        "questions".into(),
        s.questions
            .iter()
            .map(|q| question_iri(&at, q.id))
            .collect(),
    );
    Record { id: at, values: v }
}

/// Every survey, all pages.
pub(crate) fn all(ctx: &Ctx) -> Result<Vec<Survey>, RunError> {
    let mut out = Vec::new();
    let mut cursor = None;
    loop {
        let page = ctx.block(ctx.repo.list(cursor.as_ref()))?;
        out.extend(page.items);
        match page.next {
            Some(next) => cursor = Some(next),
            None => return Ok(out),
        }
    }
}

impl Typed for SurveyBackend {
    fn data_type(&self) -> &'static str {
        SURVEY
    }

    fn records(&self, query: &Values) -> Result<Vec<Record>, RunError> {
        let term = input::text(query, "term").map(str::to_lowercase);
        let state = input::text(query, "state");
        Ok(all(&self.ctx)?
            .iter()
            .filter(|s| {
                term.as_ref()
                    .is_none_or(|t| s.envelope.name.to_lowercase().contains(t))
            })
            .filter(|s| state.is_none_or(|k| kebab(s.state.as_str()) == k))
            .map(|s| record(&self.ctx, s))
            .collect())
    }

    fn dispatch(&self, command: &'static Command, input: &Values) -> Result<Outcome, RunError> {
        crate::survey_run::run(&self.ctx, command.name, input)
    }
}
