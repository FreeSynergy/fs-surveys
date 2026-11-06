//! Envelope building for every object kind of the main store.
//!
//! Pattern: Adapter — maps typed values onto AS2 envelopes (ADR 0020). The `name` is never empty.

use fs_store::{Envelope, ObjectId};
use serde_json::{Value, json};

use super::consts::CONTEXT;
use super::handle::SurveyRepo;
use crate::model::NS;

/// The JSON-LD context of the content written here.
pub(super) fn context() -> Value {
    json!(["https://www.w3.org/ns/activitystreams", {"fs": NS}])
}

impl SurveyRepo {
    /// A fresh envelope of `object_type` with this program's context, actor and IRI.
    pub(super) fn fresh_envelope(&self, object_type: &str, name: &str) -> Envelope {
        let mut e = Envelope::new(object_type, name);
        e.iri = Some(self.iri(e.id));
        e.context = Some(CONTEXT.into());
        e.attributed_to = Some(self.actor.clone());
        e.to = vec![self.actor.clone()];
        e
    }

    /// The external IRI of a survey, used as `inReplyTo`.
    pub(super) fn survey_ref(&self, id: ObjectId) -> String {
        self.iri(id)
    }
}
