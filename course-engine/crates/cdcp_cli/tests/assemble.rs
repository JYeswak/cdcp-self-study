//! Live CLI assemble path (`cdcp assemble`).
//!
//! Bead: `bd-hardening-g-assess-64t.3`.
//!
//! 64t.2 landed `AssembleError::NotLetterMcq` on `assemble_input`. This suite
//! is the wire: a planted multi-select through the CLI assemble entry is RED
//! (named error, no four shuffled strings). Letter-MCQ assemble still works.
//!
//! FLOOR-RAISE: this cannot decide that a letter item is any good, and it
//! does not migrate the 804-item bank or re-freeze C2 goldens.

use assert_cmd::Command;
use serde_json::{json, Value};
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

fn uniq(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cdcp_assemble_{tag}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ))
}

/// Distinctive plant options — if these leak as four shuffled strings, flatten.
const PLANT_OPTIONS: [&str; 4] = [
    "PLANT_MS_ALPHA",
    "PLANT_MS_BRAVO",
    "PLANT_MS_CHARLIE",
    "PLANT_MS_DELTA",
];

fn write_assess(tag: &str, rows: Value) -> (PathBuf, PathBuf) {
    let dir = uniq(tag);
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("assess.json");
    fs::write(&path, serde_json::to_string_pretty(&rows).unwrap()).expect("write assess");
    (dir, path)
}

#[test]
fn help_lists_assemble_and_assess_flag() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("assemble"),
        "cdcp --help must list assemble: {stdout}"
    );
    let assert = cdcp().args(["assemble", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("--assess"),
        "cdcp assemble --help must list --assess: {stdout}"
    );
}

/// GOOD control: live bank assemble still produces the sampler's seed-42 ids.
/// Without this, the plant below could pass because `cdcp assemble` always fails.
#[test]
fn assemble_live_bank_seed42_item_ids_match_sampler() {
    let assert = cdcp()
        .args(["assemble", "--bank", "bank/items", "--seed", "42"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let exam: Value = serde_json::from_str(&stdout).expect("assemble stdout is exam JSON");
    let ids: Vec<String> = exam["item_ids"]
        .as_array()
        .expect("item_ids")
        .iter()
        .map(|v| v.as_str().expect("id").to_string())
        .collect();
    assert_eq!(
        ids.len(),
        40,
        "letter assemble must still be a 40-item form"
    );

    let bank = cdcp_bank::Bank::load_dir(&workspace_root().join("bank/items")).expect("load bank");
    let cfg = cdcp_assemble::AssembleConfig::default();
    let sampled = cdcp_assemble::assemble(&bank, 42, cfg).expect("sampler");
    assert_eq!(
        ids, sampled.item_ids,
        "cdcp assemble must be the same entry as cdcp_assemble::assemble"
    );
}

/// KNOWN-BAD: a valid multi-select through the CLI assemble entry is RED.
/// Flattening it would emit the four plant option strings as shuffled choices.
#[test]
fn assemble_planted_multi_select_is_red_not_flattened() {
    let plant = json!([{
        "id": "plant-ms",
        "module": 1,
        "stem": "select all that apply",
        "item": {
            "kind": "multi-select",
            "options": PLANT_OPTIONS,
            "correct": ["PLANT_MS_ALPHA", "PLANT_MS_CHARLIE"],
            "credit": "all-or-nothing"
        }
    }]);
    let (dir, assess) = write_assess("ms", plant);
    let out = dir.join("form.json");

    let assert = cdcp()
        .args([
            "assemble",
            "--bank",
            "bank/items",
            "--seed",
            "42",
            "--assess",
            assess.to_str().expect("utf-8"),
            "--out",
            out.to_str().expect("utf-8"),
        ])
        .assert()
        .failure();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("multi-select")
            && combined.contains("will not flatten")
            && combined.contains("A–D")
            && combined.contains("plant-ms"),
        "named NotLetterMcq must reach the CLI, got: {combined}"
    );
    for opt in PLANT_OPTIONS {
        assert!(
            !stdout.contains(opt),
            "stdout must not leak plant option {opt} as a shuffled choice: {stdout}"
        );
    }
    assert!(
        !out.exists(),
        "refuse must not write a form file ({})",
        out.display()
    );

    let _ = fs::remove_dir_all(&dir);
}

/// GOOD control for --assess: a single-select extra is admitted.
/// Without this, the plant could pass because --assess is broken for every kind.
#[test]
fn assemble_single_select_assess_is_admitted() {
    let extra = json!([{
        "id": "ok-ss",
        "module": 1,
        "stem": "which source",
        "item": {
            "kind": "single-select",
            "options": ["utility", "genset", "both", "neither"],
            "correct": "genset"
        }
    }]);
    let (dir, assess) = write_assess("ss", extra);
    let out = dir.join("form.json");

    let assert = cdcp()
        .args([
            "assemble",
            "--bank",
            "bank/items",
            "--seed",
            "42",
            "--assess",
            assess.to_str().expect("utf-8"),
            "--out",
            out.to_str().expect("utf-8"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("n_items=41"),
        "single-select extra must be appended to the 40-item letter form: {stdout}"
    );

    let exam: Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read form")).expect("form JSON");
    let ids: Vec<&str> = exam["item_ids"]
        .as_array()
        .expect("item_ids")
        .iter()
        .map(|v| v.as_str().expect("id"))
        .collect();
    assert_eq!(ids.len(), 41);
    assert_eq!(*ids.last().expect("last"), "ok-ss");
    let last = exam["items"]
        .as_array()
        .expect("items")
        .last()
        .expect("item");
    assert_eq!(last["id"], "ok-ss");
    assert_eq!(last["correct"], "genset");
    assert!(
        !matches!(last["correct"].as_str(), Some("A" | "B" | "C" | "D")),
        "semantic single-select must not be flattened to a letter, got {:?}",
        last["correct"]
    );

    let _ = fs::remove_dir_all(&dir);
}

/// ANTI-VACUOUS: an empty --assess list is an ERROR, not a silent bank-only form.
#[test]
fn assemble_empty_assess_file_is_an_error() {
    let (dir, assess) = write_assess("empty", json!([]));
    let assert = cdcp()
        .args([
            "assemble",
            "--bank",
            "bank/items",
            "--assess",
            assess.to_str().expect("utf-8"),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("empty"),
        "empty assess list must be named, got: {stderr}"
    );
    let _ = fs::remove_dir_all(&dir);
}
