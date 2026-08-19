//! Fail-closed proof that load-bearing Rust tests still exist and ran.
//!
//! This is a process/observability gate, not a claim about the quality of the
//! assertions inside the named tests. It measures the current checkout
//! (`tree=worktree`) through Cargo; it does not inspect or certify the Git
//! index. A required test name missing from a successful suite is a violation,
//! a filtered suite is an evaluation error, and an empty or malformed registry
//! is a schema error.

#![forbid(unsafe_code)]

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;

pub const NAME: &str = "required-tests";
pub const SUMMARY: &str = "required test identities ran unfiltered in the worktree";
pub const POLICY: &str = "registries/required_tests.toml";

const BANK_LIB_SUITE: &str = "cdcp_bank-lib";
const ANKI_EXPORT_SUITE: &str = "cdcp_anki-export";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eval {
    Ok(String),
    Violation(Vec<String>),
    Error(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryFile {
    #[serde(rename = "test", default)]
    tests: Vec<RawRequiredTest>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequiredTest {
    name: Option<String>,
    suite: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequiredTest {
    name: String,
    suite: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuiteOutput {
    code: i32,
    stdout: String,
    stderr: String,
}

fn schema_error(message: impl Into<String>) -> String {
    format!("SCHEMA ERROR: {}", message.into())
}

fn load_registry(root: &Path) -> Result<Vec<RequiredTest>, String> {
    let path = root.join(POLICY);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let parsed: RegistryFile = toml::from_str(&text)
        .map_err(|e| schema_error(format!("{} is invalid TOML: {e}", path.display())))?;
    if parsed.tests.is_empty() {
        return Err(schema_error(format!(
            "{} has no [[test]] rows",
            path.display()
        )));
    }

    let mut seen = BTreeSet::new();
    let mut tests = Vec::with_capacity(parsed.tests.len());
    for (index, raw) in parsed.tests.into_iter().enumerate() {
        let row = index + 1;
        let name = raw
            .name
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| schema_error(format!("[[test]] row {row} has no name")))?;
        if name.chars().any(char::is_whitespace) {
            return Err(schema_error(format!(
                "[[test]] row {row} name {name:?} contains whitespace"
            )));
        }

        let suite = raw
            .suite
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| schema_error(format!("[[test]] row {row} has no suite")))?;
        if suite_args(&suite).is_none() {
            return Err(schema_error(format!(
                "[[test]] row {row} names unknown suite {suite:?}"
            )));
        }

        let reason = raw
            .reason
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                schema_error(format!(
                    "[[test]] row {row} ({name}) has a missing or empty reason"
                ))
            })?;

        let key = (suite.clone(), name.clone());
        if !seen.insert(key) {
            return Err(schema_error(format!(
                "duplicate required test {name:?} in suite {suite:?}"
            )));
        }
        tests.push(RequiredTest {
            name,
            suite,
            reason,
        });
    }
    Ok(tests)
}

fn suite_args(suite: &str) -> Option<Vec<&'static str>> {
    match suite {
        BANK_LIB_SUITE => Some(vec!["test", "--locked", "-p", "cdcp_bank", "--lib"]),
        ANKI_EXPORT_SUITE => Some(vec![
            "test",
            "--locked",
            "-p",
            "cdcp_anki",
            "--test",
            "export",
        ]),
        _ => None,
    }
}

