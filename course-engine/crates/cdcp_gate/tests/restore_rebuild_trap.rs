//! THE STALE-BINARY RESTORE TRAP, reproduced on demand — bd-stale-binary-mtime-trap-p65w.
//!
//! **The argued pattern lives in `tests/support/rebuild.rs`. Read that header
//! before writing a meta-test pair.** This file is its known-bad: it makes the
//! trap fire, on purpose, in both directions, so the fix can be verified rather
//! than believed.
//!
//! WHAT IS REPRODUCED. The house meta-test pair is
//!   (1) mutate the gate, assert the suite goes non-zero;
//!   (2) with the mutation STILL IN PLACE, delete the assertion, assert it
//!       returns to zero;
//!   (3) restore.
//! Step 3 done with `mv file.bak file` hands the restored file the BACKUP'S
//! mtime — older than the artifact cargo already built from the perturbed
//! source — so cargo skips the rebuild and the next run reads a verdict off a
//! binary that does not match the tree.
//!
//! BOTH DIRECTIONS MATTER AND ONLY ONE OF THEM IS LOUD.
//!
//! * FALSE RED — restore the mutated GATE naively. The tree is correct, the
//!   binary is not, the suite goes red. This is the direction measured
//!   2026-08-14 and recorded in `goldens/PROVENANCE.md`. Someone investigates,
//!   so it self-corrects.
//! * FALSE GREEN — restore the deleted ASSERTION naively, at the end of leg 2,
//!   while the mutation is still in place. The tree now contains a live defect
//!   AND the assertion that catches it, so the truth is RED — and the suite
//!   reports 0, because the binary it runs has no assertion compiled into it.
//!   Nobody investigates a green. **A meta-test that cannot fail is a fooled
//!   certificate, which this project classes as Sev-0.**
//!   Worse, the corruption is DEFERRED: a bad step 3 does not spoil the pair
//!   that performed it, it spoils the NEXT pair to run.
//!
//! HOW IT IS REPRODUCED WITHOUT TOUCHING THIS REPO. Every case builds a
//! throwaway two-file cargo crate in a tempdir — `src/lib.rs` holding a
//! `VERDICT` constant that plays the part of the gate, `tests/verdict.rs`
//! holding the assertion that catches a wrong one — and runs the real `cargo`
//! against it. Nothing here reads or writes the working tree, so the file can
//! be run concurrently with any other agent's suite.
//!
//! ANTI-VACUOUS. Every case first proves its own fixture discriminates: the
//! unperturbed crate must go GREEN and the perturbed crate must go RED before
//! any claim is made about what a restore did. A reproduction whose "trap" and
//! "no trap" arms report the same thing has demonstrated nothing.

mod support;
use support::rebuild::{build_proving_rebuild, nested_cargo, Restorable};

use std::path::{Path, PathBuf};
use std::process::Command;

const PASS: i32 = 0;
const TEST_FAILURE: i32 = 101;

// ── fixture ────────────────────────────────────────────────────────────────

/// A throwaway crate: `VERDICT` is the gate, `tests/verdict.rs` is the
/// assertion that catches a wrong one.
struct Crate {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

const GOOD_GATE: &str = "pub const VERDICT: &str = \"GOOD\";\n";
const MUTATED_GATE: &str = "pub const VERDICT: &str = \"BAD\";\n";

const ASSERTION: &str = "#[test]\n\
     fn verdict_is_good() {\n    \
         assert_eq!(trapdemo::VERDICT, \"GOOD\", \"the gate reported a wrong verdict\");\n\
     }\n";

/// Leg 2 of the house pair: the assertion deleted, the file still present and
/// still a valid test target, so the suite runs and simply catches nothing.
const ASSERTION_DELETED: &str = "// assertion deleted (meta-test leg 2)\n";

impl Crate {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonicalize");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"trapdemo\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
             [lib]\npath = \"src/lib.rs\"\n\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(root.join("src/lib.rs"), GOOD_GATE).unwrap();
        std::fs::write(root.join("tests/verdict.rs"), ASSERTION).unwrap();
        Crate { _dir: dir, root }
    }

    fn gate(&self) -> PathBuf {
        self.root.join("src/lib.rs")
    }

    fn assertion(&self) -> PathBuf {
        self.root.join("tests/verdict.rs")
    }

    fn write(&self, rel: &str, body: &str) {
        std::fs::write(self.root.join(rel), body).unwrap();
    }

    /// Run the suite the way a meta-test author does, and read the TRUE exit
    /// code off the child — never through a pipe.
    fn suite(&self) -> (i32, String) {
        let out = nested_cargo(&self.root)
            .args(["test", "--offline", "--test", "verdict"])
            .output()
            .expect("cargo test");
        let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
        s.push_str(&String::from_utf8_lossy(&out.stderr));
        (out.status.code().unwrap_or(-1), s)
    }

