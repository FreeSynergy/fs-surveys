//! The decision methods as description keys.
//!
//! Pattern: Adapter — `plurality` ... `systemic-consensing` are the keys of the choice field `method`;
//! the supermajority carries its threshold in a field of its own (`thresholdPercent`).

use fs_surveys_core::Method;

use crate::refuse::RunError;

/// The key of a method and its threshold, if it has one.
pub fn key_of(m: Method) -> (&'static str, Option<u8>) {
    match m {
        Method::Plurality => ("plurality", None),
        Method::Majority => ("majority", None),
        Method::Supermajority { threshold_percent } => ("supermajority", Some(threshold_percent)),
        Method::Approval => ("approval", None),
        Method::Score => ("score", None),
        Method::Schulze => ("schulze", None),
        Method::InstantRunoff => ("instant-runoff", None),
        Method::Borda => ("borda", None),
        Method::Consent => ("consent", None),
        Method::SystemicConsensing => ("systemic-consensing", None),
    }
}

/// The method of a key; `threshold` is needed for the supermajority only.
pub fn method_of(key: &str, threshold: Option<u8>) -> Result<Method, RunError> {
    Ok(match key {
        "plurality" => Method::Plurality,
        "majority" => Method::Majority,
        "supermajority" => Method::Supermajority {
            threshold_percent: threshold.ok_or(RunError::Missing("thresholdPercent"))?,
        },
        "approval" => Method::Approval,
        "score" => Method::Score,
        "schulze" => Method::Schulze,
        "instant-runoff" => Method::InstantRunoff,
        "borda" => Method::Borda,
        "consent" => Method::Consent,
        "systemic-consensing" => Method::SystemicConsensing,
        _ => {
            return Err(RunError::Bad {
                field: "method".into(),
                index: 0,
            });
        }
    })
}
