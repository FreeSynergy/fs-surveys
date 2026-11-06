//! Constants of the surveys objects.
//!
//! Pattern: Registry (constants) — store types and context used by this program in one place.

/// `context` of all objects of this program in the main store.
pub const CONTEXT: &str = "surveys";
/// AS2 type of a survey envelope.
pub const SURVEY_TYPE: &str = "OrderedCollection";
/// Store type of a named answer.
pub const RESPONSE_TYPE: &str = "SurveyResponse";
/// Store type of a decision; AS2 has none.
pub const DECISION_TYPE: &str = "Decision";
/// Store type of an invited person (main store, apart from the ballot box).
pub const INVITEE_TYPE: &str = "SurveyInvitee";
/// Store type of the ballot box (ballot store).
pub const BOX_TYPE: &str = "SurveyBox";
