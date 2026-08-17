//! Operator surface: doctor / health --robot / repair (bd-engine-not-gate-ar39.4).
//!
//! These tests pin the product commands. They do not move the gate_shrink
//! ratchet and they never write the live goldens/.
//!
//! Learner doctor (bd-installability-sm4g.11) must pass on a tree that has
//! ONLY `web/` + shipped wasm. A test that cannot construct that tree is
//! vacuous and FAILS.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::net::TcpListener;
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
    let index = live.join("web/index.html");
    if index.is_file() {
        fs::copy(&index, dst.join("web/index.html")).expect("copy index.html");
    }
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

/// INSTALLED layer only: `web/index.html` + shipped wasm. No bank/, no
/// goldens/, no content.lock. A helper that cannot copy those two files
/// fails the test — it must not skip and look green.
fn web_only_tree(tag: &str) -> PathBuf {
    let live = workspace_root();
    let src_index = live.join("web/index.html");
    let src_wasm = live.join("web/assets/wasm/cdcp_wasm.wasm");
    assert!(
        src_index.is_file(),
        "cannot construct web-only tree: {} missing — this test is vacuous without a live web/",
        src_index.display()
    );
    assert!(
        src_wasm.is_file(),
        "cannot construct web-only tree: {} missing — this test is vacuous without a shipped wasm",
        src_wasm.display()
    );
    let dst = uniq(tag);
    let dest_web = dst.join("web");
    fs::create_dir_all(dest_web.join("assets/wasm")).expect("mkdir web/assets/wasm");
    fs::copy(&src_index, dest_web.join("index.html")).expect("copy index.html");
    let dest_wasm = dest_web.join("assets/wasm/cdcp_wasm.wasm");
    fs::copy(&src_wasm, &dest_wasm).expect("copy wasm");
    assert!(
        dest_web.join("index.html").is_file() && dest_wasm.is_file(),
        "web-only tree failed to materialize under {}",
        dst.display()
    );
    assert!(
        !dst.join("bank").exists()
            && !dst.join("goldens").exists()
            && !dst.join("content.lock").exists(),
        "web-only tree must not contain bank/goldens/content.lock"
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

fn file_sha256(path: &Path) -> String {
    let try_cmd = |bin: &str, args: &[&str]| -> Option<String> {
        let out = std::process::Command::new(bin)
            .args(args)
            .arg(path)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        String::from_utf8(out.stdout)
            .ok()?
            .split_whitespace()
            .next()
            .map(|s| s.to_ascii_lowercase())
    };
    try_cmd("sha256sum", &[])
        .or_else(|| try_cmd("shasum", &["-a", "256"]))
        .unwrap_or_else(|| {
            panic!(
                "need sha256sum or shasum to plant a receipt for {}",
                path.display()
            )
        })
}

fn write_receipt(root: &Path, files: &[(&Path, &str)]) {
    assert!(
        !files.is_empty(),
        "a planted receipt with 0 files is the vacuous case this command refuses"
    );
    let entries: Vec<Value> = files
        .iter()
        .map(|(p, h)| {
            serde_json::json!({
                "path": p.display().to_string(),
                "sha256": h,
            })
        })
        .collect();
    let rec = serde_json::json!({
        "version": "0.1.0",
        "installed_at": "2026-08-17T00:00:00Z",
        "triple": "test",
        "source_build": true,
        "artifact": {"url": "test", "sha256": "00", "triple": "test"},
        "files": entries,
        "config_touched": [],
        "learner_progress_kept": true,
        "learner_progress_paths": []
    });
    fs::write(
        root.join("install-receipt.json"),
        serde_json::to_vec_pretty(&rec).expect("serialize receipt"),
    )
    .expect("write receipt");
}

/// web/ + wasm + a matching install-receipt.json. No bank/.
fn receipt_tree(tag: &str) -> PathBuf {
    let tree = web_only_tree(tag);
    let index = tree.join("web/index.html");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    let index_hash = file_sha256(&index);
    let wasm_hash = file_sha256(&wasm);
    write_receipt(
        &tree,
        &[(&index, index_hash.as_str()), (&wasm, wasm_hash.as_str())],
    );
    tree
}

fn fingerprint(paths: &[PathBuf]) -> Vec<(SystemTime, String, u64)> {
    paths
        .iter()
        .map(|p| {
            let meta = fs::metadata(p).unwrap_or_else(|e| panic!("stat {}: {e}", p.display()));
            (meta.modified().expect("mtime"), file_sha256(p), meta.len())
        })
        .collect()
}

// ── help ──────────────────────────────────────────────────────────────────

#[test]
fn help_lists_operator_verbs() {
    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .arg("--help")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for verb in ["doctor", "repair"] {
        assert!(
            stdout.contains(verb),
            "learner --help must list {verb}: {stdout}"
        );
    }
    let dev = cdcp().env("CDCP_DEV", "1").arg("--help").assert().success();
    let dev_out = String::from_utf8_lossy(&dev.get_output().stdout);
    assert!(
        dev_out.contains("health"),
        "CDCP_DEV=1 --help must list health: {dev_out}"
    );
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
fn doctor_help_lists_json() {
    let assert = cdcp().args(["doctor", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("--json"),
        "doctor --help must list --json: {stdout}"
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
    for flag in ["--dry-run", "--apply", "--json"] {
        assert!(
            stdout.contains(flag),
            "repair --help must list {flag}: {stdout}"
        );
    }
}

// ── doctor ────────────────────────────────────────────────────────────────

/// ANTI-VACUOUS: doctor on an empty tree is an ERROR naming what is missing.
/// A green "nothing to check" is the defect this command exists to close.
#[test]
fn doctor_on_an_empty_tree_is_error_naming_what_is_missing() {
    let empty = uniq("empty");
    fs::create_dir_all(&empty).expect("mkdir empty");
    let wasm_abs = empty.join("web/assets/wasm/cdcp_wasm.wasm");

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
    for needle in ["web", "wasm"] {
        assert!(
            combined.contains(needle),
            "doctor on an empty tree must name {needle}, got: {combined}"
        );
    }
    assert!(
        combined.contains(&wasm_abs.display().to_string()),
        "missing wasm must name the absolute path {}, got: {combined}",
        wasm_abs.display()
    );
    assert!(
        combined.contains("check(s) failed"),
        "doctor on an empty tree must fail, not pass: {combined}"
    );
    assert!(
        !combined.contains("check(s) passed"),
        "doctor must not claim a pass on an empty tree: {combined}"
    );
    assert!(
        !combined.contains("FAIL doctor bank")
            && !combined.contains("FAIL doctor goldens")
            && !combined.contains("FAIL doctor python3")
            && !combined.contains("FAIL doctor content.lock"),
        "learner doctor must not probe authoring layer on an empty tree: {combined}"
    );

    let _ = fs::remove_dir_all(&empty);
}

/// KNOWN-BAD: delete the wasm artifact → doctor RED naming the absolute path.
#[test]
fn doctor_red_when_wasm_artifact_is_deleted() {
    let tree = web_only_tree("nowasm");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    assert!(wasm.is_file(), "fixture must start with a wasm artifact");
    let abs = wasm.display().to_string();
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
        combined.contains(&abs),
        "deleting the wasm artifact must name the absolute path {abs}, got: {combined}"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// KNOWN-BAD: a 0-byte wasm satisfies is_file() and is not fresh.
#[test]
fn doctor_red_when_wasm_is_present_but_not_fresh() {
    let tree = web_only_tree("stale-wasm");
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

/// KNOWN-BAD: corrupt content.lock → doctor RED, but only on the authoring path.
#[test]
fn doctor_red_when_content_lock_is_corrupt_under_cdcp_dev() {
    let tree = operator_tree("badlock");
    fs::write(tree.join("content.lock"), "this is not toml {{{").expect("corrupt lock");

    let assert = cdcp()
        .env("CDCP_DEV", "1")
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
        "a corrupt content.lock must be named under CDCP_DEV=1, got: {combined}"
    );
    assert!(
        combined.contains("corrupt") || combined.contains("TOML") || combined.contains("toml"),
        "the error must say the lock is corrupt, got: {combined}"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// Learner path ignores a corrupt content.lock — that file is not installed.
#[test]
fn doctor_learner_path_ignores_corrupt_content_lock() {
    let tree = web_only_tree("lock-ignored");
    fs::write(tree.join("content.lock"), "this is not toml {{{").expect("corrupt lock");
    cdcp()
        .env_remove("CDCP_DEV")
        .args([
            "doctor",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .success();
    let _ = fs::remove_dir_all(&tree);
}

/// KNOWN-GOOD: a web-only tree (the installed layer) passes learner doctor.
/// A test that cannot construct that tree is vacuous and FAILS (see
/// `web_only_tree`).
#[test]
fn doctor_passes_on_a_web_only_tree() {
    let tree = web_only_tree("web-only");
    let assert = cdcp()
        .env_remove("CDCP_DEV")
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
        stdout.contains("4 check(s) passed"),
        "web-only tree must pass the four installed-layer probes: {stdout}"
    );
    assert!(
        !stdout.contains("doctor bank")
            && !stdout.contains("doctor goldens")
            && !stdout.contains("doctor python3")
            && !stdout.contains("doctor content.lock"),
        "learner doctor must not probe authoring layer: {stdout}"
    );
    let _ = fs::remove_dir_all(&tree);
}

/// Occupied default 8766 is not "tool broken" on the learner path.
#[test]
fn doctor_occupied_default_port_is_not_red() {
    let tree = web_only_tree("busy-port");
    let _hold = TcpListener::bind("127.0.0.1:8766");
    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args(["doctor", "--root", tree.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("4 check(s) passed"),
        "occupied 8766 must not fail learner doctor: {stdout}"
    );
    let _ = fs::remove_dir_all(&tree);
}

/// `cdcp doctor --json` emits a versioned envelope with named probes + pass/fail.
#[test]
fn doctor_json_emits_versioned_envelope() {
    let tree = web_only_tree("json-ok");
    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args([
            "doctor",
            "--json",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let line = stdout.trim();
    assert_eq!(
        line.lines().count(),
        1,
        "doctor --json must be one JSON line, got: {stdout:?}"
    );
    let v: Value = serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("doctor --json must be parseable JSON ({e}): {line}");
    });
    assert_eq!(
        v["schema_version"].as_u64(),
        Some(1),
        "doctor --json schema_version must be 1: {line}"
    );
    assert_eq!(v["ok"].as_bool(), Some(true), "web-only must pass: {line}");
    assert_eq!(v["layer"].as_str(), Some("installed"));
    let probes = v["probes"]
        .as_array()
        .unwrap_or_else(|| panic!("probes must be an array: {line}"));
    assert!(!probes.is_empty(), "empty probe list is ERROR, got: {line}");
    let names: Vec<&str> = probes
        .iter()
        .map(|p| p["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        names,
        ["web", "wasm", "receipt", "port"],
        "learner probe list must be the installed layer: {line}"
    );
    for p in probes {
        assert!(p["ok"].is_boolean(), "each probe needs pass/fail: {p}");
        assert!(p.get("detail").is_some(), "each probe needs detail: {p}");
    }
    let wasm = probes.iter().find(|p| p["name"] == "wasm").expect("wasm");
    let wasm_path = wasm["path"].as_str().expect("wasm.path");
    assert!(
        Path::new(wasm_path).is_absolute(),
        "wasm path must be absolute: {wasm_path}"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// Delete wasm → `--json` is still RED and names the absolute path.
#[test]
fn doctor_json_names_absolute_wasm_path_when_missing() {
    let tree = web_only_tree("json-nowasm");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    let abs = wasm.display().to_string();
    fs::remove_file(&wasm).expect("delete wasm");

    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args([
            "doctor",
            "--json",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let v: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("doctor --json must still emit an envelope on RED ({e}): {stdout}");
    });
    assert_eq!(v["ok"].as_bool(), Some(false));
    let combined = format!("{stdout}\n{stderr}");
    assert!(
        combined.contains(&abs),
        "stderr/json must name the absolute wasm path {abs}, got: {combined}"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// `env -i PATH=<dir-without-python3>` still exit 0 on the learner path.
#[test]
fn doctor_learner_path_does_not_require_python3() {
    let tree = web_only_tree("nopy");
    let empty_path = uniq("nopath");
    fs::create_dir_all(&empty_path).expect("mkdir empty PATH");
    let home = uniq("home");
    fs::create_dir_all(&home).expect("mkdir home");

    let mut cmd = Command::cargo_bin("cdcp").expect("cdcp binary");
    cmd.env_clear()
        .env("PATH", &empty_path)
        .env("HOME", &home)
        .env("TMPDIR", std::env::temp_dir())
        .args([
            "doctor",
            "--root",
            tree.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .success();

    let _ = fs::remove_dir_all(&tree);
    let _ = fs::remove_dir_all(&empty_path);
    let _ = fs::remove_dir_all(&home);
}

/// `CDCP_DEV=1` still runs authoring probes (bank / goldens / lock / python3).
#[test]
fn doctor_cdcp_dev_runs_authoring_probes() {
    let tree = web_only_tree("dev-red");
    let assert = cdcp()
        .env("CDCP_DEV", "1")
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
    for needle in ["bank", "goldens", "content.lock", "python3"] {
        assert!(
            combined.contains(needle),
            "CDCP_DEV=1 doctor must run authoring probe {needle}: {combined}"
        );
    }

    let good = operator_tree("dev-good");
    let assert = cdcp()
        .env("CDCP_DEV", "1")
        .args([
            "doctor",
            "--root",
            good.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("8 check(s) passed"),
        "CDCP_DEV=1 on a faithful tree must pass installed+authoring: {stdout}"
    );

    let _ = fs::remove_dir_all(&tree);
    let _ = fs::remove_dir_all(&good);
}

/// Source-level belt: emptying DOCTOR_CHECKS is a compile-time assert, and
/// the learner list must not contain authoring probes.
#[test]
fn doctor_checks_source_is_learner_layer_and_nonempty() {
    let src = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/operator.rs"))
        .expect("read operator.rs");
    assert!(
        src.contains("empty DOCTOR_CHECKS certifies nothing"),
        "DOCTOR_CHECKS must have a compile-time empty-list assert"
    );
    let start = src
        .find("pub(crate) const DOCTOR_CHECKS")
        .expect("DOCTOR_CHECKS must exist");
    let rest = &src[start..];
    let end = rest.find(';').expect("DOCTOR_CHECKS assignment");
    let decl = &rest[..=end];
    assert!(
        decl.contains("\"web\"")
            && decl.contains("\"wasm\"")
            && decl.contains("\"receipt\"")
            && decl.contains("\"port\""),
        "DOCTOR_CHECKS must list the installed layer: {decl}"
    );
    for banned in ["bank", "goldens", "content.lock", "python3"] {
        assert!(
            !decl.contains(banned),
            "learner DOCTOR_CHECKS must not contain {banned}: {decl}"
        );
    }
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
        "web",
        "wasm",
        "receipt",
        "engine_identities",
        "attempts_store",
    ];
    // Anti-vacuous: a zero-length pin list would make this test pass by
    // checking nothing.
    assert_eq!(pinned.len(), 6, "the pinned field list must not shrink");
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
    assert_eq!(ver, 3, "health --robot schema_version must be 3, got {ver}");

    for key in ["web", "wasm", "receipt"] {
        let fact = v[key]
            .as_object()
            .unwrap_or_else(|| panic!("{key} is an object"));
        assert!(fact.contains_key("state"), "{key} missing state");
        assert!(fact.contains_key("path"), "{key} missing path");
        let path = fact["path"].as_str().unwrap_or("");
        assert!(
            Path::new(path).is_absolute(),
            "{key}.path must be absolute: {path}"
        );
    }
    assert_eq!(v["web"]["state"].as_str(), Some("present"));
    assert_eq!(v["wasm"]["state"].as_str(), Some("present"));
    assert!(
        v["wasm"]["bytes"].as_u64().unwrap_or(0) > 0,
        "shipped wasm must be non-empty"
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

    let store = v["attempts_store"]
        .as_object()
        .expect("attempts_store is an object");
    for k in ["state", "path", "n", "export_policy"] {
        assert!(store.contains_key(k), "attempts_store missing {k}");
    }
    assert_eq!(
        store["export_policy"].as_str(),
        Some("off"),
        "export policy default is OFF"
    );
    assert_eq!(store["path"].as_str(), Some("var/attempts"));
}

/// Bundle-only (web/ + wasm, no bank/goldens) must exit 0. That is the
/// W12 contract: health --robot is no longer RED-by-construction on an
/// installed machine.
#[test]
fn health_robot_exits_0_on_bundle_only() {
    let tree = web_only_tree("health-bundle");
    let assert = cdcp()
        .args(["health", "--robot", "--root", tree.to_str().unwrap()])
        .assert()
        .success();
    let line = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(line.trim()).expect("health --robot JSON");
    assert_eq!(v["schema_version"].as_u64(), Some(3));
    assert_eq!(v["web"]["state"].as_str(), Some("present"));
    assert_eq!(v["wasm"]["state"].as_str(), Some("present"));
    assert_eq!(v["receipt"]["state"].as_str(), Some("absent"));
    assert!(
        v.get("bank_hash").is_none() && v.get("goldens").is_none(),
        "v3 must not require bank/goldens: {line}"
    );
    let _ = fs::remove_dir_all(&tree);
}

/// ANTI-VACUOUS: health without the installed web/ is an ERROR, never a
/// versioned envelope a consumer could read as "healthy, empty".
#[test]
fn health_without_web_is_error() {
    let empty = uniq("health0");
    fs::create_dir_all(&empty).expect("mkdir empty");

    let assert = cdcp()
        .args(["health", "--robot", "--root", empty.to_str().unwrap()])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stderr.contains("web") || stderr.contains("bundle"),
        "health without web/ must name the missing installed layer, got stderr={stderr} stdout={stdout}"
    );
    // Must not emit a success-shaped envelope the consumer could parse as ok.
    if let Ok(v) = serde_json::from_str::<Value>(stdout.trim()) {
        panic!("health --robot without web/ must not emit a JSON envelope, got: {v}");
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

/// `--dry-run` (the default) writes nothing. Hashes and mtimes stay put.
#[test]
fn repair_dry_run_writes_nothing_hashes_unchanged() {
    let tree = receipt_tree("dry");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    let index = tree.join("web/index.html");
    let receipt = tree.join("install-receipt.json");
    let decoy = tree.join("goldens/bank_hash.txt");
    fs::create_dir_all(decoy.parent().unwrap()).expect("mkdir goldens");
    fs::write(&decoy, b"do-not-touch\n").expect("plant decoy golden");
    let watched = [index, wasm, receipt, decoy];
    assert_eq!(watched.len(), 4, "dry-run watch list must not shrink");
    for p in &watched {
        set_mtime_past(p);
    }
    let before = fingerprint(&watched);

    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args(["repair", "--dry-run", "--root", tree.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("dry-run") && stdout.contains("writes nothing"),
        "dry-run must say it writes nothing: {stdout}"
    );

    let after = fingerprint(&watched);
    assert_eq!(
        before, after,
        "dry-run must leave dest hashes and mtimes unchanged"
    );
    assert!(
        !tree.join("web/data/units_index.json").exists(),
        "dry-run must not invent web/data from bank/"
    );

    // Bare `cdcp repair` is the same as --dry-run (breaking change: no mutate).
    cdcp()
        .env_remove("CDCP_DEV")
        .args(["repair", "--root", tree.to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(
        before,
        fingerprint(&watched),
        "bare repair must default to dry-run and write nothing"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// `--apply` on a matching receipt is a no-op. Second run planned_restore=0.
#[test]
fn repair_apply_matching_receipt_is_idempotent_noop() {
    let tree = receipt_tree("apply-ok");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    let index = tree.join("web/index.html");
    let receipt = tree.join("install-receipt.json");
    let watched = [index.clone(), wasm.clone(), receipt.clone()];
    for p in &watched {
        set_mtime_past(p);
    }
    let before = fingerprint(&watched);

    let first = cdcp()
        .env_remove("CDCP_DEV")
        .args([
            "repair",
            "--apply",
            "--json",
            "--root",
            tree.to_str().unwrap(),
        ])
        .assert()
        .success();
    let v1: Value =
        serde_json::from_str(String::from_utf8_lossy(&first.get_output().stdout).trim())
            .expect("first --json");
    assert_eq!(v1["schema_version"].as_u64(), Some(1));
    assert_eq!(v1["mode"].as_str(), Some("apply"));
    assert_eq!(v1["ok"].as_bool(), Some(true));
    assert_eq!(v1["planned_restore"].as_u64(), Some(0));
    assert_eq!(v1["actual_restore"].as_u64(), Some(0));
    let actual = v1["actual"].as_array().expect("actual");
    assert!(
        actual.is_empty(),
        "matching apply writes nothing: {actual:?}"
    );
    let planned = v1["planned"].as_array().expect("planned");
    assert!(
        !planned.is_empty(),
        "planned must list the receipt files — empty is vacuous"
    );
    for row in planned {
        assert_eq!(row["status"].as_str(), Some("ok"));
    }

    let second = cdcp()
        .env_remove("CDCP_DEV")
        .args([
            "repair",
            "--apply",
            "--json",
            "--root",
            tree.to_str().unwrap(),
        ])
        .assert()
        .success();
    let v2: Value =
        serde_json::from_str(String::from_utf8_lossy(&second.get_output().stdout).trim())
            .expect("second --json");
    assert_eq!(
        v2["planned_restore"].as_u64(),
        Some(0),
        "second apply must be planned=0 / no-op: {v2}"
    );
    assert_eq!(
        before,
        fingerprint(&watched),
        "apply on a matching receipt must not write dest files"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// No receipt → refuse, not guess. Empty dest + no receipt is RED.
#[test]
fn repair_no_receipt_refuses_and_names_the_absolute_path() {
    let empty = uniq("repair-empty");
    fs::create_dir_all(&empty).expect("mkdir");
    let rec = empty.join("install-receipt.json");
    let rec_abs = rec.display().to_string();
    assert!(!rec.exists(), "planted dest must have no receipt");

    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args(["repair", "--root", empty.to_str().unwrap()])
        .assert()
        .failure();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        combined.contains(&rec_abs),
        "missing receipt must name the absolute path {rec_abs}, got: {combined}"
    );
    assert!(
        combined.contains("refus") || combined.contains("guess"),
        "missing receipt must refuse, not guess: {combined}"
    );
    assert!(
        !empty.join("web").exists() && !empty.join("goldens").exists(),
        "refuse must not invent a tree"
    );

    let _ = fs::remove_dir_all(&empty);
}

/// Drifted wasm hash: name the path + expected sha256, write nothing, exit RED.
#[test]
fn repair_drifted_wasm_hash_is_red_and_does_not_invent() {
    let tree = receipt_tree("drift-wasm");
    let wasm = tree.join("web/assets/wasm/cdcp_wasm.wasm");
    let index = tree.join("web/index.html");
    let receipt = tree.join("install-receipt.json");
    let expected = file_sha256(&wasm);
    let original = fs::read(&wasm).expect("read wasm");
    fs::write(&wasm, b"not-the-shipped-wasm\n").expect("drift wasm");
    let drifted = file_sha256(&wasm);
    assert_ne!(expected, drifted, "plant must actually change the hash");
    set_mtime_past(&wasm);
    set_mtime_past(&index);
    set_mtime_past(&receipt);
    let before = fingerprint(&[index.clone(), wasm.clone(), receipt.clone()]);

    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args([
            "repair",
            "--apply",
            "--json",
            "--root",
            tree.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let combined = format!("{stdout}\n{stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("drift --json must still emit an envelope ({e}): {stdout}");
    });
    assert_eq!(v["ok"].as_bool(), Some(false));
    assert!(
        v["planned_restore"].as_u64().unwrap_or(0) >= 1,
        "drift must plan a restore: {v}"
    );
    assert_eq!(
        v["actual_restore"].as_u64(),
        Some(0),
        "apply must not invent bytes: {v}"
    );
    let wasm_abs = wasm.display().to_string();
    assert!(
        combined.contains(&wasm_abs),
        "drift must name the absolute wasm path {wasm_abs}, got: {combined}"
    );
    assert!(
        combined.contains(&expected),
        "drift must name the expected sha256 {expected}, got: {combined}"
    );
    assert!(
        combined.contains("hash-mismatch") || combined.contains("drifted"),
        "drift must say hash-mismatch: {combined}"
    );

    let after_bytes = fs::read(&wasm).expect("re-read wasm");
    assert_eq!(
        after_bytes, b"not-the-shipped-wasm\n",
        "apply must not rewrite a drifted wasm (cannot invent from bank/)"
    );
    assert_ne!(
        after_bytes.as_slice(),
        original.as_slice(),
        "sanity: drifted bytes must still differ from the original"
    );
    assert_eq!(
        before,
        fingerprint(&[index, wasm, receipt]),
        "apply-on-drift must write nothing"
    );
    assert!(
        !tree.join("goldens").exists(),
        "repair must never create goldens/"
    );

    // Dry-run on the same drift also writes nothing (it REPORTS).
    cdcp()
        .env_remove("CDCP_DEV")
        .args(["repair", "--dry-run", "--root", tree.to_str().unwrap()])
        .assert()
        .failure();
    assert_eq!(
        fs::read(tree.join("web/assets/wasm/cdcp_wasm.wasm")).unwrap(),
        b"not-the-shipped-wasm\n"
    );

    let _ = fs::remove_dir_all(&tree);
}

/// Empty `files[]` is RED — a receipt that pins nothing certifies nothing.
#[test]
fn repair_empty_files_array_is_error() {
    let tree = web_only_tree("empty-files");
    let rec = serde_json::json!({
        "version": "0.1.0",
        "files": []
    });
    fs::write(
        tree.join("install-receipt.json"),
        serde_json::to_vec(&rec).unwrap(),
    )
    .unwrap();
    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args(["repair", "--root", tree.to_str().unwrap()])
        .assert()
        .failure();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        combined.contains("0 files") || combined.contains("pins nothing"),
        "empty files[] must be an ERROR, got: {combined}"
    );
    let _ = fs::remove_dir_all(&tree);
}

/// Authoring rebuild (CDCP_DEV=1 --apply, no receipt, bank present) is
/// idempotent: run twice, second run writes nothing. Asserted by mtime.
#[test]
fn repair_authoring_apply_second_run_writes_nothing_by_mtime() {
    let tree = operator_tree("repair-auth");
    let watched = [
        tree.join("web/data/units_index.json"),
        tree.join("web/data/glossary.json"),
        tree.join("web/data/module_learn_slugs.js"),
        tree.join("web/data/mock40_seed42.json"),
        tree.join("web/data/keys_seed42.json"),
        tree.join("web/data/bank_items_seed42.json"),
    ];
    assert_eq!(
        watched.len(),
        6,
        "authoring repair must rebuild six artifacts"
    );

    cdcp()
        .env("CDCP_DEV", "1")
        .args(["repair", "--apply", "--root", tree.to_str().unwrap()])
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
        .env("CDCP_DEV", "1")
        .args(["repair", "--apply", "--root", tree.to_str().unwrap()])
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
        .env("CDCP_DEV", "1")
        .args(["repair", "--apply", "--root", tree.to_str().unwrap()])
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

/// Learner dry-run on a receipt tree must not create or rewrite goldens/.
#[test]
fn repair_learner_never_writes_goldens() {
    let tree = receipt_tree("no-goldens");
    assert!(
        !tree.join("goldens").exists(),
        "receipt tree must start without goldens/"
    );
    cdcp()
        .env_remove("CDCP_DEV")
        .env("UPDATE_GOLDENS", "1")
        .args(["repair", "--apply", "--root", tree.to_str().unwrap()])
        .assert()
        .success();
    assert!(
        !tree.join("goldens").exists(),
        "learner repair must never create goldens/"
    );
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
    let from_receipt = src
        .find("fn repair_from_receipt(")
        .expect("repair_from_receipt must exist");
    let receipt_body = &src[from_receipt..];
    for forbidden in ["export_web", "repair_learn", "repair_authoring("] {
        assert!(
            !receipt_body
                .split("fn repair_authoring(")
                .next()
                .unwrap_or(receipt_body)
                .contains(forbidden),
            "learner repair_from_receipt must not call {forbidden}"
        );
    }
}

/// repair on an empty tree is an ERROR, not a green no-op.
#[test]
fn repair_on_an_empty_tree_is_error() {
    let empty = uniq("repair-empty-legacy");
    fs::create_dir_all(&empty).expect("mkdir");
    let rec_abs = empty.join("install-receipt.json").display().to_string();
    let assert = cdcp()
        .env_remove("CDCP_DEV")
        .args(["repair", "--root", empty.to_str().unwrap()])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains(&rec_abs) || stderr.contains("install-receipt.json"),
        "repair on an empty tree must name the missing receipt, got: {stderr}"
    );
    let _ = fs::remove_dir_all(&empty);
}