    /// The compiled integration-test binary, i.e. the artifact whose staleness
    /// is the whole subject of this file.
    fn test_binary(&self) -> Option<PathBuf> {
        let deps = self.root.join("target/debug/deps");
        let mut found: Option<(std::time::SystemTime, PathBuf)> = None;
        for e in std::fs::read_dir(&deps).ok()? {
            let p = e.ok()?.path();
            let name = p.file_name()?.to_string_lossy().into_owned();
            if !name.starts_with("verdict-") || p.extension().is_some() {
                continue;
            }
            let m = p.metadata().ok()?.modified().ok()?;
            if found.as_ref().is_none_or(|(t, _)| m > *t) {
                found = Some((m, p));
            }
        }
        found.map(|(_, p)| p)
    }
}

/// The floor every case stands on: an unperturbed crate is GREEN and a
/// perturbed one is RED. Without this, "the restore reported 0" proves nothing,
/// because a fixture that can only ever report 0 also reports 0.
fn assert_fixture_discriminates(c: &Crate) {
    let (green, out) = c.suite();
    assert_eq!(
        green, PASS,
        "fixture is vacuous: the unperturbed crate did not pass\n{out}"
    );
    c.write("src/lib.rs", MUTATED_GATE);
    let (red, out) = c.suite();
    assert_eq!(
        red, TEST_FAILURE,
        "fixture is vacuous: the mutation did not turn the suite red\n{out}"
    );
    c.write("src/lib.rs", GOOD_GATE);
    let (green_again, out) = c.suite();
    assert_eq!(
        green_again, PASS,
        "fixture is vacuous: the crate did not come back green\n{out}"
    );
}

// ── 1. the trap: FALSE GREEN, the Sev-0 direction ─────────────────────────

/// THE ASSERTION THIS FILE EXISTS FOR. Run the house pair to completion with a
/// `mv`-shaped step 3 and the suite certifies a tree that is RED.
#[test]
fn a_naive_restore_of_the_assertion_reports_a_false_green() {
    let c = Crate::new();
    assert_fixture_discriminates(&c);

    // Leg 1 — mutate the gate; the suite must go non-zero.
    c.write("src/lib.rs", MUTATED_GATE);
    let (leg1, out) = c.suite();
    assert_eq!(leg1, TEST_FAILURE, "leg 1 did not bite\n{out}");

    // Leg 2 — with the mutation STILL IN PLACE, delete the assertion; the
    // suite must return to zero, proving THAT assertion is what bit.
    let assertion = Restorable::capture(c.assertion());
    c.write("tests/verdict.rs", ASSERTION_DELETED);
    let (leg2, out) = c.suite();
    assert_eq!(leg2, PASS, "leg 2 did not come back green\n{out}");

    // Step 3 — restore the assertion the way that rots.
    assertion.restore_the_unsafe_way();
    assert_eq!(
        std::fs::read(c.assertion()).unwrap(),
        assertion.original(),
        "the restore must put the ORIGINAL BYTES back — the trap is about mtime, \
         not about content, and a content difference here would prove the wrong thing"
    );

    // GROUND TRUTH: gate mutated AND assertion present. The only honest verdict
    // is RED. Anything else is a fooled certificate.
    let (verdict, out) = c.suite();
    assert_eq!(
        verdict, PASS,
        "the trap did not reproduce — a naive restore was expected to report a \
         FALSE GREEN on a tree whose only honest verdict is RED.\n\
         If this ever fails, the fix cannot be verified from this file and that \
         must be reported, not papered over.\n{out}"
    );
}

// ── 2. the fix: the same tree, restored safely, reports the truth ─────────

/// Same sequence, same tree, one difference: step 3 writes bytes into the
/// existing file and the rebuild is PROVEN before the verdict is read.
#[test]
fn a_safe_restore_of_the_assertion_reports_the_true_red() {
    let c = Crate::new();
    assert_fixture_discriminates(&c);

    c.write("src/lib.rs", MUTATED_GATE);
    let (leg1, out) = c.suite();
    assert_eq!(leg1, TEST_FAILURE, "leg 1 did not bite\n{out}");

    let assertion = Restorable::capture(c.assertion());
    c.write("tests/verdict.rs", ASSERTION_DELETED);
    let (leg2, out) = c.suite();
    assert_eq!(leg2, PASS, "leg 2 did not come back green\n{out}");

    // Step 3, done right: byte rewrite, then a build that is PROVEN to rebuild.
    assertion.restore();
    let artifact = c.test_binary().expect("the leg-2 test binary must exist");
    let built = build_proving_rebuild(
        &c.root,
        &artifact,
        &["test", "--offline", "--no-run", "--test", "verdict"],
    )
    .expect("the restore must be followed by an OBSERVED rebuild");
    assert!(
        built.before.is_some(),
        "the artifact must have existed before the build, or 'it was rebuilt' is trivially true"
    );

    let (verdict, out) = c.suite();
    assert_eq!(
        verdict, TEST_FAILURE,
        "with the mutation live and the assertion restored, the only honest verdict is RED\n{out}"
    );
}

