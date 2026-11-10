//! The gate in front of every backend: unsupported commands are refused with a typed problem.
//!
//! Pattern: Proxy (protection) — `Gated<T>` wraps a [`Typed`] backend. It looks the command up in the one
//! table of the core; one that is not there is `NotFound`, one marked unsupported is `Unsupported` and is
//! never dispatched. So no backend can forget the rule, and the flag the frontend sees on the wire is the
//! same flag that decides here.

use fs_admin_frontend::{Backend, BackendError, Outcome, Record, Values};
use fs_surveys_core::commands::{self, Command};
use fs_types::Edit;

use crate::refuse::RunError;

/// What a data type's backend implements; `Gated` adds the rule about unsupported commands.
pub trait Typed: Send + Sync {
    /// The data type this backend serves (`Survey` ...).
    fn data_type(&self) -> &'static str;
    /// The records `find` selects with `query` (empty = all).
    fn records(&self, query: &Values) -> Result<Vec<Record>, RunError>;
    /// Carry out a supported command of the table.
    fn dispatch(&self, command: &'static Command, input: &Values) -> Result<Outcome, RunError>;
}

/// A typed backend behind the gate.
pub struct Gated<T: Typed>(pub T);

impl<T: Typed> Gated<T> {
    /// Run a command, with the reason typed.
    ///
    /// # Errors
    /// [`RunError::Unsupported`] for a command the core does not carry out, [`RunError::NotFound`] for an
    /// unknown name, otherwise whatever the command says.
    pub fn try_run(
        &self,
        name: &str,
        record: Option<&str>,
        input: &Values,
    ) -> Result<Outcome, RunError> {
        let ty = self.0.data_type();
        let command = commands::for_type(ty)
            .into_iter()
            .find(|c| c.name == name)
            .ok_or(RunError::NotFound)?;
        if !command.supported {
            return Err(RunError::Unsupported {
                data_type: command.data_type,
                command: command.name,
                reason: command.reason,
            });
        }
        // The record travels beside the input (frontend rule); the commands read it as `id`.
        let mut input = input.clone();
        if let Some(record) = record {
            input.insert("id".to_owned(), vec![record.to_owned()]);
        }
        self.0.dispatch(command, &input)
    }

    /// The command `change`: an [`Edit`] on one record. A field not in the edit stays, one with no values is
    /// cleared (`Edit::apply` rule), so the dispatch gets exactly the fields of the edit.
    ///
    /// # Errors
    /// As [`Gated::try_run`].
    pub fn try_change(&self, record: &str, edit: &Edit) -> Result<Outcome, RunError> {
        let input: Values = edit
            .fields
            .iter()
            .filter(|f| f.path.len() == 1)
            .map(|f| (f.path[0].clone(), f.values.clone()))
            .collect();
        self.try_run("edit", Some(record), &input)
    }
}

impl<T: Typed> Backend for Gated<T> {
    fn list(&self, query: &Values) -> Result<Vec<Record>, BackendError> {
        self.0.records(query).map_err(Into::into)
    }

    fn supported(&self, command: &str) -> bool {
        commands::for_type(self.0.data_type())
            .into_iter()
            .any(|c| c.name == command && c.supported)
    }

    fn change(&self, record: &str, edit: &Edit) -> Result<Outcome, BackendError> {
        self.try_change(record, edit).map_err(Into::into)
    }

    fn run(
        &self,
        name: &str,
        record: Option<&str>,
        input: &Values,
    ) -> Result<Outcome, BackendError> {
        self.try_run(name, record, input).map_err(Into::into)
    }
}
