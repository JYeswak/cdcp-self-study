//! Known-good + known-bad injection for the snapshot loader.
//!
//! Plants:
//! - strip the licence line → refuse
//! - PROHIBITED → refuse
//! - redistribution != permitted → refuse
//! - corrupt one body byte → hash mismatch names both hashes
//! - empty pin list → ERROR
//! - registered ≥1, nothing on disk → ERROR
//!
//! Meta-tests: deleting the may_load call or the hash comparison is RED.

use cdcp_data::{
    compiled_pins, engine_root, load_compiled, load_one, load_registry, parse_pins, sha256_hex,
    DataError, SnapshotPin, ANTI_VACUOUS_EMPTY, ANTI_VACUOUS_NONE_LOADED, COMPILED_PINS,
    HASH_MISMATCH, MISSING_LICENCE_REFUSAL,
};
use cdcp_evidence::LicenceFault;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const GOOD_BODY: &[u8] = b"cdcp-data fixture v1\n";
const GOOD_HASH: &str = "d68e259dc654ec6b91fd06697dfc9182b48e735b12787cd58d7d52ba9842537b";

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn good_root() -> PathBuf {
    fixtures().join("good")
}

fn good_pin() -> SnapshotPin {
    SnapshotPin {
        id: "src-fixture-hello".into(),
        body: "knowledge/corpus/snapshots/hello.txt".into(),
        sidecar: "knowledge/corpus/snapshots/hello.meta.toml".into(),
        sha256: GOOD_HASH.into(),
    }
}

static SEQ: AtomicU64 = AtomicU64::new(1);

fn scratch(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cdcp-data-{}-{}-{}-{}",
        tag,
        std::process::id(),
        n,
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn write_snapshot(root: &Path, pin: &SnapshotPin, meta: &str, body: &[u8]) {
    let side = root.join(&pin.sidecar);
    let body_path = root.join(&pin.body);
    if let Some(p) = side.parent() {
        fs::create_dir_all(p).unwrap();
    }
    if let Some(p) = body_path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&side, meta).unwrap();
    fs::write(&body_path, body).unwrap();
}

fn permitted_meta(id: &str, body: &str, hash: &str) -> String {
    format!(
        r#"source_id = "{id}"
rights = "own-work-this-repo"
licence = "own-work-this-repo"
redistribution = "permitted"
ai_ingestion = "permitted"
capture = "body-retained"
path = "{body}"
sha256 = "{hash}"
"#
    )
}

fn production_src() -> &'static str {
    include_str!("../src/lib.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests")
}

#[test]
fn known_good_fixture_loads() {
    let report = load_registry(&good_root(), &[good_pin()]).expect("good fixture");
    assert_eq!(report.loaded.len(), 1);
    let s = &report.loaded[0];
    assert_eq!(s.id, "src-fixture-hello");
    assert_eq!(s.sha256, GOOD_HASH);
    assert_eq!(s.bytes, GOOD_BODY);
    assert!(s.eligible_for_agent_index);
    assert!(report.to_string().contains("load_snapshots: PASS"));
}

