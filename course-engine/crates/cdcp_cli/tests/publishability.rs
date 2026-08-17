//! Product CLI for `cdcp publishability`
//! (`bd-extract-publishability-bar-python-9tji`).
//!
//! `doctor-errors` prints sorted codes (or DOCTOR_UNPARSEABLE) and stays
//! exit 0. `corpus-rights` is RED on an empty list or a missing rights
//! field. Not a gate: a printed code is not a score.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
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

fn scratch(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "cdcp_cli_publishability_{}_{}_{name}",
        std::process::id(),
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn wipe(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

const ABOUT: &str = "Helpers for tests/publishability-bar.sh";

#[test]
fn help_lists_publishability() {
    let assert = cdcp().env("CDCP_DEV", "1").arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(ABOUT),
        "cdcp --help must name publishability: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("publishability")),
        "cdcp --help must list the `publishability` command: {stdout}"
    );
}

#[test]
fn publishability_help_lists_jobs() {
    let assert = cdcp().args(["publishability", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in ["doctor-errors", "corpus-rights"] {
        assert!(
            stdout.contains(verb),
            "cdcp publishability --help must list {verb}: {stdout}"
        );
    }
}

#[test]
fn planted_two_codes_prints_sorted() {
    let dir = scratch("two_codes");
    let path = dir.join("doctor.json");
    fs::write(
        &path,
        r#"{"errors":[{"code":"z_last"},{"code":"a_first"}]}"#,
    )
    .unwrap();
    let assert = cdcp()
        .args(["publishability", "doctor-errors", "--json"])
        .arg(&path)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    wipe(&dir);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), ["a_first,z_last"]);
}

#[test]
fn planted_unparseable_prints_token_and_stays_zero() {
    let dir = scratch("unparseable");
    let path = dir.join("doctor.json");
    fs::write(&path, "not-json").unwrap();
    let assert = cdcp()
        .args(["publishability", "doctor-errors", "--json"])
        .arg(&path)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    wipe(&dir);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), ["DOCTOR_UNPARSEABLE"]);
}

#[test]
fn missing_doctor_file_prints_token_and_stays_zero() {
    let assert = cdcp()
        .args([
            "publishability",
            "doctor-errors",
            "--json",
            "does-not-exist-doctor.json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), ["DOCTOR_UNPARSEABLE"]);
}

#[test]
fn live_corpus_manifest_passes() {
    let assert = cdcp()
        .args([
            "publishability",
            "corpus-rights",
            "--file",
            "knowledge/corpus/public/manifest.json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("corpus-rights: ok"),
        "live manifest must PASS: {stdout}"
    );
}

#[test]
fn planted_empty_sources_is_red() {
    let dir = scratch("empty_sources");
    let path = dir.join("manifest.json");
    fs::write(&path, r#"{"sources":[]}"#).unwrap();
    let assert = cdcp()
        .args(["publishability", "corpus-rights", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    wipe(&dir);
    assert!(
        out.contains("empty sources"),
        "empty sources must RED: {out}"
    );
}

#[test]
fn planted_missing_rights_is_red() {
    let dir = scratch("missing_rights");
    let path = dir.join("manifest.json");
    fs::write(
        &path,
        r#"{"sources":[{"url":"https://planted.example","title":"x"}]}"#,
    )
    .unwrap();
    let assert = cdcp()
        .args(["publishability", "corpus-rights", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    wipe(&dir);
    assert!(
        out.contains("missing rights"),
        "missing rights must RED: {out}"
    );
    assert!(
        out.contains("https://planted.example"),
        "must name the url: {out}"
    );
}

#[test]
fn planted_rights_present_is_green() {
    let dir = scratch("rights_ok");
    let path = dir.join("manifest.json");
    fs::write(
        &path,
        r#"{"sources":[{"url":"https://ok.example","rights":"publisher-retains-copyright"}]}"#,
    )
    .unwrap();
    let assert = cdcp()
        .args(["publishability", "corpus-rights", "--file"])
        .arg(&path)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    wipe(&dir);
    assert!(
        stdout.contains("corpus-rights: ok (1 sources)"),
        "one sourced plant must PASS: {stdout}"
    );
}

#[test]
fn missing_manifest_is_nonzero() {
    let assert = cdcp()
        .args([
            "publishability",
            "corpus-rights",
            "--file",
            "does-not-exist-manifest.json",
        ])
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains("publishability corpus-rights: read"),
        "missing file must name the read: {out}"
    );
}

/// Meta: delete the verb or put python3 back → this file is non-zero.
#[test]
fn publishability_source_has_no_python() {
    let src = include_str!("../src/publishability.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(
        src.contains("rights_present"),
        "delete the rights predicate → selftest non-zero"
    );
    assert!(
        !src.contains("python3"),
        "publishability.rs production must not mention python3"
    );
    assert!(
        !src.contains("cdcp_gate"),
        "helper must not live in / depend on cdcp_gate"
    );
}

/// Acceptance: no live `python3` left in the bar script.
#[test]
fn publishability_bar_sh_has_no_live_python3() {
    let sh = include_str!("../../../tests/publishability-bar.sh");
    for (i, line) in sh.lines().enumerate() {
        let body = line.trim();
        if body.starts_with('#') {
            continue;
        }
        // Live invocation, not a mention: `python3 -`, `python3 -c`, `if python3`.
        let live = body.contains("python3 -")
            || body.contains("python3 <<")
            || body.starts_with("python3")
            || body.contains("$(python3")
            || body.contains("`python3")
            || body.contains("if python3")
            || body.contains("command -v python3");
        assert!(
            !live,
            "tests/publishability-bar.sh:{} is a live python3: {line}",
            i + 1
        );
    }
    assert!(
        sh.contains("publishability doctor-errors"),
        "bar script must call cdcp publishability doctor-errors"
    );
    assert!(
        sh.contains("publishability corpus-rights"),
        "bar script must call cdcp publishability corpus-rights"
    );
}
