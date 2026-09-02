use std::ops::RangeInclusive;

use crate::comment::Comments;
use crate::rule::Rule;

const PREFIX: &str = "rabot:";

/// A documented exception: `// rabot: allow(rule, other-rule) because ...`.
///
/// The reason is mandatory. An exception that lives only in someone's head is
/// not an exception; it is chaos with better intentions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Allowance {
    pub file_wide: bool,
    pub line: usize,
    pub reason: String,
    pub rules: Vec<Rule>,
    /// Lines this allowance silences. Filled in by [`Allowances::attach`].
    pub scope: RangeInclusive<usize>,
}

/// A malformed allow comment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Problem {
    pub line: usize,
    pub message: String,
    pub rule: Rule,
}

/// A region of source that an allowance can attach to: an item, a field, a
/// method.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        for comment in comments.iter().filter(|comment| !comment.is_doc()) {
            let Some(directive) = comment.text.trim().strip_prefix(PREFIX) else {
                continue;
            };
            allowances.parse_directive(directive.trim(), comment.line);
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
    fn trailing_allowance_covers_its_own_line() {
        let allowances = parse("let x = y.unwrap(); // rabot: allow(panic-in-production) checked above");
        assert!(allowances.covers(Rule::PanicInProduction, 1));
    }
}
