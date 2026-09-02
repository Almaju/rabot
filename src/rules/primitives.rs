//! Wrap every primitive that carries domain meaning in a dedicated type.
//! <https://almaju.github.io/blog/docs/fundamentals/modeling/primitives>

use std::collections::BTreeMap;

use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings, has_cfg_test, has_test_attr};

pub struct Primitives;

impl Check for Primitives {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            cx,
            findings: Findings::default(),
            test_depth: usize::from(cx.in_test_file()),
        };
        visitor.visit_file(&cx.file.ast);
        visitor.findings
    }
}

const PRIMITIVES: [&str; 20] = [
    "String", "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8", "u16",
    "u32", "u64", "u128", "usize", "Vec<u8>", "&[u8]",
];

struct Visitor<'a> {
    cx: &'a Context<'a>,
    findings: Findings,
    test_depth: usize,
}

impl Visitor<'_> {
    fn check_signature(&mut self, sig: &syn::Signature) {
        if self.test_depth > 0 {
            return;
        }
        let mut by_type: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for input in &sig.inputs {
            let syn::FnArg::Typed(arg) = input else {
                continue;
            };
            let Some(primitive) = primitive_name(&arg.ty) else {
                continue;
            };
            let name = match &*arg.pat {
                syn::Pat::Ident(ident) => ident.ident.to_string(),
                other => self.cx.file.text_of(other).to_string(),
            };
            by_type.entry(primitive).or_default().push(name);
        }
        let threshold = self.cx.config.thresholds.primitive_soup.max(2);
        for (primitive, names) in by_type {
            if names.len() < threshold {
                continue;
            }
            let list = names
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ");
            self.findings.report_with_help(
                self.cx,
                Rule::PrimitiveSoup,
                sig.ident.span(),
                format!(
                    "`{}` takes {} `{primitive}` parameters ({list}): the compiler cannot tell them apart, so a swapped call site type-checks",
                    sig.ident,
                    names.len()
                ),
                Some("give each its own newtype (`struct UserId(String);`) and parse at the boundary".to_string()),
            );
        }
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let is_test = has_test_attr(&node.attrs);
        self.test_depth += usize::from(is_test);
        self.check_signature(&node.sig);
        self.test_depth -= usize::from(is_test);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let is_test = has_test_attr(&node.attrs);
        self.test_depth += usize::from(is_test);
        self.check_signature(&node.sig);
        syn::visit::visit_item_fn(self, node);
        self.test_depth -= usize::from(is_test);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let is_test = has_cfg_test(&node.attrs);
        self.test_depth += usize::from(is_test);
        syn::visit::visit_item_mod(self, node);
        self.test_depth -= usize::from(is_test);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.check_signature(&node.sig);
    }
}

/// `&str`, `String`, `u64`, `Option<String>`... the name two swappable
/// parameters share. `None` for anything that is not a primitive.
fn primitive_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Reference(reference) => {
            let inner = primitive_name(&reference.elem)?;
            Some(if reference.mutability.is_some() {
                format!("&mut {inner}")
            } else {
                format!("&{inner}")
            })
        }
        syn::Type::Slice(slice) => {
            let inner = primitive_name(&slice.elem)?;
            Some(format!("[{inner}]"))
        }
        syn::Type::Path(path) if path.qself.is_none() => {
            let segment = path.path.segments.last()?;
            let name = segment.ident.to_string();
            if name == "Option" {
                if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = arguments.args.first()
                {
                    return primitive_name(inner).map(|inner| format!("Option<{inner}>"));
                }
                return None;
            }
            (path.path.segments.len() == 1 && PRIMITIVES.contains(&name.as_str())).then_some(name)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(source: &str) -> Option<String> {
        primitive_name(&syn::parse_str(source).unwrap())
    }

    #[test]
    fn normalises_primitives() {
        assert_eq!(name("&str"), Some("&str".to_string()));
        assert_eq!(name("Option<String>"), Some("Option<String>".to_string()));
        assert_eq!(name("&mut u64"), Some("&mut u64".to_string()));
        assert_eq!(name("UserId"), None);
        assert_eq!(name("&[u8]"), Some("&[u8]".to_string()));
    }
}
