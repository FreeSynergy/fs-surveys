//! The content of one box: pending ballots, released batches, token hashes, once for people and once for AIs.
//!
//! Pattern: Aggregate — one value per survey, changed only through its methods. A ballot has no
//! person, time or id; it enters at a random position; release shuffles it. The two classes have a lane
//! each, so AI ballots never fill a batch of people, nor the other way round. A box of version 0.1.0 has
//! no classes; what it holds is read as AI (doubt).

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::{Answer, Class, Signals, Voluntary};
use crate::random::{insert_random, shuffle};

/// One anonymous ballot: answers in question order, nothing else.
pub type Ballot = Vec<Answer>;

/// Ballots released together at one stage.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Batch {
    stage: u16,
    ballots: Vec<Ballot>,
    /// The pattern counts of these ballots (open surveys), released with them.
    #[serde(default)]
    signals: Signals,
}

/// Everything of one class.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lane {
    pending: Vec<Ballot>,
    batches: Vec<Batch>,
    tokens: Vec<String>,
    /// Pattern counts of the pending ballots; they join a batch when it is released.
    pending_signals: Signals,
}

impl Lane {
    fn release(&mut self, stage: u16, min_group: u16) -> Result<()> {
        if self.pending.is_empty() || self.pending.len() < usize::from(min_group) {
            return Ok(());
        }
        let mut ballots = std::mem::take(&mut self.pending);
        shuffle(&mut ballots)?;
        let signals = std::mem::take(&mut self.pending_signals);
        self.batches.push(Batch {
            stage,
            ballots,
            signals,
        });
        Ok(())
    }
}

/// The box of a survey.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Stored")]
pub struct BallotBox {
    people: Lane,
    ai: Lane,
    /// Details given after the answers, in random order, tied to no ballot.
    voluntary: Vec<Vec<Voluntary>>,
}

/// The stored form. Version 0.1.0 wrote one lane at the top with no class; such ballots and token
/// hashes are not known to be people's, so doubt counts them as AI and they are moved into that lane.
#[derive(Deserialize)]
struct Stored {
    #[serde(default)]
    people: Lane,
    #[serde(default)]
    ai: Lane,
    #[serde(default)]
    voluntary: Vec<Vec<Voluntary>>,
    #[serde(flatten)]
    legacy: Lane,
}

impl From<Stored> for BallotBox {
    fn from(mut s: Stored) -> Self {
        s.ai.pending.append(&mut s.legacy.pending);
        s.ai.batches.append(&mut s.legacy.batches);
        s.ai.tokens.append(&mut s.legacy.tokens);
        s.ai.tokens.sort();
        s.ai.tokens.dedup();
        Self {
            people: s.people,
            ai: s.ai,
            voluntary: s.voluntary,
        }
    }
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

    /// Add a ballot of an open survey together with its pattern counts (kept as a count, not with the ballot).
    pub fn add_open_ballot(
        &mut self,
        class: Class,
        ballot: Ballot,
        signals: &Signals,
    ) -> Result<()> {
        self.add_ballot(class, ballot)?;
        self.lane_mut(class).pending_signals.add(signals);
        Ok(())
    }

    /// Add voluntary details at a random position among the others.
    pub fn add_voluntary(&mut self, details: Vec<Voluntary>) -> Result<()> {
        insert_random(&mut self.voluntary, details)
    }

    /// The voluntary details given so far.
    pub fn voluntary(&self) -> &[Vec<Voluntary>] {
        &self.voluntary
    }

    /// Pattern counts of the released batches of `class` and `stage` (`0xFFFF` = all).
    pub fn released_signals(&self, class: Class, stage: u16) -> Signals {
        let mut total = Signals::default();
        for b in &self.lane(class).batches {
            if stage == crate::model::ALL_STAGES || b.stage == stage {
                total.add(&b.signals);
            }
        }
        total
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
