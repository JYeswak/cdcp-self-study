//! content.lock [data] covers every snapshots.toml file (bd-hardening-e-data-4up.3).
//!
//! Plants:
//! - snapshots.toml non-empty + [data] empty → ERROR
//! - omit one referenced path → unpinned RED
//! - flip one byte of a vendored body → hash mismatch RED
//!
//! The committed tree is never written.

use cdcp_data::{
    load_pins_from_disk, parse_data_section, referenced_data_paths, selftest_flip_one_byte,
    sha256_hex, verify_data_lock, DataError, ANTI_VACUOUS_DATA_LOCK, LOCK_REL, SNAPSHOTS_REL,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQ: AtomicU64 = AtomicU64::new(1);

fn engine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("engine")
}

fn scratch(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cdcp-data-lock-{}-{}-{}-{}",
        tag,
        std::process::id(),
        n,
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn write_rel(root: &Path, rel: &str, bytes: &[u8]) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, bytes).unwrap();
}

fn one_pin_snapshots() -> &'static str {
    r#"schema = "cdcp.data.snapshots.v1"

[[snapshot]]
id = "src-fixture-hello"
body = "knowledge/corpus/public/osha/hello.txt"
sidecar = "knowledge/corpus/public/osha/hello.meta.toml"
sha256 = "d68e259dc654ec6b91fd06697dfc9182b48e735b12787cd58d7d52ba9842537b"
"#
}

const HELLO_BODY: &[u8] = b"cdcp-data fixture v1\n";
const HELLO_META: &[u8] = b"source_id = \"src-fixture-hello\"\n";

fn hello_hashes() -> (String, String) {
    (sha256_hex(HELLO_BODY), sha256_hex(HELLO_META))
}

fn plant_one_pin(root: &Path, data_section: &str) {
    write_rel(root, SNAPSHOTS_REL, one_pin_snapshots().as_bytes());
    write_rel(root, "knowledge/corpus/public/osha/hello.txt", HELLO_BODY);
    write_rel(
        root,
        "knowledge/corpus/public/osha/hello.meta.toml",
        HELLO_META,
    );
    let lock = format!("schema_version = 1\nbank_hash = \"00\"\n\n{data_section}");
    write_rel(root, LOCK_REL, lock.as_bytes());
}

#[test]
fn live_tree_lock_lists_every_snapshots_file() {
    let root = engine();
    let pins = load_pins_from_disk(&root).expect("live snapshots.toml");
    assert!(
        !pins.is_empty(),
        "live snapshots.toml is empty — a loader that registers nothing certifies nothing"
    );
    let required = referenced_data_paths(&pins);
    assert!(
        required.iter().any(|p| p.contains("29cfr-1910.147")),
        "OSHA 147 must be referenced: {required:?}"
    );
    assert!(
        required.iter().any(|p| p.contains("nist_sp800_123")),
        "NIST body must be referenced: {required:?}"
    );
    for rel in &required {
        assert!(
            root.join(rel).is_file(),
            "snapshots.toml names {rel} but the file is missing"
        );
    }

    let lock = fs::read_to_string(root.join(LOCK_REL)).expect("content.lock");
    let pinned = parse_data_section(&lock, LOCK_REL).expect("parse [data]");
    assert!(!pinned.is_empty(), "{ANTI_VACUOUS_DATA_LOCK}");
    for rel in &required {
        assert!(
            pinned.contains_key(rel),
            "content.lock [data] missing {rel}"
        );
    }

    let report = verify_data_lock(&root).expect("live data lock");
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.required, required.len());
    assert!(report.pinned >= required.len());
    assert!(report.to_string().contains("verify_data_lock: PASS"));
}

