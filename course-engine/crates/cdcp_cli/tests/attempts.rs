//! Product CLI for `cdcp_attempts` (`bd-hardening-l-attempts-bg2.2`).
//!
//! Record / list / export through the crate. Export without `--opt-in`
//! is non-zero. An empty store is an ERROR, never empty JSON. No
//! psychometrics verbs exist on this surface.

use assert_cmd::Command;
use cdcp_attempts::{
    AttemptEvent, AttemptLog, AttemptMode, EMPTY_STORE, EXPORT_NOT_OPTED_IN, JSONL_NAME,
    SQLITE_NAME,
};
use serde_json::Value;
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

fn uniq(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cdcp_cli_attempts_{tag}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

const BANK_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const TS: &str = "1724000000000";

fn record_args(store: &Path) -> Vec<String> {
    vec![
        "attempts".into(),
        "record".into(),
        "--store".into(),
        store.display().to_string(),
        "--item-version".into(),
        "item-v1".into(),
        "--bank-hash".into(),
        BANK_HASH.into(),
        "--learner-pseudonym".into(),
        "learner-aa11".into(),
        "--mode".into(),
        "quiz".into(),
        "--exposure-count".into(),
        "1".into(),
        "--chosen-option".into(),
        "A".into(),
        "--correctness".into(),
        "true".into(),
        "--latency-ms".into(),
        "1500".into(),
        "--timestamp-unix-ms".into(),
        TS.into(),
        "--prior-attempts".into(),
        "0".into(),
    ]
}

fn sample() -> AttemptEvent {
    AttemptEvent::new(
        "item-v1",
        BANK_HASH,
        "learner-aa11",
        AttemptMode::Quiz,
        1,
        "A",
        true,
        1500,
        1_724_000_000_000,
        0,
    )
    .unwrap()
}

#[test]
fn attempts_help_lists_record_list_export() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("no psychometrics"),
        "cdcp --help must say the attempts verb has no psychometrics: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("attempts")),
        "cdcp --help must list the `attempts` command: {stdout}"
    );

    let assert = cdcp().args(["attempts", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in ["record", "list", "export"] {
        assert!(
            stdout.contains(verb),
            "cdcp attempts --help must list {verb}: {stdout}"
        );
    }
}

#[test]
fn attempts_help_has_no_psychometrics_commands() {
    let assert = cdcp().args(["attempts", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for banned in ["irt", "difficulty", "discrimination", "theta", "ability"] {
        let as_command = stdout.lines().any(|l| {
            let tok = l.split_whitespace().next().unwrap_or("");
            tok.eq_ignore_ascii_case(banned)
        });
        assert!(
            !as_command,
            "attempts must not grow a {banned} command: {stdout}"
        );
    }
}

#[test]
fn attempts_unknown_psychometrics_command_is_nonzero() {
    for verb in ["irt", "difficulty", "discrimination"] {
        let assert = cdcp().args(["attempts", verb]).assert().failure();
        let out = combined(&assert);
        assert!(
            !out.contains("event_count") && !out.contains('{'),
            "unknown {verb} must not emit an analysis: {out}"
        );
    }
}

#[test]
fn attempts_record_then_list_roundtrips() {
    let store = uniq("roundtrip");
    let assert = cdcp().args(record_args(&store)).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("recorded") && stdout.contains("item-v1"),
        "record must acknowledge the write: {stdout}"
    );
    assert!(
        store.join(SQLITE_NAME).is_file() && store.join(JSONL_NAME).is_file(),
        "record must create both store files"
    );

    let assert = cdcp()
        .args(["attempts", "list", "--store"])
        .arg(&store)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let line = stdout.trim();
    assert_eq!(line.lines().count(), 1, "one recorded event: {stdout:?}");
    let parsed: AttemptEvent = serde_json::from_str(line).expect("list emits JSONL");
    assert_eq!(parsed, sample());

    let _ = fs::remove_dir_all(&store);
}

#[test]
fn attempts_export_without_opt_in_is_nonzero() {
    let store = uniq("no-opt-in");
    cdcp().args(record_args(&store)).assert().success();

    let assert = cdcp()
        .args(["attempts", "export", "--store"])
        .arg(&store)
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains(EXPORT_NOT_OPTED_IN),
        "export without --opt-in must name the crate token: {out}"
    );
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.trim().is_empty(),
        "refused export must write no JSON: {stdout:?}"
    );

    let _ = fs::remove_dir_all(&store);
}

