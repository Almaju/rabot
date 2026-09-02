use std::cmp::Ordering;
use std::ops::Range;

use crate::edit::Edit;

/// The key rabot sorts identifiers by: case-insensitive, with digit runs
/// compared numerically so that `field2` comes before `field10`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortKey {
    original: String,
    pieces: Vec<Piece>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Piece {
    Number(u128),
    Text(String),
}

impl SortKey {
    pub fn new(name: &str) -> Self {
        let name = name.strip_prefix("r#").unwrap_or(name);
        let mut pieces = Vec::new();
        let mut digits = String::new();
        let mut text = String::new();
        for c in name.chars() {
            if c.is_ascii_digit() {
                if !text.is_empty() {
                    pieces.push(Piece::Text(std::mem::take(&mut text)));
                }
                digits.push(c);
            } else {
                if !digits.is_empty() {
                    pieces.push(Piece::Number(
                        std::mem::take(&mut digits).parse().unwrap_or(u128::MAX),
                    ));
                }
                text.extend(c.to_lowercase());
            }
        }
        if !text.is_empty() {
            pieces.push(Piece::Text(text));
        }
        if !digits.is_empty() {
            pieces.push(Piece::Number(digits.parse().unwrap_or(u128::MAX)));
        }
        Self {
            original: name.to_string(),
            pieces,
        }
    }

    /// A key that sorts as `sort_as` but is displayed as `label`, for names
    /// whose position is decided by something other than their spelling.
    pub fn labelled(label: &str, sort_as: &str) -> Self {
        Self {
            original: label.to_string(),
            ..Self::new(sort_as)
        }
    }

    pub fn original(&self) -> &str {
        &self.original
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> Ordering {
        for (a, b) in self.pieces.iter().zip(&other.pieces) {
            let ordering = match (a, b) {
                (Piece::Number(a), Piece::Number(b)) => a.cmp(b),
                (Piece::Text(a), Piece::Text(b)) => a.cmp(b),
                (Piece::Number(_), Piece::Text(_)) => Ordering::Less,
                (Piece::Text(_), Piece::Number(_)) => Ordering::Greater,
            };
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        self.pieces
            .len()
            .cmp(&other.pieces.len())
            .then_with(|| self.original.cmp(&other.original))
    }
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// One entry of a [`SourceList`]: a group rank first, then the name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank {
    pub group: u8,
    pub key: SortKey,
}

impl Rank {
    pub fn new(group: u8, name: &str) -> Self {
        Self {
            group,
            key: SortKey::new(name),
        }
    }
}

/// The first adjacent pair that is out of order, for a precise message.
pub fn first_disorder(ranks: &[Rank]) -> Option<(usize, usize)> {
    ranks
        .windows(2)
        .position(|pair| pair[0] > pair[1])
        .map(|index| (index, index + 1))
}

/// The permutation that sorts `ranks`, or `None` when they are already sorted.
pub fn sorted_order(ranks: &[Rank]) -> Option<Vec<usize>> {
    let mut order: Vec<usize> = (0..ranks.len()).collect();
    order.sort_by(|a, b| ranks[*a].cmp(&ranks[*b]));
    let identity = order.iter().enumerate().all(|(index, item)| index == *item);
    (!identity).then_some(order)
}

/// A bracketed list in source text whose members can be permuted without
/// touching anything else: the whitespace and comments before a member move
/// with it, the trailing comma and same-line comment after it too.
#[derive(Debug)]
pub struct SourceList<'a> {
    /// Offset of the closing delimiter (or of `..rest`).
    close: usize,
    members: Vec<Member>,
    /// Offset just after the opening delimiter.
    open: usize,
    separator: Option<char>,
    text: &'a str,
}

#[derive(Debug)]
struct Member {
    body: Range<usize>,
    /// Offset at the end of everything that belongs to this member.
    chunk_end: usize,
    has_separator: bool,
    /// Same-line comment after the member, if any.
    trailing_comment: Range<usize>,
}

impl<'a> SourceList<'a> {
    /// `interior` runs from just after the opening delimiter to the closing
    /// delimiter (or to `..rest`).
    pub fn new(
        text: &'a str,
        interior: Range<usize>,
        bodies: Vec<Range<usize>>,
        separator: Option<char>,
    ) -> Self {
        let (open, close) = (interior.start, interior.end);
        let mut members = Vec::with_capacity(bodies.len());
        for body in bodies {
            let mut cursor = skip_blanks(text, body.end);
            let has_separator = separator.is_some_and(|sep| text[cursor..].starts_with(sep));
            if has_separator {
                cursor += 1;
            }
            let after_separator = skip_blanks(text, cursor);
            let trailing_comment = if text[after_separator..].starts_with("//") {
                let end = text[after_separator..]
                    .find('\n')
                    .map_or(text.len(), |offset| after_separator + offset);
                after_separator..end
            } else {
                cursor..cursor
            };
            let chunk_end = if !trailing_comment.is_empty() {
                trailing_comment.end
            } else if has_separator {
                cursor
            } else {
                body.end
            };
            members.push(Member {
                body,
                chunk_end,
                has_separator,
                trailing_comment,
            });
        }
        Self {
            close,
            members,
            open,
            separator,
            text,
        }
    }

