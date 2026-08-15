//! CLI surface for `cdcp recon` (selftest_reconstructed.sh helpers).
//!
//! Not a gate. Unit tests in `src/recon.rs` cover the predicates. These
//! cover the clap wire: the verb is listed, a planted two-job json-set is
//! RED, an empty watch set is RED.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn cdcp() -> Command {
    Command::cargo_bin("cdcp").expect("cdcp binary")
}

fn scratch(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "cdcp_cli_recon_{}_{}_{name}",
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

fn combined(assert: &assert_cmd::assert::Assert) -> String {
    let out = assert.get_output();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn help_lists_recon() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("recon")),
        "cdcp --help must list recon: {stdout}"
    );
}

#[test]
fn recon_help_lists_jobs() {
    let assert = cdcp().args(["recon", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in [
        "snapshot-live",
        "assert-unmoved",
        "assert-git-unmoved",
        "samefile",
        "archive-head",
        "json-set",
        "newest-bin",
        "mtime-ns",
    ] {
        assert!(
            stdout.contains(verb),
            "cdcp recon --help must list {verb}: {stdout}"
        );
    }
}

#[test]
fn json_set_two_jobs_is_red() {
    let dir = scratch("two_jobs");
    let path = dir.join("pack.json");
    fs::write(&path, "{\"n_items\":40,\"items\":[]}\n").unwrap();
    let assert = cdcp()
        .args(["recon", "json-set", "--file"])
        .arg(&path)
        .args(["--n-items", "39", "--flip-first-key"])
        .assert()
        .failure();
    let err = combined(&assert);
    assert!(err.contains("exactly one"), "{err}");
    wipe(&dir);
}

#[test]
fn json_set_n_items_cli_writes() {
    let dir = scratch("n_items");
    let path = dir.join("pack.json");
    fs::write(&path, "{\"n_items\":40,\"items\":[]}\n").unwrap();
    cdcp()
        .args(["recon", "json-set", "--file"])
        .arg(&path)
        .args(["--n-items", "39"])
        .assert()
        .success();
    let body = fs::read_to_string(&path).unwrap();
    assert!(
        body.contains("\"n_items\": 39") || body.contains("\"n_items\":39"),
        "{body}"
    );
    wipe(&dir);
}

#[test]
fn samefile_self_prints_one() {
    let dir = scratch("same");
    let path = dir.join("a.txt");
    fs::write(&path, "x\n").unwrap();
    let assert = cdcp()
        .args(["recon", "samefile"])
        .arg(&path)
        .arg(&path)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_eq!(stdout.trim(), "1", "{stdout}");
    wipe(&dir);
}

#[test]
fn snapshot_live_empty_watch_is_red() {
    let dir = scratch("empty_watch");
    let assert = cdcp()
        .args(["recon", "snapshot-live", "--root"])
        .arg(&dir)
        .args(["--fp"])
        .arg(dir.join("fp"))
        .args(["--clean-out"])
        .arg(dir.join("clean"))
        .assert()
        .failure();
    let err = combined(&assert);
    assert!(
        err.contains("required") || err.contains("empty watch") || err.contains("FILES"),
        "{err}"
    );
    wipe(&dir);
}

#[test]
fn mtime_ns_prints_integer() {
    let dir = scratch("mtime");
    let path = dir.join("f");
    fs::write(&path, "x").unwrap();
    let assert = cdcp()
        .args(["recon", "mtime-ns"])
        .arg(&path)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let n: u128 = stdout.trim().parse().expect("mtime integer");
    assert!(n > 0, "{stdout}");
    wipe(&dir);
}
