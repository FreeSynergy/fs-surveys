//! The content of one box: pending ballots, released batches, token hashes.
//!
//! Pattern: Aggregate — one value per survey, changed only through its methods. A ballot has no
//! person, time or id; it enters at a random position; release shuffles it.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::Answer;
use crate::random::{insert_random, shuffle};

/// One anonymous ballot: answers in question order, nothing else.
pub type Ballot = Vec<Answer>;

/// Ballots released together at one stage.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Batch {
    stage: u16,
    ballots: Vec<Ballot>,
}

/// The box of a survey.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BallotBox {
    pending: Vec<Ballot>,
    batches: Vec<Batch>,
    tokens: Vec<String>,
}

impl BallotBox {
    /// Add a ballot at a random position among the pending ones.
    pub fn add_ballot(&mut self, ballot: Ballot) -> Result<()> {
        insert_random(&mut self.pending, ballot)
    }

    /// Keep a token hash (sorted, so the order says nothing about who was invited when).
    pub fn add_token(&mut self, hash: String) {
        if let Err(at) = self.tokens.binary_search(&hash) {
            self.tokens.insert(at, hash);
        }
    }

    /// Remove a token hash; false if it is not there (spent or never issued).
    pub fn spend(&mut self, hash: &str) -> bool {
        self.tokens
            .binary_search_by(|t| t.as_str().cmp(hash))
            .map(|at| self.tokens.remove(at))
            .is_ok()
    }

    /// Release the pending ballots as one shuffled batch of `stage`, if there are at least
    /// `min_group`; a smaller rest stays. Returns whether a batch was made.
    pub fn release(&mut self, stage: u16, min_group: u16) -> Result<bool> {
        if self.pending.is_empty() || self.pending.len() < usize::from(min_group) {
            return Ok(false);
        }
        let mut ballots = std::mem::take(&mut self.pending);
        shuffle(&mut ballots)?;
        self.batches.push(Batch { stage, ballots });
        Ok(true)
    }

    /// Released ballots of `stage` (`0xFFFF` = all).
    pub fn released(&self, stage: u16) -> Vec<&Ballot> {
        self.batches
            .iter()
            .filter(|b| stage == crate::model::ALL_STAGES || b.stage == stage)
            .flat_map(|b| b.ballots.iter())
            .collect()
    }

    /// Ballots not released yet.
    pub fn held_back(&self) -> usize {
        self.pending.len()
    }
}
