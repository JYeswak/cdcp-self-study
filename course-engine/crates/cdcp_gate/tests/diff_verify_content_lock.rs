//! Differential harness for `verify-content-lock` (bd-substrate-rust-migration-jhd.8).
//!
//! Every case here runs BOTH `scripts/verify_content_lock.py` — the oracle,
//! which stays in the tree for exactly this purpose — and the Rust gate, then
//! asserts stdout, stderr, and exit code are identical byte for byte. A case
//! that only ran one side would be a test of nothing.
//!
//! Cases 2..N build a throwaway engine tree in a tempdir and copy the oracle
//! into it, because the oracle hard-codes its root as
//! `Path(__file__).resolve().parents[1]` and there is no way to point it at a
//! different tree without moving the file. The committed `content.lock` is
//! never mutated: a port must not modify the artifact it verifies.
//!
//! In the fixture neither side finds `target/debug/cdcp` and neither finds a
//! `Cargo.toml`, so both fall through the same two candidates to
//! `goldens/bank_hash.txt`. That is a property of the fixture layout, not of
//! either implementation, and it is the same for both sides by construction.
//! Two cases deliberately break that property by planting a stub `cdcp` (see
//! `plant_bank_hash_stub`) to exercise the bank-hash failure paths of bd-hw3.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE_REL: &str = "scripts/verify_content_lock.py";

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("engine root")
        .canonicalize()
        .expect("canonical engine root")
}

#[derive(Debug, PartialEq, Eq)]
struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn show(r: &Run) -> String {
    format!(
        "exit={}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        r.code,
        String::from_utf8_lossy(&r.stdout),
        String::from_utf8_lossy(&r.stderr)
    )
}

/// Every env var either side reads. Cleared on BOTH before each run so an
/// ambient value in the developer's shell cannot make the two sides disagree —
/// or, worse, agree for the wrong reason.
const READ_ENV: [&str; 2] = ["CDCP_CONTENT_LOCK_SELFTEST", "CDCP_BANK_HASH_TIMEOUT_S"];

