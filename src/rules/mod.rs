//! One module per article. Each rule reads a [`SourceFile`] and returns
//! [`Findings`]: diagnostics, plus edits when the rule can fix itself.

pub mod ambient;
pub mod comments;
pub mod dependencies;
pub mod errors;
pub mod naming;
pub mod primitives;
pub mod sorting;
pub mod structs;
pub mod tests;

use std::collections::BTreeSet;

use proc_macro2::Span;
use syn::visit::Visit;

use crate::config::Config;
use crate::diagnostic::{Diagnostic, Level};
use crate::edit::Edit;
use crate::rule::Rule;
use crate::source_file::SourceFile;

/// What every rule needs to look at one file.
pub struct Context<'a> {
    pub config: &'a Config,
    pub file: &'a SourceFile,
    pub local_types: &'a LocalTypes,
}

impl Context<'_> {
    /// Build a diagnostic unless the rule is allowed, globally or right here.
    pub fn diagnostic(&self, rule: Rule, span: Span, message: String) -> Option<Diagnostic> {
        let level = self.config.level(rule);
        if level == Level::Allow {
            return None;
        }
        let offset = self.file.range(span).start;
        if self.config.tests.relax.contains(&rule) && self.file.test_regions.contains(offset) {
            return None;
        }
        let position = self.file.position(offset);
        if self.file.allowances.covers(rule, position.line) {
            return None;
        }
        Some(Diagnostic {
            help: None,
            level,
            message,
            path: self.file.path.clone(),
            position,
            rule,
        })
    }

    /// Whether `span` lies in test code (see [`crate::test_regions`]).
    pub fn in_test_region(&self, span: Span) -> bool {
        self.file.test_regions.contains(self.file.range(span).start)
    }

    /// Every rule on this file, plus the problems with its allow comments.
    pub fn run_all(&self) -> Findings {
        let mut findings = Findings::default();
        for problem in self.file.allowances.problems() {
            let level = self.config.level(problem.rule);
            if level != Level::Allow {
                findings.diagnostics.push(Diagnostic {
                    help: None,
                    level,
                    message: problem.message.clone(),
                    path: self.file.path.clone(),
                    position: crate::diagnostic::Position {
                        column: 1,
                        line: problem.line,
                    },
                    rule: problem.rule,
                });
            }
        }
        for check in all() {
            findings.extend(check.run(self));
        }
        findings
    }
}

/// What one rule found in one file.
#[derive(Debug, Default)]
pub struct Findings {
    pub diagnostics: Vec<Diagnostic>,
    pub edits: Vec<Edit>,
}

impl Findings {
    pub fn extend(&mut self, other: Findings) {
        self.diagnostics.extend(other.diagnostics);
        self.edits.extend(other.edits);
    }

    /// Report a diagnostic; returns whether it was actually recorded (it is
    /// not when the rule is allowed), so callers can skip building a fix.
    pub fn report(&mut self, cx: &Context, rule: Rule, span: Span, message: String) -> bool {
        self.report_with_help(cx, rule, span, message, None)
    }

    pub fn report_with_help(
        &mut self,
        cx: &Context,
        rule: Rule,
        span: Span,
        message: String,
        help: Option<String>,
    ) -> bool {
        match cx.diagnostic(rule, span, message) {
            Some(mut diagnostic) => {
                diagnostic.help = help;
                self.diagnostics.push(diagnostic);
                true
            }
            None => false,
        }
    }
}

/// A rule: reads one file, reports what it finds.
pub trait Check {
    fn run(&self, cx: &Context) -> Findings;
}

/// Every rule, in the order they run.
pub fn all() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(ambient::Ambient),
        Box::new(comments::Comments),
        Box::new(dependencies::Dependencies),
        Box::new(errors::Errors),
        Box::new(naming::Naming),
        Box::new(primitives::Primitives),
        Box::new(sorting::Sorting),
        Box::new(structs::Structs),
        Box::new(tests::Tests),
    ]
}

/// Names of the structs, enums, unions and type aliases defined anywhere in
/// the files being checked. Rules that talk about "your types" need this.
#[derive(Clone, Debug, Default)]
pub struct LocalTypes {
    names: BTreeSet<String>,
}

impl LocalTypes {
    pub fn collect<'a>(files: impl IntoIterator<Item = &'a SourceFile>) -> Self {
        let mut local_types = LocalTypes::default();
        for file in files {
            local_types.visit_file(&file.ast);
        }
        local_types
    }

    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn insert(&mut self, name: impl Into<String>) {
        self.names.insert(name.into());
    }
}

impl<'ast> Visit<'ast> for LocalTypes {
    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.insert(node.ident.to_string());
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.insert(node.ident.to_string());
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.insert(node.ident.to_string());
    }

    fn visit_item_union(&mut self, node: &'ast syn::ItemUnion) {
        self.insert(node.ident.to_string());
    }
}

/// The identifier a type is known by: `Foo` for `Foo`, `&Foo`, `&mut Foo`,
/// `a::b::Foo` and `Foo<T>`.
pub fn type_ident(ty: &syn::Type) -> Option<&syn::Ident> {
    match ty {
        syn::Type::Group(group) => type_ident(&group.elem),
        syn::Type::Paren(paren) => type_ident(&paren.elem),
        syn::Type::Path(path) if path.qself.is_none() => {
            path.path.segments.last().map(|segment| &segment.ident)
        }
        syn::Type::Reference(reference) => type_ident(&reference.elem),
        _ => None,
    }
}

/// The type inside `Option<T>`, `Result<T, E>`, `Box<T>`, `Arc<T>`, `Rc<T>`
/// (or `ty` itself when it is none of these).
pub fn unwrapped(ty: &syn::Type) -> &syn::Type {
    let syn::Type::Path(path) = ty else {
        return ty;
    };
    let Some(segment) = path.path.segments.last() else {
        return ty;
    };
    if !matches!(
        segment.ident.to_string().as_str(),
        "Arc" | "Box" | "Option" | "Rc" | "Result"
    ) {
        return ty;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return ty;
    };
    match arguments.args.first() {
        Some(syn::GenericArgument::Type(inner)) => unwrapped(inner),
        _ => ty,
    }
}
