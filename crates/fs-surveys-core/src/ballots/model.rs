//! The content of one box: pending ballots, released batches, token hashes, once for people and once for AIs.
//!
//! Pattern: Aggregate — one value per survey, changed only through its methods. A ballot has no
//! person, time or id; it enters at a random position; release shuffles it. The two classes have a lane
//! each, so AI ballots never fill a batch of people, nor the other way round.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::{Answer, Class};
use crate::random::{insert_random, shuffle};

/// One anonymous ballot: answers in question order, nothing else.
pub type Ballot = Vec<Answer>;

/// Ballots released together at one stage.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Batch {
    stage: u16,
    ballots: Vec<Ballot>,
}

/// Everything of one class.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lane {
    pending: Vec<Ballot>,
    batches: Vec<Batch>,
    tokens: Vec<String>,
}

impl Lane {
    fn release(&mut self, stage: u16, min_group: u16) -> Result<()> {
        if self.pending.is_empty() || self.pending.len() < usize::from(min_group) {
            return Ok(());
        }
        let mut ballots = std::mem::take(&mut self.pending);
        shuffle(&mut ballots)?;
        self.batches.push(Batch { stage, ballots });
        Ok(())
    }
}

/// The box of a survey. The lane of people keeps the layout of version 0.1.0.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BallotBox {
    #[serde(flatten)]
    people: Lane,
    #[serde(default)]
    ai: Lane,
}

impl BallotBox {
    fn lane(&self, class: Class) -> &Lane {
        if class.is_ai() {
            &self.ai
        } else {
            &self.people
        }
    }

    fn lane_mut(&mut self, class: Class) -> &mut Lane {
        if class.is_ai() {
            &mut self.ai
        } else {
            &mut self.people
        }
    }

    /// Add a ballot at a random position among the pending ones of its class.
    pub fn add_ballot(&mut self, class: Class, ballot: Ballot) -> Result<()> {
        insert_random(&mut self.lane_mut(class).pending, ballot)
    }

    /// Keep a token hash in the lane of the class it was issued to (sorted, so the order says nothing
    /// about who was invited when).
    pub fn add_token(&mut self, class: Class, hash: String) {
        let tokens = &mut self.lane_mut(class).tokens;
        if let Err(at) = tokens.binary_search(&hash) {
            tokens.insert(at, hash);
        }
    }

    /// The class a token hash was issued to; none if it is spent or was never issued.
    pub fn token_class(&self, hash: &str) -> Option<Class> {
        Class::BOTH.into_iter().find(|c| {
            let t = &self.lane(*c).tokens;
            t.binary_search_by(|x| x.as_str().cmp(hash)).is_ok()
        })
    }

    /// Remove a token hash from either lane; false if it is not there (spent or never issued).
    pub fn spend(&mut self, hash: &str) -> bool {
        let Some(class) = self.token_class(hash) else {
            return false;
        };
        let tokens = &mut self.lane_mut(class).tokens;
        tokens
            .binary_search_by(|t| t.as_str().cmp(hash))
            .map(|at| tokens.remove(at))
            .is_ok()
    }

    /// Release the pending ballots of each class as one shuffled batch of `stage`, if there are at
    /// least `min_group` of that class; a smaller rest stays.
    pub fn release(&mut self, stage: u16, min_group: u16) -> Result<()> {
        for c in Class::BOTH {
            self.lane_mut(c).release(stage, min_group)?;
        }
        Ok(())
    }

    /// Released ballots of `class` and `stage` (`0xFFFF` = all), each with the number of its batch.
    pub fn released(&self, class: Class, stage: u16) -> Vec<(usize, &Ballot)> {
        self.lane(class)
            .batches
            .iter()
            .enumerate()
            .filter(|(_, b)| stage == crate::model::ALL_STAGES || b.stage == stage)
            .flat_map(|(i, b)| b.ballots.iter().map(move |x| (i, x)))
            .collect()
    }

    /// Ballots of `class` not released yet.
    pub fn held_back(&self, class: Class) -> usize {
        self.lane(class).pending.len()
    }

    /// Whether the AI lane holds any ballot, released or not.
    pub fn has_ai_ballots(&self) -> bool {
        !self.ai.pending.is_empty() || !self.ai.batches.is_empty()
    }
}
