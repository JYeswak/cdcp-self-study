//! CLI surface for `cdcp snap-rewrite` (check.sh CHARTER helpers).
//!
//! Not a gate. The unit tests in `src/snap_rewrite.rs` cover the rewrite
//! predicates. These cover the clap wire: the verb is listed, a planted
//! two-hit file is RED, an unknown kind is RED.

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
        "cdcp_cli_snap_rewrite_{}_{}_{name}",
        std::process::id(),
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir.join("body.txt")
}

fn wipe(path: &Path) {
    if let Some(dir) = path.parent() {
        let _ = fs::remove_dir_all(dir);
    }
}

#[test]
fn help_lists_snap_rewrite() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("snap-rewrite")),
        "cdcp --help must list snap-rewrite: {stdout}"
    );
}

#[test]
fn snap_rewrite_help_lists_both_jobs() {
    let assert = cdcp().args(["snap-rewrite", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in ["replace-once", "charter"] {
        assert!(
            stdout.contains(verb),
            "cdcp snap-rewrite --help must list {verb}: {stdout}"
        );
    }
}

#[test]
fn charter_help_lists_weaken_if() {
    let assert = cdcp()
        .args(["snap-rewrite", "charter", "--help"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for kind in ["skip-exec", "delete-assert", "weaken-if"] {
        assert!(
            stdout.contains(kind),
            "cdcp snap-rewrite charter --help must list {kind}: {stdout}"
        );
    }
}

#[test]
fn replace_once_cli_swaps_the_single_needle() {
    let path = scratch("cli_ok");
    fs::write(&path, "keep NEEDLE keep\n").unwrap();
    cdcp()
        .args(["snap-rewrite", "replace-once", "--file"])
        .arg(&path)
        .args(["--from", "NEEDLE", "--to", "DONE"])
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&path).unwrap(), "keep DONE keep\n");
    wipe(&path);
}

#[test]
fn replace_once_two_needles_is_red_and_does_not_write() {
    let path = scratch("cli_two");
    fs::write(&path, "NEEDLE NEEDLE\n").unwrap();
    let assert = cdcp()
        .args(["snap-rewrite", "replace-once", "--file"])
        .arg(&path)
        .args(["--from", "NEEDLE", "--to", "DONE"])
        .assert()
        .failure();
    let err = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(err.contains("2 time"), "{err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "NEEDLE NEEDLE\n");
    wipe(&path);
}

#[test]
fn charter_unknown_kind_is_red() {
    let assert = cdcp()
        .args([
            "snap-rewrite",
            "charter",
            "--file",
            "/tmp/unused",
            "--kind",
            "hollow-assert",
        ])
        .assert()
        .failure();
    let err = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        err.contains("skip-exec") && err.contains("delete-assert") && err.contains("weaken-if"),
        "unknown kind must name the allowed values: {err}"
    );
}

#[test]
fn weaken_if_cli_rewrites_the_next_test() {
    let path = scratch("cli_weaken");
    let mark = format!("{}{}", "CHARTER-NEEDLE", "-CHECK");
    fs::write(
        &path,
        format!("# {mark}\nif [ 1 -eq 1 ]; then\necho stay\n"),
    )
    .unwrap();
    cdcp()
        .args(["snap-rewrite", "charter", "--file"])
        .arg(&path)
        .args(["--kind", "weaken-if"])
        .assert()
        .success();
    let got = fs::read_to_string(&path).unwrap();
    assert!(got.contains("if false && [ 1 -eq 1 ]; then"), "{got}");
    assert!(got.contains("echo stay"), "{got}");
    wipe(&path);
}
