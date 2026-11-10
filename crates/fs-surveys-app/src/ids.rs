//! IRIs of the things this program owns, and the ids inside them.
//!
//! Pattern: Adapter (identity mapping) — the frontend and the description know references as IRIs (AS2
//! `id`); the core knows `ObjectId`s. A survey is `https://<instance>/surveys/<id>`, a question
//! `<survey>/questions/<id>`, an answer `<survey>/responses/<id>`, a decision `https://<instance>/decisions/<id>`;
//! groups of `fs-groups` are `https://<instance>/groups/<id>`. Parsing takes the last segment, so an IRI of
//! another instance still names the same object (provisional, ADR 0027).

use fs_objects::ObjectId;

use crate::refuse::RunError;

/// `https://<instance>/<kind>/<id>`.
pub fn iri(instance: &str, kind: &str, id: ObjectId) -> String {
    format!("https://{instance}/{kind}/{id}")
}

/// The id at the end of an IRI (or a bare id).
pub fn tail(text: &str) -> Result<ObjectId, RunError> {
    text.trim()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or(RunError::Bad {
            field: String::new(),
            index: 0,
        })
}

/// The question IRI inside a survey.
pub fn question_iri(survey_iri: &str, question: ObjectId) -> String {
    format!("{survey_iri}/questions/{question}")
}

/// `<survey>/questions/<id>` -> both ids.
pub fn split_question(text: &str) -> Result<(ObjectId, ObjectId), RunError> {
    let question = tail(text)?;
    let (survey, _) = text
        .trim()
        .rsplit_once("/questions/")
        .ok_or(RunError::Bad {
            field: String::new(),
            index: 0,
        })?;
    Ok((tail(survey)?, question))
}
