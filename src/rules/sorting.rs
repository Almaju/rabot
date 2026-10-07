//! Sort your code alphabetically unless you have a documented reason not to.
//! <https://almaju.github.io/blog/docs/fundamentals/style/sorting>

use std::ops::Range;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::ordering::{Rank, SortKey, SourceList, first_disorder, sorted_order};
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
const DERIVE_GROUPS: [&str; 3] = [
    "derives pinned first by `derive-order`",
    "unpinned derives",
    "derives pinned last by `derive-order`",
];
const ALPHABETICAL: &str = "alphabetical order";
const DERIVE_ALPHABETICAL: &str = "alphabetical order, with a derive after the trait it extends";

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
    /// How members within one group are ordered, for the message.
    order_note: &'static str,
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
            candidate.order_note.to_string()
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
            "run `rabot fmt` to reorder, or say why above it: `// Deliberately unsorted: <reason>`"
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
        let pins = DerivePins::new(&self.cx.config.sorting.derive_order);
        let members = paths
            .iter()
            .map(|path| {
                let range = self.cx.file.range_of(path);
                let name = path
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string())
                    .unwrap_or_default();
                (pins.rank(&name), range)
            })
            .collect();
        self.check(
            Rule::SortedDerives,
            Candidate {
                close: self.cx.file.range(paren.span.close()).start,
                fixable: true,
                groups: &DERIVE_GROUPS,
                members,
                open: self.cx.file.range(paren.span.open()).end,
                order_note: DERIVE_ALPHABETICAL,
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
                order_note: ALPHABETICAL,
                separator: Some(','),
                span,
                subject,
            },
        );
    }

    /// Whether the declaration order of a type's fields (and variants) is
    /// behaviour rather than layout on the page: `#[repr]` fixes the memory
    /// layout, a derived `PartialOrd` compares field by field, and the
    /// configured derives number, encode or iterate in that order.
    fn order_is_semantic(&self, attrs: &[syn::Attribute]) -> bool {
        let sensitive = &self.cx.config.sorting.order_sensitive_derives;
        let metas = effective_metas(attrs);
        metas.iter().any(|meta| meta.path().is_ident("repr"))
            || derived_paths(&metas).any(|path| {
                let last = path.segments.last().map(|segment| segment.ident.to_string());
                matches!(last.as_deref(), Some("Ord" | "PartialOrd"))
                    || sensitive.iter().any(|name| path_matches(&path, name))
            })
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
                order_note: ALPHABETICAL,
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
        let fields_are_semantic = self.order_is_semantic(&node.attrs);
        let variants_are_semantic = fields_are_semantic
            || node.variants.iter().any(|variant| variant.discriminant.is_some())
            || is_untagged(&node.attrs)
            || node.variants.iter().any(|variant| is_untagged(&variant.attrs));
        if !variants_are_semantic {
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
                    order_note: ALPHABETICAL,
                    separator: Some(','),
                    span: node.ident.span(),
                    subject: format!("variants of `{}`", node.ident),
                },
            );
        }
        for variant in &node.variants {
            if let (false, syn::Fields::Named(fields)) = (fields_are_semantic, &variant.fields) {
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
                    order_note: ALPHABETICAL,
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
        if let (false, syn::Fields::Named(fields)) = (self.order_is_semantic(&node.attrs), &node.fields) {
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
                    order_note: ALPHABETICAL,
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
                order_note: ALPHABETICAL,
                separator: Some(','),
                span: node.path.span(),
                subject: format!("fields of pattern `{name} {{ .. }}`"),
            },
        );
        syn::visit::visit_pat_struct(self, node);
    }
}

/// The `derive-order` setting: names pinned first, names pinned last, and
/// the supertrait rule for everything in between.
struct DerivePins {
    first: Vec<String>,
    last: Vec<String>,
}

impl DerivePins {
    fn new(order: &[String]) -> Self {
        let ellipsis = order.iter().position(|name| name == "...");
        let (first, last) = match ellipsis {
            Some(index) => (order[..index].to_vec(), order[index + 1..].to_vec()),
            None => (order.to_vec(), Vec::new()),
        };
        Self { first, last }
    }

    fn rank(&self, name: &str) -> Rank {
        if let Some(index) = self.first.iter().position(|pinned| pinned == name) {
            return Rank::new(0, &format!("{index:06}"));
        }
        if let Some(index) = self.last.iter().position(|pinned| pinned == name) {
            return Rank::new(2, &format!("{index:06}"));
        }
        Rank {
            group: 1,
            key: SortKey::new(&supertrait_key(name)).labelled(name),
        }
    }
}