#[test]
fn attempts_export_empty_store_is_error_not_empty_json() {
    let missing = uniq("missing-export");
    let assert = cdcp()
        .args(["attempts", "export", "--opt-in", "--store"])
        .arg(&missing)
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains(EMPTY_STORE),
        "empty/missing store export must name EMPTY_STORE: {out}"
    );
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains('[') && !stdout.contains('{'),
        "empty store must not emit JSON: {stdout:?}"
    );
    assert!(
        !missing.exists(),
        "export must not create a store as a side effect of a read"
    );

    let empty = uniq("empty-export");
    fs::create_dir_all(&empty).unwrap();
    let _ = AttemptLog::open(&empty).unwrap();
    let assert = cdcp()
        .args(["attempts", "export", "--opt-in", "--store"])
        .arg(&empty)
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains(EMPTY_STORE),
        "opened-but-empty store export must name EMPTY_STORE: {out}"
    );
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_ne!(stdout.trim(), "[]");
    assert!(
        !stdout.contains('{'),
        "empty store must not emit JSON objects: {stdout:?}"
    );

    let _ = fs::remove_dir_all(&empty);
}

#[test]
fn attempts_list_empty_store_is_error_not_empty_json() {
    let missing = uniq("missing-list");
    let assert = cdcp()
        .args(["attempts", "list", "--store"])
        .arg(&missing)
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains(EMPTY_STORE),
        "empty/missing store list must name EMPTY_STORE: {out}"
    );
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains('[') && !stdout.contains('{'),
        "empty list must not emit JSON: {stdout:?}"
    );

    let _ = fs::remove_dir_all(&missing);
}

#[test]
fn attempts_export_with_opt_in_writes_jsonl() {
    let store = uniq("opt-in");
    cdcp().args(record_args(&store)).assert().success();

    let assert = cdcp()
        .args(["attempts", "export", "--opt-in", "--store"])
        .arg(&store)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let parsed: AttemptEvent =
        serde_json::from_str(stdout.trim()).expect("opted-in export emits JSONL");
    assert_eq!(parsed.item_version, "item-v1");
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("event_count=1"),
        "receipt must name the exported N: {stderr}"
    );
    assert!(
        stderr.contains("minimum_n=warning"),
        "N=1 is below the designed-in floor: {stderr}"
    );

    let _ = fs::remove_dir_all(&store);
}

#[test]
fn attempts_doctor_mentions_store() {
    let empty = uniq("doctor");
    fs::create_dir_all(&empty).unwrap();
    let assert = cdcp()
        .args([
            "doctor",
            "--root",
            empty.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains("attempts-store")
            && out.contains("var/attempts")
            && out.contains("export=off"),
        "doctor must mention the attempt store and default-OFF export: {out}"
    );
    assert!(
        out.contains("state=absent"),
        "empty tree has no store: {out}"
    );

    let planted = uniq("doctor-plant");
    fs::create_dir_all(&planted).unwrap();
    let mut log = AttemptLog::open(planted.join("var/attempts")).unwrap();
    log.record(&sample()).unwrap();
    let assert = cdcp()
        .args([
            "doctor",
            "--root",
            planted.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains("state=ready") && out.contains("n=1"),
        "doctor must mention a planted store: {out}"
    );

    let _ = fs::remove_dir_all(&empty);
    let _ = fs::remove_dir_all(&planted);
}

#[test]
fn attempts_health_mentions_store() {
    let assert = cdcp().args(["health", "--robot"]).assert().success();
    let line = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(line.trim()).expect("health --robot JSON");
    let store = v["attempts_store"]
        .as_object()
        .expect("health must mention attempts_store");
    assert_eq!(store["export_policy"].as_str(), Some("off"));
    assert_eq!(store["path"].as_str(), Some("var/attempts"));
    assert!(
        store["state"].as_str().is_some(),
        "attempts_store.state must be present: {line}"
    );
}

/// Meta: delete the crate calls or add a network import → this is non-zero.
#[test]
fn attempts_cli_source_calls_the_crate_and_has_no_network() {
    let src = include_str!("../src/attempts.rs");
    for needle in [
        "AttemptLog",
        "ExportPolicy",
        "EMPTY_STORE",
        "EXPORT_NOT_OPTED_IN",
        "export_jsonl",
    ] {
        assert!(
            src.contains(needle),
            "delete {needle} from the attempts CLI → selftest non-zero"
        );
    }
    for banned in [
        "estimate_item_response_model",
        "compute_item_difficulty",
        "compute_item_discrimination",
    ] {
        assert!(!src.contains(banned), "attempts CLI must not call {banned}");
    }
    for needle in [
        "TcpStream",
        "UdpSocket",
        "TcpListener",
        "std::net",
        "::net::",
        "reqwest",
        "ureq",
        "hyper::",
    ] {
        assert!(
            !src.contains(needle),
            "attempts CLI must not mention {needle}"
        );
    }
}
