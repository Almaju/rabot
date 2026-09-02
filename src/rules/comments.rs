//! Delete the comment. Fix the code.
//! <https://almaju.github.io/blog/docs/fundamentals/style/comments>

use syn::visit::Visit;

use crate::comment::Comment;
use crate::diagnostic::Diagnostic;
use crate::rule::Rule;
use crate::rules::{Check, Context, Findings};

pub struct Comments;

impl Check for Comments {
    fn run(&self, cx: &Context) -> Findings {
        let mut findings = Findings::default();
        let blocks = CommentBlock::all(cx);
        for block in &blocks {
            if let Some(diagnostic) = block.commented_out_code(cx) {
                findings.diagnostics.push(diagnostic);
            }
            if let Some(diagnostic) = block.vague_todo(cx) {
                findings.diagnostics.push(diagnostic);
            }
        }
        let mut sections = Sections {
            blocks: &blocks,
            cx,
            findings,
        };
        sections.visit_file(&cx.file.ast);
        sections.findings
    }
}

/// Consecutive line comments (or one block comment) read as a unit, since
/// commented-out code and TODO explanations span lines.
struct CommentBlock {
    line: usize,
    start: usize,
    /// True when nothing but whitespace precedes the comment on its line.
    starts_line: bool,
    text: String,
}

impl CommentBlock {
    /// A comment that introduces the code below it rather than trailing the
    /// code beside it, and is not one of the two legitimate kinds (context
    /// for the future, a ticket, a safety argument).
    fn is_section_header(&self) -> bool {
        let lowered = self.text.trim().to_ascii_lowercase();
        self.starts_line
            && !["todo", "fixme", "xxx", "hack", "safety", "rabot:", "see ", "http"]
                .iter()
                .any(|marker| lowered.contains(marker))
    }
}

/// `// step 1: validate`, `// step 2: transform`, `// step 3: persist`: three
/// functions trapped inside one, with an informal table of contents.
struct Sections<'a> {
    blocks: &'a [CommentBlock],
    cx: &'a Context<'a>,
    findings: Findings,
}

impl Sections<'_> {
    fn check_body(&mut self, ident: &syn::Ident, block: &syn::Block) {
        let range = self.cx.file.range_of(block);
        let headers: Vec<&CommentBlock> = self
            .blocks
            .iter()
            .filter(|comment| range.contains(&comment.start) && comment.is_section_header())
            .collect();
        let threshold = self.cx.config.thresholds.section_comments;
        if headers.len() < threshold {
            return;
        }
        let mut names: Vec<String> = headers
            .iter()
            .take(3)
            .map(|header| {
                let first_line = header.text.lines().next().unwrap_or_default().trim();
                format!("`{}`", first_line.trim_end_matches(['.', ':']))
            })
            .collect();
        if headers.len() > names.len() {
            names.push("...".to_string());
        }
        self.findings.report_with_help(
            self.cx,
            Rule::SectionedFunction,
            ident.span(),
            format!(
                "`{ident}` is narrated by {} section comments ({}): a table of contents for code that should have been split",
                headers.len(),
                names.join(", ")
            ),
            Some("extract each section into a function; the comment becomes its name and disappears".to_string()),
        );
    }
}

impl<'ast> Visit<'ast> for Sections<'_> {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_body(&node.sig.ident, &node.block);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_body(&node.sig.ident, &node.block);
        syn::visit::visit_item_fn(self, node);
    }
}

impl CommentBlock {
    /// Consecutive line comments merge into one block; a marker such as
    /// `TODO` always starts a new one.
    fn all(cx: &Context) -> Vec<CommentBlock> {
        let mut blocks: Vec<CommentBlock> = Vec::new();
        let mut previous: Option<&Comment> = None;
        for comment in cx.file.comments.iter().filter(|comment| !comment.is_doc()) {
            let starts_line = cx.file.text[..comment.start]
                .rsplit('\n')
                .next()
                .is_some_and(|prefix| prefix.trim().is_empty());
            // A trailing comment and the leading comment on the next line are
            // two different thoughts, not one block.
            let continues = comment.is_line()
                && starts_line
                && previous.is_some_and(|prev| prev.is_line() && prev.line + 1 == comment.line)
                && blocks.last().is_some_and(|block| block.starts_line)
                && !starts_marker(&comment.text);
            match (continues, blocks.last_mut()) {
                (true, Some(block)) => {
                    block.text.push('\n');
                    block.text.push_str(&comment.text);
                }
                _ => blocks.push(CommentBlock {
                    line: comment.line,
                    start: comment.start,
                    starts_line,
                    text: comment.text.clone(),
                }),
            }
            previous = Some(comment);
        }
        blocks.retain(|block| block.line > 0);
        blocks
    }