fn run_oracle(engine: &Path, selftest: bool, env: &[(&str, &str)]) -> Run {
    let mut c = Command::new("python3");
    c.arg(engine.join(ORACLE_REL)).current_dir(engine);
    for k in READ_ENV {
        c.env_remove(k);
    }
    if selftest {
        c.env("CDCP_CONTENT_LOCK_SELFTEST", "1");
    }
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().expect("spawn python3");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

fn run_gate(engine: &Path, selftest: bool, env: &[(&str, &str)]) -> Run {
    let mut c = Command::new(BIN);
    c.arg("--root")
        .arg(engine)
        .arg("verify-content-lock")
        .current_dir(engine);
    for k in READ_ENV {
        c.env_remove(k);
    }
    if selftest {
        c.env("CDCP_CONTENT_LOCK_SELFTEST", "1");
    }
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().expect("spawn cdcp_gate");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

/// Run both sides and assert every byte agrees.
///
/// Returns the **Rust** run, deliberately. Every case then adds its own
/// assertions about WHAT that output says, and because those read the port's
/// bytes rather than the oracle's, they are a second load-bearing leg: deleting
/// the comparison below still leaves a real check on the port, and deleting a
/// case's own assertions still leaves the comparison. Returning the oracle's
/// run instead — the obvious first draft — collapses both legs onto the
/// comparison, and a meta-test on 2026-08-14 caught exactly that: with the
/// stderr comparison removed, a truncation defect planted in the gate went
/// undetected by the whole suite.
fn assert_identical(engine: &Path, selftest: bool, case: &str) -> Run {
    assert_identical_env(engine, selftest, &[], case)
}

fn assert_identical_env(engine: &Path, selftest: bool, env: &[(&str, &str)], case: &str) -> Run {
    let py = run_oracle(engine, selftest, env);
    let rs = run_gate(engine, selftest, env);
    assert_eq!(
        rs.code,
        py.code,
        "{case}: exit code differs\npython:\n{}\nrust:\n{}",
        show(&py),
        show(&rs)
    );
    assert_eq!(
        String::from_utf8_lossy(&rs.stdout),
        String::from_utf8_lossy(&py.stdout),
        "{case}: stdout differs"
    );
    assert_eq!(
        String::from_utf8_lossy(&rs.stderr),
        String::from_utf8_lossy(&py.stderr),
        "{case}: stderr differs"
    );
    assert_eq!(rs.stdout, py.stdout, "{case}: stdout bytes differ");
    assert_eq!(rs.stderr, py.stderr, "{case}: stderr bytes differ");
    rs
}

// ── fixture ────────────────────────────────────────────────────────────────

struct Fixture {
    _dir: tempfile::TempDir,
    /// The engine root inside the fixture (`<tmp>/engine`).
    engine: PathBuf,
    /// The parent corpus (`<tmp>`) — where `modules/*.md` lives, matching the
    /// oracle's `ROOT.parent` fallback in `resolve_pinned`.
    base: PathBuf,
}

impl Fixture {
    /// A tree the oracle reports GREEN on, built by copying the real one.
    fn green() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = dir.path().canonicalize().expect("canonicalize");
        let engine = base.join("engine");
        let src = engine_root();

        fs::create_dir_all(engine.join("scripts")).unwrap();
        fs::create_dir_all(engine.join("registries")).unwrap();
        fs::create_dir_all(engine.join("goldens")).unwrap();

        fs::copy(src.join(ORACLE_REL), engine.join(ORACLE_REL)).expect("copy oracle");
        // The Rust dispatcher anchors the engine root on this file.
        fs::write(
            engine.join("registries/claims.toml"),
            "schema_version = 1\n",
        )
        .unwrap();
        fs::copy(
            src.join("goldens/bank_hash.txt"),
            engine.join("goldens/bank_hash.txt"),
        )
        .expect("copy golden");
        fs::copy(src.join("content.lock"), engine.join("content.lock")).expect("copy lock");

        let f = Fixture {
            _dir: dir,
            engine,
            base,
        };

        // Copy every pinned file to the same place, relative to whichever root
        // the oracle resolves it against in the real tree.
        let mut copied = 0usize;
        for rel in pinned_paths(&src) {
            let from_engine = src.join(&rel);
            let (from, to) = if from_engine.exists() {
                (from_engine, f.engine.join(&rel))
            } else {
                (src.parent().unwrap().join(&rel), f.base.join(&rel))
            };
            if !from.is_file() {
                continue;
            }
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(&from, &to).unwrap_or_else(|e| panic!("copy {}: {e}", from.display()));
            copied += 1;
        }
        assert!(
            copied >= 30,
            "fixture copied only {copied} pinned files — a fixture that pins nothing is an ERROR"
        );
        f
    }

    fn lock(&self) -> PathBuf {
        self.engine.join("content.lock")
    }

    fn read_lock(&self) -> String {
        fs::read_to_string(self.lock()).expect("read fixture lock")
    }

    fn write_lock(&self, body: &str) {
        fs::write(self.lock(), body).expect("write fixture lock");
    }

    /// The `bank_hash` this fixture's lock actually pins — DERIVED, never typed.
    ///
    /// bd-5sp5: these three cases used to carry the 64-hex literal by hand, and
    /// it went stale the moment C2 changed what `hash_payload` covers. A test
    /// that pins a value by hand is a doc with a compiler; deriving it means the
    /// next bank_hash move cannot leave a false literal behind.
    ///
    /// Anti-vacuous: a lock with no `bank_hash`, or one that is not 64 hex
    /// characters, is an ERROR here — otherwise a malformed lock would silently
    /// make every mutation below a no-op that still reported green.
    fn bank_hash(&self) -> String {
        let text = self.read_lock();
        let table: toml::Table = text.parse().expect("fixture lock parses");
        let hash = table
            .get("bank_hash")
            .and_then(|v| v.as_str())
            .expect("fixture lock pins a bank_hash")
            .to_string();
        assert!(
            hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()),
            "fixture lock's bank_hash is not 64 hex characters: {hash:?}"
        );
        hash
    }

    /// Replace the first occurrence of `needle` in the lock with `sub`.
    fn edit_lock(&self, needle: &str, sub: &str) {
        let text = self.read_lock();
        assert!(
            text.contains(needle),
            "fixture lock does not contain {needle:?} — the mutation would have been a no-op"
        );
        self.write_lock(&text.replacen(needle, sub, 1));
    }
}

