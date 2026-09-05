//! Some things are genuinely hard to test: the clock, random number
//! generators, the environment. The instinct is to mock them. The right move
//! is to make them injectable.
//! <https://almaju.github.io/blog/docs/fundamentals/architecture/testing>
//! <https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies>

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings, type_ident};

pub struct Ambient;

impl Check for Ambient {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            config_depth: 0,
            cx,
            findings: Findings::default(),
            startup_depth: 0,
        };
        visitor.visit_file(&cx.file.ast);
        visitor.findings
    }
}

const CLOCKS: [&str; 7] = [
    "Instant",
    "Local",
    "OffsetDateTime",
    "SystemTime",
    "Timestamp",
    "Utc",
    "Zoned",
];
const NOW: [&str; 3] = ["now", "now_local", "now_utc"];
const SLEEPERS: [&str; 3] = ["task", "thread", "time"];

struct Visitor<'a> {
    /// Inside a function or impl that exists to read configuration
    /// (`load_config`, `Config::from_env`, `impl Settings`).
    config_depth: usize,
    cx: &'a Context<'a>,
    findings: Findings,
    /// Inside `fn main`, where the clock and the environment are read once
    /// and handed down.
    startup_depth: usize,
}

impl Visitor<'_> {
    fn check_call(&mut self, path: &syn::Path) {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let [.., owner, name] = segments.as_slice() else {
            return;
        };
        let call = AssociatedCall { name, owner };
        let shown = format!("`{owner}::{name}()`");
        if call.reads_clock() {
            if self.startup_depth == 0 {
                self.findings.report_with_help(
                    self.cx,
                    Rule::AmbientTime,
                    path.span(),
                    format!("{shown} reads the wall clock from inside the logic: nothing can freeze time to test this"),
                    Some(
                        "take a `Clock` (a trait with `fn now(&self)`) as a dependency; a `FixedClock` is useful in staging and demos, not only in tests"
                            .to_string(),
                    ),
                );
            }
        } else if call.draws_randomness() {
            if self.startup_depth == 0 {
                self.findings.report_with_help(
                    self.cx,
                    Rule::AmbientRandomness,
                    path.span(),
                    format!("{shown} draws from a global generator: this code cannot be replayed"),
                    Some(
                        "take an `Rng` as a parameter; a seeded generator makes the failure reproducible"
                            .to_string(),
                    ),
                );
            }
        } else if call.reads_environment() {
            if self.startup_depth == 0 && self.config_depth == 0 {
                self.findings.report_with_help(
                    self.cx,
                    Rule::AmbientConfig,
                    path.span(),
                    format!("{shown} reads configuration on the spot: a dependency hidden from the signature"),
                    Some(
                        "read it once in `main` (or in `Config::from_env`) and pass the value in; the signature then says what this needs"
                            .to_string(),
                    ),
                );
            }
        } else if call.sleeps() && self.cx.in_test_region(path.span()) {
            self.findings.report_with_help(
                self.cx,
                Rule::SleepInTests,
                path.span(),
                format!("{shown} in a test: it passes on your machine and fails on a loaded CI runner"),
                Some(
                    "inject the clock and advance it, or wait on the event itself (a channel, a notify), never on the time it usually takes"
                        .to_string(),
                ),
            );
        }
    }

    fn is_config_function(&self, ident: &syn::Ident) -> bool {
        let name = ident.to_string().to_ascii_lowercase();
        name.contains("config") || name.contains("env") || name.contains("settings")
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = &*node.func {
            self.check_call(&callee.path);
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let is_config = self.is_config_function(&node.sig.ident);
        self.config_depth += usize::from(is_config);
        syn::visit::visit_impl_item_fn(self, node);
        self.config_depth -= usize::from(is_config);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let is_main = node.sig.ident == "main";
        let is_config = self.is_config_function(&node.sig.ident);
        self.startup_depth += usize::from(is_main);
        self.config_depth += usize::from(is_config);
        syn::visit::visit_item_fn(self, node);
        self.config_depth -= usize::from(is_config);
        self.startup_depth -= usize::from(is_main);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let is_config = type_ident(&node.self_ty).is_some_and(|ident| self.is_config_function(ident));
        self.config_depth += usize::from(is_config);
        syn::visit::visit_item_impl(self, node);
        self.config_depth -= usize::from(is_config);
    }
}

/// The last two segments of a call path: `Utc::now`, `env::var`,
/// `thread::sleep`.
struct AssociatedCall<'a> {
    name: &'a str,
    owner: &'a str,
}

impl AssociatedCall<'_> {
    fn draws_randomness(&self) -> bool {
        matches!(
            (self.owner, self.name),
            (
                "rand",
                "random" | "rng" | "thread_rng" | "random_range" | "random_bool"
            ) | ("OsRng", "default")
                | ("StdRng", "from_entropy" | "from_os_rng")
                | (
                    "fastrand",
                    "u64" | "u32" | "usize" | "bool" | "f64" | "shuffle" | "choice"
                )
        )
    }

    fn reads_clock(&self) -> bool {
        CLOCKS.contains(&self.owner) && NOW.contains(&self.name)
    }

    fn reads_environment(&self) -> bool {
        self.owner == "env" && matches!(self.name, "var" | "var_os" | "vars" | "vars_os")
    }

    fn sleeps(&self) -> bool {
        self.name == "sleep" && SLEEPERS.contains(&self.owner)
    }
}