// ── 3. the anti-vacuous leg: why "just build first" is not the fix ────────

/// The obvious remedy — run `cargo build` after restoring, before asserting —
/// PASSES IN EXACTLY THE CASE IT WAS WRITTEN TO CATCH. After a rename-restore
/// cargo finds nothing newer than the artifact, exits 0, and rebuilds nothing.
///
/// This is the bead's anti-vacuous clause aimed at its own remedy: a rebuild
/// step that finds nothing to rebuild, when a file demonstrably changed, is an
/// ERROR and not a pass. `build_proving_rebuild` is the thing that makes it one.
#[test]
fn a_forced_build_after_a_naive_restore_is_itself_vacuous() {
    let c = Crate::new();
    assert_fixture_discriminates(&c);

    let gate = Restorable::capture(c.gate());
    c.write("src/lib.rs", MUTATED_GATE);
    let (red, out) = c.suite();
    assert_eq!(red, TEST_FAILURE, "the mutation must bite first\n{out}");

    gate.restore_the_unsafe_way();
    assert_eq!(
        std::fs::read(c.gate()).unwrap(),
        gate.original(),
        "the source on disk is correct — that is what makes the stale artifact deceptive"
    );

    let artifact = c.test_binary().expect("test binary must exist");
    let plain = nested_cargo(&c.root)
        .args(["build", "--offline", "--tests"])
        .output()
        .expect("cargo build");
    assert_eq!(
        plain.status.code().unwrap_or(-1),
        PASS,
        "the naive remedy reports SUCCESS, which is the whole problem: {}{}",
        String::from_utf8_lossy(&plain.stdout),
        String::from_utf8_lossy(&plain.stderr),
    );

    let err = build_proving_rebuild(
        &c.root,
        &artifact,
        &["test", "--offline", "--no-run", "--test", "verdict"],
    )
    .expect_err(
        "a build that rebuilt nothing, after a file demonstrably changed, must be an ERROR",
    );
    assert!(
        err.contains("ANTI-VACUOUS FAILURE"),
        "the error must name what happened rather than fail obscurely: {err}"
    );
}

// ── 4. the recovery move, kept honest ─────────────────────────────────────

/// `touch` after a rename-restore does work — `goldens/PROVENANCE.md` reaches
/// for it correctly. It is pinned here as the RECOVERY move so the record shows
/// why it is not the pattern: it is a second step, separate from the restore,
/// and its omission is exactly what case 1 above certifies green.
#[test]
fn touch_recovers_a_tree_that_was_already_restored_by_rename() {
    let c = Crate::new();
    assert_fixture_discriminates(&c);

    let gate = Restorable::capture(c.gate());
    c.write("src/lib.rs", MUTATED_GATE);
    let (red, out) = c.suite();
    assert_eq!(red, TEST_FAILURE, "the mutation must bite first\n{out}");

    gate.restore_the_unsafe_way();
    let (still_red, out) = c.suite();
    assert_eq!(
        still_red, TEST_FAILURE,
        "the FALSE RED direction, as measured 2026-08-14: correct source, stale binary\n{out}"
    );

    touch(&c.gate());
    let artifact = c.test_binary().expect("test binary must exist");
    build_proving_rebuild(
        &c.root,
        &artifact,
        &["test", "--offline", "--no-run", "--test", "verdict"],
    )
    .expect("after a touch the rebuild must be observable");

    let (green, out) = c.suite();
    assert_eq!(
        green, PASS,
        "with the source correct and the artifact rebuilt, the tree is GREEN\n{out}"
    );
}

fn touch(path: &Path) {
    let out = std::process::Command::new("touch")
        .arg(path)
        .output()
        .expect("touch");
    assert!(out.status.success(), "touch {}", path.display());
}

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

