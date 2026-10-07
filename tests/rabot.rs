//! End-to-end checks against the fixtures: every rule fires where expected,
//! `fmt` produces the golden output, and a second `fmt` changes nothing.

use std::path::{Path, PathBuf};

use rabot::app::{App, FormatMode};
use rabot::config::Config;
use rabot::file_set::Scope;
use rabot::rule::Rule;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn lint_findings() -> Vec<(Rule, usize)> {
    let root = fixtures().join("lint");
    let app = App::new(Config::default(), root.clone());
    let outcome = app
        .check(&Scope::Paths(vec![root.join("src")]))
        .expect("check runs");
    outcome
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path.ends_with("lib.rs"))
        .map(|diagnostic| (diagnostic.rule, diagnostic.position.line))
        .collect()
}

#[test]
fn every_rule_fires_where_expected() {
    let findings = lint_findings();
    let expected = [
        (Rule::AmbientConfig, 46),
        (Rule::AmbientConfig, 104),
        (Rule::AmbientRandomness, 47),
        (Rule::AmbientTime, 45),
        (Rule::BooleanValidation, 40),
        (Rule::BooleanValidation, 94),
        (Rule::BypassableConstructor, 23),
        (Rule::CommentedOutCode, 105),
        (Rule::DroppedErrorContext, 48),
        (Rule::EscapeHatchVariant, 37),
        (Rule::FreeFunction, 61),
        (Rule::FreeFunction, 112),
        (Rule::FreeFunction, 114),
        (Rule::FreeFunction, 118),
        (Rule::FreeFunction, 137),
        (Rule::FreeFunction, 147),
        (Rule::GlobalState, 7),
        (Rule::IgnoredTest, 175),
        (Rule::MockUsage, 173),
        (Rule::OrphanModule, 5),
        (Rule::PanicInProduction, 104),
        (Rule::PanicInProduction, 109),
        (Rule::PanicInProduction, 115),
        (Rule::PrimitiveField, 12),
        (Rule::PrimitiveField, 13),
        (Rule::PrimitiveSoup, 103),
        (Rule::SectionedFunction, 147),
        (Rule::SleepInTests, 185),
        (Rule::SortedDerives, 10),
        (Rule::SortedFields, 11),
        (Rule::SortedFields, 73),
        (Rule::SortedFields, 83),
        (Rule::SortedImplItems, 93),
        (Rule::SortedStructLiteral, 119),
        (Rule::SortedStructPattern, 138),
        (Rule::SortedTraitItems, 128),
        (Rule::SortedVariants, 70),
        (Rule::StringlyTypedField, 32),
        (Rule::SwallowedError, 62),
        (Rule::SwallowedError, 63),
        (Rule::SwallowedError, 66),
        (Rule::TooManyParameters, 126),
        (Rule::UndocumentedException, 82),
        (Rule::UnknownRule, 88),
        (Rule::UntypedError, 26),
        (Rule::UntypedError, 114),
        (Rule::UntypedError, 122),
        (Rule::VagueTodo, 107),
        (Rule::VagueTypeName, 89),
    ];
    for expectation in &expected {
        assert!(
            findings.contains(expectation),
            "missing {expectation:?} in {findings:#?}"
        );
    }
    let unexpected: Vec<_> = findings
        .iter()
        .filter(|finding| !expected.contains(finding))
        .collect();
    assert!(unexpected.is_empty(), "unexpected findings: {unexpected:#?}");
}

#[test]
fn derive_messages_name_the_derive() {
    let root = fixtures().join("lint");
    let app = App::new(Config::default(), root.clone());
    let outcome = app
        .check(&Scope::Paths(vec![root.join("src")]))
        .expect("check runs");
    let messages: Vec<&str> = outcome
        .diagnostics
        .iter()
        .filter(|d| d.rule == Rule::SortedDerives)
        .map(|d| d.message.as_str())
        .collect();
    assert!(!messages.is_empty());
    assert!(messages.iter().all(|m| !m.contains('\u{1}')), "{messages:?}");
}

