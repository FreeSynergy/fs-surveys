//! Ports to other programs.
//!
//! Pattern: Strategy (ADR 0043) — the core knows only these traits; fs-groups, fs-messenger and the
//! capnp clients plug in behind them.

use async_trait::async_trait;
use fs_store::ObjectId;

use crate::error::Result;

/// Groups as the surveys core needs them (from `groups.capnp`).
#[async_trait]
pub trait GroupLookup: Send + Sync {
    /// All parents, grandparents, ... of a group.
    async fn ancestors(&self, group: ObjectId) -> Result<Vec<ObjectId>>;
    /// The people (person IRIs) in a group, nested groups included.
    async fn members(&self, group: ObjectId) -> Result<Vec<String>>;
}

/// Sends a token straight to a person. The core never returns it to its caller.
#[async_trait]
pub trait TokenDelivery: Send + Sync {
    /// Deliver `token` for `survey` to `person`. An error means: nothing was delivered.
    async fn send(&self, person: &str, survey: ObjectId, token: &str) -> Result<()>;
}
