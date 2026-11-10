//! The English words of this frontend.
//!
//! Pattern: Strategy behind `Say` — all text in one place; translation (ADR 0053) replaces this file's
//! content, not its users. Fixed keys have a table; the keys of the descriptions (`survey.anonymity`,
//! `question.cmd.set-conditions`, `survey.invite.people`, `<label>.<choice>`) are spoken from their last
//! part, which is a plain word list in kebab case.

use fs_components_fields::Say;

/// The title of the window.
pub const TITLE: &str = "FreeSynergy Surveys";

/// The words of the frontend.
#[derive(Clone, Copy, Debug, Default)]
pub struct Words;

const FIXED: &[(&str, &str)] = &[
    ("action.add", "Add"),
    ("action.back", "Back"),
    ("action.cancel", "Cancel"),
    ("action.close", "Close"),
    ("action.remove", "Remove"),
    ("action.save", "Save"),
    ("menu.program", "Program"),
    ("menu.view", "View"),
    ("menu.help", "Help"),
    ("menu.quit", "Quit"),
    ("menu.about", "About"),
    ("message.done", "Done."),
    ("message.rejected", "The program did not accept this."),
    ("message.not-found", "Not found, or not available yet."),
    ("message.unavailable", "The program cannot be reached."),
    ("message.unsaved", "There is text that was not sent."),
    ("list.empty", "Nothing here yet."),
    ("list.stale", "Out of date"),
    ("detail.not-loaded", "Not loaded"),
    ("confirm.sure", "Are you sure?"),
    ("value.none", "none"),
    ("finding.slot", "Value {n}"),
    ("problem.missing", "This is needed."),
    ("problem.not-in-list", "This is not allowed here."),
    ("problem.character", "This cannot be read."),
    ("sort.ascending", "ascending"),
    ("sort.descending", "descending"),
];

/// `survey.cmd.set-conditions` -> `Set conditions`.
fn spoken(key: &str) -> String {
    let last = key.rsplit('.').next().unwrap_or(key);
    let mut text = last.replace('-', " ");
    if let Some(first) = text.get(..1) {
        text = first.to_uppercase() + &text[1..];
    }
    text
}

impl Say for Words {
    fn say(&self, key: &str) -> String {
        FIXED
            .iter()
            .find(|(k, _)| *k == key)
            .map_or_else(|| spoken(key), |(_, text)| (*text).to_owned())
    }
}