    /// A comment that parses as Rust and carries syntax (not just words) is
    /// code somebody could not bring themselves to delete.
    fn commented_out_code(&self, cx: &Context) -> Option<Diagnostic> {
        let text = self.text.trim();
        if text.contains("rabot:") || !has_code_signal(text) || !parses_as_rust(text) {
            return None;
        }
        let mut diagnostic = cx.diagnostic(
            Rule::CommentedOutCode,
            proc_macro2::Span::call_site(),
            "commented-out code: you have git, there is no temporary".to_string(),
        )?;
        diagnostic.position = cx.file.position(self.start);
        if cx
            .file
            .allowances
            .covers(Rule::CommentedOutCode, diagnostic.position.line)
        {
            return None;
        }
        diagnostic.help = Some("delete it; `git log` remembers".to_string());
        Some(diagnostic)
    }

    /// `// TODO: refactor this` says nothing. What? Why? When?
    fn vague_todo(&self, cx: &Context) -> Option<Diagnostic> {
        let text = self.text.trim();
        let lowered = text.to_ascii_lowercase();
        let marker = ["todo", "fixme", "xxx", "hack"]
            .into_iter()
            .find(|marker| lowered.starts_with(marker))?;
        let body = text[marker.len()..]
            .trim_start_matches([':', '-', '(', ')', ' '])
            .trim();
        let words = body.split_whitespace().count();
        let has_reference = body.split_whitespace().any(looks_like_reference);
        if has_reference || words >= cx.config.thresholds.vague_todo_min_words {
            return None;
        }
        let mut diagnostic = cx.diagnostic(
            Rule::VagueTodo,
            proc_macro2::Span::call_site(),
            format!(
                "`{}` without context is noise with a timestamp: say what needs to change, why it was not done now, or link the ticket",
                marker.to_ascii_uppercase()
            ),
        )?;
        diagnostic.position = cx.file.position(self.start);
        if cx
            .file
            .allowances
            .covers(Rule::VagueTodo, diagnostic.position.line)
        {
            return None;
        }
        Some(diagnostic)
    }
}

/// Prose parses as Rust more often than you would think (`Fetching` is a
/// valid expression). Demand at least one token that prose does not use.
fn has_code_signal(text: &str) -> bool {
    text.contains(';')
        || text.contains('{')
        || text.contains("()")
        || text.contains(" = ")
        || text.contains("::")
        || text.contains("->")
        || text.contains("=>")
        || text.contains(".await")
        || text.contains("#[")
}

fn looks_like_reference(word: &str) -> bool {
    let word =
        word.trim_matches(|c: char| !c.is_alphanumeric() && c != '#' && c != '-' && c != ':' && c != '/');
    if word.starts_with('#') && word[1..].chars().all(|c| c.is_ascii_digit()) && word.len() > 1 {
        return true;
    }
    if word.contains("://") {
        return true;
    }
    match word.split_once('-') {
        Some((project, number)) => {
            project.len() >= 2
                && project.chars().all(|c| c.is_ascii_uppercase())
                && !number.is_empty()
                && number.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

fn parses_as_rust(text: &str) -> bool {
    syn::parse_file(text).is_ok()
        || syn::parse_str::<syn::Block>(&format!("{{ {text} }}")).is_ok()
        || syn::parse_str::<syn::Block>(&format!("{{ {text}; }}")).is_ok()
}

fn starts_marker(text: &str) -> bool {
    let lowered = text.trim().to_ascii_lowercase();
    ["todo", "fixme", "xxx", "hack", "rabot:"]
        .iter()
        .any(|marker| lowered.starts_with(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prose_is_not_code() {
        assert!(!has_code_signal("Get active users from the database"));
        assert!(!parses_as_rust("Get active users from the database"));
        assert!(!parses_as_rust("see https://example.com/docs"));
    }

    #[test]
    fn code_is_code() {
        assert!(parses_as_rust("let x = compute(5);"));
        assert!(parses_as_rust("fn old() -> u32 { 1 }"));
        assert!(has_code_signal("let x = compute(5);"));
    }

    #[test]
    fn references_are_recognised() {
        assert!(looks_like_reference("PERF-112."));
        assert!(looks_like_reference("#4521"));
        assert!(looks_like_reference("https://tracker/1"));
        assert!(!looks_like_reference("refactor"));
        assert!(!looks_like_reference("well-known"));
    }
}
