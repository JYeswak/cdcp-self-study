//! Integration tests for the `cdcp` binary.
//!
//! Paths are relative to the course-engine workspace root (cwd set per command).

use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

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

#[test]
fn bank_hash_prints_64_hex_chars() {
    let assert = cdcp()
        .args(["bank-hash", "--bank", "bank/items"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let hash = stdout.trim();
    assert_eq!(
        hash.len(),
        64,
        "bank_hash should be 64 hex chars, got {hash:?}"
    );
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "bank_hash must be hex: {hash}"
    );
}

#[test]
fn goldens_check_exit_0() {
    let assert = cdcp()
        .args([
            "goldens",
            "check",
            "--bank",
            "bank/items",
            "--dir",
            "goldens",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("ok golden all-correct"),
        "missing ok golden all-correct: {stdout}"
    );
    assert!(
        stdout.contains("ok golden all-wrong"),
        "missing ok golden all-wrong: {stdout}"
    );
}

#[test]
fn grade_all_correct_score_40() {
    let assert = cdcp()
        .args([
            "grade",
            "--bank",
            "bank/items",
            "--fixture",
            "goldens/fixtures/mock40_seed42.json",
            "--mode",
            "all-correct",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("score=40/40"),
        "expected score=40/40, got: {stdout}"
    );
    assert!(
        stdout.contains("digest="),
        "expected digest= in output: {stdout}"
    );
}

#[test]
fn grade_json_all_correct_from_fixture_answers() {
    let root = workspace_root();
    let fixture_path = root.join("goldens/fixtures/mock40_seed42.json");
    let fixture: Value =
        serde_json::from_str(&fs::read_to_string(&fixture_path).expect("read fixture"))
            .expect("parse fixture");

    // Build [{item_id, chosen}, ...] from fixture items' correct letters.
    let answers: Vec<Value> = fixture["items"]
        .as_array()
        .expect("fixture.items")
        .iter()
        .map(|item| {
            json!({
                "item_id": item["id"],
                "chosen": item["correct"],
            })
        })
        .collect();
    assert_eq!(answers.len(), 40);

    let dir = std::env::temp_dir().join(format!("cdcp_cli_test_answers_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let answers_path = dir.join("all_correct.json");
    fs::write(
        &answers_path,
        serde_json::to_string_pretty(&answers).unwrap(),
    )
    .expect("write answers");

    let assert = cdcp()
        .args([
            "grade",
            "--bank",
            "bank/items",
            "--fixture",
            "goldens/fixtures/mock40_seed42.json",
            "--mode",
            "json",
            "--answers",
            answers_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("score=40/40"),
        "json mode all-correct expected score=40/40, got: {stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn grade_json_rejects_bad_letter_and_unknown_item() {
    let dir = std::env::temp_dir().join(format!("cdcp_cli_test_bad_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);

    let bad_letter = dir.join("bad_letter.json");
    fs::write(&bad_letter, r#"[{"item_id":"m01-q045","chosen":"E"}]"#).unwrap();
    let assert = cdcp()
        .args([
            "grade",
            "--bank",
            "bank/items",
            "--fixture",
            "goldens/fixtures/mock40_seed42.json",
            "--mode",
            "json",
            "--answers",
            bad_letter.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("invalid chosen"),
        "expected invalid chosen error, got: {stderr}"
    );

    let unknown = dir.join("unknown.json");
    fs::write(
        &unknown,
        r#"[{"item_id":"does-not-exist-xyz","chosen":"A"}]"#,
    )
    .unwrap();
    let assert = cdcp()
        .args([
            "grade",
            "--bank",
            "bank/items",
            "--fixture",
            "goldens/fixtures/mock40_seed42.json",
            "--mode",
            "json",
            "--answers",
            unknown.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("unknown item_id"),
        "expected unknown item_id error, got: {stderr}"
    );

    let _ = fs::remove_dir_all(&dir);
}
