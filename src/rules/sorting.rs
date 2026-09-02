//! Sort your code alphabetically unless you have a documented reason not to.
//! <https://almaju.github.io/blog/docs/fundamentals/style/sorting>

use std::ops::Range;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::ordering::{Rank, SourceList, first_disorder, sorted_order};
use crate::rule::Rule;
use crate::rules::{Check, Context, Findings, type_ident};

pub struct Sorting;

impl Check for Sorting {
    fn run(&self, cx: &Context) -> Findings {
        let mut sorter = Sorter {
            cx,
            findings: Findings::default(),
        };
        sorter.visit_file(&cx.file.ast);
        sorter.findings
    }
}

const IMPL_GROUPS: [&str; 5] = [
    "associated consts",
    "associated types",
    "constructors",
    "pub fns",
    "private fns",
];
const TRAIT_GROUPS: [&str; 3] = ["associated consts", "associated types", "fns"];

struct Sorter<'a> {
    cx: &'a Context<'a>,
    findings: Findings,
}

/// One list rabot may reorder.
struct Candidate {
    /// Offset of the closing delimiter.
    close: usize,
    /// True when rabot may rewrite the list; false when it can only complain.
    fixable: bool,
    /// Names of the rank groups, for the message.
    groups: &'static [&'static str],
    members: Vec<(Rank, Range<usize>)>,
    /// Offset just past the opening delimiter.
    open: usize,
    separator: Option<char>,
    /// Where the diagnostic points.
    span: Span,
    /// "fields of `User`", "`impl User`", ...
    subject: String,
}

impl Sorter<'_> {
    fn check(&mut self, rule: Rule, candidate: Candidate) {
        let ranks: Vec<Rank> = candidate.members.iter().map(|(rank, _)| rank.clone()).collect();
        let Some(order) = sorted_order(&ranks) else {
            return;
        };
        let Some((before, after)) = first_disorder(&ranks) else {
            return;
        };
        let (first, second) = (&ranks[before], &ranks[after]);
        let why = if first.group == second.group {
            "alphabetical order".to_string()
        } else {
            format!(
                "{} come before {}",
                candidate.groups[second.group as usize], candidate.groups[first.group as usize]
            )
        };
        let message = format!(
            "{}: `{}` should come before `{}` ({why})",
            candidate.subject,
            second.key.original(),
            first.key.original(),
        );
        let help = if candidate.fixable {
            "run `rabot fmt` to reorder, or document the exception with `// rabot: allow(...) reason`"
        } else {
            "reorder by hand: the initializers may have side effects, so rabot will not move them"
        };
        if !self
            .findings
            .report_with_help(self.cx, rule, candidate.span, message, Some(help.to_string()))
        {
            return;
        }
        if !candidate.fixable {
            return;
        }
        let bodies = candidate.members.into_iter().map(|(_, range)| range).collect();
        let list = SourceList::new(
            &self.cx.file.text,
            candidate.open..candidate.close,
            bodies,
            candidate.separator,
        );
        self.findings.edits.push(list.reordered(&order));
    }

    fn check_derive(&mut self, attr: &syn::Attribute) {
        let Ok(list) = attr.meta.require_list() else {
            return;
        };
        let syn::MacroDelimiter::Paren(paren) = &list.delimiter else {
            return;
        };
        let Ok(paths) = list.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        else {
            return;
        };
        let members = paths
            .iter()
            .map(|path| {
                let range = self.cx.file.range_of(path);
                (Rank::new(0, &self.cx.file.text[range.clone()]), range)
            })
            .collect();
        self.check(
            Rule::SortedDerives,
            Candidate {
                close: self.cx.file.range(paren.span.close()).start,
                fixable: true,
                groups: &IMPL_GROUPS,
                members,
                open: self.cx.file.range(paren.span.open()).end,
                separator: Some(','),
                span: attr.span(),
                subject: "derive list".to_string(),
            },
        );
    }

    fn check_named_fields(&mut self, subject: String, span: Span, fields: &syn::FieldsNamed) {
        let members = fields
            .named
            .iter()
            .filter_map(|field| {
                let ident = field.ident.as_ref()?;
                Some((Rank::new(0, &ident.to_string()), self.cx.file.range_of(field)))
            })
            .collect();
        let brace = fields.brace_token.span;
        self.check(
            Rule::SortedFields,
            Candidate {
                close: self.cx.file.range(brace.close()).start,
                fixable: true,
                groups: &IMPL_GROUPS,
                members,
                open: self.cx.file.range(brace.open()).end,
                separator: Some(','),
                span,
                subject,
            },
        );
    }
}