/// Every `path = hash` key under `[knowledge]` and `[modules]` of the real lock.
fn pinned_paths(src: &Path) -> Vec<String> {
    let text = fs::read_to_string(src.join("content.lock")).expect("read content.lock");
    let table: toml::Table = text.parse().expect("content.lock parses");
    let mut out = Vec::new();
    for section in ["knowledge", "modules"] {
        if let Some(toml::Value::Table(t)) = table.get(section) {
            out.extend(t.keys().cloned());
        }
    }
    assert!(!out.is_empty(), "content.lock pins nothing");
    out
}

// ── case 1: the live repo tree ─────────────────────────────────────────────

/// The committed tree, unmodified, through both implementations.
#[test]
fn live_tree_is_green_and_identical() {
    let root = engine_root();
    let got = assert_identical(&root, false, "live tree");
    assert_eq!(
        got.code,
        0,
        "the committed tree is expected GREEN: {}",
        show(&got)
    );
    let stdout = String::from_utf8(got.stdout.clone()).expect("utf-8 receipt");
    assert!(
        stdout.starts_with("verify_content_lock: PASS bank_hash="),
        "{stdout}"
    );
    // The truncation marker is U+2026, not three ASCII dots. Pin the bytes.
    assert!(
        got.stdout_contains_ellipsis(),
        "receipt must carry U+2026: {stdout}"
    );
}

trait EllipsisCheck {
    fn stdout_contains_ellipsis(&self) -> bool;
}
impl EllipsisCheck for Run {
    fn stdout_contains_ellipsis(&self) -> bool {
        self.stdout.windows(3).any(|w| w == [0xe2u8, 0x80, 0xa6])
            && !String::from_utf8_lossy(&self.stdout).contains("...")
    }
}

/// The optional mutate-selftest on the live tree: both sides must agree that
/// flipping the pinned `bank_hash` reaches the RED path.
#[test]
fn live_tree_selftest_is_identical() {
    let root = engine_root();
    let got = assert_identical(&root, true, "live tree selftest");
    assert_eq!(got.code, 0, "{}", show(&got));
    assert_eq!(
        String::from_utf8_lossy(&got.stdout),
        "verify_content_lock: ok: mutate-selftest trips RED (bank_hash drift)\n"
    );
}

// ── case 1b: the fixture reproduces GREEN ──────────────────────────────────

#[test]
fn fixture_baseline_is_green_and_identical() {
    let f = Fixture::green();
    let got = assert_identical(&f.engine, false, "fixture baseline");
    assert_eq!(
        got.code,
        0,
        "the fixture must start GREEN or every RED case below proves nothing: {}",
        show(&got)
    );
    assert!(String::from_utf8_lossy(&got.stdout).contains("knowledge=9 modules=31"));
}

// ── case 2: a tampered content hash ────────────────────────────────────────

#[test]
fn tampered_knowledge_hash_is_red_in_both() {
    let f = Fixture::green();
    f.edit_lock(
        "\"knowledge/topics.toml\" = \"07c8ace8f187ab10f0029f10dd501a28d5b31a0a803d536a50eb5674806e2479\"",
        "\"knowledge/topics.toml\" = \"0000000000000000000000000000000000000000000000000000000000000000\"",
    );
    let got = assert_identical(&f.engine, false, "tampered knowledge hash");
    assert_eq!(got.code, 1, "{}", show(&got));
    let err = String::from_utf8_lossy(&got.stderr);
    assert!(err.starts_with("verify_content_lock: FAIL\n"), "{err}");
    assert!(
        err.contains(
            "  - [knowledge] hash mismatch: knowledge/topics.toml lock=000000000000\u{2026} live=07c8ace8f187\u{2026}\n"
        ),
        "{err}"
    );
    assert!(err.contains("Regenerate (human review): UPDATE_CONTENT_LOCK=1 python3 scripts/gen_content_lock.py\n"), "{err}");
}

