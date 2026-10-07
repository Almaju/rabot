use std::ops::RangeInclusive;

use crate::comment::{Comment, Comments};
use crate::rule::Rule;

const PREFIX: &str = "rabot:";

/// Words a comment can open with to say that what follows is deliberate.
/// Matched case-insensitively, on a word boundary, at the very start.
const MARKERS: [&str; 4] = ["by design", "deliberate", "intentional", "on purpose"];

/// Words of reason a marked comment needs before it counts: "Intentionally
/// empty" says the choice was made, not why.
const MIN_REASON_WORDS: usize = 3;

/// A documented exception, written one of two ways:
///
/// - as a plain comment that says the code is deliberate and why:
///   `// Deliberately unsorted: the guard must release before the pool.`
///   The words before the first colon or comma say what is deliberate; when
///   they name rules (`unsorted` names the sorting rules), it covers those,
///   otherwise every rule on the item it precedes. It reads as an ordinary
///   note to a codebase that has never heard of rabot;
/// - as a directive naming rules: `// rabot: allow(rule, other-rule) because ...`.
///
/// The reason is mandatory. An exception that lives only in someone's head is
/// not an exception; it is chaos with better intentions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allowance {
    pub file_wide: bool,
    pub line: usize,
    pub reason: String,
    pub rules: Vec<Rule>,
    /// Lines this allowance silences. Filled in by [`Allowances::attach`].
    pub scope: RangeInclusive<usize>,
}

/// A malformed allow comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub line: usize,
    pub message: String,
    pub rule: Rule,
}

/// A region of source that an allowance can attach to: an item, a field, a
/// method.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub end_line: usize,
    pub start_line: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Allowances {
    items: Vec<Allowance>,
    problems: Vec<Problem>,
}

impl Allowances {
    pub fn parse(comments: &Comments) -> Self {
        let mut allowances = Allowances::default();
        let comments: Vec<&Comment> = comments.iter().filter(|comment| !comment.is_doc()).collect();
        for (index, comment) in comments.iter().enumerate() {
            if let Some(directive) = comment.text.trim().strip_prefix(PREFIX) {
                allowances.parse_directive(directive.trim(), comment.line);
            } else if let Some(reason) = marked_reason(&comment.text)
                && !comment.continues(comments[..index].last().copied())
            {
                allowances.parse_prose(reason, comment.line, &comments[index + 1..]);
            }
        }
        allowances
    }

    /// Bind each allowance to the scope that starts first after it, so that
    /// an allow comment above a function silences the whole function body.
    /// `lines` are the file's lines, used to skip attributes and comments
    /// between the allowance and its item.
    pub fn attach(&mut self, scopes: &[Scope], lines: &[&str]) {
        for allowance in &mut self.items {
            if allowance.file_wide {
                allowance.scope = 1..=usize::MAX;
                continue;
            }
            let Some(target_line) = next_code_line(allowance.line, lines) else {
                continue;
            };
            let scope = scopes
                .iter()
                .filter(|scope| scope.start_line == target_line)
                .max_by_key(|scope| scope.end_line);
            allowance.scope = match scope {
                Some(scope) => scope.start_line..=scope.end_line,
                None => target_line..=target_line,
            };
        }
    }

