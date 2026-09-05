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
