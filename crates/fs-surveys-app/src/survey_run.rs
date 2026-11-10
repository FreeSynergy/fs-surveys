//! The commands of the `Survey` type.
//!
//! Pattern: Command (one arm per row of the table) — each arm reads typed input, calls the core and
//! answers with the IRI of the survey. The `change` command is an `Edit` (rule of 2026-11-10): a field
//! not in the input stays, a field with no values is cleared, otherwise the values replace.

use fs_admin_frontend::{Outcome, Values};
use fs_surveys_core::commands::ANONYMITY;
use fs_surveys_core::describe::camel;
use fs_surveys_core::{Anonymity, Designer, Results, Survey};

use crate::ctx::Ctx;
use crate::ids::{iri, tail};
use crate::input::{given, id_of, ids_of, list, need, number, pick, text, yes_no};
use crate::methods::method_of;
use crate::refuse::RunError;
use crate::results;
use crate::survey::survey_iri;

const MIN_GROUP: u16 = fs_surveys_core::model::DEFAULT_MIN_GROUP;

fn anonymity(v: &Values) -> Result<Option<Anonymity>, RunError> {
    Ok(pick(v, "anonymity", ANONYMITY)?.and_then(|i| Anonymity::parse(&camel(ANONYMITY[i]))))
}

fn done(ctx: &Ctx, s: &Survey) -> Outcome {
    Outcome {
        id: Some(survey_iri(ctx, s)),
        values: Values::new(),
    }
}

fn load(ctx: &Ctx, id: fs_objects::ObjectId) -> Result<Survey, RunError> {
    ctx.block(ctx.repo.get(id))?.ok_or(RunError::NotFound)
}

/// `change` on a survey: only `name`, `anonymity`, `minGroup` and `categories` are editable.
fn edit(ctx: &Ctx, v: &Values) -> Result<Outcome, RunError> {
    let id = id_of(v, "id")?;
    let mut s = load(ctx, id)?;
    if let Some(bad) = v
        .keys()
        .find(|k| !["id", "name", "anonymity", "minGroup", "categories"].contains(&k.as_str()))
    {
        return Err(RunError::Bad {
            field: bad.clone(),
            index: 0,
        });
    }
    if given(v, "name") || given(v, "anonymity") || given(v, "minGroup") {
        let name = if given(v, "name") {
            need(v, "name")?.to_owned()
        } else {
            s.envelope.name.clone()
        };
        let level = if given(v, "anonymity") {
            anonymity(v)?.unwrap_or(Anonymity::Named)
        } else {
            s.anonymity
        };
        let min = if given(v, "minGroup") {
            number::<u16>(v, "minGroup")?.unwrap_or(MIN_GROUP)
        } else {
            s.min_group
        };
        s = ctx.block(ctx.repo.edit(id, &name, level, min))?;
    }
    if given(v, "categories") {
        let categories: Vec<String> = list(v, "categories")
            .into_iter()
            .map(str::to_owned)
            .collect();
        s = ctx.block(ctx.repo.classify(id, &categories))?;
    }
    Ok(done(ctx, &s))
}

fn create(ctx: &Ctx, v: &Values) -> Result<Outcome, RunError> {
    let name = need(v, "name")?;
    let level = anonymity(v)?.unwrap_or(Anonymity::Named);
    let scope = ids_of(v, "scope")?;
    let s = match text(v, "creator") {
        Some(creator) => ctx.block(ctx.repo.create_by(name, level, &scope, creator))?,
        None => ctx.block(ctx.repo.create(name, level, &scope))?,
    };
    Ok(done(ctx, &s))
}

fn invite(ctx: &Ctx, v: &Values) -> Result<Outcome, RunError> {
    let people: Vec<String> = list(v, "people").into_iter().map(str::to_owned).collect();
    let r = ctx.block(
        ctx.repo
            .invite(id_of(v, "id")?, &people, &ids_of(v, "groups")?),
    )?;
    let mut values = Values::new();
    values.insert("invited".into(), r.invited);
    values.insert("already".into(), r.already);
    values.insert("refused".into(), r.refused);
    Ok(Outcome { id: None, values })
}

