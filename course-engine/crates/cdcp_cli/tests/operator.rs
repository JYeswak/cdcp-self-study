//! Operator surface: doctor / health --robot / repair (bd-engine-not-gate-ar39.4).
//!
//! These tests pin the product commands. They do not move the gate_shrink
//! ratchet and they never write the live goldens/.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        "cdcp_op_{tag}_{}_{:?}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("mkdir dst");
    for entry in fs::read_dir(src).expect("read src") {
        let entry = entry.expect("dir entry");
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).expect("copy file");
        }
    }
}

#[cfg(unix)]
fn symlink_dir(src: &Path, dst: &Path) {
    std::os::unix::fs::symlink(src, dst).expect("symlink dir");
}

/// A writable engine-shaped tree: large inputs are symlinked, outputs and
/// goldens are copied so a buggy repair cannot mutate the live goldens/.
fn operator_tree(tag: &str) -> PathBuf {
    let live = workspace_root();
    let dst = uniq(tag);
    fs::create_dir_all(dst.join("web/assets/wasm")).expect("mkdir wasm");
    fs::create_dir_all(dst.join("web/data")).expect("mkdir data");
    symlink_dir(&live.join("bank"), &dst.join("bank"));
    symlink_dir(&live.join("knowledge"), &dst.join("knowledge"));
    symlink_dir(&live.join("registries"), &dst.join("registries"));
    symlink_dir(&live.join("web/content"), &dst.join("web/content"));
    copy_tree(&live.join("goldens"), &dst.join("goldens"));
    copy_tree(&live.join("web/data"), &dst.join("web/data"));
    fs::copy(live.join("content.lock"), dst.join("content.lock")).expect("copy lock");
    let wasm = live.join("web/assets/wasm/cdcp_wasm.wasm");
    if wasm.is_file() {
        fs::copy(&wasm, dst.join("web/assets/wasm/cdcp_wasm.wasm")).expect("copy wasm");
    }
    assert!(
        dst.join("bank/items").is_dir(),
        "operator tree is missing bank/items — the fixture helper copied nothing"
    );
    dst
}

fn set_mtime_past(path: &Path) {
    let past = UNIX_EPOCH + Duration::from_secs(1_000_000);
    fs::File::options()
        .write(true)
        .open(path)
        .expect("open for mtime")
        .set_modified(past)
        .expect("set mtime");
}

fn mtime(path: &Path) -> SystemTime {
    fs::metadata(path).expect("stat").modified().expect("mtime")
}

// ── help ──────────────────────────────────────────────────────────────────

#[test]
fn help_lists_operator_verbs() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in ["doctor", "health", "repair"] {
        assert!(
            stdout.contains(verb),
            "cdcp --help must list {verb}: {stdout}"
        );
    }
}