#[test]
fn snapshots_nonempty_and_zero_data_rows_is_error() {
    let root = scratch("empty-data");
    plant_one_pin(&root, "[data]\n");
    let err = verify_data_lock(&root).expect_err("empty [data] must RED");
    match err {
        DataError::EmptyDataLock { registered } => assert_eq!(registered, 1),
        other => panic!("expected EmptyDataLock, got {other:?}"),
    }
    let err = verify_data_lock(&root).unwrap_err();
    assert!(err.to_string().contains(ANTI_VACUOUS_DATA_LOCK), "{err}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn missing_data_section_is_empty_lock_error() {
    let root = scratch("no-section");
    plant_one_pin(&root, "[knowledge]\n\"x\" = \"y\"\n");
    let err = verify_data_lock(&root).expect_err("missing [data] must RED");
    assert!(matches!(err, DataError::EmptyDataLock { .. }), "{err:?}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn omitted_referenced_path_is_unpinned() {
    let root = scratch("omit");
    let (body_h, _meta_h) = hello_hashes();
    plant_one_pin(
        &root,
        &format!("[data]\n\"knowledge/corpus/public/osha/hello.txt\" = \"{body_h}\"\n"),
    );
    let err = verify_data_lock(&root).expect_err("omitted sidecar must RED");
    let faults = match err {
        DataError::DataLockFailed { faults } => faults,
        other => panic!("expected DataLockFailed, got {other:?}"),
    };
    assert!(
        faults.iter().any(|f| matches!(
            f,
            DataError::DataUnpinned { path } if path.ends_with("hello.meta.toml")
        )),
        "{faults:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn flip_one_byte_of_vendored_body_is_red() {
    let root = scratch("flip");
    let (body_h, meta_h) = hello_hashes();
    plant_one_pin(
        &root,
        &format!(
            "[data]\n\
             \"knowledge/corpus/public/osha/hello.txt\" = \"{body_h}\"\n\
             \"knowledge/corpus/public/osha/hello.meta.toml\" = \"{meta_h}\"\n"
        ),
    );
    verify_data_lock(&root).expect("faithful plant is green");

    let body_path = root.join("knowledge/corpus/public/osha/hello.txt");
    let mut body = fs::read(&body_path).unwrap();
    body[0] ^= 0xff;
    let computed = sha256_hex(&body);
    assert_ne!(computed, body_h);
    fs::write(&body_path, &body).unwrap();

    let err = verify_data_lock(&root).expect_err("flipped body must RED");
    let faults = match err {
        DataError::DataLockFailed { faults } => faults,
        other => panic!("expected DataLockFailed, got {other:?}"),
    };
    match &faults[..] {
        [DataError::DataHashMismatch {
            path,
            recorded,
            computed: got,
        }] => {
            assert_eq!(path, "knowledge/corpus/public/osha/hello.txt");
            assert_eq!(recorded, &body_h);
            assert_eq!(got, &computed);
        }
        other => panic!("expected one hash mismatch, got {other:?}"),
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn live_flip_selftest_trips_red_without_touching_the_tree() {
    let root = engine();
    let before = fs::read(root.join("knowledge/corpus/public/osha/29cfr-1910.147.txt"))
        .expect("osha 147 body");
    let msg = selftest_flip_one_byte(&root).expect("selftest");
    let after = fs::read(root.join("knowledge/corpus/public/osha/29cfr-1910.147.txt"))
        .expect("osha 147 body after");
    assert_eq!(before, after, "selftest must not write the committed body");
    assert!(msg.contains("flip-selftest trips RED"), "{msg}");
    assert!(msg.contains("29cfr-1910.147.txt"), "{msg}");
}

#[test]
fn gen_lock_writer_emits_the_data_section() {
    let gen = include_str!("../src/gen_lock.rs");
    assert!(
        gen.contains("[data]"),
        "generator must emit the [data] section or a regen wipes the pin"
    );
    assert!(
        gen.contains("hash_snapshot_files"),
        "generator must collect snapshots.toml paths"
    );
    let production = gen
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(
        !production.contains("WalkDir") && !production.contains("rglob"),
        "do not recurse the corpus; pin only snapshot-referenced files"
    );
}

#[test]
fn selftest_delete_empty_data_check_is_nonzero() {
    let src = include_str!("../src/data_lock.rs");
    assert!(src.contains("ANTI_VACUOUS_DATA_LOCK"));
    assert!(src.contains("pinned.is_empty()"));
    assert!(src.contains("EmptyDataLock"));
}
