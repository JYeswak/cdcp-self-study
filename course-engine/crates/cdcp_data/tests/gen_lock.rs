//! Writer for `content.lock` (bd-substrate-rust-migration-jhd.30).
//!
//! EXTRACT-THEN-DELETE: this is NOT a differential against
//! `scripts/gen_content_lock.py`. The Python is deleted in the same commit.
//! Identity is the live lock's pin tables. Known-bad: empty knowledge,
//! empty modules, snapshots.toml naming missing files, a nested corpus
//! toml that must not appear.

use cdcp_data::{
    generate_content_lock, lock_section, sha256_hex, write_content_lock, DataError, CANONICAL,
    LOCK_REL, SNAPSHOTS_REL,
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
        "cdcp-gen-lock-{}-{}-{}-{}",
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

const BANK64: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn one_pin_snapshots() -> &'static str {
    r#"schema = "cdcp.data.snapshots.v1"

[[snapshot]]
id = "src-fixture-hello"
body = "knowledge/corpus/public/osha/hello.txt"
sidecar = "knowledge/corpus/public/osha/hello.meta.toml"
sha256 = "00"
"#
}

fn plant_min(root: &Path) {
    write_rel(root, "knowledge/a.toml", b"schema = 1\n");
    write_rel(root, "knowledge/corpus/deep.toml", b"smuggled = true\n");
    write_rel(root, "web/content/modules/m.md", b"# m\n");
    if let Some(parent) = root.parent() {
        write_rel(parent, "modules/m.md", b"# parent m\n");
    }
    write_rel(root, SNAPSHOTS_REL, one_pin_snapshots().as_bytes());
    write_rel(
        root,
        "knowledge/corpus/public/osha/hello.txt",
        b"hello-body\n",
    );
    write_rel(
        root,
        "knowledge/corpus/public/osha/hello.meta.toml",
        b"source_id = \"src-fixture-hello\"\n",
    );
}

#[test]
fn python_generator_is_gone() {
    assert!(
        !engine().join("scripts/gen_content_lock.py").exists(),
        "EXTRACT-THEN-DELETE: scripts/gen_content_lock.py must stay gone"
    );
}

#[test]
fn writer_pin_tables_match_live_lock() {
    let root = engine();
    let live = fs::read_to_string(root.join(LOCK_REL)).expect("live content.lock");
    let bank_hash = live_scalar(&live, "bank_hash");
    let generated = generate_content_lock(&root, &bank_hash).expect("generate");

    let mut live_n = 0usize;
    let mut matched = 0usize;
    for section in ["knowledge", "modules", "data"] {
        let live_map = pin_map(&lock_section(&live, section));
        let gen_map = pin_map(&lock_section(&generated, section));
        assert_eq!(
            live_map.keys().collect::<Vec<_>>(),
            gen_map.keys().collect::<Vec<_>>(),
            "[{section}] path set must match the live lock"
        );
        for (path, live_hash) in &live_map {
            let gen_hash = gen_map.get(path).expect("path set checked");
            let abs = resolve_pinned(&root, path);
            let bytes = fs::read(&abs).unwrap_or_else(|e| panic!("read {path}: {e}"));
            let disk = sha256_hex(&bytes);
            assert_eq!(
                gen_hash, &disk,
                "writer hash for {section}/{path} must be the bytes on disk"
            );
            if live_hash == &disk {
                assert_eq!(
                    gen_hash, live_hash,
                    "[{section}] {path} is current — writer must match the live lock byte-for-byte"
                );
                matched += 1;
            }
            live_n += 1;
        }
    }
    assert!(live_n >= 30, "only {live_n} live pins — vacuous identity");
    assert!(
        matched >= 28,
        "only {matched}/{live_n} live pins still match disk — freeze the lock or the files"
    );
    assert!(
        generated.contains(&format!("canonical = \"{CANONICAL}\"")),
        "writer must emit CANONICAL"
    );
}

