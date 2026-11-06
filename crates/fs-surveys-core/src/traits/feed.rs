//! The [`SurveyFeed`] contract: state changes and named answers, never ballots.
//!
//! Pattern: Observer — a [`SurveyListener`] subscribes and is told about every change; dropping the
//! [`Subscription`] ends it. [`Registry`] is the reusable subject.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fs_store::ObjectId;

use crate::model::SurveyState;

/// What happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurveyChange {
    /// A survey was created.
    Created(ObjectId),
    /// A survey was edited, classified or widened.
    Updated(ObjectId),
    /// A survey changed state.
    State(ObjectId, SurveyState),
    /// A survey was deleted.
    Deleted(ObjectId),
    /// A person submitted a named answer (survey, response). Anonymous ballots never appear.
    Answered(ObjectId, ObjectId),
}

/// Receives changes. Called on the thread that made the change: keep it short.
pub trait SurveyListener: Send + Sync {
    /// A survey changed.
    fn changed(&self, change: &SurveyChange);
}

/// Live updates of surveys (`survey.subscribe`).
pub trait SurveyFeed: Send + Sync {
    /// Tell `listener` about every change until the returned [`Subscription`] is dropped.
    fn subscribe(&self, listener: Arc<dyn SurveyListener>) -> Subscription;
}

type Entries = Vec<(u64, Arc<dyn SurveyListener>)>;

/// The subject of the Observer: holds the listeners and notifies them.
#[derive(Default)]
pub struct Registry {
    next: Mutex<u64>,
    listeners: Mutex<Entries>,
}

impl Registry {
    /// Add a listener; the subscription removes it again.
    pub fn add(self: &Arc<Self>, listener: Arc<dyn SurveyListener>) -> Subscription {
        let mut next = self.next.lock().unwrap_or_else(PoisonError::into_inner);
        *next += 1;
        let id = *next;
        self.entries().push((id, listener));
        Subscription {
            id,
            registry: Arc::clone(self),
        }
    }

    /// Tell every listener.
    pub fn notify(&self, change: &SurveyChange) {
        let all: Vec<_> = self.entries().iter().map(|(_, l)| l.clone()).collect();
        for l in all {
            l.changed(change);
        }
    }

    fn entries(&self) -> MutexGuard<'_, Entries> {
        self.listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// Keeps a subscription alive; dropping it unsubscribes.
#[must_use = "dropping the subscription ends it"]
pub struct Subscription {
    id: u64,
    registry: Arc<Registry>,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        let id = self.id;
        self.registry.entries().retain(|(i, _)| *i != id);
    }
}
