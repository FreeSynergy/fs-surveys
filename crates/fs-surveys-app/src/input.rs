//! Reading typed input values.
//!
//! Pattern: none (small pure functions). A missing key means "not given"; a value that is empty after
//! trimming counts as not given (the rule of `Edit`, `types.capnp`).

use std::str::FromStr;

use fs_admin_frontend::Values;
use fs_objects::ObjectId;

use crate::ids::tail;
use crate::refuse::RunError;

/// The first value of `key`, trimmed, if there is one.
pub fn text<'a>(v: &'a Values, key: &str) -> Option<&'a str> {
    v.get(key)?.iter().map(|s| s.trim()).find(|s| !s.is_empty())
}

/// All non-empty values of `key`.
pub fn list<'a>(v: &'a Values, key: &str) -> Vec<&'a str> {
    v.get(key)
        .map(|l| {
            l.iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// The key is part of the input, with values or to be cleared.
pub fn given(v: &Values, key: &str) -> bool {
    v.contains_key(key)
}

/// A required text.
pub fn need<'a>(v: &'a Values, key: &'static str) -> Result<&'a str, RunError> {
    text(v, key).ok_or(RunError::Missing(key))
}

/// A number in a text field; `None` if not given.
pub fn number<T: FromStr>(v: &Values, key: &str) -> Result<Option<T>, RunError> {
    text(v, key)
        .map(|s| {
            s.parse().map_err(|_| RunError::Bad {
                field: key.to_owned(),
                index: 0,
            })
        })
        .transpose()
}

/// `yes` / `no`.
pub fn yes_no(v: &Values, key: &str) -> Result<Option<bool>, RunError> {
    match text(v, key) {
        None => Ok(None),
        Some("yes") => Ok(Some(true)),
        Some("no") => Ok(Some(false)),
        Some(_) => Err(RunError::Bad {
            field: key.to_owned(),
            index: 0,
        }),
    }
}

/// A reference to an object of this program (or a group): its id.
pub fn id_of(v: &Values, key: &'static str) -> Result<ObjectId, RunError> {
    tail(need(v, key)?).map_err(|_| RunError::Bad {
        field: key.to_owned(),
        index: 0,
    })
}

/// Several references: their ids.
pub fn ids_of(v: &Values, key: &str) -> Result<Vec<ObjectId>, RunError> {
    list(v, key)
        .into_iter()
        .enumerate()
        .map(|(index, s)| {
            tail(s).map_err(|_| RunError::Bad {
                field: key.to_owned(),
                index,
            })
        })
        .collect()
}

/// A value of this fixed list of keys, as the position in `keys`.
pub fn pick(v: &Values, key: &str, keys: &[&str]) -> Result<Option<usize>, RunError> {
    text(v, key)
        .map(|s| {
            keys.iter().position(|k| *k == s).ok_or(RunError::Bad {
                field: key.to_owned(),
                index: 0,
            })
        })
        .transpose()
}