    /// The edit that rewrites the list in `order`.
    ///
    /// The whitespace before each slot stays where it is, so a single-line
    /// list stays single-line and blank lines keep their positions. Comments
    /// travel with the member they describe.
    pub fn reordered(&self, order: &[usize]) -> Edit {
        let last_had_separator = self.members.last().is_some_and(|member| member.has_separator);
        let mut previous_end = self.open;
        let leads: Vec<Lead> = self
            .members
            .iter()
            .map(|member| {
                let lead = Lead::split(self.text, previous_end..member.body.start);
                previous_end = member.chunk_end;
                lead
            })
            .collect();
        let trailer_start = self.members.last().map_or(self.open, |member| member.chunk_end);
        let mut out = String::new();
        for (slot, index) in order.iter().enumerate() {
            let member = &self.members[*index];
            out.push_str(&self.text[leads[slot].whitespace.clone()]);
            out.push_str(&self.text[leads[*index].comments.clone()]);
            out.push_str(&self.text[member.body.clone()]);
            let is_last = slot == order.len() - 1;
            if let Some(separator) = self.separator
                && (!is_last || last_had_separator)
            {
                out.push(separator);
            }
            if !member.trailing_comment.is_empty() {
                out.push(' ');
                out.push_str(&self.text[member.trailing_comment.clone()]);
            }
        }
        out.push_str(&self.text[trailer_start..self.close]);
        Edit::new(self.open..self.close, out)
    }
}

/// The text before a member: layout whitespace that belongs to the slot,
/// then comments (and their indentation) that belong to the member.
struct Lead {
    comments: Range<usize>,
    whitespace: Range<usize>,
}

impl Lead {
    fn split(text: &str, range: Range<usize>) -> Self {
        let lead = &text[range.clone()];
        let first_visible = lead
            .find(|c: char| !c.is_whitespace())
            .map_or(range.end, |offset| range.start + offset);
        Self {
            comments: first_visible..range.end,
            whitespace: range.start..first_visible,
        }
    }
}

fn skip_blanks(text: &str, mut cursor: usize) -> usize {
    let bytes = text.as_bytes();
    while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
        cursor += 1;
    }
    cursor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_order(names: &[&str]) -> Vec<String> {
        let mut keys: Vec<SortKey> = names.iter().map(|name| SortKey::new(name)).collect();
        keys.sort();
        keys.into_iter().map(|key| key.original().to_string()).collect()
    }

    #[test]
    fn sorts_case_insensitively_and_naturally() {
        assert_eq!(
            key_order(&["b", "A", "a10", "a2", "r#type", "Zed"]),
            vec!["A", "a2", "a10", "b", "type", "Zed"]
        );
    }

    #[test]
    fn labelled_keys_keep_their_name() {
        let key = SortKey::labelled("Eq", "PartialEq\u{1}Eq");
        assert_eq!(key.original(), "Eq");
        assert!(SortKey::new("PartialEq") < key && key < SortKey::new("PartialOrd"));
    }

    #[test]
    fn detects_disorder() {
        let ranks = vec![Rank::new(0, "b"), Rank::new(0, "a"), Rank::new(0, "c")];
        assert_eq!(first_disorder(&ranks), Some((0, 1)));
        assert_eq!(sorted_order(&ranks), Some(vec![1, 0, 2]));
        let sorted = vec![Rank::new(0, "a"), Rank::new(1, "a")];
        assert_eq!(sorted_order(&sorted), None);
    }

    fn reorder(text: &str, bodies: &[&str], separator: Option<char>) -> String {
        let open = text.find('{').unwrap() + 1;
        let close = text.rfind('}').unwrap();
        let ranges: Vec<Range<usize>> = bodies
            .iter()
            .map(|body| {
                let start = text.find(body).unwrap();
                start..start + body.len()
            })
            .collect();
        let list = SourceList::new(text, open..close, ranges, separator);
        let names: Vec<Rank> = bodies
            .iter()
            .map(|body| Rank::new(0, body.split(':').next().unwrap()))
            .collect();
        let order = sorted_order(&names).unwrap();
        let edit = list.reordered(&order);
        let mut out = text.to_string();
        out.replace_range(edit.start..edit.end, &edit.replacement);
        out
    }

    #[test]
    fn moves_comments_and_commas_with_members() {
        let text = "struct A {\n    // about b\n    b: B, // trailing\n\n    a: A\n}";
        let out = reorder(text, &["b: B", "a: A"], Some(','));
        assert_eq!(
            out,
            "struct A {\n    a: A,\n\n    // about b\n    b: B // trailing\n}"
        );
    }

    #[test]
    fn keeps_single_line_lists_single_line() {
        let out = reorder("S { b: 1, a: 2 }", &["b: 1", "a: 2"], Some(','));
        assert_eq!(out, "S { a: 2, b: 1 }");
    }

    #[test]
    fn keeps_trailing_comma_when_present() {
        let out = reorder("S {\n    b: 1,\n    a: 2,\n}", &["b: 1", "a: 2"], Some(','));
        assert_eq!(out, "S {\n    a: 2,\n    b: 1,\n}");
    }

    #[test]
    fn keeps_paren_list_spacing() {
        let text = "#[derive{Serialize, Debug, Clone}]";
        let out = reorder(text, &["Serialize", "Debug", "Clone"], Some(','));
        assert_eq!(out, "#[derive{Clone, Debug, Serialize}]");
    }

    #[test]
    fn reorders_items_without_separators() {
        let text = "impl A {\n    fn b() {}\n\n    fn a() {}\n}";
        let out = reorder(text, &["fn b() {}", "fn a() {}"], None);
        assert_eq!(out, "impl A {\n    fn a() {}\n\n    fn b() {}\n}");
    }
}
