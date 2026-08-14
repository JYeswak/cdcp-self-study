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

// ── bd-golden-sampler-divergence-09q ────────────────────────────────────────
// The seed-42 golden used to be Python MT19937 output (scripts/sample_mock.py) while
// export-web silently PREFERRED that fixture at seed 42. The Rust sampler was therefore
// never exercised by any gate: measured at the time of the fix, 37 of 40 ids differed
// between cdcp_assemble::assemble(seed=42) and the committed fixture, and 0 of the 3
// shared ids sat at the same index. The two tests below close that hole.

/// Read the committed seed-42 fixture's `item_ids` in order.
fn golden_fixture_item_ids() -> Vec<String> {
    let path = workspace_root().join("goldens/fixtures/mock40_seed42.json");
    let fixture: Value = serde_json::from_str(&fs::read_to_string(&path).expect("read fixture"))
        .expect("parse fixture");
    assert_eq!(
        fixture["seed"].as_u64(),
        Some(42),
        "fixture must be the seed-42 fixture"
    );
    let ids: Vec<String> = fixture["item_ids"]
        .as_array()
        .expect("fixture.item_ids array")
        .iter()
        .map(|v| v.as_str().expect("item id is a string").to_string())
        .collect();
    // Anti-vacuous: an empty or short id list must ERROR, never silently pass.
    assert_eq!(
        ids.len(),
        40,
        "fixture must pin 40 item_ids, got {}",
        ids.len()
    );
    ids
}

/// THE ASSERTION THIS REPO WAS MISSING: the golden fixture IS the Rust sampler's output.
///
/// `cdcp_assemble::assemble()` at seed 42 must reproduce `goldens/fixtures/mock40_seed42.json`
/// exactly — same set, same presentation order. Perturb the sampler (seed derivation, module
/// shuffle order, PRNG) and this test goes RED, which is the known-bad proof that the golden
/// now certifies the assembler it claims to certify.
///
/// `shuffle_choices` is false to mirror export-web; choice shuffling draws from an independent
/// rng, so `item_ids` are identical under either config.
#[test]
fn golden_fixture_is_the_rust_sampler_output() {
    let bank = cdcp_bank::Bank::load_dir(&workspace_root().join("bank/items")).expect("load bank");
    let cfg = cdcp_assemble::AssembleConfig {
        shuffle_choices: false,
        ..Default::default()
    };
    let exam = cdcp_assemble::assemble(&bank, 42, cfg).expect("assemble seed 42");

    assert_eq!(
        exam.item_ids,
        golden_fixture_item_ids(),
        "goldens/fixtures/mock40_seed42.json does not match cdcp_assemble::assemble(seed=42). \
         Either the sampler changed (re-freeze deliberately with \
         `UPDATE_GOLDENS=1 cargo run -p cdcp_cli -- goldens fixture` then `goldens generate`), \
         or the fixture was regenerated by something other than the Rust sampler — which is the \
         fooled certificate bd-golden-sampler-divergence-09q removed."
    );
}

/// export-web must NOT replay the fixture at seed 42 — the implicit bypass is gone.
///
/// Exporting into a temp dir from the workspace root (where the old default fixture path
/// resolves) must still run the sampler: stdout reports `golden_pinned=false`, and the emitted
/// pack's item order equals the sampler's. Restore the `seed == 42` preference in
/// `export_web()` and this test goes RED.
#[test]
fn export_web_seed42_runs_the_sampler_not_the_fixture() {
    let out = std::env::temp_dir().join(format!("cdcp_cli_exportweb_{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let assert = cdcp()
        .args([
            "export-web",
            "--bank",
            "bank/items",
            "--seed",
            "42",
            "--out",
            out.to_str().expect("temp path is utf-8"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("golden_pinned=false"),
        "export-web --seed 42 must run the sampler, not replay a fixture: {stdout}"
    );

    let pack: Value = serde_json::from_str(
        &fs::read_to_string(out.join("mock40_seed42.json")).expect("read exported pack"),
    )
    .expect("parse exported pack");
    let exported: Vec<String> = pack["items"]
        .as_array()
        .expect("pack.items array")
        .iter()
        .map(|i| i["id"].as_str().expect("pack item id").to_string())
        .collect();
    assert_eq!(exported.len(), 40, "exported pack must carry 40 items");

    let bank = cdcp_bank::Bank::load_dir(&workspace_root().join("bank/items")).expect("load bank");
    let cfg = cdcp_assemble::AssembleConfig {
        shuffle_choices: false,
        ..Default::default()
    };
    let exam = cdcp_assemble::assemble(&bank, 42, cfg).expect("assemble seed 42");
    assert_eq!(
        exported, exam.item_ids,
        "export-web seed 42 output must equal the sampler's output"
    );

    let _ = fs::remove_dir_all(&out);
}

/// The retained bypass is EXPLICIT and TESTED: `--fixture <path>` still replays a recorded
/// item_ids list and says so (`golden_pinned=true`). Only the implicit seed-42 preference died.
#[test]
fn export_web_explicit_fixture_flag_still_replays() {
    let out = std::env::temp_dir().join(format!("cdcp_cli_exportweb_fix_{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let assert = cdcp()
        .args([
            "export-web",
            "--bank",
            "bank/items",
            "--seed",
            "42",
            "--out",
            out.to_str().expect("temp path is utf-8"),
            "--fixture",
            "goldens/fixtures/mock40_seed42.json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("golden_pinned=true"),
        "--fixture must report an explicit replay: {stdout}"
    );

    let _ = fs::remove_dir_all(&out);
}
