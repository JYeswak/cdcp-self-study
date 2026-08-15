//! THE SAFE RESTORE PATTERN — read this before you write a meta-test pair.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! THE HOUSE PATTERN AND THE STEP THAT ROTS
//! ─────────────────────────────────────────────────────────────────────────
//!
//! Every meta-test in this repo is a PAIR (`.flywheel/CHARTER.md`, `gauntlet`):
//!
//!   1. MUTATE the gate — one byte of its output, its exit code, its
//!      anti-vacuous branch — and confirm the suite goes non-zero.
//!   2. With that mutation STILL IN PLACE, delete the assertion and confirm
//!      the suite returns to zero.
//!   3. RESTORE.
//!
//! Step 3 is the one that rots, and **a bad step 3 does not corrupt the pair
//! that performed it — it corrupts the NEXT one.** The damage is deferred,
//! which is why it survives review.
//!
//! MECHANISM. `cp file file.bak` then `mv file.bak file` restores the CONTENT
//! but hands the file the BACKUP OBJECT'S mtime — `mv` is a rename, so the
//! inode, and its timestamps, come from the backup. That mtime is OLDER than
//! the artifact cargo already built from the perturbed source. Cargo decides
//! what to rebuild by comparing mtimes, sees nothing newer, and skips the
//! rebuild. The next `cargo test` runs the binary built from the PERTURBED
//! source while the source on disk reads correct.
//!
//! MEASURED 2026-08-14 (`goldens/PROVENANCE.md`, with the two mtimes): a
//! verification reported RED for a tree that was GREEN. A false RED is loud
//! and self-correcting — someone investigates. THE SAME MECHANISM PRODUCES A
//! FALSE GREEN, which is silent: restore the assertion after leg 2, skip the
//! rebuild, and the suite passes against a binary that has no assertion in it.
//! A meta-test that cannot fail is a fooled certificate — Sev-0 here.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! THE RULE
//! ─────────────────────────────────────────────────────────────────────────
//!
//!   RESTORE BY WRITING BYTES INTO THE EXISTING FILE — never `mv`, never
//!   `git mv`, never any rename. `cp bak file`, `git checkout -- file`,
//!   `printf … > file` and [`Restorable::restore`] all truncate-and-write the
//!   file that is already there, so its mtime becomes NOW.
//!
//!   THEN PROVE THE REBUILD RAN before any post-restore assertion:
//!   [`build_proving_rebuild`], or by hand
//!
//!       git checkout -- <file> && touch <file>
//!       cargo build -p <crate> --tests 2>&1 | tee /tmp/b.log; echo rc=${PIPESTATUS[0]}
//!       grep -q 'Compiling <crate>' /tmp/b.log || echo 'STALE — nothing rebuilt'
//!
//!   A BUILD THAT FINDS NOTHING TO REBUILD, WHEN THE FILE DEMONSTRABLY
//!   CHANGED, IS AN ERROR — NOT A PASS.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! WHY THIS SHAPE AND NOT ONE OF THE OTHER THREE
//! ─────────────────────────────────────────────────────────────────────────
//!
//! Measured on this machine, in a throwaway crate, all three candidates run
//! head to head against the same perturb-and-restore cycle
//! (`tests/restore_rebuild_trap.rs` re-runs the whole matrix):
//!
//! | restore | what `cargo build` did | verdict the suite reported |
//! |---|---|---|
//! | `mv bak file` then `cargo build` | nothing — `Finished in 0.00s`, `fresh: true` | **WRONG** |
//! | `cp bak file` then `cargo build` | recompiled — `fresh: false` | right |
//! | `mv bak file` + `touch` then `cargo build` | recompiled — `fresh: false` | right |
//!
//! **`touch` (candidate A) is the RECOVERY MOVE, not the pattern.** It works,
//! and `goldens/PROVENANCE.md` reaches for it correctly. But it is a SECOND
//! step, separate from the restore the author was already performing, and its
//! omission is silent. Between the `mv` and the `touch` there is a window in
//! which every command runs against the stale artifact. A convention whose
//! omission is invisible is the exact failure class this bead exists to close.
//! Keep `touch` for the case where you find yourself already past a `mv`.
//!
//! **Byte-rewrite (candidate B) is the mechanism, because it cannot be
//! forgotten — it IS the restore.** There is no second step to omit. Its
//! weakness is that it produces no evidence: if a helper is later "tidied"
//! back to a rename — and `mv` is the obvious idiom for "put it back" —
//! nothing complains.
//!
//! **A forced `cargo build` (candidate C) is the only leg that produces
//! evidence, AND ON ITS OWN IT IS VACUOUS.** This is the whole argument.
//! Measured above: after `mv`, `cargo build` exits 0, prints `Finished in
//! 0.00s`, and reports every artifact `fresh: true`. The obvious fix — "just
//! build before you assert" — passes in precisely the case it was written to
//! catch. So C is kept, but only with the freshness assertion attached: the
//! build must be OBSERVED to have rebuilt something.
//!
//! Hence B for the mechanism (cannot be forgotten) + C-with-an-assertion for
//! the proof (cannot be vacuous). Neither half is sufficient alone.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! THE CHECK THAT LOOKS RIGHT AND IS USELESS
//! ─────────────────────────────────────────────────────────────────────────
//!
//! Do NOT "prove" freshness by asserting `artifact_mtime > source_mtime`.
//! That is the ordering cargo itself tests, and THE TRAP SATISFIES IT: after
//! `mv`, the source carries the backup's OLD mtime and the artifact carries
//! the perturbed build's NEW one, so the comparison reads healthy for exactly
//! the tree that is poisoned. The provable fact is not an ordering between
//! two files, it is **"this build wrote this artifact"** — so compare the
//! artifact's mtime BEFORE the build with its mtime AFTER, which is what
//! [`build_proving_rebuild`] does. That comparison stays clock-free and needs
//! no agreement between the filesystem's clock and the process's.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::SystemTime;

