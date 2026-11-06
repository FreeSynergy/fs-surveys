//! The survey value and its (de)serialisation as one AS2 `OrderedCollection` object.
//!
//! Pattern: Adapter — the survey is one `StoredObject`; the AS2 JSON is the standard-shaped view and
//! the `fs:` members carry the profile (DESIGN.md, "One survey is one object").

use fs_store::{Envelope, ObjectId, StoredObject};
use serde_json::{Value, json};

use super::kinds::{Anonymity, SurveyState};
use super::question::Question;

/// Namespace of the `fs:` members.
pub const NS: &str = "https://freesynergy.net/ns#";
/// Media type of the stored content.
pub const AS2_TYPE: &str = "application/activity+json";
/// Smallest batch of an anonymous survey.
pub const MIN_GROUP_FLOOR: u16 = 3;
/// Default `minGroup`.
pub const DEFAULT_MIN_GROUP: u16 = 5;

/// A survey: AS2 collection of questions plus our profile.
#[derive(Clone, Debug, PartialEq)]
pub struct Survey {
    /// The stored envelope; `sensitive` holds the DPV categories.
    pub envelope: Envelope,
    /// Life cycle state.
    pub state: SurveyState,
    /// The promise made to respondents.
    pub anonymity: Anonymity,
    /// The questions, in order.
    pub questions: Vec<Question>,
    /// Groups asked, in the order added; empty = not bound to a group.
    pub scope: Vec<ObjectId>,
    /// 0 at publishing, +1 for each widening.
    pub stage: u16,
    /// Smallest batch of anonymous results.
    pub min_group: u16,
}

impl Survey {
    /// The id of the survey.
    pub fn id(&self) -> ObjectId {
        self.envelope.id
    }

    /// Whether answers are anonymous.
    pub fn is_anonymous(&self) -> bool {
        self.anonymity != Anonymity::Named
    }

    /// The question with this id.
    pub fn question(&self, id: ObjectId) -> Option<&Question> {
        self.questions.iter().find(|q| q.id == id)
    }

    /// Position of a question in the order.
    pub fn position(&self, id: ObjectId) -> Option<usize> {
        self.questions.iter().position(|q| q.id == id)
    }

    /// The AS2 JSON; `closed` is set once the survey is closed or archived.
    pub fn as2(&self) -> Value {
        let closed = matches!(self.state, SurveyState::Closed | SurveyState::Archived);
        let items: Vec<Value> = self.questions.iter().map(|q| q.as2(closed)).collect();
        let mut v = json!({
            "@context": ["https://www.w3.org/ns/activitystreams", {"fs": NS}],
            "type": "OrderedCollection", "name": self.envelope.name,
            "totalItems": items.len(), "orderedItems": items,
            "fs:state": self.state, "fs:anonymity": self.anonymity,
            "fs:scope": self.scope, "fs:stage": self.stage, "fs:minGroup": self.min_group,
        });
        if closed {
            v["closed"] = true.into();
        }
        v
    }

    /// The object to store.
    pub fn to_object(&self) -> StoredObject {
        StoredObject::new(self.envelope.clone(), AS2_TYPE, self.as2())
            .with_index("state", self.state.as_str())
    }

    /// Read a survey back from its stored object.
    pub fn from_object(o: &StoredObject) -> Option<Self> {
        let c = &o.content;
        let items = c.get("orderedItems")?.as_array()?;
        let questions = items
            .iter()
            .map(|i| serde_json::from_value(i.get("fs:question")?.clone()).ok())
            .collect::<Option<Vec<Question>>>()?;
        Some(Self {
            envelope: o.envelope.clone(),
            state: serde_json::from_value(c.get("fs:state")?.clone()).ok()?,
            anonymity: serde_json::from_value(c.get("fs:anonymity")?.clone()).ok()?,
            questions,
            scope: serde_json::from_value(c.get("fs:scope")?.clone()).ok()?,
            stage: u16::try_from(c.get("fs:stage")?.as_u64()?).ok()?,
            min_group: u16::try_from(c.get("fs:minGroup")?.as_u64()?).ok()?,
        })
    }
}
