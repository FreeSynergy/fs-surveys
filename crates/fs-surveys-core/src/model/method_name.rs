//! Text form of a decision method, for stored JSON.
//!
//! Pattern: Adapter — maps the methods crate's `Method` (no serde there) to `name[:threshold]`.

use fs_surveys_methods::Method;
use serde::{Deserialize, Deserializer, Serializer};

/// `plurality`, ..., `supermajority:60`.
pub fn method_text(m: Method) -> String {
    match m {
        Method::Plurality => "plurality".into(),
        Method::Majority => "majority".into(),
        Method::Supermajority { threshold_percent } => format!("supermajority:{threshold_percent}"),
        Method::Approval => "approval".into(),
        Method::Score => "score".into(),
        Method::Schulze => "schulze".into(),
        Method::InstantRunoff => "instantRunoff".into(),
        Method::Borda => "borda".into(),
        Method::Consent => "consent".into(),
        Method::SystemicConsensing => "systemicConsensing".into(),
    }
}

/// The method with this text, if it is one.
pub fn method_parse(s: &str) -> Option<Method> {
    Some(match s {
        "plurality" => Method::Plurality,
        "majority" => Method::Majority,
        "approval" => Method::Approval,
        "score" => Method::Score,
        "schulze" => Method::Schulze,
        "instantRunoff" => Method::InstantRunoff,
        "borda" => Method::Borda,
        "consent" => Method::Consent,
        "systemicConsensing" => Method::SystemicConsensing,
        other => Method::Supermajority {
            threshold_percent: other.strip_prefix("supermajority:")?.parse().ok()?,
        },
    })
}

/// `serde(with = ...)` support for a `Method` field.
pub mod serde_method {
    use super::{Deserialize, Deserializer, Method, Serializer, method_parse, method_text};

    /// Write the method as its text.
    #[allow(clippy::trivially_copy_pass_by_ref)] // the signature serde's `with` requires
    pub fn serialize<S: Serializer>(m: &Method, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&method_text(*m))
    }

    /// Read a method from its text.
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Method, D::Error> {
        let t = String::deserialize(d)?;
        method_parse(&t).ok_or_else(|| serde::de::Error::custom(format!("unknown method {t}")))
    }
}