impl<'ast> Visit<'ast> for Sorter<'_> {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if node.path().is_ident("derive") {
            self.check_derive(node);
        }
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        let mut members = Vec::new();
        let mut fixable = true;
        for field in &node.fields {
            let syn::Member::Named(ident) = &field.member else {
                return syn::visit::visit_expr_struct(self, node);
            };
            fixable &= is_pure(&field.expr);
            members.push((Rank::new(0, &ident.to_string()), self.cx.file.range_of(field)));
        }
        let brace = node.brace_token.span;
        let close = match &node.dot2_token {
            Some(dot2) => self.cx.file.range_of(dot2).start,
            None => self.cx.file.range(brace.close()).start,
        };
        let name = self.cx.file.text_of(&node.path).to_string();
        self.check(
            Rule::SortedStructLiteral,
            Candidate {
                close,
                fixable,
                groups: &IMPL_GROUPS,
                members,
                open: self.cx.file.range(brace.open()).end,
                separator: Some(','),
                span: node.path.span(),
                subject: format!("fields of `{name} {{ .. }}`"),
            },
        );
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        for attr in &node.attrs {
            self.visit_attribute(attr);
        }
        let order_is_semantic = node.attrs.iter().any(|attr| attr.path().is_ident("repr"))
            || node.variants.iter().any(|variant| variant.discriminant.is_some())
            || derives_ordering(&node.attrs);
        if !order_is_semantic {
            let members = node
                .variants
                .iter()
                .map(|variant| {
                    (
                        Rank::new(0, &variant.ident.to_string()),
                        self.cx.file.range_of(variant),
                    )
                })
                .collect();
            let brace = node.brace_token.span;
            self.check(
                Rule::SortedVariants,
                Candidate {
                    close: self.cx.file.range(brace.close()).start,
                    fixable: true,
                    groups: &IMPL_GROUPS,
                    members,
                    open: self.cx.file.range(brace.open()).end,
                    separator: Some(','),
                    span: node.ident.span(),
                    subject: format!("variants of `{}`", node.ident),
                },
            );
        }
        for variant in &node.variants {
            if let syn::Fields::Named(fields) = &variant.fields {
                let subject = format!("fields of `{}::{}`", node.ident, variant.ident);
                self.check_named_fields(subject, variant.ident.span(), fields);
            }
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let self_type = type_ident(&node.self_ty)
            .map(ToString::to_string)
            .unwrap_or_else(|| self.cx.file.text_of(&node.self_ty).to_string());
        let mut members = Vec::new();
        let mut sortable = true;
        for item in &node.items {
            let rank = match item {
                syn::ImplItem::Const(item) => Rank::new(0, &item.ident.to_string()),
                syn::ImplItem::Type(item) => Rank::new(1, &item.ident.to_string()),
                syn::ImplItem::Fn(item) => {
                    let group = if node.trait_.is_some() || is_constructor(&item.sig, &self_type) {
                        2
                    } else if matches!(item.vis, syn::Visibility::Inherited) {
                        4
                    } else {
                        3
                    };
                    Rank::new(group, &item.sig.ident.to_string())
                }
                _ => {
                    sortable = false;
                    break;
                }
            };
            members.push((rank, self.cx.file.range_of(item)));
        }
        if sortable {
            let brace = node.brace_token.span;
            let subject = match &node.trait_ {
                Some((_, path, _)) => {
                    format!("`impl {} for {self_type}`", self.cx.file.text_of(path))
                }
                None => format!("`impl {self_type}`"),
            };
            self.check(
                Rule::SortedImplItems,
                Candidate {
                    close: self.cx.file.range(brace.close()).start,
                    fixable: true,
                    groups: if node.trait_.is_some() {
                        &TRAIT_GROUPS
                    } else {
                        &IMPL_GROUPS
                    },
                    members,
                    open: self.cx.file.range(brace.open()).end,
                    separator: None,
                    span: node.impl_token.span(),
                    subject,
                },
            );
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        for attr in &node.attrs {
            self.visit_attribute(attr);
        }
        let layout_matters = node.attrs.iter().any(|attr| attr.path().is_ident("repr"));
        if let (false, syn::Fields::Named(fields)) = (layout_matters, &node.fields) {
            self.check_named_fields(format!("fields of `{}`", node.ident), node.ident.span(), fields);
        }
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let mut members = Vec::new();
        let mut sortable = true;
        for item in &node.items {
            let rank = match item {
                syn::TraitItem::Const(item) => Rank::new(0, &item.ident.to_string()),
                syn::TraitItem::Type(item) => Rank::new(1, &item.ident.to_string()),
                syn::TraitItem::Fn(item) => Rank::new(2, &item.sig.ident.to_string()),
                _ => {
                    sortable = false;
                    break;
                }
            };
            members.push((rank, self.cx.file.range_of(item)));
        }
        if sortable {
            let brace = node.brace_token.span;
            self.check(
                Rule::SortedTraitItems,
                Candidate {
                    close: self.cx.file.range(brace.close()).start,
                    fixable: true,
                    groups: &TRAIT_GROUPS,
                    members,
                    open: self.cx.file.range(brace.open()).end,
                    separator: None,
                    span: node.ident.span(),
                    subject: format!("`trait {}`", node.ident),
                },
            );
        }
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_pat_struct(&mut self, node: &'ast syn::PatStruct) {
        let mut members = Vec::new();
        for field in &node.fields {
            let syn::Member::Named(ident) = &field.member else {
                return syn::visit::visit_pat_struct(self, node);
            };
            members.push((Rank::new(0, &ident.to_string()), self.cx.file.range_of(field)));
        }
        let brace = node.brace_token.span;
        let close = match &node.rest {
            Some(rest) => self.cx.file.range_of(rest).start,
            None => self.cx.file.range(brace.close()).start,
        };
        let name = self.cx.file.text_of(&node.path).to_string();
        self.check(
            Rule::SortedStructPattern,
            Candidate {
                close,
                fixable: true,
                groups: &IMPL_GROUPS,
                members,
                open: self.cx.file.range(brace.open()).end,
                separator: Some(','),
                span: node.path.span(),
                subject: format!("fields of pattern `{name} {{ .. }}`"),
            },
        );
        syn::visit::visit_pat_struct(self, node);
    }
}

/// Deriving `PartialOrd` or `Ord` makes variant order part of the semantics.
fn derives_ordering(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("derive")
            && attr
                .parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                .is_ok_and(|paths| {
                    paths.iter().any(|path| {
                        path.segments.last().is_some_and(|segment| {
                            matches!(segment.ident.to_string().as_str(), "Ord" | "PartialOrd")
                        })
                    })
                })
    })
}

