//! The class of a participant.
//!
//! Pattern: Value object — `Human` or `Ai`, decided once (at submit, or when a token is issued) and kept
//! with the answer. Its text form is the stored one.

use serde::{Deserialize, Serialize};

/// Who an answer is counted with. Answers of the two classes are never added together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Class {
    /// Known as a human by the marking (`Some(false)`).
    Human,
    /// Marked as an AI, or not known as a human: doubt counts as AI.
    Ai,
}

impl Class {
    /// Both classes, people first.
    pub const BOTH: [Self; 2] = [Self::Human, Self::Ai];

    /// Whether this is the AI class.
    pub fn is_ai(self) -> bool {
        self == Self::Ai
    }
}
