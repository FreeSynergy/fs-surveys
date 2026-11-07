//! A recorded decision and the paging types.
//!
//! Pattern: Value objects (plain data). Paging is an Iterator over opaque cursors.

use fs_objects::{Envelope, ObjectId};
use fs_surveys_methods::Method;

/// Stage value meaning "all stages".
pub const ALL_STAGES: u16 = 0xFFFF;

/// A decision people took on a question; it may differ from the method's own winner.
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    /// Store type `Decision`; AS2 has none.
    pub envelope: Envelope,
    /// The survey.
    pub survey: ObjectId,
    /// The question.
    pub question: ObjectId,
    /// The method looked at.
    pub method: Method,
    /// Choice ids.
    pub outcome: Vec<String>,
    /// Why.
    pub note: String,
    /// Task IRIs (the tasks stay in fs-tasks).
    pub tasks: Vec<String>,
    /// The ballots it rests on; `ALL_STAGES` = all.
    pub stage: u16,
}

impl Decision {
    /// The id.
    pub fn id(&self) -> ObjectId {
        self.envelope.id
    }
}

/// Opaque position in a list (none = start).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cursor(pub String);

impl Cursor {
    /// The cursor for "skip `n` items".
    pub fn at(n: usize) -> Self {
        Self(n.to_string())
    }

    /// The offset encoded, or 0.
    pub fn offset(&self) -> usize {
        self.0.parse().unwrap_or(0)
    }
}

/// One page of results; `next` is `None` on the last page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    /// The items.
    pub items: Vec<T>,
    /// Where the next page starts.
    pub next: Option<Cursor>,
}
