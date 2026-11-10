//! The fixed lists of keys that fields and inputs choose from.
//!
//! Pattern: Registry (constants). A choice key is lower case with `-` (the description rule), so the wire
//! names of `surveys.capnp` (`anonymousOpen`) appear here as `anonymous-open`; `describe::keys` converts.

/// `Anonymity`.
pub const ANONYMITY: &[&str] = &["named", "anonymous-open", "anonymous-once", "public"];
/// `SurveyState`.
pub const STATE: &[&str] = &["draft", "open", "closed", "archived"];
/// `QuestionKind`.
pub const KIND: &[&str] = &[
    "single-choice",
    "multiple-choice",
    "text",
    "number",
    "date",
    "scale",
    "ranking",
    "option-scores",
];
/// `Method`, without the threshold (that is a field of its own).
pub const METHOD: &[&str] = &[
    "plurality",
    "majority",
    "supermajority",
    "approval",
    "score",
    "schulze",
    "instant-runoff",
    "borda",
    "consent",
    "systemic-consensing",
];
/// `ConditionOp`.
pub const OPS: &[&str] = &["answered", "equals", "includes", "at-least", "at-most"];
/// A yes/no field (there is no boolean basic type yet).
pub const YES_NO: &[&str] = &["yes", "no"];
/// `Claim` of an open answer.
pub const CLAIM: &[&str] = &["unset", "human", "ai"];
/// Media types of an export.
pub const EXPORT: &[&str] = &["text-csv"];
