use std::ops::Range;

use thiserror::Error;

/// Replace `text[start..end]` with `replacement`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub end: usize,
    pub replacement: String,
    pub start: usize,
}

impl Edit {
    pub fn new(range: Range<usize>, replacement: String) -> Self {
        Self {
            end: range.end,
            replacement,
            start: range.start,
        }
    }

    pub fn contains(&self, other: &Edit) -> bool {
        self.start <= other.start && other.end <= self.end && self != other
    }
}

#[derive(Debug, Error)]
pub enum EditError {
    #[error("edit range {start}..{end} exceeds text length {len}")]
    OutOfBounds { end: usize, len: usize, start: usize },
    #[error("edits {a_start}..{a_end} and {b_start}..{b_end} overlap")]
    Overlap {
        a_end: usize,
        a_start: usize,
        b_end: usize,
        b_start: usize,
    },
}

/// A batch of edits applied to one text.
///
/// Edits nested inside a larger edit are dropped: the larger edit rewrites
/// that region from the original text, and a second formatting pass picks the
/// nested ones up again.
#[derive(Debug, Default)]
pub struct Edits {
    items: Vec<Edit>,
}

impl Edits {
    pub fn new(items: Vec<Edit>) -> Self {
        Self { items }
    }

    pub fn apply(&self, text: &str) -> Result<String, EditError> {
        let mut edits = self.outermost();
        edits.sort_by_key(|edit| (edit.start, edit.end));
        let mut out = String::with_capacity(text.len());
        let mut cursor = 0;
        for edit in edits {
            if edit.end > text.len() || edit.start > edit.end {
                return Err(EditError::OutOfBounds {
                    end: edit.end,
                    len: text.len(),
                    start: edit.start,
                });
            }
            if edit.start < cursor {
                return Err(EditError::Overlap {
                    a_end: cursor,
                    a_start: cursor,
                    b_end: edit.end,
                    b_start: edit.start,
                });
            }
            out.push_str(&text[cursor..edit.start]);
            out.push_str(&edit.replacement);
            cursor = edit.end;
        }
        out.push_str(&text[cursor..]);
        Ok(out)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn push(&mut self, edit: Edit) {
        self.items.push(edit);
    }

    fn outermost(&self) -> Vec<&Edit> {
        self.items
            .iter()
            .filter(|edit| !self.items.iter().any(|outer| outer.contains(edit)))
            .collect()
    }
}
