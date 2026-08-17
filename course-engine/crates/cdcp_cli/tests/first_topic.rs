//! Product CLI for `cdcp first-topic-id`
//! (`bd-extract-orphan-topic-python-9lj5`).
//!
//! Prints the first `id = "..."` from a topics.toml. A missing file,
//! an empty file, or a document with zero matches is non-zero. Not a
//! gate: a printed id is not proof the topic is assessed.

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
        "cdcp_cli_first_topic_{}_{}_{name}",
        std::process::id(),
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir.join("topics.toml")
}

fn wipe(path: &Path) {
    if let Some(dir) = path.parent() {
        let _ = fs::remove_dir_all(dir);
    }
}

const ABOUT: &str = "Print the first topic id";

#[test]
fn help_lists_first_topic_id() {
    let assert = cdcp().env("CDCP_DEV", "1").arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(ABOUT),
        "cdcp --help must name first-topic-id: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("first-topic-id")),
        "cdcp --help must list the `first-topic-id` command: {stdout}"
    );
}

#[test]
fn live_topics_toml_prints_the_first_id() {
    let assert = cdcp()
        .args(["first-topic-id", "--file", "knowledge/topics.toml"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines,
        ["m01-business-org"],
        "stdout must be exactly the first live topic id: {stdout:?}"
    );
}

#[test]
fn default_file_is_knowledge_topics_toml() {
    let assert = cdcp().arg("first-topic-id").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), ["m01-business-org"]);
}

#[test]
fn planted_two_ids_prints_the_first() {
    let path = scratch("two");
    fs::write(&path, "id = \"alpha\"\nid = \"omega\"\n").unwrap();
    let assert = cdcp()
        .args(["first-topic-id", "--file"])
        .arg(&path)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    wipe(&path);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), ["alpha"]);
}

#[test]
fn missing_file_is_nonzero() {
    let assert = cdcp()
        .args(["first-topic-id", "--file", "does-not-exist-topics.toml"])
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains("first-topic-id: read"),
        "missing file must name the read: {out}"
    );
}

#[test]
fn planted_empty_file_is_red() {
    let path = scratch("empty");
    fs::write(&path, "").unwrap();
    let assert = cdcp()
        .args(["first-topic-id", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    wipe(&path);
    assert!(
        out.contains("empty document"),
        "0-byte topics file must RED: {out}"
    );
}

#[test]
fn planted_schema_only_is_red() {
    let path = scratch("schema");
    fs::write(&path, "schema_version = 1\n").unwrap();
    let assert = cdcp()
        .args(["first-topic-id", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    wipe(&path);
    assert!(
        out.contains("no topic id"),
        "schema-only document must RED: {out}"
    );
}

#[test]
fn planted_topic_id_key_is_not_a_fallback() {
    let path = scratch("topic_id");
    fs::write(&path, "topic_id = \"not-this\"\n").unwrap();
    let assert = cdcp()
        .args(["first-topic-id", "--file"])
        .arg(&path)
        .assert()
        .failure();
    let out = combined(&assert);
    wipe(&path);
    assert!(out.contains("no topic id"), "topic_id is not id: {out}");
}

/// Meta: delete the verb → this file is non-zero.
#[test]
fn first_topic_source_has_no_python() {
    let src = include_str!("../src/first_topic.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(
        src.contains("match_id_at"),
        "delete the id scanner → selftest non-zero"
    );
    assert!(
        !src.contains("python3"),
        "first_topic.rs production must not mention python3"
    );
    assert!(
        !src.contains("cdcp_gate"),
        "helper must not live in / depend on cdcp_gate"
    );
}