/// The cargo that built this test, so a nested build cannot drift to a
/// different toolchain than the one under audit.
pub fn cargo_bin() -> String {
    option_env!("CARGO").unwrap_or("cargo").to_string()
}

/// A file captured before perturbation, so it can be put back without a rename.
///
/// Capture BEFORE you mutate, restore with [`Restorable::restore`], and prove
/// the rebuild with [`build_proving_rebuild`] before asserting anything.
pub struct Restorable {
    path: PathBuf,
    original: Vec<u8>,
}

impl Restorable {
    /// Read and hold the current bytes. This is the `cp file file.bak` step,
    /// except the backup lives in memory and therefore cannot be `mv`-ed.
    pub fn capture(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let original =
            std::fs::read(&path).unwrap_or_else(|e| panic!("capture {}: {e}", path.display()));
        Restorable { path, original }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The bytes as captured — for asserting the fixture is not vacuous.
    pub fn original(&self) -> &[u8] {
        &self.original
    }

    /// THE SAFE RESTORE: truncate-and-write the EXISTING file. No rename, so
    /// the mtime becomes now and cargo cannot mistake the file for older than
    /// the artifact built from its perturbed form.
    pub fn restore(&self) {
        std::fs::write(&self.path, &self.original)
            .unwrap_or_else(|e| panic!("restore {}: {e}", self.path.display()));
    }

    /// The trap, on purpose. Reproduces `cp file file.bak; …; mv file.bak file`
    /// by writing a backup file and renaming it over the perturbed source, so
    /// the restored file inherits the backup's OLD mtime.
    ///
    /// Exists ONLY so the fixture in `tests/restore_rebuild_trap.rs` can show
    /// the trap firing. Never use it to actually restore anything.
    pub fn restore_the_unsafe_way(&self) {
        let bak = self.path.with_extension("cdcp-trap-bak");
        std::fs::write(&bak, &self.original).expect("write backup");
        // Age the backup so its mtime is unambiguously older than any artifact
        // built from the perturbed source. In the shell form that ageing is
        // implicit — `cp file file.bak` happens BEFORE the perturbed build —
        // and it is stamped explicitly here so the reproduction cannot become
        // a race that happens to pass on a fast machine.
        age_to_2001(&bak);
        std::fs::rename(&bak, &self.path).expect("rename backup over source");
    }
}

/// Stamp a file's mtime back to 2001, older than anything this test can build.
///
/// `touch -t` (POSIX, `[[CC]YY]MMDDhhmm[.SS]`, interpreted in LOCAL time) is
/// used rather than a `filetime` dependency: the exact instant does not matter,
/// only that it precedes the build, and a whole-crate dependency to move one
/// timestamp in one fixture would be the larger cost.
fn age_to_2001(path: &Path) {
    let out = Command::new("touch")
        .arg("-t")
        .arg("200109090146.40")
        .arg(path)
        .output()
        .expect("touch");
    assert!(
        out.status.success(),
        "touch -t on {}: {}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The outcome of a build that was required to actually rebuild something.
#[derive(Debug)]
pub struct ProvenBuild {
    pub output: Output,
    /// mtime of the artifact before the build, `None` if it did not exist.
    pub before: Option<SystemTime>,
    pub after: SystemTime,
}

impl ProvenBuild {
    pub fn code(&self) -> i32 {
        self.output.status.code().unwrap_or(-1)
    }

    pub fn text(&self) -> String {
        let mut s = String::from_utf8_lossy(&self.output.stdout).into_owned();
        s.push_str(&String::from_utf8_lossy(&self.output.stderr));
        s
    }
}

/// Run `cargo <args>` in `manifest_dir` and PROVE it produced `artifact`.
///
/// Returns `Err` when the build succeeded but wrote nothing — the anti-vacuous
/// clause. That is the exact signature of the trap: the source on disk changed,
/// cargo compared mtimes, and decided it had nothing to do.
///
/// The proof is `mtime(artifact)` before vs after, never an ordering between
/// the artifact and its source; see the module header for why that ordering is
/// satisfied by the poisoned tree.
pub fn build_proving_rebuild(
    manifest_dir: &Path,
    artifact: &Path,
    args: &[&str],
) -> Result<ProvenBuild, String> {
    let before = std::fs::metadata(artifact).and_then(|m| m.modified()).ok();

    let output = nested_cargo(manifest_dir)
        .args(args)
        .output()
        .map_err(|e| format!("spawn cargo: {e}"))?;

    let after = std::fs::metadata(artifact)
        .and_then(|m| m.modified())
        .map_err(|e| {
            format!(
                "artifact {} missing after `cargo {}`: {e}\n{}{}",
                artifact.display(),
                args.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
        })?;

    let rebuilt = match before {
        None => true,
        Some(b) => after != b,
    };
    if !rebuilt {
        return Err(format!(
            "ANTI-VACUOUS FAILURE: `cargo {}` rebuilt nothing, but {} was restored. \
             The artifact still on disk was built from the PERTURBED source and any \
             verdict read from it is fabricated. Restore by writing bytes (never `mv`), \
             or `touch` the source, then build again.",
            args.join(" "),
            artifact.display(),
        ));
    }
    Ok(ProvenBuild {
        output,
        before,
        after,
    })
}

/// A `cargo` for a THROWAWAY project: the parent's target dir, flags and
/// wrappers are stripped so the child cannot write into, or read staleness
/// from, the workspace that is running the test.
pub fn nested_cargo(manifest_dir: &Path) -> Command {
    let mut c = Command::new(cargo_bin());
    c.current_dir(manifest_dir);
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
        c.env_remove(k);
    }
    c
}
