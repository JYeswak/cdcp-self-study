//! Product CLI for `cdcp slo` (`bd-extract-smoke-slo-python-l5ke`).
//!
//! `slo budgets` prints three integers from `[budgets]`. `slo now-ms`
//! prints one epoch-ms integer. A missing table, a missing key, or a
//! non-integer wall is non-zero. Root-level keys are not a fallback.

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve course-engine workspace root")
}

fn cdcp() -> Command {
    let mut cmd = Command::cargo_bin("cdcp").expect("cdcp binary");
    cmd.current_dir(workspace_root());
    cmd
}

fn combined(assert: &assert_cmd::assert::Assert) -> String {
    let out = assert.get_output();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn uniq(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cdcp_cli_slo_{tag}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn assigned_u64(text: &str, key: &str) -> u64 {
    let prefix = format!("{key} = ");
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("{key} is not a u64: {rest}"));
        }
    }
    panic!("slo.toml has no `{key} =` line");
}

const SLO_ABOUT: &str = "Parse slo.toml wall budgets";

#[test]
fn help_lists_slo() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(SLO_ABOUT),
        "cdcp --help must name the slo verb: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("slo")),
        "cdcp --help must list the `slo` command: {stdout}"
    );
}

#[test]
fn slo_help_lists_budgets_and_now_ms() {
    let assert = cdcp().args(["slo", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in ["budgets", "now-ms"] {
        assert!(
            stdout.contains(verb),
            "cdcp slo --help must list {verb}: {stdout}"
        );
    }
}

#[test]
fn live_slo_toml_prints_the_three_walls() {
    let live = fs::read_to_string(workspace_root().join("slo.toml")).expect("read live slo.toml");
    let want = [
        assigned_u64(&live, "grade_ms"),
        assigned_u64(&live, "export_ms"),
        assigned_u64(&live, "bank_verify_ms"),
    ];
    let assert = cdcp()
        .args(["slo", "budgets", "--file", "slo.toml"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines,
        [
            want[0].to_string().as_str(),
            want[1].to_string().as_str(),
            want[2].to_string().as_str()
        ],
        "stdout must be exactly three integer lines: {stdout:?}"
    );
}

#[test]
fn missing_file_is_nonzero() {
    let assert = cdcp()
        .args(["slo", "budgets", "--file", "does-not-exist-slo.toml"])
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains("slo budgets: read"),
        "missing file must name the read: {out}"
    );
}

#[test]
fn planted_missing_table_is_red() {
    let dir = uniq("no_table");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("slo.toml");
    fs::write(&path, "schema_version = 1\n").unwrap();
    let assert = cdcp()
        .args(["slo", "budgets", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        out.contains("missing [budgets]"),
        "root-only document must RED: {out}"
    );
}

#[test]
fn planted_root_level_keys_are_not_a_fallback() {
    let dir = uniq("root_fallback");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("slo.toml");
    fs::write(&path, "grade_ms = 1\nexport_ms = 2\nbank_verify_ms = 3\n").unwrap();
    let assert = cdcp()
        .args(["slo", "budgets", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        out.contains("missing [budgets]"),
        "retired python fallback must stay gone: {out}"
    );
}

#[test]
fn planted_empty_file_is_red() {
    let dir = uniq("empty");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("slo.toml");
    fs::write(&path, "").unwrap();
    let assert = cdcp()
        .args(["slo", "budgets", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        out.contains("empty document"),
        "0-byte slo file must RED: {out}"
    );
}

#[test]
fn planted_missing_key_is_red() {
    let dir = uniq("missing_key");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("slo.toml");
    fs::write(&path, "[budgets]\ngrade_ms = 1\nexport_ms = 2\n").unwrap();
    let assert = cdcp()
        .args(["slo", "budgets", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        out.contains("missing budget key"),
        "missing wall must name the hole: {out}"
    );
    assert!(
        out.contains("bank_verify_ms"),
        "missing wall must name the key: {out}"
    );
}

#[test]
fn planted_negative_budget_is_red() {
    let dir = uniq("neg");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("slo.toml");
    fs::write(
        &path,
        "[budgets]\ngrade_ms = -5\nexport_ms = 2\nbank_verify_ms = 3\n",
    )
    .unwrap();
    let assert = cdcp()
        .args(["slo", "budgets", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    let _ = fs::remove_dir_all(&dir);
    assert!(out.contains("grade_ms"), "must name the key: {out}");
    assert!(out.contains(">= 0"), "must refuse a negative: {out}");
}

#[test]
fn now_ms_prints_one_recent_integer() {
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let assert = cdcp().args(["slo", "now-ms"]).assert().success();
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "now-ms must print one line: {stdout:?}");
    let got: u128 = lines[0]
        .parse()
        .unwrap_or_else(|_| panic!("now-ms is not an integer: {stdout:?}"));
    assert!(got >= before, "got {got} < before {before}");
    assert!(got <= after, "got {got} > after {after}");
}

/// Meta: delete the verb or the planted RED tokens → this file is non-zero.
#[test]
fn slo_source_looks_up_budgets_and_has_no_python() {
    let src = include_str!("../src/slo.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(
        src.contains("doc.get(\"budgets\")"),
        "delete the [budgets] lookup → selftest non-zero"
    );
    assert!(
        src.contains("REQUIRED_BUDGET_KEYS"),
        "delete the required-keys list → selftest non-zero"
    );
    assert!(
        !src.contains("python3"),
        "slo.rs production must not mention python3"
    );
}
