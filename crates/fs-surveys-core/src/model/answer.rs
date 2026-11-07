//! Answers, credentials and named responses.
//!
//! Pattern: Tagged union (`AnswerValue`, `Credential`) mirroring the unions of `surveys.capnp`.

use fs_objects::{Envelope, ObjectId};
use serde::{Deserialize, Serialize};

/// One value for one option of an `optionScores` question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionScore {
    /// Choice id.
    pub choice: String,
    /// The score.
    pub score: i32,
}

/// The value of an answer; which arm fits follows from the question kind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnswerValue {
    /// `singleChoice`.
    Choice(String),
    /// `multipleChoice`.
    Choices(Vec<String>),
    /// `text`.
    Text(String),
    /// `number`.
    Number(f64),
    /// `date`, an RFC 3339 full-date.
    Date(String),
    /// `scale`.
    Scale(i32),
    /// `ranking`, most preferred first.
    Ranking(Vec<String>),
    /// `optionScores`.
    Scores(Vec<OptionScore>),
}

/// An answer to one question.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    /// The question answered.
    pub question: ObjectId,
    /// The value.
    #[serde(flatten)]
    pub value: AnswerValue,
}

/// Who is answering; the arm needed follows from the anonymity of the survey.
#[derive(Clone, PartialEq, Eq)]
pub enum Credential {
    /// `anonymousOpen`.
    None,
    /// `named`: the person's IRI.
    Person(String),
    /// `anonymousOnce`: the token sent to the person.
    Token(String),
    /// `public`: the answer to `challenge` (not built).
    Proof(Vec<u8>),
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A token must not end up in a log line.
        f.write_str(match self {
            Self::None => "Credential::None",
            Self::Person(_) => "Credential::Person(..)",
            Self::Token(_) => "Credential::Token(..)",
            Self::Proof(_) => "Credential::Proof(..)",
        })
    }
}

/// An answer that knows its author (`named` surveys only).
#[derive(Clone, Debug, PartialEq)]
pub struct NamedResponse {
    /// The stored envelope.
    pub envelope: Envelope,
    /// The survey answered.
    pub survey: ObjectId,
    /// The person's IRI.
    pub person: String,
    /// The answers so far.
    pub answers: Vec<Answer>,
    /// False while it is a saved draft.
    pub submitted: bool,
    /// Counted as an AI answer: kept and shown apart from the answers of people.
    pub ai: bool,
}

/// Who was newly invited and who had been before (never a token).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InviteReport {
    /// People newly invited.
    pub invited: Vec<String>,
    /// Invited before; no new token.
    pub already: Vec<String>,
    /// An AI, or not known as a human, and the survey does not allow AIs: nothing was sent.
    pub refused: Vec<String>,
}
