//! People whose token is being delivered right now.
//!
//! Pattern: RAII / Guard — the mark is removed when the `Mark` is dropped, on every path. Delivery
//! runs outside the survey lock, so this in-memory set (shared by clones) is what stops two
//! parallel invites of one person from sending two tokens.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};

use fs_objects::ObjectId;

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

/// Open submits of the last minute per survey, in memory only.
pub(super) type Arrivals =
    Arc<Mutex<std::collections::HashMap<ObjectId, std::collections::VecDeque<std::time::Instant>>>>;

/// Note an arrival for `survey` and tell whether more than `limit` arrived within the last minute.
pub(super) fn note_arrival(set: &Arrivals, survey: ObjectId, limit: usize) -> bool {
    let now = std::time::Instant::now();
    let window = std::time::Duration::from_secs(60);
    let mut all = set.lock().unwrap_or_else(PoisonError::into_inner);
    // Forget what is older than a minute, in every survey, so the map stays small.
    all.retain(|_, marks| {
        while marks
            .front()
            .is_some_and(|t| now.saturating_duration_since(*t) > window)
        {
            marks.pop_front();
        }
        !marks.is_empty()
    });
    let marks = all.entry(survey).or_default();
    marks.push_back(now);
    // Counting past the limit is not needed, so a flood cannot grow the list.
    while marks.len() > limit + 1 {
        marks.pop_front();
    }
    marks.len() > limit
}
