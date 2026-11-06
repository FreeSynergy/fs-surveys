//! The root of the contract: what this provider can do.
//!
//! Pattern: Null Object (report) — every capability id of `surveys.capnp` is listed, supported or not (ADR 0045).

use fs_interfaces::Capabilities;

/// Root interface `Surveys`; the other interfaces are the other traits of this module.
pub trait Surveys: Send + Sync {
    /// What this provider can do, per capability id.
    fn capabilities(&self) -> Capabilities;
}
