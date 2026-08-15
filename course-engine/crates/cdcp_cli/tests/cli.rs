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
fn help_lists_learn_compilers() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in [
        "build-units",
        "build-glossary",
        "build-learn-slugs",
        "smoke-learn",
        "smoke-learn-chrome",
        "smoke-feedback-links",
        "smoke-diagrams",
        "smoke-a11y",
        "smoke-weak-links",
        "smoke-learn-v2",
        "export-anki",
    ] {
        assert!(
            stdout.contains(verb),
            "cdcp --help must list {verb}: {stdout}"
        );
    }
}

#[test]
fn smoke_diagrams_live_tree_passes() {
    let assert = cdcp().arg("smoke-diagrams").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("smoke_diagrams: PASS"),
        "live diagram smoke must PASS: {stdout}"
    );
}

#[test]
fn smoke_a11y_live_tree_passes() {
    let assert = cdcp().arg("smoke-a11y").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("PASS: smoke_a11y"),
        "live a11y smoke must PASS: {stdout}"
    );
}

#[test]
fn smoke_learn_chrome_live_tree_passes() {
    let assert = cdcp().arg("smoke-learn-chrome").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("smoke_learn_chrome: PASS"),
        "live Learn chrome smoke must PASS: {stdout}"
    );
}

#[test]
fn smoke_feedback_links_live_tree_passes() {
    let assert = cdcp().arg("smoke-feedback-links").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("PASS: smoke_feedback_links"),
        "live feedback-link smoke must PASS: {stdout}"
    );
}

#[test]
fn smoke_weak_links_live_tree_passes() {
    let assert = cdcp().arg("smoke-weak-links").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("PASS: smoke_weak_links"),
        "live weak-links smoke must PASS: {stdout}"
    );
}

#[test]
fn smoke_learn_v2_live_tree_passes() {
    let assert = cdcp().arg("smoke-learn-v2").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("smoke_learn_v2: PASS"),
        "live Learn v2 smoke must PASS: {stdout}"
    );
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

// ── bd-goldens-check-is-file-hole-7v9p ──────────────────────────────────────
// `goldens check` guarded its bank_hash comparison with `if bh_path.is_file()`.
// Measured before the fix, against a byte-identical temp copy of goldens/ with
// bank_hash.txt removed: exit 0, stdout `ok golden all-correct / ok golden
// all-wrong`. The only evidence of the skipped comparison was one MISSING
// stdout line. That is the absent-input-reads-as-success shape.
//
// FLOOR RAISED, not a proof: these tests establish that a required golden which
// is absent, empty, or unreachable exits non-zero, and that the command cannot
// report ok having compared fewer than EXPECTED_COMPARISONS legs. They CANNOT
// decide whether a present, matching pin was frozen against a correct bank — a
// deliberate re-freeze of a wrong value still passes here. That is the coupling
// ledger's question (cdcp_gate goldens-couplings) and PROVENANCE.md review.
//
// Every case below runs against a TEMP COPY. goldens/ is never mutated: another
// agent owns that surface, and its bank_hash content changes under this test.

/// Byte-copy `goldens/` into a fresh temp dir and return the copy's path.
///
/// The copy is compared against the LIVE bank, so it exercises exactly the
/// artifacts `goldens check --dir goldens` would.
fn goldens_copy(tag: &str) -> PathBuf {
    let dst = std::env::temp_dir().join(format!(
        "cdcp_goldens_{tag}_{}_{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dst).expect("create temp goldens dir");
    copy_tree(&workspace_root().join("goldens"), &dst);
    // Anti-vacuous: a copy helper that silently copied nothing would make every
    // known-bad case below pass for the wrong reason.
    assert!(
        dst.join("bank_hash.txt").is_file(),
        "temp copy is missing bank_hash.txt — the copy helper copied nothing"
    );
    dst
}

fn copy_tree(src: &std::path::Path, dst: &std::path::Path) {
    for entry in fs::read_dir(src).expect("read goldens dir") {
        let entry = entry.expect("dir entry");
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            fs::create_dir_all(&to).expect("mkdir");
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).expect("copy golden");
        }
    }
}

fn check_goldens_dir(dir: &std::path::Path) -> assert_cmd::assert::Assert {
    cdcp()
        .args([
            "goldens",
            "check",
            "--bank",
            "bank/items",
            "--dir",
            dir.to_str().expect("temp path is utf-8"),
        ])
        .assert()
}

