//! Wrap every primitive that carries domain meaning in a dedicated type.
//! <https://almaju.github.io/blog/docs/fundamentals/modeling/primitives>

use std::collections::BTreeMap;

use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings};

pub struct Primitives;

impl Check for Primitives {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            cx,
            findings: Findings::default(),
            in_trait_impl: false,
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
    /// Inside `impl Trait for T`, where the signatures are the trait's choice.
    in_trait_impl: bool,
}

impl Visitor<'_> {
    /// `email: String` in a domain struct: the name promises an email, the
    /// type accepts anything.
    fn check_fields(&mut self, item: &syn::ItemStruct) {
        if self.is_boundary(&item.ident.to_string()) {
            return;
        }
        let syn::Fields::Named(fields) = &item.fields else {
            return;
        };
        for field in &fields.named {
            let Some(ident) = &field.ident else {
                continue;
            };
            let name = ident.to_string();
            let Some(primitive) = primitive_name(&field.ty) else {
                continue;
            };
            if primitive.contains("bool") || primitive.contains("char") || !self.is_domain_field(&name) {
                continue;
            }
            let suggestion = newtype_name(&name);
            let inner = primitive.trim_start_matches("Option<").trim_end_matches('>');
            self.findings.report_with_help(
                self.cx,
                Rule::PrimitiveField,
                ident.span(),
                format!(
                    "`{name}: {primitive}` in `{}`: the name promises a {}, the type accepts anything",
                    item.ident,
                    name.trim_start_matches('_').replace('_', " ")
                ),
                Some(format!(
                    "wrap it (`struct {suggestion}({inner});`) and validate once, in `{suggestion}::parse`, at the boundary"
                )),
            );
        }
    }

    fn check_signature(&mut self, sig: &syn::Signature) {
        if self.in_trait_impl {
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

    fn is_boundary(&self, type_name: &str) -> bool {
        self.cx
            .config
            .naming
            .boundary_suffixes
            .iter()
            .any(|suffix| type_name.ends_with(suffix.as_str()) && type_name != suffix)
    }

    fn is_domain_field(&self, name: &str) -> bool {
        let lowered = name.to_ascii_lowercase();
        self.cx.config.naming.domain_fields.iter().any(|pattern| {
            if let Some(suffix) = pattern.strip_prefix('_') {
                lowered.ends_with(&format!("_{suffix}"))
            } else {
                lowered == *pattern || lowered.ends_with(&format!("_{pattern}"))
            }
        })
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_signature(&node.sig);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_signature(&node.sig);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let was = self.in_trait_impl;
        self.in_trait_impl = node.trait_.is_some();
        syn::visit::visit_item_impl(self, node);
        self.in_trait_impl = was;
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_fields(node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.check_signature(&node.sig);
    }
}

/// `user_id` becomes `UserId`, `created_at` becomes `CreatedAt`.
fn newtype_name(field: &str) -> String {
    field
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
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
    fn builds_newtype_names() {
        assert_eq!(newtype_name("user_id"), "UserId");
        assert_eq!(newtype_name("email"), "Email");
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
