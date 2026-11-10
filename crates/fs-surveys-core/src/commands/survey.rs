//! The commands of the `Survey` type.
//!
//! Pattern: Registry rows (data only). One row per call of `Designer` and `Results`, plus a row for
//! every optional requirement the core does not carry (`supported = false`, with the reason).

use fs_types::BasicType::{MultilineText, Text};
use fs_types::CommandKind as K;

use super::choices::{ANONYMITY, EXPORT, METHOD, STATE, YES_NO};
use super::table::{Command, Field};

const T: &str = "Survey";
const ID: Field = Field::reference("id", "Survey").required();
const STAGE: Field = Field::new("stage", Text);
const SUMMARY: &[Field] = &[
    Field::new("ballots", Text),
    Field::choice("hidden", YES_NO),
    Field::new("heldBack", Text),
    Field::new("counts", Text).multiple(),
    Field::new("ai", Text).multiple(),
];
const LATER: &str = "optional in requirements.md, not built yet";

/// All rows of the type.
pub(super) const ROWS: &[Command] = &[
    Command::new(
        T,
        "create",
        K::Create,
        &["survey.create"],
        &[
            Field::new("name", Text).required(),
            Field::choice("anonymity", ANONYMITY),
            Field::reference("scope", "Group").multiple(),
            Field::reference("creator", "Person"),
        ],
        &[Field::reference("id", "Survey").required()],
    ),
    // The standard `change`: its input is an `Edit` over the fields of the type, nothing else (types.capnp).
    Command::new(T, "edit", K::Change, &["survey.edit"], &[], &[]),
    Command::new(
        T,
        "anonymity",
        K::Other,
        &["survey.anonymity"],
        &[
            ID,
            Field::choice("anonymity", ANONYMITY).required(),
            Field::new("minGroup", Text),
        ],
        &[],
    ),
    Command::new(
        T,
        "classify",
        K::Other,
        &["survey.classify"],
        &[ID, Field::new("categories", Text).multiple()],
        &[],
    ),
    Command::new(T, "publish", K::Other, &["survey.publish"], &[ID], &[]),
    Command::new(T, "close", K::Other, &["survey.close"], &[ID], &[]),
    Command::new(T, "reopen", K::Other, &["survey.close"], &[ID], &[]),
    Command::new(T, "archive", K::Other, &["survey.close"], &[ID], &[]),
    Command::new(T, "delete", K::Delete, &["survey.create"], &[ID], &[]),
    Command::new(
        T,
        "invite",
        K::Other,
        &["survey.invite", "survey.tokens"],
        &[
            ID,
            Field::reference("people", "Person").multiple(),
            Field::reference("groups", "Group").multiple(),
        ],
        &[
            Field::reference("invited", "Person").multiple(),
            Field::reference("already", "Person").multiple(),
            Field::reference("refused", "Person").multiple(),
        ],
    ),
    Command::new(
        T,
        "widen",
        K::Other,
        &["survey.widen"],
        &[ID, Field::reference("parent", "Group").required()],
        &[],
    ),
    Command::new(
        T,
        "aiAllowed",
        K::Other,
        &["survey.aiAllowed"],
        &[
            ID,
            Field::reference("by", "Person").required(),
            Field::choice("allowed", YES_NO).required(),
        ],
        &[],
    ),
    Command::new(
        T,
        "find",
        K::Find,
        &[],
        &[
            Field::new("term", Text),
            Field::choice("state", STATE),
        ],
        &[Field::reference("items", "Survey").multiple()],
    ),
    Command::new(
        T,
        "summary",
        K::Other,
        &["result.summary"],
        &[ID, STAGE],
        SUMMARY,
    ),
    Command::new(
        T,
        "evaluate",
        K::Other,
        &["survey.method"],
        &[
            ID,
            Field::reference("question", "Question").required(),
            Field::choice("method", METHOD).required(),
            Field::new("thresholdPercent", Text),
            STAGE,
        ],
        &[
            Field::new("winners", Text).multiple(),
            Field::new("options", Text).multiple(),
            Field::new("note", Text),
            Field::choice("hidden", YES_NO),
        ],
    ),
    Command::new(
        T,
        "filter",
        K::Other,
        &["result.filter"],
        &[
            ID,
            Field::reference("question", "Question").required(),
            Field::new("choice", Text).required(),
            STAGE,
        ],
        SUMMARY,
    ),
    Command::new(
        T,
        "export",
        K::Other,
        &["result.export"],
        &[ID, Field::choice("mediaType", EXPORT)],
        &[Field::new("data", MultilineText)],
    ),
    Command::new(
        T,
        "exportAi",
        K::Other,
        &["result.export"],
        &[ID, Field::choice("mediaType", EXPORT)],
        &[Field::new("data", MultilineText)],
    ),
    Command::new(T, "subscribe", K::Other, &["subscribe", "survey.subscribe"], &[ID], &[])
        .unsupported("the core has a subscription, but a window has no listener to keep; frontends follow by reading again"),
    Command::new(T, "embed", K::Other, &["poll.embed"], &[ID], &[])
        .unsupported("the host program embeds a survey; the core has no call for it"),
    Command::new(T, "publicLink", K::Other, &["survey.publicLink"], &[ID], &[])
        .unsupported("not built yet: adaptive proof of work, global limit, quality score"),
    Command::new(T, "date", K::Create, &["poll.date"], &[ID], &[]).unsupported(LATER),
    Command::new(T, "template", K::Create, &["template.use"], &[ID], &[]).unsupported(LATER),
    Command::new(T, "translate", K::Other, &["survey.translate"], &[ID], &[]).unsupported(LATER),
    Command::new(T, "federate", K::Other, &["poll.federate"], &[ID], &[]).unsupported(LATER),
    Command::new(T, "thread", K::Other, &["survey.thread"], &[ID], &[]).unsupported(LATER),
    Command::new(T, "groups", K::Other, &["result.groups"], &[ID], &[]).unsupported(LATER),
    Command::new(T, "delegate", K::Other, &["vote.delegate"], &[ID], &[]).unsupported(LATER),
    Command::new(T, "compose", K::Create, &["space.compose"], &[], &[]).unsupported(LATER),
    Command::new(T, "modes", K::Other, &["space.modes"], &[], &[]).unsupported(LATER),
];
