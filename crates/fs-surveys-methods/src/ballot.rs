//! The four ballot forms.
//!
//! Pattern: Tagged union (one variant per form; methods declare which forms they accept).

/// One voter's ballot, by choice id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ballot {
    /// One choice.
    Single(String),
    /// Any number of approved choices (may be empty).
    Multiple(Vec<String>),
    /// Choices from best to worst; unlisted choices are ranked below all listed ones.
    Ranking(Vec<String>),
    /// A score per choice; choices left out are not scored.
    Scores(Vec<(String, i32)>),
}

impl Ballot {
    /// Plain-English name of the ballot form.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Single(_) => "single",
            Self::Multiple(_) => "multiple",
            Self::Ranking(_) => "ranking",
            Self::Scores(_) => "scores",
        }
    }
}
