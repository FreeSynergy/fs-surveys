//! The [`AiMarking`] strategy: does the host consider a participant an AI?
//!
//! Pattern: Strategy (injected) — this crate does not depend on fs-people. The host wires the answer
//! (for example to the role book), as `fs_groups_core::AiMarking` does. Why a strategy: who counts as
//! an AI is the host's knowledge, and the survey rules must never be fed by what a participant says.

use async_trait::async_trait;

use crate::error::Result;

/// Tells whether a person is marked as an AI. Interface status: draft (ADR 0045).
///
/// **`Some(false)` means "known to be a human", not "not marked".** An implementation answers `None`
/// for an identity it does not know. This crate treats everything except `Some(false)` as an AI:
/// `Some(true)`, `None`, no marking configured and no identity at all. An error is passed on; it
/// never counts as "human".
#[async_trait]
pub trait AiMarking: Send + Sync {
    /// `Some(true)`: marked as an AI. `Some(false)`: known, and a human. `None`: no answer.
    async fn is_ai(&self, person: &str) -> Result<Option<bool>>;
}