#[test]
fn tampered_module_content_is_red_in_both() {
    let f = Fixture::green();
    // Tamper the FILE this time, not the lock — the other direction of drift.
    let target = f.engine.join("web/content/modules/12-fire.md");
    let mut body = fs::read(&target).unwrap();
    body.extend_from_slice(b"\nsmuggled paragraph\n");
    fs::write(&target, body).unwrap();

    let got = assert_identical(&f.engine, false, "tampered module content");
    assert_eq!(got.code, 1, "{}", show(&got));
    let err = String::from_utf8_lossy(&got.stderr);
    assert!(
        err.contains("  - [modules] hash mismatch: web/content/modules/12-fire.md lock=2192c1af59a3\u{2026} live="),
        "{err}"
    );
}

#[test]
fn tampered_bank_hash_is_drift_in_both() {
    let f = Fixture::green();
    let live = f.bank_hash();
    f.edit_lock(
        &format!("bank_hash = \"{live}\""),
        "bank_hash = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
    );
    let got = assert_identical(&f.engine, false, "tampered bank_hash");
    assert_eq!(got.code, 1, "{}", show(&got));
    let err = String::from_utf8_lossy(&got.stderr);
    // 16 characters of each side, then U+2026 — the wider truncation the drift
    // message uses, distinct from the 12 the receipt and the mismatch use.
    assert!(
        err.contains(&format!(
            "  - bank_hash drift: lock=aaaaaaaaaaaaaaaa\u{2026} live={}\u{2026}\n",
            &live[..16]
        )),
        "{err}"
    );
}

// ── case 3: a lock entry whose file no longer exists ───────────────────────

#[test]
fn missing_pinned_file_is_red_in_both() {
    let f = Fixture::green();
    fs::remove_file(f.engine.join("knowledge/sources.toml")).expect("remove pinned file");
    let got = assert_identical(&f.engine, false, "missing pinned file");
    assert_eq!(got.code, 1, "{}", show(&got));
    let err = String::from_utf8_lossy(&got.stderr);
    assert!(
        err.contains("  - [knowledge] missing file: knowledge/sources.toml\n"),
        "{err}"
    );
}

#[test]
fn missing_parent_relative_pinned_file_is_red_in_both() {
    let f = Fixture::green();
    // `modules/*.md` resolves against ROOT.parent, not ROOT — the second leg of
    // `resolve_pinned`. Deleting one exercises that leg on both sides.
    fs::remove_file(f.base.join("modules/07-emf.md")).expect("remove parent-relative pin");
    let got = assert_identical(&f.engine, false, "missing parent-relative pin");
    assert_eq!(got.code, 1, "{}", show(&got));
    assert!(
        String::from_utf8_lossy(&got.stderr)
            .contains("  - [modules] missing file: modules/07-emf.md\n"),
        "{}",
        show(&got)
    );
}

// ── case 4: the tree-side walk (bd-z3v) ────────────────────────────────────
//
// These two replace `an_unlisted_file_slips_past_both_implementations` and
// `deleting_a_row_silently_stops_checking_that_file_in_both`, which pinned the
// two directions of the hole as agreed behaviour. The behaviour they pinned no
// longer exists, so they were deleted rather than amended.

/// A file under a locked root that no row pins is RED, on both sides, in both
/// the engine-relative root and the parent-relative one.
#[test]
fn an_unlisted_file_is_red_in_both() {
    let f = Fixture::green();
    fs::write(
        f.engine.join("knowledge/smuggled_policy.toml"),
        "schema_version = 1\nsmuggled = true\n",
    )
    .unwrap();
    fs::write(f.base.join("modules/99-smuggled.md"), "# not pinned\n").unwrap();

    let got = assert_identical(&f.engine, false, "unlisted file");
    assert_eq!(
        got.code,
        1,
        "an unpinned file under a locked root must be RED: {}",
        show(&got)
    );
    let err = String::from_utf8_lossy(&got.stderr);
    assert!(
        err.contains(
            "  - [knowledge] in the tree but not pinned in content.lock: \
             knowledge/smuggled_policy.toml\n"
        ),
        "{err}"
    );
    assert!(
        err.contains(
            "  - [modules] in the tree but not pinned in content.lock: modules/99-smuggled.md\n"
        ),
        "{err}"
    );
}