    pub fn covers(&self, rule: Rule, line: usize) -> bool {
        self.items
            .iter()
            .any(|allowance| allowance.rules.contains(&rule) && allowance.scope.contains(&line))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Allowance> {
        self.items.iter()
    }

    pub fn problems(&self) -> &[Problem] {
        &self.problems
    }

    fn parse_directive(&mut self, directive: &str, line: usize) {
        let (keyword, rest) = match directive.find('(') {
            Some(open) => (directive[..open].trim(), &directive[open..]),
            None => (directive, ""),
        };
        let file_wide = match keyword {
            "allow" => false,
            "allow-file" => true,
            other => {
                self.problems.push(Problem {
                    line,
                    message: format!(
                        "unknown rabot directive `{other}`; use `rabot: allow(rule) reason` or `rabot: allow-file(rule) reason`"
                    ),
                    rule: Rule::UnknownRule,
                });
                return;
            }
        };
        let Some(close) = rest.find(')') else {
            self.problems.push(Problem {
                line,
                message: "allow comment is missing its closing `)`".to_string(),
                rule: Rule::UnknownRule,
            });
            return;
        };
        let mut rules = Vec::new();
        for name in rest[1..close]
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            match Rule::parse(name) {
                Some(rule) => rules.push(rule),
                None => self.problems.push(Problem {
                    line,
                    message: format!("unknown rule `{name}`; run `rabot rules` to list them"),
                    rule: Rule::UnknownRule,
                }),
            }
        }
        let reason = rest[close + 1..].trim().trim_start_matches(':').trim();
        if reason.is_empty() {
            self.problems.push(Problem {
                line,
                message: "allow comment has no reason; write down why the rule does not apply here"
                    .to_string(),
                rule: Rule::UndocumentedException,
            });
            return;
        }
        self.items.push(Allowance {
            file_wide,
            line,
            reason: reason.to_string(),
            rules,
            scope: line..=line,
        });
    }

    /// A comment opening with a marker. Line comments directly below it
    /// continue the sentence, so a reason can wrap. When the words that say
    /// what is deliberate name rules ("Deliberately unsorted"), only those
    /// are covered; otherwise ("Intentional: ...") every rule is.
    fn parse_prose(&mut self, text: &str, line: usize, following: &[&Comment]) {
        let mut text = text.trim().to_string();
        for (offset, next) in following.iter().enumerate() {
            if !next.is_line() || next.line != line + offset + 1 {
                break;
            }
            text.push(' ');
            text.push_str(next.text.trim());
        }
        let reason = text.trim_start_matches([':', ',', '-', ' ']);
        if reason.split_whitespace().count() < MIN_REASON_WORDS {
            return;
        }
        let topic = topic(&text);
        let coverable = Rule::all().iter().copied().filter(|rule| !rule.is_meta());
        let named: Vec<Rule> = coverable
            .clone()
            .filter(|rule| topic.iter().any(|word| rule.is_named_by(word)))
            .collect();
        self.items.push(Allowance {
            file_wide: false,
            line,
            reason: reason.to_string(),
            rules: if named.is_empty() {
                coverable.collect()
            } else {
                named
            },
            scope: line..=line,
        });
    }
}

/// Whether a comment's text is an exception in either form. The comment
/// rules skip these: an exception is neither code nor a section header.
pub fn is_exception(text: &str) -> bool {
    text.contains(PREFIX) || marked_reason(text).is_some()
}

/// The text after a leading marker ("Intentionally", "By design", ...),
/// or `None` when the comment does not open with one.
fn marked_reason(text: &str) -> Option<&str> {
    let text = text.trim();
    let lowered = text.to_ascii_lowercase();
    MARKERS.iter().find_map(|marker| {
        let rest = lowered.strip_prefix(marker)?;
        // The marker's own suffix (`-ly`, `-ally`) is part of the word.
        let word_end = rest
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(rest.len());
        let suffix = &rest[..word_end];
        if !matches!(suffix, "" | "ly") {
            return None;
        }
        Some(&text[marker.len() + word_end..])
    })
}

/// The words that say what is deliberate: those after the marker, up to the
/// first colon, comma or full stop. `unsorted` in "Deliberately unsorted:
/// drop order matters", nothing in "Intentional: checked above". Lowercase,
/// with surrounding punctuation and backticks removed.
fn topic(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    for token in text.split_whitespace() {
        if matches!(token, "-" | "--" | "\u{2013}" | "\u{2014}") || token.starts_with('(') {
            break;
        }
        words.push(token.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase());
        if token.ends_with([':', ',', ';', '.', '!', '?']) {
            break;
        }
    }
    words
}

