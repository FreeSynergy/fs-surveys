//! [`SurveyRepo`]: the store-backed implementation of the surveys traits.
//!
//! Pattern: Repository + Facade + Builder — one small type hides the two stores (main and ballot) and
//! the envelope rules; ports (groups, delivery) are injected with `with_*`.

use std::collections::HashMap;
use std::sync::{Arc, PoisonError};

use fs_objects::traits::{ObjectQuery, ObjectRepository, UnloggedObjects};
use fs_objects::{ObjectId, Store};
use tokio::sync::{Mutex, OwnedMutexGuard};

use super::backend::StoreAdapter;
use crate::traits::{GroupLookup, Registry, TokenDelivery};

/// Surveys over a main store and a ballot store. Cheap to clone; clones share one box lock.
#[derive(Clone)]
pub struct SurveyRepo {
    pub(super) objects: Arc<dyn ObjectRepository>,
    pub(super) reads: Arc<dyn ObjectQuery>,
    pub(crate) ballots: Arc<dyn UnloggedObjects>,
    pub(super) actor: String,
    pub(super) instance: String,
    pub(super) page_size: usize,
    pub(super) groups: Option<Arc<dyn GroupLookup>>,
    pub(super) delivery: Option<Arc<dyn TokenDelivery>>,
    pub(super) feed: Arc<Registry>,
    /// One lock per survey (its box and decisions of it included), from read to save.
    pub(super) inflight: super::inflight::InFlight,
    pub(crate) locks: Arc<std::sync::Mutex<HashMap<ObjectId, Arc<Mutex<()>>>>>,
}

impl SurveyRepo {
    /// Surveys over `main` (surveys, named answers, invitations, decisions) and `ballots`, a store
    /// of its own that is never connected to the hub. `actor` is the IRI of the acting program.
    pub fn new(
        main: Arc<dyn Store>,
        ballots: Arc<dyn UnloggedObjects>,
        actor: impl Into<String>,
    ) -> Self {
        let adapter = Arc::new(StoreAdapter(main));
        Self {
            objects: adapter.clone(),
            reads: adapter,
            ballots,
            actor: actor.into(),
            instance: "local.invalid".into(),
            page_size: 100,
            groups: None,
            delivery: None,
            feed: Arc::new(Registry::default()),
            locks: Arc::default(),
            inflight: Arc::default(),
        }
    }

    /// Instance domain for external IRIs `https://<instance>/surveys/<id>` (ADR 0027).
    #[must_use]
    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = instance.into();
        self
    }

    /// Groups, for inviting groups and for widening.
    #[must_use]
    pub fn with_groups(mut self, groups: Arc<dyn GroupLookup>) -> Self {
        self.groups = Some(groups);
        self
    }

    /// How tokens reach people; without it `anonymousOnce` cannot invite.
    #[must_use]
    pub fn with_delivery(mut self, delivery: Arc<dyn TokenDelivery>) -> Self {
        self.delivery = Some(delivery);
        self
    }

    /// Items per page of `list` (default 100, at least 1).
    #[must_use]
    pub fn with_page_size(mut self, n: usize) -> Self {
        self.page_size = n.max(1);
        self
    }

    /// Take the lock of one object (a survey, or a decision) until the guard is dropped. The
    /// lock is not reentrant: the public operation takes it, the helpers below it do not.
    pub(crate) async fn lock_for(&self, id: ObjectId) -> OwnedMutexGuard<()> {
        let lock = {
            let mut map = self.locks.lock().unwrap_or_else(PoisonError::into_inner);
            map.entry(id).or_default().clone()
        };
        lock.lock_owned().await
    }

    pub(super) fn iri(&self, id: ObjectId) -> String {
        format!("https://{}/surveys/{}", self.instance, id)
    }
}