/// KNOWN-GOOD: a faithful copy still passes, and reports how much it compared.
///
/// The count line is the anti-vacuous surface: before the fix, a run that
/// compared 2 legs and a run that compared 3 differed only by an absent line.
#[test]
fn goldens_check_passes_on_a_faithful_copy_and_reports_its_coverage() {
    let dir = goldens_copy("good");
    let assert = check_goldens_dir(&dir).success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("ok bank_hash pin"),
        "faithful copy must compare the bank_hash pin: {stdout}"
    );
    assert!(
        stdout.contains("3 comparison(s)"),
        "goldens check must report the number of comparisons it performed: {stdout}"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// KNOWN-BAD, the bead's headline case plus its siblings: deleting ANY required
/// golden exits non-zero and names the file. Deleting `bank_hash.txt` used to
/// exit 0.
#[test]
fn goldens_check_errors_when_any_required_golden_is_absent() {
    let required = [
        "fixtures/mock40_seed42.json",
        "mock40_seed42_all_correct.sha256",
        "mock40_seed42_all_wrong.sha256",
        "bank_hash.txt",
    ];
    // Anti-vacuous: a zero-length loop would make this test pass by testing
    // nothing, which is the exact bug under repair.
    assert_eq!(
        required.len(),
        4,
        "the required-golden list must not shrink"
    );

    for rel in required {
        let dir = goldens_copy("absent");
        fs::remove_file(dir.join(rel)).expect("delete the golden under test");
        let assert = check_goldens_dir(&dir).failure();
        let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
        assert!(
            stderr.contains(rel.rsplit('/').next().expect("basename")),
            "deleting {rel} must produce an error naming it, got: {stderr}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// KNOWN-BAD: a 0-byte pin file satisfies `is_file()` and pins nothing.
/// `exists()` is not the same claim as `a value was compared`.
#[test]
fn goldens_check_errors_on_a_zero_byte_pin() {
    for rel in ["bank_hash.txt", "mock40_seed42_all_correct.sha256"] {
        let dir = goldens_copy("zerobyte");
        let target = dir.join(rel);
        fs::write(&target, b"").expect("truncate pin to 0 bytes");
        assert_eq!(
            fs::metadata(&target).expect("stat pin").len(),
            0,
            "the planted pin must really be 0 bytes"
        );
        assert!(target.is_file(), "a 0-byte file still satisfies is_file()");

        let assert = check_goldens_dir(&dir).failure();
        let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
        assert!(
            stderr.contains("empty pin file"),
            "a 0-byte {rel} must be an error, got: {stderr}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// ANTI-VACUOUS: a goldens dir with nothing to check is an ERROR. An empty scan
/// must not report the way a scan that checked everything reports.
#[test]
fn goldens_check_errors_when_zero_goldens_are_discovered() {
    // (a) genuinely empty directory
    let empty = std::env::temp_dir().join(format!("cdcp_goldens_empty_{}", std::process::id()));
    let _ = fs::remove_dir_all(&empty);
    fs::create_dir_all(&empty).expect("create empty dir");
    let assert = check_goldens_dir(&empty).failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("discovered 0 golden files"),
        "an empty goldens dir must fail as a vacuous scan, got: {stderr}"
    );

    // (b) prose only. PROVENANCE.md is documentation, not a pinned artifact, so
    // a directory holding only prose has still discovered nothing.
    fs::write(empty.join("PROVENANCE.md"), "# prose only\n").expect("write prose");
    let assert = check_goldens_dir(&empty).failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("discovered 0 golden files"),
        "prose alone is not a golden, got: {stderr}"
    );

    // (c) a missing directory is an error that names it, not a silent pass.
    let gone = empty.join("does-not-exist");
    let assert = check_goldens_dir(&gone).failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("goldens dir not found"),
        "an absent --dir must be named, got: {stderr}"
    );

    let _ = fs::remove_dir_all(&empty);
}

/// ANTI-VACUOUS: a fixture pinning zero items grades nothing. Both attempts
/// would digest the empty case and `goldens generate` would freeze that.
#[test]
fn goldens_check_errors_on_a_fixture_that_pins_no_items() {
    let dir = goldens_copy("nofixtureids");
    let fixture_path = dir.join("fixtures/mock40_seed42.json");
    let mut fixture: Value =
        serde_json::from_str(&fs::read_to_string(&fixture_path).expect("read fixture"))
            .expect("parse fixture");
    fixture["item_ids"] = json!([]);
    fs::write(&fixture_path, serde_json::to_string(&fixture).unwrap()).expect("write fixture");

    let assert = check_goldens_dir(&dir).failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("item_ids is empty"),
        "a fixture with no ids must be named as the cause, got: {stderr}"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// KNOWN-GOOD, over-strictness leg: a legitimately-optional input must NOT fail.
/// `goldens/PROVENANCE.md` is prose the CLI does not consume, and an extra
/// unrelated `.md` is not a golden. A gate that reddens on documentation gets
/// routed around, so absence and addition of prose are both green here.
#[test]
fn goldens_check_ignores_prose_alongside_the_artifacts() {
    let dir = goldens_copy("prose");
    let provenance = dir.join("PROVENANCE.md");
    if provenance.is_file() {
        fs::remove_file(&provenance).expect("remove prose");
    }
    check_goldens_dir(&dir).success();

    fs::write(dir.join("NOTES.md"), "# scratch prose\n").expect("write extra prose");
    let assert = check_goldens_dir(&dir).success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("3 comparison(s)"),
        "adding prose must not change what is compared: {stdout}"
    );
    let _ = fs::remove_dir_all(&dir);
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