/// The first line after `line` that holds code rather than a comment,
/// attribute or blank line. A trailing allow comment on a code line applies
/// to that line.
fn next_code_line(line: usize, lines: &[&str]) -> Option<usize> {
    let own = lines.get(line - 1)?.trim();
    if !own.starts_with("//") && !own.starts_with("/*") {
        return Some(line);
    }
    (line + 1..=lines.len()).find(|candidate| {
        let text = lines[candidate - 1].trim();
        !(text.is_empty() || text.starts_with("//") || text.starts_with("#[") || text.starts_with("#!["))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Allowances {
        let mut allowances = Allowances::parse(&Comments::scan(source));
        let lines: Vec<&str> = source.lines().collect();
        allowances.attach(&[], &lines);
        allowances
    }

    #[test]
    fn parses_rules_and_reason() {
        let allowances = parse("// rabot: allow(sorted-fields, free-function) drop order matters\nstruct A;");
        let items: Vec<_> = allowances.iter().collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rules, vec![Rule::SortedFields, Rule::FreeFunction]);
        assert_eq!(items[0].reason, "drop order matters");
        assert!(allowances.covers(Rule::SortedFields, 2));
        assert!(!allowances.covers(Rule::SortedFields, 3));
    }

    #[test]
    fn missing_reason_is_a_problem() {
        let allowances = parse("// rabot: allow(sorted-fields)\nstruct A;");
        assert_eq!(allowances.problems().len(), 1);
        assert_eq!(allowances.problems()[0].rule, Rule::UndocumentedException);
        assert!(!allowances.covers(Rule::SortedFields, 2));
    }

    #[test]
    fn unknown_rule_is_a_problem() {
        let allowances = parse("// rabot: allow(nope) because\nstruct A;");
        assert_eq!(allowances.problems()[0].rule, Rule::UnknownRule);
    }

    #[test]
    fn attaches_to_scope() {
        let source = "// rabot: allow(panic-in-production) startup\n#[inline]\nfn a() {\n  x.unwrap();\n}\n";
        let mut allowances = Allowances::parse(&Comments::scan(source));
        let lines: Vec<&str> = source.lines().collect();
        allowances.attach(
            &[Scope {
                end_line: 5,
                start_line: 3,
            }],
            &lines,
        );
        assert!(allowances.covers(Rule::PanicInProduction, 4));
        assert!(!allowances.covers(Rule::PanicInProduction, 6));
    }

    #[test]
    fn file_wide_allowance() {
        let allowances = parse("// rabot: allow-file(mock-usage) legacy suite\n\nfn a() {}");
        assert!(allowances.covers(Rule::MockUsage, 300));
    }

    #[test]
    fn a_deliberate_comment_covers_every_rule_on_its_item() {
        for source in [
            "// intentional: the C header defines this order\nstruct A;",
            "// On purpose: this mirrors the wire format.\nstruct A;",
            "// By design, callers own the retry loop.\nstruct A;",
            "// Intentionally slow: the backoff is the whole point.\nstruct A;",
        ] {
            let allowances = parse(source);
            assert!(allowances.covers(Rule::SortedFields, 2), "{source}");
            assert!(allowances.covers(Rule::PanicInProduction, 2), "{source}");
            assert!(!allowances.covers(Rule::SortedFields, 3), "{source}");
            assert!(!allowances.covers(Rule::UnknownRule, 2), "{source}");
            assert_eq!(allowances.problems(), &[], "{source}");
        }
    }

    #[test]
    fn saying_what_is_deliberate_covers_only_the_rules_it_names() {
        for source in [
            "// Deliberately unsorted: the guard must release before the pool.\nstruct A;",
            "// Intentionally unsorted, to match the C header.\nstruct A;",
            "// Intentional sorting: the C header defines this order.\nstruct A;",
            "// By design the order matches the wire format.\nstruct A;",
            "// Deliberately left unsorted\n// so the guard releases first.\nstruct A;",
        ] {
            let allowances = parse(source);
            let item = source.lines().count();
            assert!(allowances.covers(Rule::SortedFields, item), "{source}");
            assert!(allowances.covers(Rule::SortedDerives, item), "{source}");
            assert!(!allowances.covers(Rule::PanicInProduction, item), "{source}");
        }
    }

    #[test]
    fn the_words_after_the_colon_are_the_reason_not_the_subject() {
        let allowances = parse("// Intentional: the unwrap is checked two lines above.\nstruct A;");
        assert!(allowances.covers(Rule::PanicInProduction, 2));
        assert!(allowances.covers(Rule::SortedFields, 2));
    }

    #[test]
    fn words_are_matched_loosely() {
        for (source, rule) in [
            (
                "let x = y.unwrap(); // Intentional unwraps: checked above.",
                Rule::PanicInProduction,
            ),
            (
                "// Intentionally a `String`: a free-form user label.\nstruct A;",
                Rule::StringlyTypedField,
            ),
            (
                "// Deliberately PUBLIC: any f64 is a valid Meters.\nstruct A;",
                Rule::BypassableConstructor,
            ),
            (
                "// Deliberately a catch-all: the C library is free-form.\nenum E;",
                Rule::EscapeHatchVariant,
            ),
        ] {
            let allowances = parse(source);
            let item = source.lines().count();
            assert!(allowances.covers(rule, item), "{source}");
            assert!(!allowances.covers(Rule::SortedFields, item), "{source}");
        }
    }

    #[test]
    fn several_words_cover_several_rules() {
        let allowances =
            parse("// Deliberately unsorted and unwrapped: generated from the C header.\nstruct A;");
        assert!(allowances.covers(Rule::SortedFields, 2));
        assert!(allowances.covers(Rule::PanicInProduction, 2));
        assert!(!allowances.covers(Rule::GlobalState, 2));
    }

    /// Every rule page shows how to silence it. The comment it shows has to
    /// do that by naming the rule, not by covering everything.
    #[test]
    fn every_rule_page_example_names_its_rule() {
        for rule in Rule::all() {
            let page = rule.documentation();
            let Some((_, silence)) = page.split_once("## Silence it") else {
                continue;
            };
            let Some((example, reason)) = silence.lines().find_map(|line| {
                let (_, text) = line.split_once("//")?;
                Some((line, marked_reason(text)?))
            }) else {
                continue;
            };
            assert!(
                topic(reason).iter().any(|word| rule.is_named_by(word)),
                "{rule}: `{example}` does not name it"
            );
            let item = if example.trim_start().starts_with("//") {
                2
            } else {
                1
            };
            let allowances = parse(&format!("{example}\nstruct A;"));
            assert!(
                allowances.covers(*rule, item),
                "{rule}: `{example}` does not silence it"
            );
        }
    }

    #[test]
    fn a_deliberate_comment_can_wrap() {
        let allowances = parse("// Deliberately unsorted:\n// the guard must release first.\nstruct A;");
        assert!(allowances.covers(Rule::SortedFields, 3));
        assert_eq!(
            allowances.iter().next().map(|a| a.reason.as_str()),
            Some("unsorted: the guard must release first.")
        );
    }

    #[test]
    fn a_marker_without_a_reason_or_inside_a_word_is_not_an_exception() {
        for source in [
            "// Intentionally empty\nstruct A;",
            "// Deliberately.\nstruct A;",
            "// Intentionality is overrated, says the docs.\nstruct A;",
            "// This is intentional: the guard must release first.\nstruct A;",
            "// The buffer size is\n// intentionally controlled by the caller.\nstruct A;",
        ] {
            let allowances = parse(source);
            assert_eq!(allowances.iter().count(), 0, "{source}");
            assert_eq!(allowances.problems(), &[], "{source}");
        }
    }

    #[test]
    fn a_trailing_deliberate_comment_covers_its_own_line() {
        let allowances = parse("let x = y.unwrap(); // Intentional: checked two lines above");
        assert!(allowances.covers(Rule::PanicInProduction, 1));
    }

    #[test]
    fn trailing_allowance_covers_its_own_line() {
        let allowances = parse("let x = y.unwrap(); // rabot: allow(panic-in-production) checked above");
        assert!(allowances.covers(Rule::PanicInProduction, 1));
    }
}