/// The dangerous direction: deleting a ROW used to delete the CHECK, and the
/// removal looked exactly like a pass. Now the tree-side walk finds the file
/// with no row behind it.
#[test]
fn deleting_a_row_is_red_in_both() {
    let f = Fixture::green();
    let text = f.read_lock();
    let line = "\"knowledge/claims.toml\" = \"f38eaf89f3bd61bfc7d1a4f5d6a0757e7a2a0399000afae2fc0e3182c78df867\"\n";
    assert!(text.contains(line));
    f.write_lock(&text.replacen(line, "", 1));
    // Corrupt the file the row used to pin: before the fix this stayed GREEN.
    fs::write(f.engine.join("knowledge/claims.toml"), "corrupted\n").unwrap();

    let got = assert_identical(&f.engine, false, "deleted row");
    assert_eq!(got.code, 1, "deleting a pin must be RED: {}", show(&got));
    assert!(
        String::from_utf8_lossy(&got.stderr).contains(
            "  - [knowledge] in the tree but not pinned in content.lock: knowledge/claims.toml\n"
        ),
        "{}",
        show(&got)
    );
}

/// The receipt names the roots it walked and their counts, so a reader of a
/// GREEN verdict is told what the verdict ranges over.
#[test]
fn the_green_receipt_names_its_coverage_in_both() {
    let f = Fixture::green();
    let got = assert_identical(&f.engine, false, "coverage receipt");
    assert_eq!(got.code, 0, "{}", show(&got));
    let out = String::from_utf8_lossy(&got.stdout);
    assert!(
        out.contains(
            "verify_content_lock: covered roots (every file found under these is pinned and \
             matched): knowledge/*.toml=9 web/content/modules/*.md=16 ../modules/*.md=15\n"
        ),
        "{out}"
    );
    assert!(
        out.contains("verify_content_lock: NOT covered: anything outside those roots"),
        "{out}"
    );
}

/// Anti-vacuous, tree side: a locked root that matched zero files must not
/// report like a root whose every file matched.
#[test]
fn a_locked_root_with_zero_files_is_red_in_both() {
    let f = Fixture::green();
    let dir = f.base.join("modules");
    let mut removed = 0usize;
    for e in fs::read_dir(&dir).unwrap().flatten() {
        fs::remove_file(e.path()).unwrap();
        removed += 1;
    }
    assert!(removed >= 15, "emptied only {removed} files");
    let got = assert_identical(&f.engine, false, "root matched zero files");
    assert_ne!(got.code, 0, "a root that checked nothing is not a pass");
    assert!(
        String::from_utf8_lossy(&got.stderr).contains(
            "  - [modules] locked root matched zero files: ../modules/*.md \
             (nothing was checked there \u{2014} vacuous ERROR)\n"
        ),
        "{}",
        show(&got)
    );
}

#[test]
fn a_missing_locked_root_is_red_in_both() {
    let f = Fixture::green();
    let dir = f.base.join("modules");
    for e in fs::read_dir(&dir).unwrap().flatten() {
        fs::remove_file(e.path()).unwrap();
    }
    fs::remove_dir(&dir).unwrap();
    let got = assert_identical(&f.engine, false, "locked root absent");
    assert_ne!(got.code, 0);
    assert!(
        String::from_utf8_lossy(&got.stderr).contains(
            "  - [modules] locked root is not a directory: ../modules/*.md \
             (nothing was checked there \u{2014} vacuous ERROR)\n"
        ),
        "{}",
        show(&got)
    );
}

// ── case 4b: the bank-hash subprocess (bd-hw3) ─────────────────────────────
//
// Both of these used to be uncaught exceptions in the oracle — a traceback where
// a verdict belonged — and a silent skip-to-the-next-candidate in the port.