#[test]
fn writer_writes_to_a_temp_tree_and_omits_nested_corpus() {
    let tmp = scratch("plant");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    plant_min(&root);

    let dest = root.join("out.lock");
    let report = write_content_lock(&root, BANK64, &dest).expect("write");
    assert_eq!(report.knowledge, 1);
    assert_eq!(report.modules, 2);
    assert_eq!(report.data, 2);
    assert_eq!(report.bank_hash, BANK64);

    let text = fs::read_to_string(&dest).expect("read dest");
    assert!(text.contains("[knowledge]"));
    assert!(text.contains("[modules]"));
    assert!(text.contains("[data]"));
    assert!(
        !text.contains("knowledge/corpus/deep.toml"),
        "nested corpus toml must not be pinned: {text}"
    );
    assert!(text.contains("\"knowledge/a.toml\""));
    assert!(text.contains("\"web/content/modules/m.md\""));
    assert!(text.contains("\"modules/m.md\""));
    assert!(text.contains("\"knowledge/corpus/public/osha/hello.txt\""));
    assert!(text.contains("\"knowledge/corpus/public/osha/hello.meta.toml\""));
    assert!(text.contains("cdcp content-lock"));
    assert!(!text.contains("python3 scripts/gen_content_lock.py"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn empty_knowledge_is_red_and_writes_nothing() {
    let tmp = scratch("no-knowledge");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    plant_min(&root);
    fs::remove_file(root.join("knowledge/a.toml")).unwrap();

    let dest = root.join(LOCK_REL);
    let err = write_content_lock(&root, BANK64, &dest).expect_err("empty knowledge");
    assert!(matches!(err, DataError::EmptyKnowledge), "{err:?}");
    assert!(!dest.exists(), "RED must write nothing");
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn empty_modules_is_red() {
    let tmp = scratch("no-modules");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    plant_min(&root);
    fs::remove_file(root.join("web/content/modules/m.md")).unwrap();
    if let Some(parent) = root.parent() {
        let _ = fs::remove_file(parent.join("modules/m.md"));
    }

    let err = generate_content_lock(&root, BANK64).expect_err("empty modules");
    assert!(matches!(err, DataError::EmptyModules), "{err:?}");
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn snapshots_naming_missing_files_is_red() {
    let tmp = scratch("missing-snap");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    plant_min(&root);
    fs::remove_file(root.join("knowledge/corpus/public/osha/hello.txt")).unwrap();

    let err = generate_content_lock(&root, BANK64).expect_err("missing snapshot body");
    match err {
        DataError::SnapshotFilesMissing { paths } => {
            assert!(paths.contains("hello.txt"), "{paths}");
        }
        other => panic!("expected SnapshotFilesMissing, got {other:?}"),
    }
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn snapshots_nonempty_and_zero_data_files_is_already_red_in_data_lock() {
    // The writer refuses missing files before it can emit an empty [data].
    // The verifier still refuses a hand-written lock with empty [data].
    let src = include_str!("../src/data_lock.rs");
    assert!(src.contains("EmptyDataLock"));
    assert!(src.contains("ANTI_VACUOUS_DATA_LOCK"));
}

#[test]
fn invalid_bank_hash_is_red_before_write() {
    let tmp = scratch("bad-hash");
    let root = tmp.join("engine");
    fs::create_dir_all(&root).unwrap();
    plant_min(&root);
    let dest = root.join(LOCK_REL);
    let err = write_content_lock(&root, "nope", &dest).expect_err("bad hash");
    assert!(matches!(err, DataError::InvalidBankHash), "{err:?}");
    assert!(!dest.exists());
    let _ = fs::remove_dir_all(&tmp);
}

fn live_scalar(text: &str, key: &str) -> String {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim().strip_prefix('=').unwrap_or(rest).trim();
        if let Some(inner) = rest.strip_prefix('"').and_then(|s| s.split('"').next()) {
            return inner.to_string();
        }
    }
    panic!("no {key} in live lock");
}

fn pin_map(section: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for line in section.lines() {
        if let Some((path, hx)) = parse_pin_line(line) {
            out.insert(path.to_string(), hx.to_string());
        }
    }
    out
}

fn parse_pin_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if !line.starts_with('"') {
        return None;
    }
    let rest = &line[1..];
    let end = rest.find('"')?;
    let path = &rest[..end];
    let rest = rest[end + 1..].trim();
    let rest = rest.strip_prefix('=')?.trim();
    let rest = rest.strip_prefix('"')?;
    let hex_end = rest.find('"')?;
    Some((path, &rest[..hex_end]))
}

fn resolve_pinned(root: &Path, rel: &str) -> PathBuf {
    let cand = root.join(rel);
    if cand.is_file() {
        return cand;
    }
    root.parent().unwrap_or(root).join(rel)
}
