//! Open surveys: what a respondent says about themselves, and the patterns shown beside the result.
//!
//! Pattern: Value objects (plain data). Nobody has to prove anything on an open survey. The respondent
//! says whether they are an AI; that is a **claim**, never a proof, and the interface says so. Nothing
//! is dropped silently: every answer is counted, and the patterns below are shown next to the figures.

use serde::{Deserialize, Serialize};

/// What an open respondent hands in with the answers.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct OpenMeta {
    /// The answer to "Are you an AI?". Required: no answer, no ballot.
    pub declared_ai: Option<bool>,
    /// Milliseconds between showing the form and sending it, measured by the frontend. Only used to
    /// count the pattern "too fast"; it is not stored with the ballot.
    pub elapsed_ms: Option<u32>,
    /// The hidden field. A person never sees it; any text in it is a pattern.
    pub honeypot: Option<String>,
}

impl std::fmt::Debug for OpenMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OpenMeta(..)")
    }
}

/// Counts of the patterns, kept per lane and released with its batch, never per ballot, so no flag
/// can be tied to one answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Signals {
    /// Open ballots that were looked at.
    pub ballots: u32,
    /// Sent faster than a person can read the questions.
    pub too_fast: u32,
    /// The hidden field was filled.
    pub honeypot: u32,
    /// The same choice in every row of a block of choice or scale questions.
    pub straight_line: u32,
    /// Arrived while many others arrived in the same minute.
    pub burst: u32,
    /// The frontend gave no timing, so "too fast" could not be told.
    pub no_timing: u32,
}

impl Signals {
    /// Add another batch's counts.
    pub fn add(&mut self, other: &Self) {
        self.ballots = self.ballots.saturating_add(other.ballots);
        self.too_fast = self.too_fast.saturating_add(other.too_fast);
        self.honeypot = self.honeypot.saturating_add(other.honeypot);
        self.straight_line = self.straight_line.saturating_add(other.straight_line);
        self.burst = self.burst.saturating_add(other.burst);
        self.no_timing = self.no_timing.saturating_add(other.no_timing);
    }
}

/// The patterns shown beside the figures of a lane of an open survey. Nothing here removes an answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Patterns {
    /// The counts from submit time.
    pub signals: Signals,
    /// Answers whose free text equals the free text of another answer (counted when read).
    pub same_text: u32,
}

/// One voluntary detail given after the answers (for example `age` = `30-39`). It is kept apart from
/// the ballots, with no link to one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voluntary {
    /// The field.
    pub key: String,
    /// Its value.
    pub value: String,
}
