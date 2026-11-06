//! People whose token is being delivered right now.
//!
//! Pattern: RAII / Guard — the mark is removed when the `Mark` is dropped, on every path. Delivery
//! runs outside the survey lock, so this in-memory set (shared by clones) is what stops two
//! parallel invites of one person from sending two tokens.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};

use fs_store::ObjectId;

/// The shared set of `(survey, person)` pairs in flight.
pub(super) type InFlight = Arc<Mutex<HashSet<(ObjectId, String)>>>;

/// Proof that this caller alone is delivering to the person; drops the claim when dropped.
pub(super) struct Mark {
    set: InFlight,
    key: (ObjectId, String),
}

impl Mark {
    /// Claim `(survey, person)`; `None` if somebody else holds it.
    pub(super) fn claim(set: &InFlight, survey: ObjectId, person: &str) -> Option<Self> {
        let key = (survey, person.to_string());
        let fresh = set
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.clone());
        fresh.then(|| Self {
            set: Arc::clone(set),
            key,
        })
    }
}

impl Drop for Mark {
    fn drop(&mut self) {
        self.set
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.key);
    }
}
