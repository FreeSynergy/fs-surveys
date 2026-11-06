//! CSV writing with proper escaping and protection against spreadsheet formulas.
//!
//! Pattern: Builder — rows are added cell by cell, then `finish` yields the bytes.

/// Characters that make a spreadsheet read a cell as a formula.
const FORMULA: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

/// Quote a cell if it needs it (RFC 4180).
pub fn escape(cell: &str) -> String {
    if cell.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell.to_string()
    }
}

/// Free text: a leading formula character gets an apostrophe in front, then it is escaped.
pub fn text_cell(cell: &str) -> String {
    if cell.starts_with(FORMULA) {
        escape(&format!("'{cell}"))
    } else {
        escape(cell)
    }
}

/// A CSV document under construction.
#[derive(Default)]
pub struct Csv(String);

impl Csv {
    /// Add one row of ready (already escaped) cells.
    pub fn row(&mut self, cells: &[String]) {
        self.0.push_str(&cells.join(","));
        self.0.push_str("\r\n");
    }

    /// The document as UTF-8 bytes.
    pub fn finish(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}
