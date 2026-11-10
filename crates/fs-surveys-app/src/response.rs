//! The backend of the `SurveyResponse` type.
//!
//! Pattern: Repository over the core (`Typed`) — only named answers are records. A list needs a survey and
//! a person (the core resumes one person's answer); anonymous ballots have no id and no time on purpose, so
//! they are never listed, only sent with `submit`.

use std::sync::Arc;

use fs_admin_frontend::{Outcome, Record, Values};
use fs_surveys_core::commands::{CLAIM, Command};
use fs_surveys_core::describe::RESPONSE;
use fs_surveys_core::{Credential, NamedResponse, OpenMeta, Respond, Survey, Voluntary};

use crate::answers;
use crate::ctx::Ctx;
use crate::gate::{Gated, Typed};
use crate::input::{id_of, list, need, number, pick, text};
use crate::refuse::RunError;
use crate::survey::survey_iri;

/// Named answers over the core.
pub struct ResponseBackend {
    ctx: Arc<Ctx>,
}

impl ResponseBackend {
    /// The backend, behind the gate for unsupported commands.
    pub fn new(ctx: Arc<Ctx>) -> Gated<Self> {
        Gated(Self { ctx })
    }
}

fn record(ctx: &Ctx, s: &Survey, r: &NamedResponse) -> Record {
    let at = survey_iri(ctx, s);
    let mut v = Values::new();
    v.insert("survey".into(), vec![at.clone()]);
    v.insert("person".into(), vec![r.person.clone()]);
    v.insert("answers".into(), answers::format(&at, &r.answers));
    v.insert(
        "submitted".into(),
        vec![if r.submitted { "yes" } else { "no" }.into()],
    );
    v.insert("ai".into(), vec![if r.ai { "yes" } else { "no" }.into()]);
    Record {
        id: format!("{at}/responses/{}", r.envelope.id),
        values: v,
    }
}

fn survey(ctx: &Ctx, v: &Values) -> Result<Survey, RunError> {
    ctx.block(ctx.repo.open(id_of(v, "survey")?))?
        .ok_or(RunError::NotFound)
}

fn credential(v: &Values) -> Result<Credential, RunError> {
    let claim = pick(v, "declaredAi", CLAIM)?;
    let meta = claim.is_some() || text(v, "elapsedMs").is_some() || text(v, "honeypot").is_some();
    Ok(if let Some(token) = text(v, "token") {
        Credential::Token(token.to_owned())
    } else if meta {
        Credential::Open(OpenMeta {
            declared_ai: match claim.map(|i| CLAIM[i]) {
                Some("human") => Some(false),
                Some("ai") => Some(true),
                _ => None,
            },
            elapsed_ms: number(v, "elapsedMs")?,
            honeypot: text(v, "honeypot").map(str::to_owned),
        })
    } else if let Some(person) = text(v, "person") {
        Credential::Person(person.to_owned())
    } else {
        Credential::None
    })
}

impl Typed for ResponseBackend {
    fn data_type(&self) -> &'static str {
        RESPONSE
    }

    fn records(&self, query: &Values) -> Result<Vec<Record>, RunError> {
        let (Some(_), Some(person)) = (text(query, "survey"), text(query, "person")) else {
            return Ok(Vec::new());
        };
        let s = survey(&self.ctx, query)?;
        let found = self.ctx.block(self.ctx.repo.resume(s.id(), person))?;
        Ok(found.iter().map(|r| record(&self.ctx, &s, r)).collect())
    }

    fn dispatch(&self, command: &'static Command, v: &Values) -> Result<Outcome, RunError> {
        let ctx = &*self.ctx;
        match command.name {
            "open" => {
                let s = survey(ctx, v)?;
                let mut values = Values::new();
                values.insert("survey".into(), vec![survey_iri(ctx, &s)]);
                Ok(Outcome { id: None, values })
            }
            "save" => {
                let s = survey(ctx, v)?;
                let given = answers::parse(&s, &list(v, "answers"))?;
                let r = ctx.block(ctx.repo.save_response(s.id(), need(v, "person")?, given))?;
                Ok(Outcome {
                    id: Some(record(ctx, &s, &r).id),
                    values: Values::new(),
                })
            }
            "submit" => {
                let s = survey(ctx, v)?;
                let given = answers::parse(&s, &list(v, "answers"))?;
                ctx.block(ctx.repo.submit(s.id(), credential(v)?, given))?;
                Ok(Outcome::default())
            }
            "voluntary" => {
                let details = list(v, "details")
                    .into_iter()
                    .enumerate()
                    .map(|(index, d)| {
                        d.split_once('=')
                            .map(|(k, val)| Voluntary {
                                key: k.trim().to_owned(),
                                value: val.trim().to_owned(),
                            })
                            .ok_or(RunError::Bad {
                                field: "details".into(),
                                index,
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                ctx.block(ctx.repo.add_voluntary(id_of(v, "survey")?, details))?;
                Ok(Outcome::default())
            }
            _ => Err(RunError::NotFound),
        }
    }
}