fn run_suite(root: &Path, suite: &str) -> Result<SuiteOutput, String> {
    let args = suite_args(suite).ok_or_else(|| format!("unknown suite {suite:?}"))?;
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let output = Command::new(cargo)
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run cargo for suite {suite:?}: {e}"))?;
    Ok(SuiteOutput {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn suite_output(output: &SuiteOutput) -> String {
    let mut text = output.stdout.clone();
    if !output.stderr.is_empty() {
        text.push('\n');
        text.push_str(&output.stderr);
    }
    text
}

fn has_unfiltered_summary(output: &SuiteOutput) -> bool {
    let text = suite_output(output);
    let summaries: Vec<&str> = text
        .lines()
        .filter(|line| line.contains("filtered out"))
        .collect();
    !summaries.is_empty() && summaries.iter().all(|line| line.contains("0 filtered out"))
}

fn evaluate_with_runner<F>(root: &Path, runner: F) -> Eval
where
    F: Fn(&Path, &str) -> Result<SuiteOutput, String>,
{
    let tests = match load_registry(root) {
        Ok(tests) => tests,
        Err(message) => return Eval::Error(format!("tree=worktree {message}")),
    };

    let mut by_suite: BTreeMap<String, Vec<RequiredTest>> = BTreeMap::new();
    for test in tests {
        by_suite.entry(test.suite.clone()).or_default().push(test);
    }

    let mut violations = Vec::new();
    let mut reports = Vec::new();
    for (suite, required) in &by_suite {
        let output = match runner(root, suite) {
            Ok(output) => output,
            Err(message) => {
                return Eval::Error(format!(
                    "tree=worktree suite={suite} could not be evaluated: {message}"
                ));
            }
        };
        if !has_unfiltered_summary(&output) {
            return Eval::Error(format!(
                "tree=worktree suite={suite} did not report a clean `0 filtered out` summary; filtered runs are an ERROR"
            ));
        }
        let text = suite_output(&output);
        reports.push(format!("{suite}:required={}", required.len()));
        if output.code != 0 {
            violations.push(format!(
                "tree=worktree suite={suite} exited {} (the suite is RED)",
                output.code
            ));
        }
        for test in required {
            let needle = format!("test {} ... ok", test.name);
            if !text.contains(&needle) {
                violations.push(format!(
                    "tree=worktree suite={suite} missing required test {:?}; expected output {needle:?}; reason={}",
                    test.name, test.reason
                ));
            }
        }
    }

    if !violations.is_empty() {
        return Eval::Violation(violations);
    }

    Eval::Ok(format!(
        "{NAME}: PASS: tree=worktree (current checkout, not Git index); suites={}; required={}; every suite reported `0 filtered out`; {}",
        by_suite.len(),
        by_suite.values().map(Vec::len).sum::<usize>(),
        reports.join(" ")
    ))
}

pub fn evaluate(root: &Path) -> Eval {
    evaluate_with_runner(root, run_suite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const ONE_TEST: &str = r#"
[[test]]
name = "required_test"
suite = "cdcp_bank-lib"
reason = "This test is the load-bearing known-good leg."
"#;

    fn fixture(policy: &str) -> TempDir {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("registries")).unwrap();
        fs::write(temp.path().join(POLICY), policy).unwrap();
        temp
    }

    fn output(code: i32, text: &str) -> SuiteOutput {
        SuiteOutput {
            code,
            stdout: text.to_string(),
            stderr: String::new(),
        }
    }

    #[test]
    fn required_registry_passes_when_named_test_is_present_and_unfiltered() {
        let temp = fixture(ONE_TEST);
        let result = evaluate_with_runner(temp.path(), |_, _| {
            Ok(output(
                0,
                "test required_test ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
            ))
        });
        assert!(matches!(result, Eval::Ok(_)), "{result:?}");
    }

    #[test]
    fn deleting_a_registry_named_test_is_red_and_names_it() {
        let temp = fixture(ONE_TEST);
        let result = evaluate_with_runner(temp.path(), |_, _| {
            Ok(output(
                0,
                "test some_other_test ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
            ))
        });
        match result {
            Eval::Violation(items) => assert!(items[0].contains("required_test"), "{items:?}"),
            other => panic!("missing registry test was not RED: {other:?}"),
        }
    }

    #[test]
    fn a_filtered_suite_is_an_error_even_if_the_named_test_line_is_present() {
        let temp = fixture(ONE_TEST);
        let result = evaluate_with_runner(temp.path(), |_, _| {
            Ok(output(
                0,
                "test required_test ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\n",
            ))
        });
        match result {
            Eval::Error(message) => {
                assert!(message.contains("0 filtered out"), "{message}");
                assert!(message.contains("filtered runs are an ERROR"), "{message}");
            }
            other => panic!("filtered suite was not an ERROR: {other:?}"),
        }
    }

    #[test]
    fn a_filter_that_matches_nothing_is_an_error() {
        let temp = fixture(ONE_TEST);
        let result = evaluate_with_runner(temp.path(), |_, _| {
            Ok(output(
                0,
                "running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\n",
            ))
        });
        match result {
            Eval::Error(message) => {
                assert!(message.contains("0 filtered out"), "{message}");
                assert!(message.contains("filtered runs are an ERROR"), "{message}");
            }
            other => panic!("empty filtered suite was not an ERROR: {other:?}"),
        }
    }

    #[test]
    fn empty_reason_is_a_schema_error() {
        let temp = fixture(
            r#"
[[test]]
name = "required_test"
suite = "cdcp_bank-lib"
reason = ""
"#,
        );
        let result = evaluate_with_runner(temp.path(), |_, _| {
            panic!("schema error must not run a suite")
        });
        match result {
            Eval::Error(message) => {
                assert!(message.contains("SCHEMA ERROR"), "{message}");
                assert!(message.contains("missing or empty reason"), "{message}");
            }
            other => panic!("empty reason was not a schema error: {other:?}"),
        }
    }

    #[test]
    fn missing_reason_is_a_schema_error() {
        let temp = fixture(
            r#"
[[test]]
name = "required_test"
suite = "cdcp_bank-lib"
"#,
        );
        let result = evaluate_with_runner(temp.path(), |_, _| {
            panic!("schema error must not run a suite")
        });
        match result {
            Eval::Error(message) => {
                assert!(message.contains("SCHEMA ERROR"), "{message}");
                assert!(message.contains("missing or empty reason"), "{message}");
            }
            other => panic!("missing reason was not a schema error: {other:?}"),
        }
    }

    #[test]
    fn empty_registry_is_an_error() {
        let temp = fixture("# intentionally empty\n");
        let result = evaluate_with_runner(temp.path(), |_, _| {
            panic!("empty registry must not run a suite")
        });
        match result {
            Eval::Error(message) => {
                assert!(message.contains("SCHEMA ERROR"), "{message}");
                assert!(message.contains("no [[test]] rows"), "{message}");
            }
            other => panic!("empty registry was not an error: {other:?}"),
        }
    }

    #[test]
    fn suite_commands_have_no_test_filter() {
        for suite in [BANK_LIB_SUITE, ANKI_EXPORT_SUITE] {
            let args = suite_args(suite).unwrap();
            assert!(
                args.iter().all(|arg| *arg != "required_test"),
                "suite command unexpectedly filters: {args:?}"
            );
        }
    }
}