/// Alphabetical, except that a derive sorts right after the trait it
/// extends: `Eq` after `PartialEq`, `Ord` after `PartialOrd`, `Copy` after
/// `Clone`. The key of `Eq` is "PartialEq" plus a suffix that sorts before
/// any other continuation, so it lands immediately after `PartialEq`.
fn supertrait_key(name: &str) -> String {
    match name {
        "Copy" => "Clone\u{1}Copy".to_string(),
        "Eq" => "PartialEq\u{1}Eq".to_string(),
        "Ord" => "PartialOrd\u{1}Ord".to_string(),
        other => other.to_string(),
    }
}

/// The attributes an item may carry: each attribute as written, and the
/// ones inside `#[cfg_attr(predicate, ..)]`, since a type that derives
/// `uniffi::Record` behind a feature still has to keep its order.
fn effective_metas(attrs: &[syn::Attribute]) -> Vec<syn::Meta> {
    let mut metas = Vec::new();
    let mut pending: Vec<syn::Meta> = attrs.iter().map(|attr| attr.meta.clone()).collect();
    while let Some(meta) = pending.pop() {
        let syn::Meta::List(list) = &meta else {
            metas.push(meta);
            continue;
        };
        if !list.path.is_ident("cfg_attr") {
            metas.push(meta);
            continue;
        }
        if let Ok(inner) = list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated) {
            pending.extend(inner.into_iter().skip(1));
        }
    }
    metas
}

/// Every path in every `derive(..)` among `metas`.
fn derived_paths(metas: &[syn::Meta]) -> impl Iterator<Item = syn::Path> + '_ {
    metas
        .iter()
        .filter_map(|meta| match meta {
            syn::Meta::List(list) if list.path.is_ident("derive") => list
                .parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                .ok(),
            _ => None,
        })
        .flatten()
}

/// `#[serde(untagged)]`: serde tries the variants top to bottom and keeps
/// the first that fits, so their order decides what a value becomes.
fn is_untagged(attrs: &[syn::Attribute]) -> bool {
    effective_metas(attrs).iter().any(|meta| match meta {
        syn::Meta::List(list) if list.path.is_ident("serde") => list
            .tokens
            .clone()
            .into_iter()
            .any(|token| matches!(token, proc_macro2::TokenTree::Ident(ident) if ident == "untagged")),
        _ => false,
    })
}

/// A configured derive name against a derive path: a plain name matches
/// the last segment, a path (`uniffi::Record`) matches the trailing segments.
fn path_matches(path: &syn::Path, name: &str) -> bool {
    let wanted: Vec<&str> = name.split("::").filter(|part| !part.is_empty()).collect();
    let segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    !wanted.is_empty()
        && segments.len() >= wanted.len()
        && segments[segments.len() - wanted.len()..]
            .iter()
            .zip(&wanted)
            .all(|(segment, wanted)| segment == wanted)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ordering::sorted_order;

    fn order(pins: &[&str], derives: &[&str]) -> Vec<String> {
        let pins = DerivePins::new(&pins.iter().map(|p| p.to_string()).collect::<Vec<_>>());
        let ranks: Vec<Rank> = derives.iter().map(|name| pins.rank(name)).collect();
        let order = sorted_order(&ranks).unwrap_or_else(|| (0..derives.len()).collect());
        order
            .into_iter()
            .map(|index| derives[index].to_string())
            .collect()
    }

    #[test]
    fn configured_derives_match_by_name_or_by_path() {
        let path = |text: &str| syn::parse_str::<syn::Path>(text).expect("path");
        assert!(path_matches(&path("clap::Parser"), "Parser"));
        assert!(path_matches(&path("Parser"), "Parser"));
        assert!(path_matches(&path("::uniffi::Record"), "uniffi::Record"));
        assert!(!path_matches(&path("Record"), "uniffi::Record"));
        assert!(!path_matches(&path("thiserror::Error"), "uniffi::Error"));
        assert!(!path_matches(&path("clap::Parser"), "clap"));
        assert!(!path_matches(&path("Parser"), ""));
    }

    #[test]
    fn a_derive_follows_the_trait_it_extends() {
        assert_eq!(
            order(
                &[],
                &[
                    "Ord",
                    "Eq",
                    "PartialOrd",
                    "Hash",
                    "PartialEq",
                    "Debug",
                    "Copy",
                    "Clone"
                ]
            ),
            vec![
                "Clone",
                "Copy",
                "Debug",
                "Hash",
                "PartialEq",
                "Eq",
                "PartialOrd",
                "Ord"
            ]
        );
    }

    #[test]
    fn pins_go_first_then_last_with_the_rest_in_between() {
        assert_eq!(
            order(
                &["Debug", "...", "Serialize"],
                &["Serialize", "Eq", "Debug", "Clone", "PartialEq"]
            ),
            vec!["Debug", "Clone", "PartialEq", "Eq", "Serialize"]
        );
        assert_eq!(
            order(&["Serialize", "Deserialize"], &["Deserialize", "Serialize"]),
            vec!["Serialize", "Deserialize"]
        );
    }
}
