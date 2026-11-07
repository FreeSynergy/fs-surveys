//! CSV writing with proper escaping and protection against spreadsheet formulas.
//!
//! Pattern: Builder — rows are added cell by cell, then `finish` yields the bytes.

/// Characters that make a spreadsheet read a cell as a formula.
const FORMULA: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

/// Quote a cell if it needs it (RFC 4180).
pub fn escape(cell: &str) -> String {
    if cell.contains([',', ';', '"', '\n', '\r', '\t']) {
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
pub struct Csv {
    text: String,
    /// A first column on every row: `participants` in the header, this label below.
    tag: Option<&'static str>,
}

impl Csv {
    /// A document whose rows all begin with the participant class, so a row cannot be mistaken for
    /// one of the other class when files are joined.
    pub fn tagged(tag: &'static str) -> Self {
        Self {
            text: String::new(),
            tag: Some(tag),
        }
    }

    /// Add one row of ready (already escaped) cells.
    pub fn row(&mut self, cells: &[String]) {
        if let Some(tag) = self.tag {
            let first = if self.text.is_empty() {
                "participants"
            } else {
                tag
            };
            self.text.push_str(first);
            self.text.push(',');
        }
        self.text.push_str(&cells.join(","));
        self.text.push_str("\r\n");
    }

    /// The document as UTF-8 bytes.
    pub fn finish(self) -> Vec<u8> {
        self.text.into_bytes()
    }
}
