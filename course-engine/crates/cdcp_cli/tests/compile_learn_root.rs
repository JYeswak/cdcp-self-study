//! bd-installability-sm4g.9 — `--root` is the tree, not a walk start.
//!
//! `compile_learn` used to re-resolve via `resolve_engine_root`, so
//! `build-learn --root <emptydir-under-the-engine>` walked up, mutated the
//! live `web/`, wrote nothing to the empty dir, and exited 0.
//!
//! A test that only asserts a non-zero exit is insufficient: the escape is a
//! WRITE. Each case plants a sentinel in the live repo, snapshots the files
//! an escaped compiler would rewrite, and asserts both stay byte-and-mtime
//! identical. The empty dir is a descendant of the engine so a re-walk would
//! find `registries/claims.toml` — a `/tmp` empty dir would make the old
//! walk fail closed and the test pass for the wrong reason.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The six verbs that route through `compile_learn`.
const LEARN_VERBS: &[&str] = &[
    "build-learn",
    "build-reference",
    "build-units",
    "build-glossary",
    "build-learn-slugs",
    "smoke-learn",
];

/// Live files an escaped compile would `fs::write` (always-write, not if-changed).
const WRITE_TARGETS: &[&str] = &[
    "web/learn.html",
    "web/data/modules_index.json",
    "web/reference.html",
    "web/data/units_index.json",
    "web/data/glossary.json",
    "web/data/module_learn_slugs.js",
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve course-engine workspace root")
}

fn cdcp() -> Command {
    let mut cmd = Command::cargo_bin("cdcp").expect("cdcp binary");
    cmd.current_dir(workspace_root());
    cmd.env("CDCP_DEV", "1");
    cmd
}

struct Fingerprint {
    bytes: Vec<u8>,
    mtime: SystemTime,
}

fn fingerprint(path: &Path) -> Fingerprint {
    let meta = fs::metadata(path).unwrap_or_else(|e| panic!("stat {}: {e}", path.display()));
    Fingerprint {
        bytes: fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
        mtime: meta.modified().expect("mtime"),
    }
}

fn assert_untouched(path: &Path, before: &Fingerprint, verb: &str) {
    let after = fingerprint(path);
    assert_eq!(
        before.bytes,
        after.bytes,
        "{verb} --root <emptydir> mutated bytes of {}",
        path.display()
    );
    assert_eq!(
        before.mtime,
        after.mtime,
        "{verb} --root <emptydir> mutated mtime of {} (escape is a write)",
        path.display()
    );
}

fn dir_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| {
            e.expect("dirent")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

struct Plant {
    sentinel: PathBuf,
    empty: PathBuf,
}

impl Drop for Plant {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.sentinel);
        let _ = fs::remove_dir_all(&self.empty);
    }
}

fn plant(tag: &str) -> (Plant, Fingerprint, Vec<(PathBuf, Fingerprint)>) {
    let root = workspace_root();
    let nonce = format!(
        "{}_{}_{}",
        tag,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );

    // Descendant of the engine: a re-walk from here finds claims.toml.
    let empty = root.join(format!("target/cdcp-sm4g9-empty_{nonce}"));
    fs::create_dir_all(&empty).expect("mkdir empty root under the engine");
    assert!(
        empty.starts_with(&root),
        "empty root must sit under the engine so a re-walk would escape"
    );
    assert!(
        dir_names(&empty).is_empty(),
        "planted empty root must start empty"
    );

    let sentinel = root.join(format!(
        "web/data/bd-installability-sm4g.9.sentinel_{nonce}"
    ));
    let payload = format!("sm4g.9-sentinel-{nonce}\n");
    fs::write(&sentinel, payload.as_bytes()).expect("plant sentinel in the live repo");
    let plant = Plant {
        sentinel: sentinel.clone(),
        empty,
    };
    let sentinel_fp = fingerprint(&plant.sentinel);
    let targets: Vec<(PathBuf, Fingerprint)> = WRITE_TARGETS
        .iter()
        .map(|rel| {
            let path = root.join(rel);
            assert!(
                path.is_file(),
                "write-target {rel} missing — snapshot would be vacuous"
            );
            (path.clone(), fingerprint(&path))
        })
        .collect();
    (plant, sentinel_fp, targets)
}

fn assert_verb_honors_empty_root(verb: &str) {
    let (plant, sentinel_fp, targets) = plant(verb);
    let output = cdcp()
        .args([verb, "--root"])
        .arg(&plant.empty)
        .output()
        .unwrap_or_else(|e| panic!("spawn {verb}: {e}"));
    let code = output.status.code().unwrap_or(-1);
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(
        code, 0,
        "{verb} --root <emptydir> must exit non-zero, got {code}\n{out}"
    );
    assert!(
        dir_names(&plant.empty).is_empty(),
        "{verb} --root <emptydir> wrote into the empty dir: {:?}\n{out}",
        dir_names(&plant.empty)
    );
    assert_untouched(&plant.sentinel, &sentinel_fp, verb);
    for (path, before) in &targets {
        assert_untouched(path, before, verb);
    }
    println!("{verb} --root <emptydir> exit={code} sentinel_untouched empty_dir_empty");
}

#[test]
fn six_compile_learn_verbs_are_exactly_the_owned_set() {
    assert_eq!(
        LEARN_VERBS,
        [
            "build-learn",
            "build-reference",
            "build-units",
            "build-glossary",
            "build-learn-slugs",
            "smoke-learn",
        ]
    );
    assert_eq!(
        WRITE_TARGETS.len(),
        6,
        "write-target snapshot must cover every compile_learn writer"
    );
}

#[test]
fn build_learn_empty_root_is_red_and_writes_nothing() {
    assert_verb_honors_empty_root("build-learn");
}

#[test]
fn build_reference_empty_root_is_red_and_writes_nothing() {
    assert_verb_honors_empty_root("build-reference");
}

#[test]
fn build_units_empty_root_is_red_and_writes_nothing() {
    assert_verb_honors_empty_root("build-units");
}

#[test]
fn build_glossary_empty_root_is_red_and_writes_nothing() {
    assert_verb_honors_empty_root("build-glossary");
}

#[test]
fn build_learn_slugs_empty_root_is_red_and_writes_nothing() {
    assert_verb_honors_empty_root("build-learn-slugs");
}

#[test]
fn smoke_learn_empty_root_is_red_and_writes_nothing() {
    assert_verb_honors_empty_root("smoke-learn");
}
