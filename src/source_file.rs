use std::ops::Range;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use thiserror::Error;

use crate::allowance::{Allowances, Scope};
use crate::comment::Comments;
use crate::diagnostic::Position;

#[derive(Debug, Error)]
#[error("{path}:{line}:{column}: {message}")]
pub struct ParseError {
    pub column: usize,
    pub line: usize,
    pub message: String,
    pub path: PathBuf,
}

/// One Rust file: its text, its syntax tree, its comments and its documented
/// exceptions, all addressable by byte offset.
#[derive(Debug)]
pub struct SourceFile {
    pub allowances: Allowances,
    pub ast: syn::File,
    pub comments: Comments,
    line_starts: Vec<usize>,
    /// Bytes `syn::parse_file` skipped (a BOM or a shebang line) before the
    /// text its spans are relative to.
    offset: usize,
    pub path: PathBuf,
    pub text: String,
}

impl SourceFile {
    pub fn parse(path: impl Into<PathBuf>, text: String) -> Result<Self, ParseError> {
        let path = path.into();
        let offset = skipped_prefix(&text);
        let ast = syn::parse_file(&text).map_err(|error| {
            let start = error.span().start();
            ParseError {
                column: start.column + 1,
                line: start.line,
                message: error.to_string(),
                path: path.clone(),
            }
        })?;
        let comments = Comments::scan(&text);
        let mut allowances = Allowances::parse(&comments);
        let mut file = Self {
            allowances: Allowances::default(),
            ast,
            comments,
            line_starts: line_starts(&text),
            offset,
            path,
            text,
        };
        let lines: Vec<&str> = file.text.lines().collect();
        allowances.attach(&file.scopes(), &lines);
        file.allowances = allowances;
        Ok(file)
    }

    /// True for files that hold tests, benchmarks or examples rather than
    /// production code: anything under `tests/`, `benches/` or `examples/`
    /// inside the nearest crate, or a `tests.rs` / `test.rs` module.
    pub fn is_test_file(&self) -> bool {
        let inside_crate = self
            .path
            .ancestors()
            .skip(1)
            .find(|dir| dir.join("Cargo.toml").is_file())
            .and_then(|root| self.path.strip_prefix(root).ok())
            .unwrap_or(&self.path);
        inside_crate.components().any(|component| {
            matches!(
                component.as_os_str().to_str(),
                Some("tests" | "benches" | "examples" | "tests.rs" | "test.rs")
            )
        })
    }

    pub fn lines(&self) -> Vec<&str> {
        self.text.lines().collect()
    }

    pub fn position(&self, offset: usize) -> Position {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        let column = self.text[self.line_starts[line]..offset].chars().count();
        Position {
            column: column + 1,
            line: line + 1,
        }
    }

    pub fn position_of(&self, span: Span) -> Position {
        self.position(self.range(span).start)
    }

    pub fn range(&self, span: Span) -> Range<usize> {
        let range = span.byte_range();
        range.start + self.offset..range.end + self.offset
    }

    pub fn range_of<T: Spanned>(&self, node: &T) -> Range<usize> {
        self.range(node.span())
    }

    pub fn relative_path(&self, root: &Path) -> PathBuf {
        self.path
            .strip_prefix(root)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| self.path.clone())
    }

    pub fn text_of<T: Spanned>(&self, node: &T) -> &str {
        &self.text[self.range_of(node)]
    }

    fn scopes(&self) -> Vec<Scope> {
        let mut collector = ScopeCollector {
            file: self,
            scopes: Vec::new(),
        };
        collector.visit_file(&self.ast);
        collector.scopes
    }
}

struct ScopeCollector<'a> {
    file: &'a SourceFile,
    scopes: Vec<Scope>,
}

impl ScopeCollector<'_> {
    fn record<T: Spanned>(&mut self, node: &T) {
        let range = self.file.range_of(node);
        self.scopes.push(Scope {
            end_line: self.file.position(range.end).line,
            start_line: self.file.position(range.start).line,
        });
    }
}

impl<'ast> Visit<'ast> for ScopeCollector<'_> {
    fn visit_field(&mut self, node: &'ast syn::Field) {
        self.record(node);
        syn::visit::visit_field(self, node);
    }

    fn visit_impl_item(&mut self, node: &'ast syn::ImplItem) {
        self.record(node);
        syn::visit::visit_impl_item(self, node);
    }

    fn visit_item(&mut self, node: &'ast syn::Item) {
        self.record(node);
        syn::visit::visit_item(self, node);
    }

    fn visit_stmt(&mut self, node: &'ast syn::Stmt) {
        self.record(node);
        syn::visit::visit_stmt(self, node);
    }

    fn visit_trait_item(&mut self, node: &'ast syn::TraitItem) {
        self.record(node);
        syn::visit::visit_trait_item(self, node);
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        self.record(node);
        syn::visit::visit_variant(self, node);
    }
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(text.match_indices('\n').map(|(index, _)| index + 1));
    starts
}

/// `syn::parse_file` strips a BOM and a shebang line before parsing.
fn skipped_prefix(text: &str) -> usize {
    let mut offset = 0;
    let mut rest = text;
    if let Some(stripped) = rest.strip_prefix('\u{feff}') {
        offset += '\u{feff}'.len_utf8();
        rest = stripped;
    }
    if rest.starts_with("#!") && !rest.starts_with("#![") {
        offset += rest.find('\n').unwrap_or(rest.len());
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_are_one_based() {
        let file = SourceFile::parse("a.rs", "fn a() {}\n  fn b() {}\n".to_string()).unwrap();
        assert_eq!(file.position(0), Position { column: 1, line: 1 });
        assert_eq!(file.position(12), Position { column: 3, line: 2 });
    }

    #[test]
    fn spans_survive_a_shebang() {
        let text = "#!/usr/bin/env rust-script\nfn a() {}\n".to_string();
        let file = SourceFile::parse("a.rs", text).unwrap();
        let item = &file.ast.items[0];
        assert_eq!(file.text_of(item), "fn a() {}");
        assert_eq!(file.position_of(item.span()).line, 2);
    }

    #[test]
    fn reports_parse_errors() {
        let error = SourceFile::parse("a.rs", "fn a( {}".to_string()).err().unwrap();
        assert_eq!(error.line, 1);
    }
}
