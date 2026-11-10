//! The marking this window wires: the operator is a person, nobody else is known.
//!
//! Pattern: Strategy (the injected `AiMarking`) — whoever opens this window takes the responsibility for
//! the actor being a human; every other identity gets no answer, which the core counts as an AI ("doubt
//! counts as AI", `DESIGN.md`). Provisional: when a people provider is reachable, a marking over `fs-people`
//! replaces this.

use async_trait::async_trait;
use fs_surveys_core::{AiMarking, Result};

/// Says `Some(false)` for the operator and nothing for anybody else.
pub struct OperatorIsHuman(String);

impl OperatorIsHuman {
    /// The operator's IRI.
    pub fn new(actor: &str) -> Self {
        Self(actor.trim().to_lowercase())
    }
}

#[async_trait]
impl AiMarking for OperatorIsHuman {
    async fn is_ai(&self, person: &str) -> Result<Option<bool>> {
        Ok((person.trim().to_lowercase() == self.0).then_some(false))
    }
}