/// Install a fake `target/debug/cdcp` that both sides prefer over `cargo run`.
fn plant_bank_hash_stub(engine: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let dir = engine.join("target/debug");
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("cdcp");
    fs::write(&p, body).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn a_bank_hash_that_says_nothing_is_red_in_both() {
    let f = Fixture::green();
    plant_bank_hash_stub(&f.engine, "#!/bin/sh\nexit 0\n");
    let got = assert_identical(&f.engine, false, "bank-hash exits 0 with no output");
    assert_eq!(
        got.code,
        1,
        "a hash oracle that says nothing must not read as a pass: {}",
        show(&got)
    );
    assert!(
        String::from_utf8_lossy(&got.stderr)
            .contains("  - bank-hash exited 0 with no output (cannot obtain live bank_hash)\n"),
        "{}",
        show(&got)
    );
}

#[test]
fn a_bank_hash_that_hangs_is_red_in_both() {
    let f = Fixture::green();
    plant_bank_hash_stub(&f.engine, "#!/bin/sh\nsleep 60\n");
    let got = assert_identical_env(
        &f.engine,
        false,
        &[("CDCP_BANK_HASH_TIMEOUT_S", "1")],
        "bank-hash hangs",
    );
    assert_eq!(got.code, 1, "{}", show(&got));
    assert!(
        String::from_utf8_lossy(&got.stderr)
            .contains("  - bank-hash timed out (cannot obtain live bank_hash)\n"),
        "{}",
        show(&got)
    );
}

#[test]
fn an_unparseable_timeout_is_red_in_both() {
    let f = Fixture::green();
    let got = assert_identical_env(
        &f.engine,
        false,
        &[("CDCP_BANK_HASH_TIMEOUT_S", "banana")],
        "unparseable timeout",
    );
    assert_eq!(got.code, 1, "{}", show(&got));
    assert!(
        String::from_utf8_lossy(&got.stderr)
            .contains("  - invalid CDCP_BANK_HASH_TIMEOUT_S (want a positive number of seconds)\n"),
        "{}",
        show(&got)
    );
}

#[test]
fn a_non_positive_timeout_is_red_in_both() {
    let f = Fixture::green();
    let got = assert_identical_env(
        &f.engine,
        false,
        &[("CDCP_BANK_HASH_TIMEOUT_S", "0")],
        "zero timeout",
    );
    assert_eq!(got.code, 1, "{}", show(&got));
    assert!(
        String::from_utf8_lossy(&got.stderr)
            .contains("  - invalid CDCP_BANK_HASH_TIMEOUT_S (want a positive number of seconds)\n"),
        "{}",
        show(&got)
    );
}

// ── case 5: anti-vacuous — empty or missing lock is never a pass ───────────

#[test]
fn a_missing_lock_is_red_in_both_never_a_pass() {
    let f = Fixture::green();
    fs::remove_file(f.lock()).expect("remove lock");
    let got = assert_identical(&f.engine, false, "missing lock");
    assert_ne!(got.code, 0, "a missing lock must never report like a pass");
    assert_eq!(got.code, 1, "{}", show(&got));
    let err = String::from_utf8_lossy(&got.stderr);
    assert!(
        err.contains(&format!(
            "  - missing content.lock at {}\n",
            f.engine.join("content.lock").display()
        )),
        "{err}"
    );
}

#[test]
fn an_empty_lock_is_red_in_both_on_all_four_counts() {
    let f = Fixture::green();
    f.write_lock("");
    let got = assert_identical(&f.engine, false, "empty lock");
    assert_ne!(got.code, 0, "an empty lock must never report like a pass");
    let err = String::from_utf8_lossy(&got.stderr);
    for want in [
        "  - unsupported schema_version=None (want 1)\n",
        "  - content.lock missing bank_hash\n",
        "  - content.lock [knowledge] empty (vacuous ERROR)\n",
        "  - content.lock [modules] empty (vacuous ERROR)\n",
    ] {
        assert!(err.contains(want), "missing {want:?} in:\n{err}");
    }
}

#[test]
fn a_comments_only_lock_is_red_in_both() {
    let f = Fixture::green();
    f.write_lock("# every hash deleted, header kept\n\n");
    let got = assert_identical(&f.engine, false, "comments-only lock");
    assert_ne!(got.code, 0);
}

#[test]
fn empty_sections_are_red_in_both() {
    let f = Fixture::green();
    let text = f.read_lock();
    let head = text
        .split("[knowledge]")
        .next()
        .expect("lock has a [knowledge] section");
    f.write_lock(&format!("{head}[knowledge]\n\n[modules]\n"));
    let got = assert_identical(&f.engine, false, "empty sections");
    assert_ne!(
        got.code, 0,
        "a lock pinning zero files must not read as a pass"
    );
    let err = String::from_utf8_lossy(&got.stderr);
    assert!(
        err.contains("  - content.lock [knowledge] empty (vacuous ERROR)\n"),
        "{err}"
    );
    assert!(
        err.contains("  - content.lock [modules] empty (vacuous ERROR)\n"),
        "{err}"
    );
}

// ── schema and type edges ──────────────────────────────────────────────────

#[test]
fn wrong_schema_version_is_red_in_both_with_python_repr() {
    let f = Fixture::green();
    f.edit_lock("schema_version = 1\n", "schema_version = 2\n");
    let got = assert_identical(&f.engine, false, "schema 2");
    assert_eq!(got.code, 1);
    assert!(
        String::from_utf8_lossy(&got.stderr)
            .contains("  - unsupported schema_version=2 (want 1)\n"),
        "{}",
        show(&got)
    );
}

#[test]
fn string_schema_version_reprs_with_quotes_in_both() {
    let f = Fixture::green();
    f.edit_lock("schema_version = 1\n", "schema_version = \"1\"\n");
    let got = assert_identical(&f.engine, false, "schema '1'");
    assert!(
        String::from_utf8_lossy(&got.stderr)
            .contains("  - unsupported schema_version='1' (want 1)\n"),
        "{}",
        show(&got)
    );
}

#[test]
fn boolean_true_schema_version_passes_the_check_in_both() {
    // `True == 1` in Python. This is the oracle's behaviour, faithfully ported;
    // it is recorded here so the equivalence is deliberate rather than lucky.
    let f = Fixture::green();
    f.edit_lock("schema_version = 1\n", "schema_version = true\n");
    let got = assert_identical(&f.engine, false, "schema true");
    assert_eq!(got.code, 0, "{}", show(&got));
}

#[test]
fn empty_bank_hash_is_red_in_both() {
    let f = Fixture::green();
    f.edit_lock(
        &format!("bank_hash = \"{}\"", f.bank_hash()),
        "bank_hash = \"\"",
    );
    let got = assert_identical(&f.engine, false, "empty bank_hash");
    assert_eq!(got.code, 1);
    assert!(
        String::from_utf8_lossy(&got.stderr).contains("  - content.lock missing bank_hash\n"),
        "{}",
        show(&got)
    );
}

#[test]
fn a_non_string_section_is_red_in_both() {
    let f = Fixture::green();
    let text = f.read_lock();
    let head = text.split("[knowledge]").next().unwrap();
    f.write_lock(&format!("{head}knowledge = \"nope\"\nmodules = \"nope\"\n"));
    let got = assert_identical(&f.engine, false, "sections are not tables");
    assert_eq!(got.code, 1);
    let err = String::from_utf8_lossy(&got.stderr);
    assert!(
        err.contains("  - [knowledge] must be a table of path = hash\n"),
        "{err}"
    );
    assert!(
        err.contains("  - [modules] must be a table of path = hash\n"),
        "{err}"
    );
}

// ── the fallback chain for the live bank digest ────────────────────────────

#[test]
fn a_missing_golden_makes_the_digest_unobtainable_in_both() {
    let f = Fixture::green();
    fs::remove_file(f.engine.join("goldens/bank_hash.txt")).expect("remove golden");
    let got = assert_identical(&f.engine, false, "no golden, no binary, no cargo manifest");
    assert_eq!(got.code, 1, "{}", show(&got));
    assert!(
        String::from_utf8_lossy(&got.stderr).contains("  - cannot obtain live bank_hash\n"),
        "{}",
        show(&got)
    );
}

#[test]
fn a_non_hex_golden_is_compared_verbatim_in_both() {
    // The oracle does NOT hex-validate the golden fallback; this port does not
    // either. Pinning that here keeps a later "improvement" honest.
    let f = Fixture::green();
    let live = f.bank_hash();
    fs::write(f.engine.join("goldens/bank_hash.txt"), "  not-a-hash  \n").unwrap();
    let got = assert_identical(&f.engine, false, "non-hex golden");
    assert_eq!(got.code, 1, "{}", show(&got));
    assert!(
        String::from_utf8_lossy(&got.stderr).contains(&format!(
            "  - bank_hash drift: lock={}\u{2026} live=not-a-hash\u{2026}\n",
            &live[..16]
        )),
        "{}",
        show(&got)
    );
}

// ── the selftest branch, on a fixture ──────────────────────────────────────

#[test]
fn selftest_on_the_fixture_is_identical() {
    let f = Fixture::green();
    let got = assert_identical(&f.engine, true, "fixture selftest");
    assert_eq!(got.code, 0, "{}", show(&got));
    assert_eq!(
        String::from_utf8_lossy(&got.stdout),
        "verify_content_lock: ok: mutate-selftest trips RED (bank_hash drift)\n"
    );
}

#[test]
fn selftest_without_a_lock_fails_the_same_way_in_both() {
    let f = Fixture::green();
    fs::remove_file(f.lock()).unwrap();
    let got = assert_identical(&f.engine, true, "selftest, no lock");
    assert_eq!(got.code, 1);
    assert_eq!(
        String::from_utf8_lossy(&got.stderr),
        "FAIL: content.lock missing; cannot selftest\n"
    );
}

#[test]
fn selftest_without_a_bank_hash_line_fails_the_same_way_in_both() {
    let f = Fixture::green();
    let text = f.read_lock();
    let stripped: String = text
        .lines()
        .filter(|l| !l.starts_with("bank_hash = "))
        .map(|l| format!("{l}\n"))
        .collect();
    f.write_lock(&stripped);
    let got = assert_identical(&f.engine, true, "selftest, no bank_hash line");
    assert_eq!(got.code, 1);
    assert_eq!(
        String::from_utf8_lossy(&got.stderr),
        "FAIL: selftest could not locate bank_hash line\n"
    );
}

// ── the harness must not be able to pass vacuously ─────────────────────────

/// If the oracle ever stops being runnable, every `assert_identical` above
/// would compare two failures-to-run. Check the oracle really is there and
/// really is the file the port was written against.
#[test]
fn the_oracle_is_present_and_runnable() {
    let root = engine_root();
    let oracle = root.join(ORACLE_REL);
    assert!(
        oracle.is_file(),
        "{} is the differential oracle for this port; it must stay in the tree",
        oracle.display()
    );
    let text = fs::read_to_string(&oracle).unwrap();
    assert!(text.contains("def verify("), "oracle lost its verify()");
    assert!(
        text.contains("bank_hash drift"),
        "oracle lost the drift message this port reproduces"
    );
    // bd-z3v: the tree-side walk is the half of the gate whose coverage the lock
    // cannot narrow. If it leaves the oracle, every case above compares two
    // closed-world checks and the differential suite stops meaning what it says.
    assert!(
        text.contains("def tree_side_errors("),
        "oracle lost its tree-side walk"
    );
    assert!(
        text.contains("LOCKED_ROOTS"),
        "oracle lost its locked-root list"
    );
    // The marker really is U+2026 in the oracle's source, not three dots.
    assert!(
        text.as_bytes().windows(3).any(|w| w == [0xe2, 0x80, 0xa6]),
        "oracle no longer uses U+2026"
    );
    let out = Command::new("python3")
        .arg("--version")
        .output()
        .expect("python3 must be on PATH for the differential suite to mean anything");
    assert!(out.status.success());
}
