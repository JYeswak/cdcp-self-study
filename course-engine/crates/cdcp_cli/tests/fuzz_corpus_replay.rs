//! L5 crash floor: corpus-replay of the committed seed inputs (`bd-p228`).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! The two libFuzzer targets under `fuzz/fuzz_targets/` are not a workspace
//! member, so `cargo test --workspace` never builds them. This suite is the
//! floor that actually gates a change: it feeds the committed seed corpora
//! through the same entry points the libFuzzer targets call
//! (`ChoiceLetter::parse`, `canonical_json` on an arbitrary `Value`).
//!
//! A panic is `ReplayVerdict::Crash`. The live seed walk asserts `Ok` for
//! every file, so a crash fails cargo test and therefore `scripts/check.sh`.
//! `planted_crashing_subject_is_red` plants a panicking subject and requires
//! `Crash`. An empty seed directory is an error, not a pass.
//!
//! # WHAT THIS SUITE CANNOT DECIDE
//!
//! It is corpus-replay, not a live libFuzzer campaign. It cannot discover new
//! inputs, cannot decide the seed set is complete, and cannot decide that a
//! function which does not panic is correct. Property tests cover digest
//! stability and bank_hash reorder.

use cdcp_core::{canonical_json, ChoiceLetter};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// A floor with fewer committed inputs than this is not a floor.
const MIN_SEED_FILES: usize = 8;

const CHOICE_TARGET: &str = "choice_letter_parse";
const JSON_TARGET: &str = "canonical_json_bytes";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplayVerdict {
    Ok,
    Crash,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve course-engine workspace root")
}

fn seed_dir(root: &Path, target: &str) -> PathBuf {
    root.join("fuzz/seed_corpus").join(target)
}

fn list_corpus_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir.is_dir() {
        return Err(format!(
            "{} is not a directory — a missing corpus is an error, not a pass",
            dir.display()
        ));
    }
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!(
            "{} is empty — a floor with no inputs is not a floor",
            dir.display()
        ));
    }
    Ok(files)
}

/// Serialize hook swap: cargo test runs this file's tests in parallel.
static HOOK: Mutex<()> = Mutex::new(());

fn replay_one(data: &[u8], subject: fn(&[u8])) -> ReplayVerdict {
    let _guard = HOOK.lock().unwrap_or_else(|p| p.into_inner());
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subject(data)));
    std::panic::set_hook(prev);
    match result {
        Ok(()) => ReplayVerdict::Ok,
        Err(_) => ReplayVerdict::Crash,
    }
}

/// Same entry point as `fuzz/fuzz_targets/choice_letter_parse.rs`.
fn choice_letter_subject(data: &[u8]) {
    let s = String::from_utf8_lossy(data);
    let _ = ChoiceLetter::parse(&s);
}

/// Same entry point as `fuzz/fuzz_targets/canonical_json_bytes.rs`.
fn canonical_json_subject(data: &[u8]) {
    let Ok(v) = serde_json::from_slice::<Value>(data) else {
        return;
    };
    let _ = canonical_json(&v);
}

fn planted_boom(_data: &[u8]) {
    panic!("planted crash [bd-p228]");
}

fn replay_files(files: &[PathBuf], subject: fn(&[u8])) {
    for path in files {
        let data = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert_eq!(
            replay_one(&data, subject),
            ReplayVerdict::Ok,
            "crash replaying {}",
            path.display()
        );
    }
}

fn uniq(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cdcp_cli_fuzz_{tag}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn maybe_replay_local_working_corpus(root: &Path, target: &str, subject: fn(&[u8])) {
    let dir = root.join("fuzz/corpus").join(target);
    if !dir.is_dir() {
        return;
    }
    let Ok(files) = list_corpus_files(&dir) else {
        return;
    };
    replay_files(&files, subject);
}

/// Live floor: every committed seed file must replay without crash.
#[test]
fn live_seed_corpus_replays_without_crash() {
    let root = workspace_root();
    let choice_files = list_corpus_files(&seed_dir(&root, CHOICE_TARGET))
        .expect("choice_letter_parse seed corpus");
    let json_files =
        list_corpus_files(&seed_dir(&root, JSON_TARGET)).expect("canonical_json_bytes seed corpus");
    assert!(
        choice_files.len() >= MIN_SEED_FILES,
        "choice_letter_parse seed corpus shrank to {} file(s); floor is {MIN_SEED_FILES}",
        choice_files.len()
    );
    assert!(
        json_files.len() >= MIN_SEED_FILES,
        "canonical_json_bytes seed corpus shrank to {} file(s); floor is {MIN_SEED_FILES}",
        json_files.len()
    );
    replay_files(&choice_files, choice_letter_subject);
    replay_files(&json_files, canonical_json_subject);
    maybe_replay_local_working_corpus(&root, CHOICE_TARGET, choice_letter_subject);
    maybe_replay_local_working_corpus(&root, JSON_TARGET, canonical_json_subject);
}

/// Fires-on-known-bad: a panicking subject is Crash, through both the
/// single-input judge and a one-file corpus directory.
#[test]
fn planted_crashing_subject_is_red() {
    assert_eq!(
        replay_one(b"CRASH", planted_boom),
        ReplayVerdict::Crash,
        "a planted panic must be Crash, not Ok"
    );

    let dir = uniq("plant_crash");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("plant"), b"CRASH").unwrap();
    let files = list_corpus_files(&dir).expect("planted one-file corpus");
    let mut saw_crash = false;
    for path in files {
        let data = fs::read(&path).unwrap();
        if replay_one(&data, planted_boom) == ReplayVerdict::Crash {
            saw_crash = true;
        }
    }
    let _ = fs::remove_dir_all(&dir);
    assert!(
        saw_crash,
        "a planted crash in a corpus directory must be observed as Crash"
    );
}

/// Control: the same judge must stay green on a quiet subject, or the plant
/// above would be an always-Crash spoof.
#[test]
fn planted_quiet_subject_stays_green() {
    assert_eq!(replay_one(b"A", choice_letter_subject), ReplayVerdict::Ok);
    assert_eq!(
        replay_one(b"{\"z\":1}", canonical_json_subject),
        ReplayVerdict::Ok
    );
}

/// Anti-vacuous: an empty directory is an error, not a pass.
#[test]
fn empty_corpus_dir_is_an_error_not_a_pass() {
    let dir = uniq("empty_corpus");
    fs::create_dir_all(&dir).unwrap();
    let err = list_corpus_files(&dir).expect_err("empty dir must be an error");
    let _ = fs::remove_dir_all(&dir);
    assert!(
        err.contains("empty"),
        "empty-corpus error must name emptiness: {err}"
    );
}

/// Anti-vacuous: a path that is not a directory is an error, not a pass.
#[test]
fn missing_corpus_dir_is_an_error_not_a_pass() {
    let dir = uniq("missing_corpus");
    let err = list_corpus_files(&dir).expect_err("missing dir must be an error");
    assert!(
        err.contains("not a directory"),
        "missing-corpus error must name the miss: {err}"
    );
}