/// The agent-facing receipt: `sh scripts/restore_safe.inc.sh prove-rebuild`.
fn run_prove_rebuild(artifact: &Path, crate_root: &Path, cargo_args: &[&str]) -> (i32, String) {
    let helper = engine_root().join("scripts/restore_safe.inc.sh");
    let mut cmd = Command::new("sh");
    cmd.arg(&helper)
        .arg("prove-rebuild")
        .arg("--artifact")
        .arg(artifact)
        .arg("--")
        .arg(support::rebuild::cargo_bin())
        .args(cargo_args)
        .current_dir(crate_root);
    for k in [
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTFLAGS",
        "RUSTDOCFLAGS",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "CARGO_MAKEFLAGS",
        "CARGO_MANIFEST_DIR",
        "CARGO_PKG_NAME",
        "LLVM_PROFILE_FILE",
    ] {
        cmd.env_remove(k);
    }
    let out = cmd.output().expect("prove-rebuild");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}

/// OBSERVED: the command refuses a real stale tree. Not merely authored.
#[test]
fn prove_rebuild_command_refuses_a_stale_tree() {
    let c = Crate::new();
    assert_fixture_discriminates(&c);

    let gate = Restorable::capture(c.gate());
    c.write("src/lib.rs", MUTATED_GATE);
    let (red, out) = c.suite();
    assert_eq!(red, TEST_FAILURE, "the mutation must bite first\n{out}");

    gate.restore_the_unsafe_way();
    let artifact = c.test_binary().expect("test binary must exist");
    let (code, out) = run_prove_rebuild(
        &artifact,
        &c.root,
        &["test", "--offline", "--no-run", "--test", "verdict"],
    );
    assert_ne!(
        code, PASS,
        "prove-rebuild must refuse a stale tree; got {code}\n{out}"
    );
    assert!(
        out.contains("ANTI-VACUOUS"),
        "prove-rebuild must name the vacuous rebuild, not fail obscurely:\n{out}"
    );
}

/// Empty/absent artifact is ERROR, never "nothing to check".
#[test]
fn prove_rebuild_command_errors_on_absent_artifact() {
    let helper = engine_root().join("scripts/restore_safe.inc.sh");
    let missing = std::env::temp_dir().join("cdcp-prove-rebuild-no-such-artifact");
    let _ = std::fs::remove_file(&missing);
    let out = Command::new("sh")
        .arg(&helper)
        .arg("prove-rebuild")
        .arg("--artifact")
        .arg(&missing)
        .arg("--")
        .arg("true")
        .output()
        .expect("prove-rebuild absent");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(
        out.status.code().unwrap_or(-1),
        PASS,
        "absent artifact must not be GREEN\n{text}"
    );
    assert!(
        text.contains("artifact missing") || text.contains("ERROR"),
        "absent artifact must be named as ERROR, not 'nothing to check':\n{text}"
    );
}

/// After a safe restore the same command is GREEN — the rebuild is observed.
#[test]
fn prove_rebuild_command_accepts_a_safe_restore() {
    let c = Crate::new();
    assert_fixture_discriminates(&c);

    let assertion = Restorable::capture(c.assertion());
    c.write("src/lib.rs", MUTATED_GATE);
    let (leg1, out) = c.suite();
    assert_eq!(leg1, TEST_FAILURE, "leg 1 did not bite\n{out}");
    c.write("tests/verdict.rs", ASSERTION_DELETED);
    let (leg2, out) = c.suite();
    assert_eq!(leg2, PASS, "leg 2 did not come back green\n{out}");

    assertion.restore();
    let artifact = c.test_binary().expect("the leg-2 test binary must exist");
    let (code, out) = run_prove_rebuild(
        &artifact,
        &c.root,
        &["test", "--offline", "--no-run", "--test", "verdict"],
    );
    assert_eq!(
        code, PASS,
        "prove-rebuild must observe a rebuild after a safe restore\n{out}"
    );
    assert!(
        out.contains("prove-rebuild: ok:"),
        "the receipt must be the command's, not a silent cargo 0:\n{out}"
    );
}

/// The helper's executable selftest (plants + CHARTER pair) is the wired
/// receipt. `cargo test -p cdcp_gate --test restore_rebuild_trap` is already
/// on the check.sh path; this is what makes a convention a gate.
#[test]
fn restore_safe_selftest_refuses_stale_and_runs_the_charter_pair() {
    let helper = engine_root().join("scripts/restore_safe.inc.sh");
    let out = Command::new("sh")
        .arg(&helper)
        .output()
        .expect("restore_safe selftest");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "restore_safe selftest must be GREEN on this tree\n{text}"
    );
    assert!(
        text.contains("prove-rebuild refused") || text.contains("known-bad RED — prove-rebuild"),
        "selftest must OBSERVE prove-rebuild refuse a stale tree:\n{text}"
    );
    assert!(
        text.contains("CHARTER pair 2/2"),
        "selftest must run the CHARTER pair, not only leg 1:\n{text}"
    );
    assert!(
        text.contains("absent artifact is ERROR"),
        "selftest must plant an absent artifact:\n{text}"
    );
}