#[test]
fn strip_licence_line_is_refused() {
    let root = scratch("strip-licence");
    let pin = good_pin();
    let meta = r#"source_id = "src-fixture-hello"
redistribution = "permitted"
ai_ingestion = "permitted"
capture = "body-retained"
path = "knowledge/corpus/snapshots/hello.txt"
sha256 = "d68e259dc654ec6b91fd06697dfc9182b48e735b12787cd58d7d52ba9842537b"
"#;
    write_snapshot(&root, &pin, meta, GOOD_BODY);
    let err = load_one(&root, &pin).expect_err("stripped licence must refuse");
    match &err {
        DataError::Refused {
            id,
            fault: LicenceFault::MissingRights { field, .. },
        } => {
            assert_eq!(id, "src-fixture-hello");
            assert_eq!(*field, "rights");
        }
        other => panic!("expected MissingRights refusal, got {other:?}"),
    }
    assert!(
        err.to_string().contains("refusing to load"),
        "refusal must not be a warning: {err}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn prohibited_is_refused() {
    let root = scratch("prohibited");
    let pin = good_pin();
    let meta = r#"source_id = "src-fixture-hello"
rights = "publisher-copyright"
licence = "publisher-copyright"
redistribution = "permitted"
ai_ingestion = "PROHIBITED"
capture = "body-retained"
path = "knowledge/corpus/snapshots/hello.txt"
sha256 = "d68e259dc654ec6b91fd06697dfc9182b48e735b12787cd58d7d52ba9842537b"
"#;
    write_snapshot(&root, &pin, meta, GOOD_BODY);
    let err = load_one(&root, &pin).expect_err("PROHIBITED must refuse");
    match err {
        DataError::Refused { id, fault } => {
            assert_eq!(id, "src-fixture-hello");
            assert!(
                matches!(fault, LicenceFault::ProhibitedInAgentIndex { .. }),
                "{fault:?}"
            );
        }
        other => panic!("expected PROHIBITED refusal, got {other:?}"),
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn redistribution_not_permitted_is_refused() {
    let root = scratch("not-permitted");
    let pin = good_pin();
    let meta = r#"source_id = "src-fixture-hello"
rights = "own-work-this-repo"
licence = "own-work-this-repo"
redistribution = "not-licensed"
ai_ingestion = "permitted"
capture = "body-retained"
path = "knowledge/corpus/snapshots/hello.txt"
sha256 = "d68e259dc654ec6b91fd06697dfc9182b48e735b12787cd58d7d52ba9842537b"
"#;
    write_snapshot(&root, &pin, meta, GOOD_BODY);
    let err = load_one(&root, &pin).expect_err("not-licensed must refuse");
    match err {
        DataError::Refused { id, fault } => {
            assert_eq!(id, "src-fixture-hello");
            assert!(
                matches!(fault, LicenceFault::PublishedUnlicensed { .. }),
                "{fault:?}"
            );
        }
        other => panic!("expected PublishedUnlicensed refusal, got {other:?}"),
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn corrupt_one_byte_is_hash_mismatch_naming_both() {
    let root = scratch("corrupt-byte");
    let pin = good_pin();
    let mut body = GOOD_BODY.to_vec();
    body[0] ^= 0xff;
    let computed = sha256_hex(&body);
    assert_ne!(computed, GOOD_HASH);
    write_snapshot(
        &root,
        &pin,
        &permitted_meta(&pin.id, &pin.body, GOOD_HASH),
        &body,
    );
    let err = load_one(&root, &pin).expect_err("corrupt byte must RED");
    match &err {
        DataError::HashMismatch {
            id,
            recorded,
            computed: got,
        } => {
            assert_eq!(id, "src-fixture-hello");
            assert_eq!(recorded, GOOD_HASH);
            assert_eq!(got, &computed);
            assert_ne!(recorded, got);
        }
        other => panic!("expected HashMismatch, got {other:?}"),
    }
    let text = err.to_string();
    assert!(text.contains(HASH_MISMATCH), "{text}");
    assert!(text.contains(GOOD_HASH), "must name recorded: {text}");
    assert!(text.contains(&computed), "must name computed: {text}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn empty_pin_list_is_error() {
    let err = load_registry(&good_root(), &[]).expect_err("empty registry");
    assert!(matches!(err, DataError::EmptyRegistry), "{err:?}");
    assert!(err.to_string().contains(ANTI_VACUOUS_EMPTY));
}

#[test]
fn registered_but_nothing_on_disk_is_error() {
    let root = scratch("empty-disk");
    let err = load_registry(&root, &[good_pin()]).expect_err("registered, nothing loaded");
    match err {
        DataError::NoneLoaded { registered, faults } => {
            assert_eq!(registered, 1);
            assert!(!faults.is_empty());
        }
        other => panic!("expected NoneLoaded, got {other:?}"),
    }
    let err = load_registry(&root, &[good_pin()]).unwrap_err();
    assert!(err.to_string().contains(ANTI_VACUOUS_NONE_LOADED));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn missing_sidecar_sha256_is_error() {
    let root = scratch("no-hash");
    let pin = good_pin();
    let meta = r#"source_id = "src-fixture-hello"
rights = "own-work-this-repo"
licence = "own-work-this-repo"
redistribution = "permitted"
ai_ingestion = "permitted"
capture = "body-retained"
path = "knowledge/corpus/snapshots/hello.txt"
"#;
    write_snapshot(&root, &pin, meta, GOOD_BODY);
    let err = load_one(&root, &pin).expect_err("missing recorded sha256");
    assert!(
        matches!(err, DataError::MissingRecordedHash { .. }),
        "{err:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unknown_ingestion_loads_but_is_not_index_eligible() {
    let root = scratch("unknown-ai");
    let pin = good_pin();
    let meta = r#"source_id = "src-fixture-hello"
rights = "own-work-this-repo"
licence = "own-work-this-repo"
redistribution = "permitted"
ai_ingestion = "unknown"
capture = "body-retained"
path = "knowledge/corpus/snapshots/hello.txt"
sha256 = "d68e259dc654ec6b91fd06697dfc9182b48e735b12787cd58d7d52ba9842537b"
"#;
    write_snapshot(&root, &pin, meta, GOOD_BODY);
    let s = load_one(&root, &pin).expect("Unknown may load");
    assert!(
        !s.eligible_for_agent_index,
        "eligible_for_agent_index must exclude Unknown"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn live_compiled_pins_load() {
    let engine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let engine = engine.canonicalize().expect("engine root");
    let report = load_compiled(&engine).expect("live compiled pins");
    assert!(
        report.loaded.iter().any(|s| s.id == "src-nist-sp800-123"),
        "{report}"
    );
    let nist = report
        .loaded
        .iter()
        .find(|s| s.id == "src-nist-sp800-123")
        .unwrap();
    assert_eq!(
        nist.sha256,
        "182ae5d23011108fba08c3a56c5029b8a26a428480ae38ac61877f6f3365db41"
    );
    assert!(nist.eligible_for_agent_index);
    assert!(!nist.bytes.is_empty());
}

#[test]
fn engine_root_finds_claims_toml() {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = engine_root(&start).expect("engine root");
    assert!(root.join("registries/claims.toml").is_file());
}

/// Meta-test: delete the may_load call → this selftest is non-zero.
#[test]
fn selftest_delete_may_load_is_nonzero() {
    let src = production_src();
    assert!(
        src.contains("may_load(&meta)"),
        "delete the may_load call → selftest non-zero"
    );
    assert!(
        src.contains("MISSING_LICENCE_REFUSAL"),
        "delete the refusal token interpolation → selftest non-zero"
    );
    assert!(
        MISSING_LICENCE_REFUSAL.contains("lacks a licence line"),
        "{MISSING_LICENCE_REFUSAL}"
    );
}

/// Meta-test: delete the hash comparison → this selftest is non-zero.
#[test]
fn selftest_delete_hash_compare_is_nonzero() {
    let src = production_src();
    assert!(
        src.contains("HASH_MISMATCH"),
        "delete the hash-mismatch token interpolation → selftest non-zero"
    );
    assert!(
        src.contains("recorded") && src.contains("computed"),
        "delete the both-hashes naming → selftest non-zero"
    );
    assert!(
        src.contains("hash_eq(&computed, &pin.sha256)"),
        "delete the body-vs-pin comparison → selftest non-zero"
    );
}

/// Meta-test: delete the empty-set ERROR → this selftest is non-zero.
#[test]
fn selftest_delete_anti_vacuous_is_nonzero() {
    let src = production_src();
    assert!(src.contains("ANTI_VACUOUS_EMPTY"));
    assert!(src.contains("ANTI_VACUOUS_NONE_LOADED"));
    assert!(src.contains("EmptyRegistry"));
    assert!(src.contains("NoneLoaded"));
}

#[test]
fn compiled_pins_text_is_the_include() {
    let pins = compiled_pins().expect("compiled");
    let parsed = parse_pins(COMPILED_PINS, "snapshots.toml").expect("parse include");
    assert_eq!(pins, parsed);
    assert!(!pins.is_empty());
}