#[test]
fn health_help_lists_robot() {
    let assert = cdcp().args(["health", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("--robot"),
        "health --help must list --robot: {stdout}"
    );
}

#[test]
fn repair_help_does_not_offer_to_freeze_goldens() {
    let assert = cdcp().args(["repair", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("Never re-freezes goldens") || stdout.contains("Never re-freeze"),
        "repair --help must say it does not re-freeze goldens: {stdout}"
    );
    assert!(
        !stdout.contains("UPDATE_GOLDENS"),
        "repair --help must not advertise UPDATE_GOLDENS: {stdout}"
    );
}

// ── doctor ────────────────────────────────────────────────────────────────

/// ANTI-VACUOUS: doctor on an empty tree is an ERROR naming what is missing.
/// A green "nothing to check" is the defect this command exists to close.
#[test]
fn doctor_on_an_empty_tree_is_error_naming_what_is_missing() {
    let empty = uniq("empty");
    fs::create_dir_all(&empty).expect("mkdir empty");

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
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    for needle in ["bank", "wasm", "goldens", "content.lock"] {
        assert!(
            combined.contains(needle),
            "doctor on an empty tree must name {needle}, got: {combined}"
        );
    }
    assert!(
        combined.contains("check(s) failed"),
        "doctor on an empty tree must fail, not pass: {combined}"
    );
    assert!(
        !combined.contains("check(s) passed"),
        "doctor must not claim a pass on an empty tree: {combined}"
    );

    let _ = fs::remove_dir_all(&empty);
}

/// KNOWN-BAD: delete the wasm artifact → doctor RED naming it.
#[test]
fn doctor_red_when_wasm_artifact_is_deleted() {
    let tree = operator_tree("nowasm");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    assert!(wasm.is_file(), "fixture must start with a wasm artifact");
    fs::remove_file(&wasm).expect("delete wasm");
    assert!(!wasm.is_file(), "wasm must actually be gone");

    let assert = cdcp()
        .args([
            "doctor",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .failure();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        combined.contains("cdcp_wasm.wasm") || combined.contains("wasm"),
        "deleting the wasm artifact must produce an error naming it, got: {combined}"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// KNOWN-BAD: a 0-byte wasm satisfies is_file() and is not fresh.
#[test]
fn doctor_red_when_wasm_is_present_but_not_fresh() {
    let tree = operator_tree("stale-wasm");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    fs::write(&wasm, b"").expect("truncate wasm");
    assert_eq!(fs::metadata(&wasm).unwrap().len(), 0);

    let assert = cdcp()
        .args([
            "doctor",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .failure();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        combined.contains("0 bytes") || combined.contains("not fresh") || combined.contains("wasm"),
        "a 0-byte wasm must be RED as not fresh, got: {combined}"
    );

    fs::write(&wasm, b"not a wasm module at all").expect("write junk wasm");
    let assert = cdcp()
        .args([
            "doctor",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .failure();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        combined.contains("magic") || combined.contains("not a wasm") || combined.contains("fresh"),
        "a non-wasm wasm path must be RED as not fresh, got: {combined}"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// KNOWN-BAD: corrupt content.lock → doctor RED.
#[test]
fn doctor_red_when_content_lock_is_corrupt() {
    let tree = operator_tree("badlock");
    fs::write(tree.join("content.lock"), "this is not toml {{{").expect("corrupt lock");

    let assert = cdcp()
        .args([
            "doctor",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .failure();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        combined.contains("content.lock"),
        "a corrupt content.lock must be named, got: {combined}"
    );
    assert!(
        combined.contains("corrupt") || combined.contains("TOML") || combined.contains("toml"),
        "the error must say the lock is corrupt, got: {combined}"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// KNOWN-GOOD: a faithful operator tree passes doctor (port 0 so this cannot
/// flake on a busy 8766).
#[test]
fn doctor_passes_on_a_faithful_tree() {
    let tree = operator_tree("good");
    let assert = cdcp()
        .args([
            "doctor",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("6 check(s) passed"),
        "faithful tree must run and pass all compiled-in checks: {stdout}"
    );
    let _ = fs::remove_dir_all(&tree);
}

// ── health --robot ────────────────────────────────────────────────────────

/// Field names pinned so a consumer can rely on them. Extra or missing
/// top-level keys is a schema change, not a silent reshape.
#[test]
fn health_robot_emits_a_versioned_envelope_with_pinned_fields() {
    let assert = cdcp().args(["health", "--robot"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let line = stdout.trim();
    assert!(!line.is_empty(), "health --robot must emit one NDJSON line");
    assert_eq!(
        line.lines().count(),
        1,
        "health --robot must be one NDJSON line, got: {stdout:?}"
    );

    let v: Value = serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("health --robot must be parseable JSON ({e}): {line}");
    });
    let obj = v
        .as_object()
        .unwrap_or_else(|| panic!("health --robot must be a JSON object: {line}"));

    let pinned = [
        "schema_version",
        "bank_hash",
        "approved_n",
        "manifest_n",
        "unit_count",
        "engine_identities",
        "goldens",
    ];
    // Anti-vacuous: a zero-length pin list would make this test pass by
    // checking nothing.
    assert_eq!(pinned.len(), 7, "the pinned field list must not shrink");
    for key in pinned {
        assert!(
            obj.contains_key(key),
            "health --robot missing pinned field {key}: {line}"
        );
    }
    assert_eq!(
        obj.len(),
        pinned.len(),
        "health --robot grew extra top-level keys {:?}; that is a schema change",
        obj.keys()
            .filter(|k| !pinned.contains(&k.as_str()))
            .collect::<Vec<_>>()
    );

    let ver = v["schema_version"]
        .as_u64()
        .expect("schema_version must be a number, not a string");
    assert!(ver >= 1, "schema_version must be >= 1, got {ver}");

    let hash = v["bank_hash"].as_str().expect("bank_hash is a string");
    assert_eq!(hash.len(), 64, "bank_hash must be 64 hex chars");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "bank_hash must be hex: {hash}"
    );

    let approved = v["approved_n"].as_u64().expect("approved_n is a number");
    let manifest = v["manifest_n"].as_u64().expect("manifest_n is a number");
    let units = v["unit_count"].as_u64().expect("unit_count is a number");
    assert!(approved > 0, "approved_n must be > 0, got {approved}");
    assert!(manifest > 0, "manifest_n must be > 0, got {manifest}");
    assert!(units > 0, "unit_count must be > 0, got {units}");
    assert!(
        approved <= manifest,
        "approved_n={approved} must be <= manifest_n={manifest}"
    );

    let engine = v["engine_identities"]
        .as_object()
        .expect("engine_identities is an object");
    assert_eq!(
        engine.get("oracle").and_then(|x| x.as_str()),
        Some("cdcp_grade-native")
    );
    assert_eq!(
        engine.get("subject").and_then(|x| x.as_str()),
        Some("cdcp_wasm-wasm32")
    );

    let goldens = v["goldens"].as_object().expect("goldens is an object");
    for k in ["state", "required_n", "present_n"] {
        assert!(goldens.contains_key(k), "goldens missing {k}");
    }
    assert_eq!(goldens["state"].as_str(), Some("present"));
    assert!(
        goldens["required_n"].as_u64().unwrap() > 0,
        "goldens.required_n must not be 0"
    );
}

/// ANTI-VACUOUS: health with zero items is an ERROR, never a versioned
/// envelope full of zeros that a consumer could read as "healthy, empty".
#[test]
fn health_with_zero_items_is_error() {
    let empty = uniq("health0");
    fs::create_dir_all(empty.join("bank/items")).expect("mkdir empty bank");
    fs::write(
        empty.join("bank/MANIFEST.toml"),
        "schema_version = 1\nitem_count = 0\nitems = []\n",
    )
    .expect("write empty manifest");
    fs::create_dir_all(empty.join("web/data")).expect("mkdir data");
    fs::write(
        empty.join("web/data/units_index.json"),
        r#"{"schema_version":1,"unit_count":0,"units":[]}"#,
    )
    .expect("write empty units");

    let assert = cdcp()
        .args(["health", "--robot", "--root", empty.to_str().unwrap()])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stderr.contains("zero")
            || stderr.contains("empty")
            || stderr.contains("0 items")
            || stderr.contains("approved_n"),
        "health on zero items must name the emptiness, got stderr={stderr} stdout={stdout}"
    );
    // Must not emit a success-shaped envelope the consumer could parse as ok.
    if let Ok(v) = serde_json::from_str::<Value>(stdout.trim()) {
        panic!("health --robot on zero items must not emit a JSON envelope, got: {v}");
    }

    let _ = fs::remove_dir_all(&empty);
}

/// Unversioned / unparseable is the command's own ERROR floor: the live
/// envelope always carries a numeric schema_version, and pretty-printed
/// multi-line JSON is rejected by the NDJSON pin above.
#[test]
fn health_robot_schema_version_is_numeric_not_a_string() {
    let assert = cdcp().args(["health", "--robot"]).assert().success();
    let line = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(line.trim()).expect("parseable");
    assert!(
        v["schema_version"].is_u64() || v["schema_version"].is_i64(),
        "schema_version must be a JSON number, got {}",
        v["schema_version"]
    );
    assert!(
        !v["schema_version"].is_string(),
        "a string schema_version is unversioned-as-far-as-a-consumer-is-concerned"
    );
}

// ── repair ────────────────────────────────────────────────────────────────

/// repair is idempotent: run twice, second run writes nothing. Asserted by
/// mtime, not by exit code — a tree-cleanliness check cannot see an
/// idempotent write.
#[test]
fn repair_second_run_writes_nothing_by_mtime() {
    let tree = operator_tree("repair");
    let watched = [
        tree.join("web/data/units_index.json"),
        tree.join("web/data/glossary.json"),
        tree.join("web/data/module_learn_slugs.js"),
        tree.join("web/data/mock40_seed42.json"),
        tree.join("web/data/keys_seed42.json"),
        tree.join("web/data/bank_items_seed42.json"),
    ];
    // Anti-vacuous: a watch list of 0 files would make this pass by checking
    // nothing, which is the exact bug under repair.
    assert_eq!(watched.len(), 6, "repair must rebuild six artifacts");

    cdcp()
        .args(["repair", "--root", tree.to_str().unwrap()])
        .assert()
        .success();
    for p in &watched {
        assert!(p.is_file(), "repair must produce {}", p.display());
        assert!(
            fs::metadata(p).unwrap().len() > 0,
            "{} must be non-empty after repair",
            p.display()
        );
        set_mtime_past(p);
        assert_eq!(
            mtime(p),
            UNIX_EPOCH + Duration::from_secs(1_000_000),
            "planted mtime must actually land on {}",
            p.display()
        );
    }

    cdcp()
        .args(["repair", "--root", tree.to_str().unwrap()])
        .assert()
        .success();
    for p in &watched {
        assert_eq!(
            mtime(p),
            UNIX_EPOCH + Duration::from_secs(1_000_000),
            "second repair must not write {} (mtime moved)",
            p.display()
        );
    }

    let _ = fs::remove_dir_all(&tree);
}

/// THE HARD RULE: repair is not a golden laundromat. Even with
/// UPDATE_GOLDENS=1 in the environment, goldens/ mtimes stay put.
#[test]
fn repair_does_not_refreeze_goldens_even_with_update_goldens() {
    let tree = operator_tree("nolaundry");
    let goldens = [
        tree.join("goldens/bank_hash.txt"),
        tree.join("goldens/mock40_seed42_all_correct.sha256"),
        tree.join("goldens/mock40_seed42_all_wrong.sha256"),
        tree.join("goldens/fixtures/mock40_seed42.json"),
    ];
    assert_eq!(goldens.len(), 4, "the golden watch list must not shrink");
    for p in &goldens {
        assert!(p.is_file(), "fixture missing {}", p.display());
        set_mtime_past(p);
    }

    let mut cmd = cdcp();
    cmd.env("UPDATE_GOLDENS", "1")
        .args(["repair", "--root", tree.to_str().unwrap()])
        .assert()
        .success();

    for p in &goldens {
        assert_eq!(
            mtime(p),
            UNIX_EPOCH + Duration::from_secs(1_000_000),
            "repair must not re-freeze {} even with UPDATE_GOLDENS=1",
            p.display()
        );
    }

    let _ = fs::remove_dir_all(&tree);
}

/// Source-level belt: the repair function must not name the golden writers.
/// A later edit that calls them is a test failure, not a silent B2 hole.
#[test]
fn repair_source_does_not_call_golden_writers() {
    let src = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/operator.rs"))
        .expect("read operator.rs");
    // Isolate the repair function so comments elsewhere cannot hide a call.
    let start = src
        .find("pub(crate) fn repair(")
        .expect("repair function must exist");
    let rest = &src[start..];
    let end = rest[1..]
        .find("\npub(crate) fn ")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    let body = &rest[..end];
    for forbidden in ["goldens_generate", "goldens_fixture", "GoldensCmd"] {
        assert!(
            !body.contains(forbidden),
            "repair must not mention {forbidden}: that is the golden laundromat"
        );
    }
}

/// repair on an empty tree is an ERROR, not a green no-op.
#[test]
fn repair_on_an_empty_tree_is_error() {
    let empty = uniq("repair-empty");
    fs::create_dir_all(&empty).expect("mkdir");
    let assert = cdcp()
        .args(["repair", "--root", empty.to_str().unwrap()])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("bank") || stderr.contains("nothing to rebuild"),
        "repair on an empty tree must name what is missing, got: {stderr}"
    );
    let _ = fs::remove_dir_all(&empty);
}