/// An associated function (no receiver) that hands back the type.
fn is_constructor(sig: &syn::Signature, self_type: &str) -> bool {
    if sig.receiver().is_some() {
        return false;
    }
    let syn::ReturnType::Type(_, ty) = &sig.output else {
        return false;
    };
    mentions_type(ty, self_type)
}

fn mentions_type(ty: &syn::Type, self_type: &str) -> bool {
    struct Finder<'a> {
        found: bool,
        self_type: &'a str,
    }
    impl<'ast> Visit<'ast> for Finder<'_> {
        fn visit_path_segment(&mut self, node: &'ast syn::PathSegment) {
            if node.ident == "Self" || node.ident == self.self_type {
                self.found = true;
            }
            syn::visit::visit_path_segment(self, node);
        }
    }
    let mut finder = Finder {
        found: false,
        self_type,
    };
    finder.visit_type(ty);
    finder.found
}

/// Whether reordering this initializer relative to its siblings could change
/// behaviour. Only expressions that are plainly side-effect free pass.
fn is_pure(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(_) | syn::Expr::Path(_) => true,
        syn::Expr::Cast(cast) => is_pure(&cast.expr),
        syn::Expr::Field(field) => is_pure(&field.base),
        syn::Expr::Group(group) => is_pure(&group.expr),
        syn::Expr::Paren(paren) => is_pure(&paren.expr),
        syn::Expr::Reference(reference) => is_pure(&reference.expr),
        syn::Expr::Unary(unary) => is_pure(&unary.expr),
        syn::Expr::Tuple(tuple) => tuple.elems.iter().all(is_pure),
        syn::Expr::Array(array) => array.elems.iter().all(is_pure),
        syn::Expr::Struct(inner) => inner.fields.iter().all(|field| is_pure(&field.expr)),
        syn::Expr::MethodCall(call) => {
            call.args.is_empty()
                && matches!(
                    call.method.to_string().as_str(),
                    "clone" | "into" | "to_owned" | "to_string" | "as_ref" | "as_str" | "len" | "is_empty"
                )
                && is_pure(&call.receiver)
        }
        syn::Expr::Call(call) => {
            let syn::Expr::Path(callee) = &*call.func else {
                return false;
            };
            let Some(last) = callee.path.segments.last() else {
                return false;
            };
            match last.ident.to_string().as_str() {
                "Some" | "Ok" | "Err" => call.args.iter().all(is_pure),
                "new" | "default" | "empty" => call.args.is_empty(),
                _ => false,
            }
        }
        _ => false,
    }
}
