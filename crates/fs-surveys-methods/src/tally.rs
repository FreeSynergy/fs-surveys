//! Result types of an evaluation.
//!
//! Pattern: Value objects (plain data, no behaviour).

/// The result of one choice.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionResult {
    /// Choice id.
    pub choice: String,
    /// The method's number for this choice (votes, sum, wins, ...).
    pub value: f64,
    /// Rank, 1 is best; equal results share a rank (1, 1, 3).
    pub rank: u16,
}

/// The outcome of evaluating ballots under one method.
#[derive(Debug, Clone, PartialEq)]
pub struct Tally {
    /// Winning choice ids; empty when nothing is decided.
    pub winners: Vec<String>,
    /// Every choice, sorted by rank, then by the order of `choices`.
    pub options: Vec<OptionResult>,
    /// Plain-English explanation when nothing is decided or a tie needs reading.
    pub note: String,
}
