//! The table type: a command with its input, result and whether the core can carry it out.
//!
//! Pattern: Registry of plain rows (single source of truth). The type description sent to a frontend, the
//! refusal of an unsupported command in the backend, and the test against `requirements.md` all read
//! this table, so they cannot drift apart. A row is data only; `supported` is the one flag a
//! frontend will later read from the wire (transitional rule of thread T, see `DESIGN.md`).

use fs_types::{BasicType, CommandKind};

/// One typed input or result field of a command.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    /// Machine name, lower camel case.
    pub name: &'static str,
    /// Basic type (ADR 0066).
    pub basic: BasicType,
    /// Must be given.
    pub required: bool,
    /// A list of values.
    pub multiple: bool,
    /// For `Reference`: the data type referred to.
    pub target: &'static str,
    /// For `Choice`: the allowed keys.
    pub choices: &'static [&'static str],
}

impl Field {
    /// A field of this basic type.
    #[must_use]
    pub const fn new(name: &'static str, basic: BasicType) -> Self {
        Self {
            name,
            basic,
            required: false,
            multiple: false,
            target: "",
            choices: &[],
        }
    }

    /// A reference to a data type.
    #[must_use]
    pub const fn reference(name: &'static str, target: &'static str) -> Self {
        let mut f = Self::new(name, BasicType::Reference);
        f.target = target;
        f
    }

    /// A choice of fixed keys.
    #[must_use]
    pub const fn choice(name: &'static str, choices: &'static [&'static str]) -> Self {
        let mut f = Self::new(name, BasicType::Choice);
        f.choices = choices;
        f
    }

    /// Must be given.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// A list of values.
    #[must_use]
    pub const fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }
}

/// One command of one data type.
#[derive(Clone, Copy, Debug)]
pub struct Command {
    /// The data type it belongs to: `Survey`, `Question`, `SurveyResponse`, `Decision`.
    pub data_type: &'static str,
    /// Machine name, unique in its type.
    pub name: &'static str,
    /// What it does to the data.
    pub kind: CommandKind,
    /// The ids in `requirements.md` it carries (`survey.create` ...); one command may carry several.
    pub requirements: &'static [&'static str],
    /// The core can carry it out. `false` rows are in the description all the same (ADR 0066) and the
    /// backend refuses them.
    pub supported: bool,
    /// For an unsupported command: why (English, for the log and `DESIGN.md`).
    pub reason: &'static str,
    /// The input, in the order of the call.
    pub input: &'static [Field],
    /// The result.
    pub result: &'static [Field],
}

impl Command {
    /// A supported command.
    #[must_use]
    pub const fn new(
        data_type: &'static str,
        name: &'static str,
        kind: CommandKind,
        requirements: &'static [&'static str],
        input: &'static [Field],
        result: &'static [Field],
    ) -> Self {
        Self {
            data_type,
            name,
            kind,
            requirements,
            supported: true,
            reason: "",
            input,
            result,
        }
    }

    /// The same command, marked as not carried out, with the reason.
    #[must_use]
    pub const fn unsupported(mut self, reason: &'static str) -> Self {
        self.supported = false;
        self.reason = reason;
        self
    }
}

/// Every row of every type.
#[must_use]
pub fn all() -> Vec<&'static Command> {
    [
        super::survey::ROWS,
        super::question::ROWS,
        super::response::ROWS,
        super::decision::ROWS,
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The rows of one data type, in table order.
#[must_use]
pub fn for_type(data_type: &str) -> Vec<&'static Command> {
    all()
        .into_iter()
        .filter(|c| c.data_type == data_type)
        .collect()
}

/// The commands the core does not carry out, as `(data type, command)`.
#[must_use]
pub fn unsupported_commands() -> Vec<(&'static str, &'static str)> {
    all()
        .into_iter()
        .filter(|c| !c.supported)
        .map(|c| (c.data_type, c.name))
        .collect()
}

/// Every requirement id the table carries, sorted, without doubles.
#[must_use]
pub fn requirement_ids() -> Vec<&'static str> {
    let mut ids: Vec<&str> = all()
        .into_iter()
        .flat_map(|c| c.requirements.iter().copied())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}
