//! The list of methods, mirroring `enum Method` in `surveys.capnp`.
//!
//! Pattern: Tagged union used as the key of the registry.

/// A decision method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Most votes wins.
    Plurality,
    /// More than half of the ballots.
    Majority,
    /// At least `threshold_percent` of the ballots.
    Supermajority {
        /// Required share in percent, 51..=100.
        threshold_percent: u8,
    },
    /// Most approvals wins.
    Approval,
    /// Highest score sum wins.
    Score,
    /// Schulze (beatpath) ranked method.
    Schulze,
    /// Instant runoff.
    InstantRunoff,
    /// Borda count.
    Borda,
    /// Passes when nobody objects (sociocracy).
    Consent,
    /// Least total resistance, 0 to 10.
    SystemicConsensing,
}
