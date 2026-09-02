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
        (Rule::CommentedOutCode, 58),
        (Rule::FreeFunction, 65),
        (Rule::FreeFunction, 67),
        (Rule::FreeFunction, 71),
        (Rule::FreeFunction, 90),
        (Rule::FreeFunction, 100),
        (Rule::GlobalState, 7),
        (Rule::MockUsage, 111),
        (Rule::OrphanModule, 5),
        (Rule::PanicInProduction, 57),
        (Rule::PanicInProduction, 62),
        (Rule::PanicInProduction, 68),
        (Rule::PrimitiveField, 12),
        (Rule::PrimitiveField, 13),
        (Rule::PrimitiveSoup, 56),
        (Rule::SectionedFunction, 100),
        (Rule::SortedDerives, 10),
        (Rule::SortedFields, 11),
        (Rule::SortedFields, 26),
        (Rule::SortedFields, 36),
        (Rule::SortedImplItems, 46),
        (Rule::SortedStructLiteral, 72),
        (Rule::SortedStructPattern, 91),
        (Rule::SortedTraitItems, 81),
        (Rule::SortedVariants, 23),
        (Rule::TooManyParameters, 79),
        (Rule::UndocumentedException, 35),
        (Rule::UnknownRule, 41),
        (Rule::UntypedError, 75),
        (Rule::VagueTodo, 60),
        (Rule::VagueTypeName, 42),
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
fn documented_exceptions_are_honoured() {
    let findings = lint_findings();
    // `Severity` is allowed with a reason; `LOGGER` is infrastructure; the
    // test module may unwrap; `main` may expect.
    assert!(
        !findings
            .iter()
            .any(|(rule, line)| *rule == Rule::SortedVariants && *line == 30)
    );
    assert!(
        !findings
            .iter()
            .any(|(rule, line)| *rule == Rule::GlobalState && *line == 8)
    );
    assert!(
        !findings
            .iter()
            .any(|(rule, line)| *rule == Rule::PanicInProduction && *line > 75)
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