#[test]
fn documented_exceptions_are_honoured() {
    let findings = lint_findings();
    // `Severity` is allowed with a reason; `LOGGER` is infrastructure; the
    // test module may unwrap; `main` may expect.
    assert!(
        !findings
            .iter()
            .any(|(rule, line)| *rule == Rule::SortedVariants && *line == 77)
    );
    assert!(
        !findings
            .iter()
            .any(|(rule, line)| *rule == Rule::GlobalState && *line == 8)
    );
    assert!(
        !findings
            .iter()
            .any(|(rule, line)| *rule == Rule::PanicInProduction && *line > 155)
    );
    // Test-gated items relax the domain rules: `#[cfg(test)] fn test_helper` and
    // `#[cfg(any(test, ..))] struct MockService` raise nothing.
    let relaxed_region = 156..=169;
    let in_region: Vec<_> = findings
        .iter()
        .filter(|(_, line)| relaxed_region.contains(line))
        .collect();
    assert!(
        in_region.is_empty(),
        "test-gated code should be silent: {in_region:?}"
    );
}

#[test]
fn fmt_matches_the_golden_file() {
    let dir = std::env::temp_dir().join(format!("rabot-fmt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("before.rs");
    std::fs::copy(fixtures().join("fmt/before.rs"), &target).expect("copy fixture");

    let app = App::new(Config::default(), dir.clone());
    let outcome = app
        .format(&Scope::Paths(vec![target.clone()]), FormatMode::Write)
        .expect("fmt runs");
    assert_eq!(outcome.changed.len(), 1);
    assert_eq!(outcome.changed[0].path, target);
    let formatted = std::fs::read_to_string(&target).expect("read result");
    let expected = std::fs::read_to_string(fixtures().join("fmt/after.rs")).expect("read golden");
    assert_eq!(formatted, expected);

    let again = app
        .format(&Scope::Paths(vec![target.clone()]), FormatMode::Check)
        .expect("second fmt runs");
    assert!(
        again.changed.is_empty(),
        "fmt is not idempotent: {:#?}",
        again.diagnostics
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn fmt_check_reports_without_writing() {
    let dir = std::env::temp_dir().join(format!("rabot-fmt-check-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("before.rs");
    std::fs::copy(fixtures().join("fmt/before.rs"), &target).expect("copy fixture");
    let before = std::fs::read_to_string(&target).expect("read");

    let app = App::new(Config::default(), dir.clone());
    let outcome = app
        .format(&Scope::Paths(vec![target.clone()]), FormatMode::Check)
        .expect("fmt runs");
    assert_eq!(outcome.changed.len(), 1);
    assert_eq!(outcome.changed[0].before, before);
    assert_ne!(outcome.changed[0].after, before);
    assert!(
        outcome
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule.fixable())
    );
    assert_eq!(std::fs::read_to_string(&target).expect("read"), before);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn changed_scope_follows_git() {
    let dir = std::env::temp_dir().join(format!("rabot-changed-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).expect("temp dir");
    let git = |args: &[&str]| {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(args)
            .status()
            .expect("git runs");
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "t"]);
    std::fs::write(dir.join("src/committed.rs"), "pub struct A { b: u8, a: u8 }\n").expect("write");
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "init"]);
    std::fs::write(dir.join("src/fresh.rs"), "pub struct B { b: u8, a: u8 }\n").expect("write");

    let app = App::new(Config::default(), dir.clone());
    let uncommitted = app.check(&Scope::Changed { since: None }).expect("check runs");
    let files: Vec<_> = uncommitted.diagnostics.iter().map(|d| d.path.clone()).collect();
    assert_eq!(files, vec![dir.join("src/fresh.rs")]);

    let since_root = app
        .check(&Scope::Changed {
            since: Some("HEAD".to_string()),
        })
        .expect("check runs");
    assert_eq!(since_root.files_seen, 1);

    let everything = app.check(&Scope::Paths(Vec::new())).expect("check runs");
    assert_eq!(everything.files_seen, 2);

    std::fs::create_dir_all(dir.join("generated/deep")).expect("dir");
    std::fs::write(
        dir.join("generated/deep/skip.rs"),
        "pub struct C { b: u8, a: u8 }\n",
    )
    .expect("write");
    let mut config = Config::default();
    config.files.exclude.push("generated".to_string());
    let excluded = App::new(config, dir.clone())
        .check(&Scope::Changed { since: None })
        .expect("check runs");
    assert_eq!(
        excluded.files_seen, 1,
        "excludes apply to nested files under --changed"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// The examples next to each rule's page: `docs/src/rules/<rule>/bad.rs`
/// breaks exactly that rule and `good.rs` breaks none. The page includes
/// both verbatim, so the published examples are the ones under test.
fn rule_example(rule: Rule, file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/src/rules")
        .join(rule.name())
        .join(file)
}

/// Lints one example on its own: local types are collected from the files
/// in scope, so checking a whole directory would let a `User` in one rule's
/// example turn a free function in another's into a finding.
fn rule_findings(path: &Path) -> Vec<(Rule, usize)> {
    let root = path.parent().expect("fixture has a directory").to_path_buf();
    let app = App::new(Config::default(), root);
    let outcome = app
        .check(&Scope::Paths(vec![path.to_path_buf()]))
        .expect("check runs");
    outcome
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule, diagnostic.position.line))
        .collect()
}

#[test]
fn every_rule_has_a_bad_example_that_breaks_only_that_rule() {
    for rule in Rule::all() {
        let findings = rule_findings(&rule_example(*rule, "bad.rs"));
        assert!(
            findings.iter().any(|(found, _)| found == rule),
            "{rule}: bad.rs does not trigger the rule; found {findings:?}"
        );
        let others: Vec<_> = findings.iter().filter(|(found, _)| found != rule).collect();
        assert!(
            others.is_empty(),
            "{rule}: bad.rs must break only its own rule, but also raised {others:?}"
        );
    }
}

#[test]
fn every_rule_has_a_good_example_that_is_silent() {
    for rule in Rule::all() {
        let findings = rule_findings(&rule_example(*rule, "good.rs"));
        assert!(
            findings.is_empty(),
            "{rule}: good.rs should be clean; found {findings:?}"
        );
    }
}

#[test]
fn fmt_turns_every_bad_sorting_example_into_the_good_one() {
    let dir = std::env::temp_dir().join(format!("rabot-rules-fmt-{}", std::process::id()));
    for rule in Rule::all().iter().filter(|rule| rule.fixable()) {
        let scratch = dir.join(rule.name());
        std::fs::create_dir_all(&scratch).expect("temp dir");
        let target = scratch.join("bad.rs");
        std::fs::copy(rule_example(*rule, "bad.rs"), &target).expect("copy example");

        let app = App::new(Config::default(), scratch.clone());
        let outcome = app
            .format(&Scope::Paths(vec![target.clone()]), FormatMode::Write)
            .expect("fmt runs");
        assert_eq!(outcome.changed.len(), 1, "{rule}: fmt changes the bad example");
        let formatted = std::fs::read_to_string(&target).expect("read result");
        let expected = std::fs::read_to_string(rule_example(*rule, "good.rs")).expect("read good");
        assert_eq!(formatted, expected, "{rule}: fmt on bad.rs must produce good.rs");

        let again = app
            .format(&Scope::Paths(vec![target]), FormatMode::Check)
            .expect("second fmt runs");
        assert!(again.changed.is_empty(), "{rule}: fmt is not idempotent");
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_deliberate_comment_silences_check_and_fmt() {
    let dir = std::env::temp_dir().join(format!("rabot-deliberate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("lib.rs");
    let source = "// Deliberately unsorted: the guard must release before the pool.\npub struct Connection {\n    guard: u8,\n    pool: u8,\n}\n\npub struct Plain {\n    b: u8,\n    a: u8,\n}\n";
    std::fs::write(&target, source).expect("write");

    let app = App::new(Config::default(), dir.clone());
    let outcome = app
        .check(&Scope::Paths(vec![target.clone()]))
        .expect("check runs");
    let lines: Vec<(Rule, usize)> = outcome
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule, diagnostic.position.line))
        .collect();
    assert_eq!(
        lines,
        vec![(Rule::SortedFields, 7)],
        "only the undocumented struct"
    );

    app.format(&Scope::Paths(vec![target.clone()]), FormatMode::Write)
        .expect("fmt runs");
    let formatted = std::fs::read_to_string(&target).expect("read");
    assert!(formatted.contains("guard: u8,\n    pool: u8,"), "{formatted}");
    assert!(formatted.contains("a: u8,\n    b: u8,"), "{formatted}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn blank_lines_group_fields_variants_and_literals() {
    let dir = std::env::temp_dir().join(format!("rabot-groups-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("lib.rs");
    let source = r#"pub struct Person {
    first_name: u8,
    last_name: u8,

    age: u8,
    birth_date: u8,
}

pub struct Contact {
    // Identity
    last_name: u8,
    first_name: u8,

    // Address
    street: u8,
    // Where the post goes.
    city: u8,
}

pub struct Spaced {
    /// Second.
    beta: u8,

    /// First.
    alpha: u8,
}

pub enum Shape {
    Square,
    Circle,

    Line,
}

impl Person {
    const B: u8 = 0;
    const A: u8 = 0;

    pub fn zeta(&self) -> Person {
        Person {
            last_name: 0,
            first_name: 0,

            age: 0,
            birth_date: 0,
        }
    }

    pub fn alpha(&self) {}
}
"#;
    std::fs::write(&target, source).expect("write");

    let app = App::new(Config::default(), dir.clone());
    let outcome = app
        .check(&Scope::Paths(vec![target.clone()]))
        .expect("check runs");
    let mut found: Vec<(Rule, usize, &str)> = outcome
        .diagnostics
        .iter()
        .map(|d| (d.rule, d.position.line, d.message.as_str()))
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            (
                Rule::SortedFields,
                9,
                "fields of `Contact`: `first_name` should come before `last_name` (alphabetical order within its group)"
            ),
            (
                Rule::SortedFields,
                20,
                "fields of `Spaced`: `alpha` should come before `beta` (alphabetical order)"
            ),
            (
                Rule::SortedImplItems,
                35,
                "`impl Person`: `A` should come before `B` (alphabetical order)"
            ),
            (
                Rule::SortedStructLiteral,
                40,
                "fields of `Person { .. }`: `first_name` should come before `last_name` (alphabetical order within its group)"
            ),
            (
                Rule::SortedVariants,
                28,
                "variants of `Shape`: `Circle` should come before `Square` (alphabetical order within its group)"
            ),
        ]
    );

    app.format(&Scope::Paths(vec![target.clone()]), FormatMode::Write)
        .expect("fmt runs");
    let formatted = std::fs::read_to_string(&target).expect("read");
    let expected = r#"pub struct Person {
    first_name: u8,
    last_name: u8,

    age: u8,
    birth_date: u8,
}

pub struct Contact {
    // Identity
    first_name: u8,
    last_name: u8,

    // Address
    // Where the post goes.
    city: u8,
    street: u8,
}

pub struct Spaced {
    /// First.
    alpha: u8,

    /// Second.
    beta: u8,
}

pub enum Shape {
    Circle,
    Square,

    Line,
}

impl Person {
    const A: u8 = 0;
    const B: u8 = 0;

    pub fn alpha(&self) {}

    pub fn zeta(&self) -> Person {
        Person {
            first_name: 0,
            last_name: 0,

            age: 0,
            birth_date: 0,
        }
    }
}
"#;
    assert_eq!(formatted, expected);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn order_sensitive_lists_are_left_alone() {
    let dir = std::env::temp_dir().join(format!("rabot-order-sensitive-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("lib.rs");
    let source = r#"#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(PartialEq, PartialOrd)]
pub enum Shape {
    Square { side: u8, at: u8 },
    Circle { radius: u8, at: u8 },
}

#[repr(u8)]
pub enum Tagged {
    B { y: u8, x: u8 },
    A,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Float(f64),
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Opened,
    Closed,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(clap::Parser)]
pub struct Copy {
    source: String,
    destination: String,
}

#[cfg_attr(feature = "ffi", derive(Debug), cfg_attr(unix, derive(uniffi::Record)))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Gated {
    Second { b: u8, a: u8 },
    First,
}

#[derive(uniffi::Record)]
pub struct Person {
    last_name: String,
    first_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    Io,
    Bad,
}

#[derive(Record)]
pub struct Row {
    b: u8,
    a: u8,
}
"#;
    std::fs::write(&target, source).expect("write");

    let lines = |config: Config| -> Vec<(Rule, usize)> {
        let outcome = App::new(config, dir.clone())
            .check(&Scope::Paths(vec![target.clone()]))
            .expect("check runs");
        outcome
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule, diagnostic.position.line))
            .collect()
    };
    assert_eq!(
        lines(Config::default()),
        vec![(Rule::SortedVariants, 56), (Rule::SortedFields, 62)],
        "only `thiserror::Error` (not `uniffi::Error`) and the bare `Record`"
    );
    let mut config = Config::default();
    config.sorting.order_sensitive_derives.push("Record".to_string());
    assert_eq!(lines(config), vec![(Rule::SortedVariants, 56)]);

    App::new(Config::default(), dir.clone())
        .format(&Scope::Paths(vec![target.clone()]), FormatMode::Write)
        .expect("fmt runs");
    let formatted = std::fs::read_to_string(&target).expect("read");
    let untouched = source
        .replace("    Io,\n    Bad,", "    Bad,\n    Io,")
        .replace("    b: u8,\n    a: u8,", "    a: u8,\n    b: u8,");
    assert_eq!(formatted, untouched);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn packages_narrow_a_workspace_run() {
    let dir = std::env::temp_dir().join(format!("rabot-workspace-{}", std::process::id()));
    let write = |relative: &str, text: &str| {
        let path = dir.join(relative);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(path, text).expect("write");
    };
    write(
        "Cargo.toml",
        "[package]\nname = \"app\"\n\n[workspace]\nmembers = [\"crates/*\"]\n",
    );
    write("src/main.rs", "pub struct A { b: u8, a: u8 }\n");
    write("crates/core/Cargo.toml", "[package]\nname = \"core\"\n");
    write("crates/core/src/lib.rs", "pub struct B { b: u8, a: u8 }\n");
    write("crates/cli/Cargo.toml", "[package]\nname = \"cli\"\n");
    write("crates/cli/src/lib.rs", "pub struct C { b: u8, a: u8 }\n");

    let files_of = |packages: &[&str]| {
        let app = App::new(Config::default(), dir.clone())
            .with_packages(packages.iter().map(|name| name.to_string()).collect());
        let outcome = app.check(&Scope::Paths(Vec::new())).expect("check runs");
        let mut files: Vec<PathBuf> = outcome.diagnostics.iter().map(|d| d.path.clone()).collect();
        files.sort();
        files.dedup();
        files
    };
    assert_eq!(files_of(&["core"]), vec![dir.join("crates/core/src/lib.rs")]);
    assert_eq!(
        files_of(&["app"]),
        vec![dir.join("src/main.rs")],
        "the root package does not own its members' files"
    );
    assert_eq!(files_of(&["cli", "core"]).len(), 2);
    assert_eq!(files_of(&[]).len(), 3);

    let explicit = App::new(Config::default(), dir.clone())
        .with_packages(vec!["core".to_string()])
        .check(&Scope::Paths(vec![dir.join("crates")]))
        .expect("check runs");
    assert_eq!(explicit.files_seen, 1, "explicit paths are narrowed too");

    let unknown = App::new(Config::default(), dir.clone())
        .with_packages(vec!["nope".to_string()])
        .check(&Scope::Paths(Vec::new()));
    assert!(unknown.is_err());
    std::fs::remove_dir_all(&dir).ok();
}
