//! End-to-end checks against the fixtures: every rule fires where expected,
//! `fmt` produces the golden output, and a second `fmt` changes nothing.

use std::path::{Path, PathBuf};

use rabot::app::{App, FormatMode};
use rabot::config::Config;
use rabot::rule::Rule;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn lint_findings() -> Vec<(Rule, usize)> {
    let root = fixtures().join("lint");
    let app = App::new(Config::default(), root.clone());
    let outcome = app.check(&[root.join("src")]).expect("check runs");
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
        (Rule::CommentedOutCode, 51),
        (Rule::FreeFunction, 58),
        (Rule::FreeFunction, 60),
        (Rule::FreeFunction, 64),
        (Rule::GlobalState, 7),
        (Rule::MockUsage, 85),
        (Rule::OrphanModule, 5),
        (Rule::PanicInProduction, 50),
        (Rule::PanicInProduction, 55),
        (Rule::PanicInProduction, 61),
        (Rule::PrimitiveSoup, 49),
        (Rule::SortedDerives, 10),
        (Rule::SortedFields, 11),
        (Rule::SortedFields, 19),
        (Rule::SortedFields, 29),
        (Rule::SortedImplItems, 39),
        (Rule::SortedStructLiteral, 65),
        (Rule::SortedTraitItems, 74),
        (Rule::SortedVariants, 16),
        (Rule::TooManyParameters, 72),
        (Rule::UndocumentedException, 28),
        (Rule::UnknownRule, 34),
        (Rule::UntypedError, 68),
        (Rule::VagueTodo, 53),
        (Rule::VagueTypeName, 35),
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
            .any(|(rule, line)| *rule == Rule::SortedVariants && *line == 23)
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
        .format(std::slice::from_ref(&target), FormatMode::Write)
        .expect("fmt runs");
    assert_eq!(outcome.changed, vec![target.clone()]);
    let formatted = std::fs::read_to_string(&target).expect("read result");
    let expected = std::fs::read_to_string(fixtures().join("fmt/after.rs")).expect("read golden");
    assert_eq!(formatted, expected);

    let again = app
        .format(std::slice::from_ref(&target), FormatMode::Check)
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
        .format(std::slice::from_ref(&target), FormatMode::Check)
        .expect("fmt runs");
    assert_eq!(outcome.changed, vec![target.clone()]);
    assert!(
        outcome
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule.fixable())
    );
    assert_eq!(std::fs::read_to_string(&target).expect("read"), before);
    std::fs::remove_dir_all(&dir).ok();
}