fn evaluate(ctx: &Ctx, v: &Values) -> Result<Outcome, RunError> {
    let id = id_of(v, "id")?;
    let question = tail(need(v, "question")?)?;
    let method = method_of(need(v, "method")?, number(v, "thresholdPercent")?)?;
    let stage = number(v, "stage")?.unwrap_or(fs_surveys_core::ALL_STAGES);
    Ok(results::evaluation(
        &ctx.block(ctx.repo.evaluate(id, question, method, stage))?,
    ))
}

fn export(ctx: &Ctx, v: &Values, ai: bool) -> Result<Outcome, RunError> {
    let id = id_of(v, "id")?;
    let data = if ai {
        ctx.block(ctx.repo.export_ai(id, "text/csv"))?
    } else {
        ctx.block(ctx.repo.export(id, "text/csv"))?
    };
    let mut values = Values::new();
    values.insert(
        "data".into(),
        vec![String::from_utf8_lossy(&data).into_owned()],
    );
    Ok(Outcome { id: None, values })
}

/// Run the command `name` (already known to be supported).
pub fn run(ctx: &Ctx, name: &str, v: &Values) -> Result<Outcome, RunError> {
    let id = || id_of(v, "id");
    let reply = |s: Result<Survey, fs_surveys_core::Error>| Ok(done(ctx, &s?));
    match name {
        "create" => create(ctx, v),
        "edit" => edit(ctx, v),
        "anonymity" => {
            let (level, min) = (
                anonymity(v)?.ok_or(RunError::Missing("anonymity"))?,
                number::<u16>(v, "minGroup")?,
            );
            let current = load(ctx, id()?)?;
            let name = current.envelope.name.clone();
            reply(ctx.block(ctx.repo.edit(
                current.id(),
                &name,
                level,
                min.unwrap_or(current.min_group),
            )))
        }
        "classify" => {
            let categories: Vec<String> = list(v, "categories")
                .into_iter()
                .map(str::to_owned)
                .collect();
            reply(ctx.block(ctx.repo.classify(id()?, &categories)))
        }
        "publish" => reply(ctx.block(ctx.repo.publish(id()?))),
        "close" => reply(ctx.block(ctx.repo.close(id()?))),
        "reopen" => reply(ctx.block(ctx.repo.reopen(id()?))),
        "archive" => reply(ctx.block(ctx.repo.archive(id()?))),
        "delete" => {
            ctx.block(ctx.repo.delete(id()?))?;
            Ok(Outcome::default())
        }
        "invite" => invite(ctx, v),
        "widen" => reply(ctx.block(ctx.repo.widen(id()?, id_of(v, "parent")?))),
        "aiAllowed" => {
            let allowed = yes_no(v, "allowed")?.ok_or(RunError::Missing("allowed"))?;
            reply(ctx.block(ctx.repo.set_ai_allowed(id()?, need(v, "by")?, allowed)))
        }
        "summary" => {
            let s = ctx.block(ctx.repo.summary(
                id()?,
                number(v, "stage")?.unwrap_or(fs_surveys_core::ALL_STAGES),
            ))?;
            Ok(results::summary(&s, &iri(&ctx.instance, "surveys", id()?)))
        }
        "filter" => {
            let (q, c) = (tail(need(v, "question")?)?, need(v, "choice")?);
            let stage = number(v, "stage")?.unwrap_or(fs_surveys_core::ALL_STAGES);
            let s = ctx.block(ctx.repo.filter(id()?, q, c, stage))?;
            Ok(results::summary(&s, &iri(&ctx.instance, "surveys", id()?)))
        }
        "evaluate" => evaluate(ctx, v),
        "export" => export(ctx, v, false),
        "exportAi" => export(ctx, v, true),
        _ => Err(RunError::NotFound),
    }
}
