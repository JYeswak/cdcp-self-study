//! substrate-guard — S0 of the Rust migration (bd-substrate-rust-migration-jhd.1).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises a floor. It enforces one property over one snapshot pair:
//! **an entry git tracks that this gate can identify as non-Rust source carries
//! a reasoned, dated, bead-linked row in `registries/substrate_allowlist.toml`,
//! or the gate is RED.** Four shapes are identified (see `scan_reason`): a
//! scanned EXTENSION (case-insensitively — `.PY` is `.py`), a SHEBANG on a file
//! the extension rule cannot classify, a tracked SYMLINK, and a SUBMODULE
//! gitlink. The scanned surface is the WHOLE engine tree.
//!
//! # WHAT IT IS AND IS NOT: A PATH-AND-BYTES POLICY, ON THE PATHS GIT REPORTS
//!
//! This is a filename-and-path policy with one two-byte content probe. It cannot
//! decide that a `.rs` file is not secretly shelling out to Python —
//! `std::process::Command::new("python3")` is invisible here, and so is a Rust
//! `include_str!` of a script, and so is a script emitted at runtime. It reads
//! none of the code it permits and none of the code it clears. Anyone who states
//! the claim as "no Python runs in this project" has stated something this gate
//! never measured; what it measured is that no unreasoned non-Rust FILE is
//! tracked under a name or shape it recognises.
//!
//! # WHERE IT IS ENFORCED, AND WHERE IT IS NOT (bd-xmn5, measured 2026-08-14)
//!
//! Enforcement has two legs with very different strengths, and they are not
//! interchangeable:
//!
//! * **PRESENCE (strong).** Every run scans the whole index and the whole
//!   working tree. This is the leg `scripts/check.sh` runs, and it catches a
//!   violation no matter HOW the file arrived — ordinary commit, merge,
//!   cherry-pick, rebase, `git am`, `git commit-tree`, `--no-verify`, or a push
//!   from a clone that never installed a hook. It is retrospective: the file is
//!   already committed by then, and it stops the BUILD rather than the commit.
//! * **PRE-COMMIT (weak, and advisory by nature).** `hooks/pre-commit` runs the
//!   `--staged` leg. Git runs that hook for `git commit` only. MEASURED on git
//!   2.53.0, all with the hook installed and firing on an ordinary commit: a
//!   merge commit, a cherry-pick, a rebase, `git am`, and
//!   `git commit-tree $(git write-tree) -p HEAD` each created a commit carrying
//!   an unlisted `.py` with the hook NEVER INVOKED, as did
//!   `git -c core.hooksPath=/dev/null commit`. Hooks are also never cloned: a
//!   fresh clone has no `.git/hooks/pre-commit` at all until someone runs
//!   `cdcp_gate install-hooks`.
//!
//! So the honest sentence is: **the pre-commit hook is a courtesy on ordinary
//! `git commit` in a clone where it is installed; the presence leg in
//! `scripts/check.sh` is the floor (bd-efm7).** It trips no matter how the file
//! arrived — merge, cherry-pick, rebase, `git am`, `commit-tree`, `--no-verify`,
//! `core.hooksPath=/dev/null`, or a clone that never installed a hook. A client
//! flag cannot be ruled out; this repo has no required GitHub check and
//! no pre-receive hook (written decision). The presence scan + `cargo test`
//! live-tree test are the enforcement a client cannot skip. `install-hooks` is
//! how a clone gets the courtesy shim; check.sh installs it so `--check` is
//! meaningful in CI (bd-m67m). Do not say "refused at commit" without saying
//! which commit path.
//!
//! The test suite is now one of the places the presence leg bites, which it was
//! not before bd-xmn5:
//! `substrate_guard_e2e::the_live_repo_tree_has_no_unlisted_non_rust_file` runs
//! the scan against THIS repository, so `cargo test` goes red on an unlisted
//! script. Until that test existed, neither `cargo build --workspace` nor
//! `cargo test --workspace` scanned the tree at all, and only `scripts/check.sh`
//! did — so "it fails the build" was true of one shell orchestrator.
//!
//! # ONE SNAPSHOT PER VERDICT (bd-how)
//!
//! The subject and the policy must come from the SAME snapshot. Until 2026-08-14
//! they did not: candidate paths came from the git INDEX while the allowlist came
//! from the WORKING TREE, so staging `scripts/payload.py` and leaving its
//! `[[allow]]` row unstaged returned exit 0 on both legs — the gate approving a
//! tree that had never existed and never would. Confirmed by injection.
//!
//! Two snapshots are now read and BOTH must be clean:
//!
//! * **working tree** — tracked files present on disk, judged by the allowlist
//!   and `scripts/check.sh` on disk. This is the developer's desk.
//! * **index** — every path `git ls-files` reports, judged by the allowlist and
//!   `scripts/check.sh` as `git show :./…` returns them. This is the tree the
//!   next commit creates.
//!
//! Each snapshot is internally consistent, so the ordinary workflows stay green:
//! staging a script together with its row passes, and deleting a script together
//! with its row passes. A policy file missing from the index is an ERROR — a
//! commit that deletes the allowlist is not a commit with nothing to check.
//!
//! # WHAT THIS GATE CANNOT DO
//!
//! It cannot decide whether a stated `reason` is honest English. One mechanical
//! exception (bd-allowlist-stale-load-bearing-seq9, extended
//! bd-retire-oracle-on-behaviour-change-gna0): a reason that claims this path
//! is still the live oracle ("load-bearing check.sh", "check.sh invokes",
//! "check.sh hard-fails if", "byte-exact oracle", "oracle required") is checked
//! against the invocation set derived from `scripts/check.sh`. Presence tests
//! (`[ -f path ]`) are not invocations but they ARE reachability: a "not on
//! the check.sh path" reason for a presence-checked file is RED. Comments
//! are neither. The rest of the sentence is still prose. It cannot tell a real migration bead from a plausible-looking
//! id. It cannot tell an achievable `expires` from a date chosen to be far away.
//! It reads none of the scripts it permits, so it says nothing about what they
//! do. An author who wants a script in this tree can still get one in by
//! writing a sentence — the change is that the sentence is now dated,
//! attributed, reviewable in a diff, and it rots.
//!
//! `[oracle_inventory]` (same bead) is the remaining
//! `scripts/{verify,validate,smoke}_*.py` table. When that table is present,
//! an empty scan of those names is an ERROR, a row whose file is gone is an
//! ERROR, and an unlisted remaining oracle is an ERROR. Fixtures omit the
//! table and the leg does not run.
//!
//! The invocation set itself is TRANSITIVE (bd-check-sh-transitive-invocation-gzvb,
//! leftover presence/mjs: bd-transitive-invocation-blindspot-lcfj).
//! A grep of `scripts/check.sh` does not enumerate what that file runs:
//! `sh scripts/smoke_slo.sh` hides `python3 scripts/verify_bank.py`, and
//! `CHECKER="scripts/verify_doc_consistency.py"` then `python3 "$CHECKER"` is
//! invisible to a filename grep. `walk_invocations` follows sourced / `sh`
//! children, resolves single-assignment `$VAR` targets for `python3` / `node` /
//! `cargo run`, reports every `[ -f path ]` presence check as a DISTINCT set
//! (not an invoke), includes `node`/`.mjs` in the inventory (`js` stays
//! out of the scan floor — bd-yp9x COST), and treats an empty walk
//! as an ERROR on the live orchestrator. A reason that says "not a check.sh
//! step" / "not on the check.sh path" for a path the walk reaches (invoke OR
//! presence) is RED; the inverse (load-bearing / oracle-required for a path
//! nothing reaches) is also RED. Comments are not invokes. This walk cannot
//! decide that a Rust `Command::new("python3")` runs, and it does not follow
//! `node` children into other `.mjs`.
//!
//! The identification legs have named blind spots, kept here rather than in a
//! bead so they are read by whoever edits the rule:
//!
//! * A script under an extension that is neither scanned nor absent —
//!   `scripts/payload.txt` holding Python — is caught ONLY if git records it
//!   executable or it has a shebang AND no extension. `has_no_extension` tests
//!   the BASENAME, so `scripts/.hidden` counts as extensioned and is not sniffed.
//! * The shebang probe reads the first two bytes of the blob. It says "this is
//!   an executable text script", not "this is Python". Any interpreter counts,
//!   deliberately: the substrate law is about non-Rust source, not about which
//!   non-Rust language.
//! * A symlink and a gitlink are reported as needing a row because the gate
//!   CANNOT see through them, not because it found anything. A row for one is a
//!   human saying they looked.
//!
//! It also cannot decide, by reading `scripts/check.sh`, that the step invoking
//! this gate EXECUTES. No text test can (bd-bo6i): `: "cargo run … "` is a no-op,
//! `true # cargo run …` is a comment, and `cargo run … || true` runs the gate and
//! throws its verdict away — all three read as an invocation. The text leg is
//! therefore demoted to what it can actually do: it SUBTRACTS. It reports ABSENT
//! when nothing names the gate, INERT when every occurrence matches a compiled-in
//! disqualifier, and otherwise UNPROVEN — never "wired". The behavioural leg is
//! `--prove-wired`, which materialises the index, plants an unlisted `.py`, runs
//! `scripts/check.sh` for real and requires check.sh ITSELF to exit non-zero. An
//! inert line cannot satisfy that. What `--prove-wired` still cannot decide: that
//! every OTHER step in check.sh propagates its own failures, and that the tree
//! outside the index (unstaged edits, untracked files) is clean.
//!
//! `--prove-wired` first asks whether planting the known-bad is still meaningful
//! against the registry the snapshot carries — see `probe_plant_vacuity`. That
//! precondition reads PARSED `[[allow]]` rows, not the registry's bytes: a
//! substring scan answered "do these characters occur in this file", which is a
//! different question from "is this path exempt", and on 2026-08-14 it took
//! check.sh RED over the file's OWN COMMENT warning nobody to add such a row
//! (bd-ip10). A registry that will not parse is an ERROR there, because bytes stay
//! readable when rows do not, and the plant must never go quietly exempt.
//!
//! # WHERE TEXT IS STILL READ AS TEXT, DELIBERATELY
//!
//! The check.sh wiring leg above matches shell lines by substring, and the probe
//! attributes a transcript to this gate by substring. Those stay text tests
//! because no parse of a shell script or of a build log settles what they are
//! asked. They are therefore worded so that only SUBTRACTION is claimed — ABSENT,
//! INERT, UNPROVEN, `Unattributable` — never "wired". A `.contains` deciding a
//! fact about STRUCTURE (which paths a registry exempts) is a defect; a
//! `.contains` forming a subtractive HEURISTIC about text is the honest ceiling.
//!
//! The floor moves from *silence* to *a signed, expiring exemption*, and from
//! *a string appears in check.sh* to *a planted known-bad stops check.sh*. That
//! is the whole of the claim; this header will not pretend otherwise.
//!
//! # ONE WIDENING, STATED (bd-n1aj, 2026-08-14)
//!
//! On 2026-08-14 this gate PERMITS something it used to reject: an `[[allow]]`
//! row whose `path` carries two dots or a backslash inside a FILENAME —
//! `scripts/payload..py`, `scripts/a\b.py`. Rows with a `.` or `..` path
//! COMPONENT, and absolute paths, are rejected exactly as before; that is
//! traversal and it stays out.
//!
//! This is a widening, so it is written down rather than slipped in. It was not
//! a policy the gate held on purpose: `is_in_scope` was moved from a substring
//! test to a component test in tick 4 and `validate_rows` was not, so those two
//! files were IN SCOPE — the gate demanded a row for them — while every row that
//! could authorise them was rejected as malformed at exit 4. Both legs measured.
//! Nothing was getting through (the state was fail-closed both ways), so the harm
//! was not exposure; the harm was that an author who did exactly what the gate
//! asked still could not go green, and a gate nobody can satisfy is a gate that
//! gets routed around. The widening is therefore bounded to precisely the paths
//! the gate itself says need a row.
//!
//! The two halves now call ONE function, `normalisation_defect`. What that buys
//! is checked directly rather than asserted: over a corpus of paths, every path
//! `unlisted` would demand a row for must accept a well-formed row
//! (`no_in_scope_path_can_be_un_allowlistable`). What it does not settle is in
//! that function's own doc — backslash-as-Windows-separator and whitespace
//! padding are both left to other legs.
//!
//! # THE SCOPE DECISION (bd-xmn5, 2026-08-14) — RECORDED, WITH ITS REASONS
//!
//! Adversarial review found the floor was a filename policy over two directories,
//! and every gap below was confirmed by injection against the built binary before
//! it was closed. Each answer is written down here because the next reader will
//! otherwise re-litigate it.
//!
//! * **Extensions are matched CASE-INSENSITIVELY.** `scripts/payload.PY` and
//!   `scripts/payload.Py` were measured at exit 0. Cost of the fix: zero — the
//!   tree tracks no upper-case `.py`/`.sh` today. A rule that a rename defeats is
//!   not a rule.
//! * **The extension floor is the SHELL, PYTHON, AND CHECK.SH-INVOKED NODE
//!   FAMILY**, not two spellings: `py`, `pyw`, `sh`, `bash`, `zsh`, `ksh`,
//!   `mjs`. `scripts/payload.bash` was measured at exit 0. bd-yp9x (2026-08-15)
//!   split what xmn5 left as one omission. The four `scripts/*.mjs` that
//!   `scripts/check.sh` runs (`node scripts/smoke_*.mjs || fail`) plus the
//!   two tracked node helpers on that path ARE the same class as the remaining
//!   `.py` — load-bearing non-Rust gates — and they are now in the floor.
//!   An unlisted `scripts/x.mjs` is RED. `.js` STAYS OUT: `web/assets/js`
//!   (18 tracked learner files) plus `web/data/module_learn_slugs.js` is the
//!   browser product surface; its migration is wasm (`cdcp_wasm` / dual-path),
//!   not a `cdcp_gate` subcommand, and folding it in would make the row count
//!   stop meaning python/shell/node-gate debt. COST (permanent, same form as
//!   the plzm `beads_compliance_audit/` exclusion): an unreasoned `.js` can
//!   be committed and this gate is silent. The L5/L6 node smokes and the wasm
//!   dual-path bind that surface, not this floor. If `[scan].extensions`
//!   names `js` or `mjs`, a live snapshot that tracks zero files of that
//!   extension is ERROR — an empty scan is not a pass (bd-yp9x).
//! * **Shebang sniffing: YES, but only where the extension rule is blind** —
//!   a basename with no extension, or an entry git records `100755`.
//!   `scripts/payload` holding `#!/usr/bin/env python3` was measured at exit 0.
//!   Sniffing every tracked file would mean one `git show` per file per snapshot;
//!   the chosen predicate costs 21 blob reads in this tree and catches the shape
//!   an author actually uses for a script. What it misses is written above, in
//!   the CANNOT section, rather than left for someone to discover.
//! * **Roots: the WHOLE ENGINE TREE, with no ignore list.** `docs/payload.py`
//!   was measured at exit 0, and four real shell files had been living outside
//!   the floor since the gate was written —
//!   `.flywheel/scripts/publishability-bar.sh`, `.flywheel/watchdog.sh`,
//!   `tests/publishability-bar.sh`, `tests/voice-slop.sh`, the last two of which
//!   `scripts/check.sh` INVOKES as gates. An ignore list was rejected on purpose:
//!   it is a widening surface with a one-line diff, which is the thing
//!   `check_floor` exists to stop. `[scan].roots` is retained as a floor-checked
//!   MINIMUM so that if whole-tree scanning is ever narrowed, `scripts/` and
//!   `crates/` cannot be dropped in the same edit.
//! * **Symlinks and gitlinks need a row.** A tracked directory symlink
//!   `scripts/linkdir -> /elsewhere` holding `hidden.py` was measured at exit 0,
//!   with `hidden.py` readable through it on disk. git reports mode `120000` and
//!   one path; the tree beneath is not in this repository at all. The gate cannot
//!   see through it, so it refuses to be silent about it. Cost today: zero, the
//!   tree tracks no symlink and no submodule.
//!
//! # WHY IT IS RUST
//!
//! The guard that bans shell is not itself shell. `hooks/pre-commit` is a shim
//! whose entire body is one `exec` of this binary; it holds no decision logic.
//!
//! # THE ALLOWLIST IS THE WORKLIST
//!
//! Every row is a debt. `expires` is what stops a temporary exemption from being
//! permanent by another name. Row count is the migration progress metric and its
//! target is zero.

use crate::date::{self, Ymd};
use crate::registry::{GateCtx, GateError};
use crate::vcs;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const NAME: &str = "substrate-guard";
pub const SUMMARY: &str =
    "no .py/.sh/.mjs may enter the engine tree without a reasoned, dated, bead-linked allowlist row";

/// Where the registry lives, relative to the engine root.
pub const REGISTRY_PATH: &str = "registries/substrate_allowlist.toml";

/// The ONE file whose wiring this gate judges.
///
/// `[wiring].check_sh` used to be free text, so pointing it at any file holding a
/// suitable string satisfied the wiring leg from a file nothing runs (bd-bo6i).
/// It is now pinned; a different value is a schema ERROR.
pub const CHECK_SH_PATH: &str = "scripts/check.sh";

/// Extensions the registry may WIDEN but may never narrow below. Matched
/// case-insensitively, so `.PY` is not a second spelling of an escape hatch.
pub const FLOOR_EXTENSIONS: &[&str] = &["py", "pyw", "sh", "bash", "zsh", "ksh", "mjs"];
/// Directories the registry may ADD to but may never drop.
///
/// Since bd-xmn5 the scanned surface is the whole engine tree, so this list is a
/// retained MINIMUM rather than the boundary: if `WHOLE_TREE_SCOPE` is ever
/// turned off, these roots are still mandatory and cannot go in the same edit.
pub const FLOOR_ROOTS: &[&str] = &["scripts", "crates"];

/// The scanned surface is every normalised path git reports under the engine
/// root. See the SCOPE DECISION section; an ignore list was rejected because it
/// is a one-line widening surface.
pub const WHOLE_TREE_SCOPE: bool = true;

/// git's mode for a symlink blob. The link is what is tracked; the tree beneath
/// it is not in this repository.
pub const SYMLINK_MODE: &str = "120000";
/// git's mode for a submodule gitlink — a commit id, opaque from here.
pub const GITLINK_MODE: &str = "160000";
/// git's mode for a file with the executable bit set.
pub const EXECUTABLE_MODE: &str = "100755";
/// A reason shorter than this is not a reason.
pub const MIN_REASON_LEN: usize = 24;

/// Set in the environment of the child `check.sh` the behavioural probe runs, so
/// a probe cannot re-enter itself.
pub const PROBE_ENV: &str = "CDCP_SUBSTRATE_PROBE";
/// The known-bad the probe plants. Unlisted on purpose; if the registry ever
/// lists it the probe is vacuous and says so.
pub const PROBE_PLANT: &str = "scripts/__cdcp_probe_unlisted__.py";
const PROBE_TIMEOUT_ENV: &str = "CDCP_SUBSTRATE_PROBE_TIMEOUT_SECS";
const PROBE_DEFAULT_TIMEOUT_SECS: u64 = 600;
/// Scratch root for the probe, under `target/` so it is git-ignored and
/// `cargo clean` disposes of it.
const PROBE_DIR: &str = "target/cdcp-substrate-probe";

const KNOWN_FLAGS: &[&str] = &["--staged", "--verify-wired", "--prove-wired", "--quiet"];

// ── registry schema ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Allowlist {
    pub schema_version: u32,
    pub scan: ScanCfg,
    pub wiring: Wiring,
    #[serde(default)]
    pub allow: Vec<Row>,
    /// Present only on the live registry. Fixtures omit it and the inventory
    /// leg does not run (bd-retire-oracle-on-behaviour-change-gna0).
    #[serde(default)]
    pub oracle_inventory: Option<OracleInventory>,
}

/// Remaining `scripts/{verify,validate,smoke}_*.py` after EXTRACT-THEN-DELETE.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OracleInventory {
    #[serde(default, rename = "file")]
    pub files: Vec<OracleInventoryFile>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OracleInventoryFile {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub disposition: String,
    #[serde(default)]
    pub why: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanCfg {
    pub roots: Vec<String>,
    pub extensions: Vec<String>,
    pub include_engine_root_files: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wiring {
    /// `"pending"` — check.sh has not been wired yet; report but do not fail.
    /// `"wired"`   — check.sh must invoke this gate; its absence is RED.
    /// Anything else (including empty) is a schema ERROR. Blank is never permissive.
    /// Moving "wired" back to "pending" is a RATCHET violation, not an edit.
    pub status: String,
    pub check_sh: String,
    pub invocation: String,
    pub bead: String,
}

/// One exemption. Every field is load-bearing; `#[serde(default)]` exists so a
/// MISSING field lands here as an empty string and is reported as the schema
/// error it is, rather than as an opaque TOML parse failure.
#[derive(Debug, Clone, Deserialize)]
pub struct Row {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub migration_bead: String,
    #[serde(default)]
    pub expires: String,
}

/// Which tree a finding came from. A finding present in both is reported once,
/// unlabelled; a finding present in only one names its snapshot, because that
/// difference is exactly the bug bd-how describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Snapshot {
    Worktree,
    Index,
}

impl Snapshot {
    pub fn label(self) -> &'static str {
        match self {
            Snapshot::Worktree => "working tree only",
            Snapshot::Index => "staged snapshot (the tree this commit creates)",
        }
    }
}

// ── pure logic (unit-tested without git, without a filesystem) ─────────────

pub fn parse_allowlist(text: &str) -> Result<Allowlist, String> {
    let a: Allowlist = toml::from_str(text).map_err(|e| format!("parse {REGISTRY_PATH}: {e}"))?;
    if a.schema_version != 1 {
        return Err(format!(
            "{REGISTRY_PATH}: schema_version {} unsupported (expected 1)",
            a.schema_version
        ));
    }
    Ok(a)
}

/// The registry configures the scan, so the registry is itself an attack surface:
/// dropping `"py"` from `extensions` would disable the gate with a one-word diff.
/// The floor is compiled in and the registry may only widen it.
pub fn check_floor(scan: &ScanCfg) -> Vec<String> {
    let mut v = Vec::new();
    for ext in FLOOR_EXTENSIONS {
        if !scan.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            v.push(format!(
                "{REGISTRY_PATH}: [scan].extensions is missing the compiled-in floor {ext:?} — the registry may widen the scan, never narrow it"
            ));
        }
    }
    for r in FLOOR_ROOTS {
        if !scan.roots.iter().any(|e| e == r) {
            v.push(format!(
                "{REGISTRY_PATH}: [scan].roots is missing the compiled-in floor {r:?} — the registry may widen the scan, never narrow it"
            ));
        }
    }
    if !scan.include_engine_root_files {
        v.push(format!(
            "{REGISTRY_PATH}: [scan].include_engine_root_files = false narrows the compiled-in floor"
        ));
    }
    if scan.extensions.iter().any(|e| e.starts_with('.')) {
        v.push(format!(
            "{REGISTRY_PATH}: [scan].extensions must be bare (\"py\"), not dotted (\".py\")"
        ));
    }
    v
}

/// Anti-vacuous for the js/mjs family (bd-yp9x).
///
/// `py`/`sh` cannot go empty here without deleting the floor itself. `js` and
/// `mjs` can: adding the token to `[scan].extensions` without any tracked file
/// of that extension is a claim that nothing judges. When the scan list names
/// `js` or `mjs`, a snapshot that tracks zero files of that extension is ERROR.
/// Callers apply this on the live registry (fixtures omit `[oracle_inventory]`
/// and list `mjs` only so `check_floor` cannot be narrowed).
pub fn empty_js_family_scan(scan: &ScanCfg, entries: &[Entry]) -> Vec<String> {
    let mut out = Vec::new();
    for ext in &scan.extensions {
        let e = ext.to_ascii_lowercase();
        if e != "js" && e != "mjs" {
            continue;
        }
        let hits = entries
            .iter()
            .filter(|ent| extension_of(&ent.path).as_deref() == Some(e.as_str()))
            .count();
        if hits == 0 {
            out.push(format!(
                "{REGISTRY_PATH}: [scan].extensions claims {e:?} but this snapshot tracks 0 .{e} files — empty scan is ERROR, not a pass"
            ));
        }
    }
    out
}

pub fn check_wiring_status(w: &Wiring) -> Vec<String> {
    let mut v = Vec::new();
    match w.status.trim() {
        "pending" | "wired" => {}
        "" => v.push(format!(
            "{REGISTRY_PATH}: [wiring].status is empty — blank is never permissive; use \"pending\" or \"wired\""
        )),
        other => v.push(format!(
            "{REGISTRY_PATH}: [wiring].status {other:?} is not \"pending\" or \"wired\""
        )),
    }
    if w.invocation.trim().is_empty() {
        v.push(format!("{REGISTRY_PATH}: [wiring].invocation is empty"));
    }
    let check_sh = w.check_sh.trim();
    if check_sh.is_empty() {
        v.push(format!("{REGISTRY_PATH}: [wiring].check_sh is empty"));
    } else if check_sh != CHECK_SH_PATH {
        v.push(format!(
            "{REGISTRY_PATH}: [wiring].check_sh is {check_sh:?}; this gate's wiring leg is pinned to {CHECK_SH_PATH:?}. Repointing it satisfies the wiring leg from a file nothing runs — ERROR, not a pass"
        ));
    }
    if w.bead.trim().is_empty() {
        v.push(format!("{REGISTRY_PATH}: [wiring].bead is empty"));
    }
    v
}

/// `[wiring].status` is a RATCHET.
///
/// Once a commit has declared the gate wired, a later commit may not quietly
/// declare it pending again: "pending" exists so the commit that INSTALLS the
/// step is not blocked by its own absence, not as an off switch. Un-wiring a live
/// gate is a decision to argue in a bead, not one to make by editing one word.
pub fn check_wiring_ratchet(head_status: Option<&str>, current: &str) -> Option<String> {
    let head = head_status.unwrap_or("").trim();
    let now = current.trim();
    if head == "wired" && now != "wired" {
        return Some(format!(
            "{REGISTRY_PATH}: [wiring].status was \"wired\" at HEAD and is {now:?} here — wiring is a ratchet, not a toggle"
        ));
    }
    None
}

/// Bead ids look like `bd-<slug>` / `cp-<slug>`, optionally dotted.
pub fn looks_like_bead_id(s: &str) -> bool {
    let s = s.trim();
    let Some(rest) = s.strip_prefix("bd-").or_else(|| s.strip_prefix("cp-")) else {
        return false;
    };
    !rest.is_empty()
        && rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// THE traversal test. One implementation, called by both halves of the gate.
///
/// Returns why `path` is not a normalised engine-root-relative path, or `None`
/// when it is. `.`/`..` are only traversal as whole COMPONENTS; a dot inside a
/// filename is a filename.
///
/// # WHY THIS IS A SHARED FUNCTION AND NOT TWO SIMILAR CHECKS
///
/// This gate asks the same structural question in two places, and the two
/// answers have to be the same answer:
///
/// * `is_in_scope` decides that a file NEEDS an `[[allow]]` row.
/// * `validate_rows` decides that a row is WELL-FORMED.
///
/// When those disagree, a path can be in scope and un-allowlistable at the same
/// time — nothing gets through (the gate is fail-closed either way), but the
/// author cannot comply, and a rule nobody can comply with is the rule that gets
/// routed around. That state existed here from 2026-08-14 (tick 4) to
/// 2026-08-14 (bd-n1aj): `is_in_scope` was moved to a component test while
/// `validate_rows` kept `path.contains("..")` and `path.contains('\\')`, so
/// `scripts/payload..py` and `scripts/a\b.py` — ordinary files in a mandatory
/// root, both measured as demanding a row — had every row that could authorise
/// them rejected at exit 4, "`path` must be a normalised engine-root-relative
/// path". Both halves had been consistently wrong before tick 4, which is why
/// nothing tripped until one of them was fixed alone.
///
/// Rewriting the second test to look like the first would have left two
/// implementations that a later edit can separate again. There is now one, and
/// the property that matters — every path the gate demands a row for can be
/// given one — is asserted directly over a corpus in
/// `no_in_scope_path_can_be_un_allowlistable`.
///
/// # WHAT THIS DOES NOT SETTLE
///
/// A backslash is an ordinary byte in a POSIX filename and is treated as one, so
/// this does not decide anything about Windows-shaped paths: a row for
/// `scripts\foo.py` is caught instead by the leg that requires a file to exist at
/// the path. Nor does it settle whitespace: `validate_rows` trims a row's `path`
/// before comparing, so a file whose name is padded with spaces cannot be given a
/// row. That trim stays, because it is what makes `path = "   "` the schema ERROR
/// it should be, and a padded filename is the rarer harm.
pub fn normalisation_defect(path: &str) -> Option<&'static str> {
    if path.is_empty() {
        return Some("empty");
    }
    if path.starts_with('/') {
        return Some("absolute; paths here are relative to the engine root");
    }
    if path.split('/').any(|c| c == ".." || c == ".") {
        return Some("has a `.` or `..` path COMPONENT, which is traversal");
    }
    None
}

/// Is this engine-root-relative path inside the scanned surface?
///
/// SECURITY NOTE (adversarial review 2026-08-14, confirmed by injection): this
/// used to reject `path.contains("..")`, which excluded any path with two dots
/// ANYWHERE in it. `scripts/payload..py` is an ordinary Python file in a
/// mandatory root, and it fell straight out of scope — measured exit 0 on both
/// the presence and staged legs. The traversal guard must test path COMPONENTS,
/// not a substring; a filename is not a traversal. That test now lives in
/// `normalisation_defect`, which `validate_rows` calls too.
/// SCOPE (bd-xmn5, 2026-08-14): the whole engine tree.
///
/// It used to be `scripts/` + `crates/` + engine-root files, and `docs/payload.py`
/// was measured at exit 0 while four real shell files — two of them invoked as
/// gates by `scripts/check.sh` — had never been inside the floor at all. A floor
/// with a listed inside is a floor with a much larger listed OUTSIDE, and nothing
/// tells you when something moves there.
///
/// `roots` / `include_engine_root_files` are still schema-checked against
/// `FLOOR_ROOTS` (see `check_floor`) so that narrowing the scan stays a
/// multi-line, reviewable act rather than a one-word edit.
pub fn is_in_scope(path: &str, scan: &ScanCfg) -> bool {
    if normalisation_defect(path).is_some() {
        return false;
    }
    if WHOLE_TREE_SCOPE {
        return true;
    }
    match path.split_once('/') {
        // Engine-root file: no directory component.
        None => scan.include_engine_root_files,
        Some((head, _)) => scan.roots.iter().any(|r| r == head),
    }
}

/// The last dot-separated component of the BASENAME, lower-cased, or `None` when
/// the basename carries no dot at all.
pub fn extension_of(path: &str) -> Option<String> {
    let base = basename(path);
    base.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())
}

pub fn basename(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((_, b)) => b,
        None => path,
    }
}

/// A basename with no dot at all — the shape an executable script takes when it
/// is meant to be typed rather than imported. `scripts/.hidden` does NOT qualify:
/// the test is on the basename, dot and all, and that limit is stated in the
/// header rather than papered over.
pub fn has_no_extension(path: &str) -> bool {
    !basename(path).contains('.')
}

/// Case-insensitive since bd-xmn5. `scripts/payload.PY` was measured at exit 0
/// on both the presence and the staged legs; a policy a `mv` defeats is not one.
pub fn has_scanned_extension(path: &str, scan: &ScanCfg) -> bool {
    let Some(ext) = extension_of(path) else {
        return false;
    };
    scan.extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext))
}

/// Does this blob begin with a `#!` line? Bytes, not text: a tracked `.wasm` is
/// mode 100755 in this very repo, and turning "is this a script" into an ERROR
/// over a UTF-8 decode would be a gate failing on the wrong question.
pub fn shebang_line(bytes: &[u8]) -> Option<String> {
    if !bytes.starts_with(b"#!") {
        return None;
    }
    let end = bytes
        .iter()
        .position(|b| *b == b'\n')
        .unwrap_or(bytes.len());
    let line = String::from_utf8_lossy(&bytes[..end.min(200)])
        .trim()
        .to_string();
    Some(line)
}

/// Is this entry one whose CONTENT the gate needs to look at?
///
/// Only where the extension rule is blind: a basename with no extension, or an
/// entry git records executable. Everything else is decided from the path, and
/// the cost stays at ~20 blob reads per snapshot instead of one per tracked file.
pub fn needs_content_probe(path: &str, mode: &str, scan: &ScanCfg) -> bool {
    if has_scanned_extension(path, scan) {
        return false;
    }
    has_no_extension(path) || mode == EXECUTABLE_MODE
}

/// One tracked entry, as ONE snapshot sees it. `shebang` is `None` both when the
/// content was not a script and when the content was never probed — the two are
/// the same answer here, and `needs_content_probe` decides which files get asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub mode: String,
    pub shebang: Option<String>,
}

impl Entry {
    /// An ordinary non-executable file, judged on its path alone.
    pub fn plain(path: &str) -> Entry {
        Entry {
            path: path.to_string(),
            mode: "100644".to_string(),
            shebang: None,
        }
    }
}

/// WHY this entry must carry an `[[allow]]` row, or `None` when it need not.
///
/// This is the one place the four identification shapes live. Adding a fifth
/// means adding it here and nowhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanReason {
    /// A scanned extension, matched case-insensitively.
    Extension(String),
    /// A `#!` line on a file the extension rule could not classify.
    Shebang(String),
    /// git tracks the LINK; whatever it points at is not in this repository.
    Symlink,
    /// A submodule: a commit id in another repository.
    Gitlink,
}

impl ScanReason {
    /// The phrase that goes in the violation, chosen so the reader can tell
    /// which leg fired without reading this file.
    pub fn describe(&self) -> String {
        match self {
            ScanReason::Extension(e) => format!("non-Rust file (.{e})"),
            ScanReason::Shebang(l) => format!(
                "executable text script — no scanned extension, but its first line is {l:?}"
            ),
            ScanReason::Symlink => "tracked SYMLINK — git records the link, not the tree beneath it, so this gate cannot see what it admits".to_string(),
            ScanReason::Gitlink => "tracked SUBMODULE gitlink — a commit id in another repository, opaque to this gate".to_string(),
        }
    }
}

pub fn scan_reason(e: &Entry, scan: &ScanCfg) -> Option<ScanReason> {
    if !is_in_scope(&e.path, scan) {
        return None;
    }
    if e.mode == GITLINK_MODE {
        return Some(ScanReason::Gitlink);
    }
    if e.mode == SYMLINK_MODE {
        return Some(ScanReason::Symlink);
    }
    if has_scanned_extension(&e.path, scan) {
        return Some(ScanReason::Extension(
            extension_of(&e.path).unwrap_or_default(),
        ));
    }
    e.shebang.clone().map(ScanReason::Shebang)
}

/// Schema validation of the rows themselves. `exists` answers "is there a file at
/// this path IN THE SNAPSHOT BEING JUDGED" — on disk for the working tree, in the
/// index for the commit — and is injected so this stays a pure function under
/// test and so neither snapshot can borrow the other's answer.
pub fn validate_rows(
    rows: &[Row],
    scan: &ScanCfg,
    today: Ymd,
    exists: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut v = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();

    for (i, r) in rows.iter().enumerate() {
        let where_ = if r.path.trim().is_empty() {
            format!("[[allow]] #{}", i + 1)
        } else {
            format!("[[allow]] {}", r.path.trim())
        };

        if r.path.trim().is_empty() {
            v.push(format!("{where_}: empty `path`"));
            continue;
        }
        let path = r.path.trim();
        // SWEEP RESOLVED (bd-ip10 -> bd-n1aj, 2026-08-14): this used to be its own
        // substring test — `contains("..") || contains('\\')` — which contradicted
        // the component test in `is_in_scope` and made every in-scope path with two
        // dots or a backslash in its NAME un-allowlistable. It now calls the same
        // function `is_in_scope` calls, so the two cannot answer differently.
        if let Some(why) = normalisation_defect(path) {
            v.push(format!(
                "{where_}: `path` must be a normalised engine-root-relative path ({why})"
            ));
        }
        if !seen.insert(path) {
            v.push(format!("{where_}: duplicate `path` row"));
        }
        if !is_in_scope(path, scan) {
            v.push(format!(
                "{where_}: outside the scanned surface ({}, or an engine-root file) — an exemption for something the gate never scans is dead weight",
                scan.roots.join("/, ")
            ));
        }
        // NOTE (bd-xmn5): the "extension is not one this gate scans" test used to
        // live here. It could not survive the shebang and symlink legs — a row for
        // `hooks/pre-commit` (no extension) or for a tracked symlink named
        // `x.md` is a row the gate DEMANDS, and this function would have rejected
        // every one of them. That is precisely the bd-n1aj shape: in scope and
        // un-allowlistable at the same time. Dead-weight detection moved to
        // `dead_rows`, which asks `scan_reason` — the same function `unlisted_entries`
        // asks — so the two answers are one answer.

        // reason — the whole point of the row
        let reason = r.reason.trim();
        if reason.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `reason` — a blank reason is a SCHEMA ERROR, never permission"
            ));
        } else if reason.len() < MIN_REASON_LEN {
            v.push(format!(
                "{where_}: `reason` is {} chars; at least {MIN_REASON_LEN} are needed to say anything a reviewer can disagree with",
                reason.len()
            ));
        }

        // migration_bead — who owns the debt
        let bead = r.migration_bead.trim();
        if bead.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `migration_bead` — an exemption nobody owns is not tracked work"
            ));
        } else if !looks_like_bead_id(bead) {
            v.push(format!(
                "{where_}: `migration_bead` {bead:?} is not a bead id (bd-… / cp-…)"
            ));
        }

        // expires — what stops "temporary" from meaning "forever"
        let expires = r.expires.trim();
        if expires.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `expires` — an exemption that cannot expire is permanent by another name"
            ));
        } else {
            match date::parse_ymd(expires) {
                Err(e) => v.push(format!("{where_}: `expires` {e}")),
                Ok(d) if date::before(d, today) => v.push(format!(
                    "{where_}: EXPIRED on {expires} (today is {:04}-{:02}-{:02}) — port it to Rust under {bead}, or re-affirm the row with a new date and a reason that survives review",
                    today.0, today.1, today.2
                )),
                Ok(_) => {}
            }
        }

        // A row for a file that is gone is the migration's own litter.
        if !exists(path) {
            v.push(format!(
                "{where_}: no file at this path — if it was ported or deleted, delete the row (the allowlist is the worklist; it shrinks to zero)"
            ));
        }
    }
    v
}

/// Entries the gate identified as non-Rust, with no row. `rows` is assumed
/// already schema-checked.
pub fn unlisted_entries(entries: &[Entry], rows: &[Row], scan: &ScanCfg) -> Vec<String> {
    let listed: BTreeSet<&str> = rows.iter().map(|r| r.path.trim()).collect();
    let mut out = Vec::new();
    for e in entries {
        let Some(reason) = scan_reason(e, scan) else {
            continue;
        };
        if !listed.contains(e.path.as_str()) {
            out.push(format!(
                "{}: {} with no row in {REGISTRY_PATH}. Port it to Rust (see epic bd-substrate-rust-migration-jhd), or add a row with a real `reason`, a `migration_bead`, and an `expires` date",
                e.path,
                reason.describe()
            ));
        }
    }
    out
}

/// Rows for entries this snapshot tracks but does NOT identify as non-Rust.
///
/// The allowlist is the worklist, so a row that exempts nothing is litter — and
/// worse, it is litter that reads like tracked debt. This is the exact
/// complement of `unlisted_entries`: that one demands a row wherever
/// `scan_reason` is `Some`, this one rejects a row wherever it is `None`. Both
/// call the SAME function, which is the property bd-n1aj was about; a row for a
/// path the snapshot does not track at all is left to `validate_rows`'s "no file
/// at this path", so the two never both fire on one row.
pub fn dead_rows(rows: &[Row], entries: &[Entry], scan: &ScanCfg) -> Vec<String> {
    let mut tracked: BTreeSet<&str> = BTreeSet::new();
    let mut identified: BTreeSet<&str> = BTreeSet::new();
    for e in entries {
        tracked.insert(e.path.as_str());
        if scan_reason(e, scan).is_some() {
            identified.insert(e.path.as_str());
        }
    }
    let mut out = Vec::new();
    for r in rows {
        let p = r.path.trim();
        if p.is_empty() || !tracked.contains(p) || identified.contains(p) {
            continue;
        }
        out.push(format!(
            "[[allow]] {p}: this snapshot tracks the path but does not identify it as non-Rust source (no scanned extension, no shebang, not a symlink or submodule) — an exemption for something the gate never demands is dead weight; delete the row"
        ));
    }
    out
}

/// Path-only view, for the callers and tests that only have names. Every entry
/// is treated as an ordinary non-executable file with no shebang, so this leg
/// decides on extension alone — which is exactly what a bare list of paths can
/// support.
pub fn unlisted(candidates: &[String], rows: &[Row], scan: &ScanCfg) -> Vec<String> {
    let entries: Vec<Entry> = candidates.iter().map(|c| Entry::plain(c)).collect();
    unlisted_entries(&entries, rows, scan)
}

// ── the wiring TEXT leg: a subtractive test, never a certificate ───────────

/// What reading `scripts/check.sh` can honestly say about the step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WiringEvidence {
    /// Nothing in check.sh names this gate. The step is not there.
    Absent,
    /// Lines name the gate, but every one matches a compiled-in disqualifier —
    /// each disqualifier establishes that THAT line cannot stop the build.
    Inert(Vec<String>),
    /// At least one line survived every disqualifier. This is the ceiling of the
    /// text leg: a surviving line may still be unreachable, shadowed, or in a
    /// function nothing calls. Use `--prove-wired` for the behavioural leg.
    Unproven,
}

impl WiringEvidence {
    pub fn tag(&self) -> &'static str {
        match self {
            WiringEvidence::Absent => "ABSENT",
            WiringEvidence::Inert(_) => "INERT",
            WiringEvidence::Unproven => "UNPROVEN(text-only)",
        }
    }
}

/// Shell operators that discard the exit status of what precedes them.
///
/// This list SUBTRACTS candidates; it never adds confidence to the ones it does
/// not match. `cargo run … || true` is the worst of the family — the gate runs in
/// full and its verdict is thrown on the floor.
const SWALLOWERS: &[&str] = &[
    "|| true",
    "||true",
    "|| :",
    "||:",
    "|| /bin/true",
    "|| exit 0",
    "; true",
    ";true",
];

/// Everything before an unquoted `#` — the part of the line the shell executes.
fn code_part(line: &str) -> &str {
    let b = line.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' if !in_single => i += 1,
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'#' if !in_single && !in_double && (i == 0 || b[i - 1].is_ascii_whitespace()) => {
                return &line[..i];
            }
            _ => {}
        }
        i += 1;
    }
    line
}

fn shorten(line: &str) -> String {
    let l = line.trim();
    if l.chars().count() <= 72 {
        return l.to_string();
    }
    let head: String = l.chars().take(69).collect();
    format!("{head}...")
}

/// Read `scripts/check.sh` and report the STRONGEST honest statement the text
/// supports. Never returns "wired" — see `WiringEvidence`.
///
/// SWEEP VERDICT (bd-ip10): the `.contains` calls below stay substring tests, and
/// so do the disqualifiers. This leg is asked a question no parse of a shell
/// script answers — "does this line execute" — so it is a HEURISTIC, not a fact
/// about structure. It is kept honest by being purely SUBTRACTIVE: matching adds
/// nothing, and the strongest thing it returns is `Unproven`. Making it stricter
/// would only move lines from `Inert` to `Unproven`, which is the same non-claim.
/// The claim that check.sh stops lives in `--prove-wired`, not here.
pub fn check_sh_wiring(text: &str) -> WiringEvidence {
    let mut inert: Vec<String> = Vec::new();
    let mut live = 0usize;

    for raw in text.lines() {
        let line = raw.trim();
        if !(line.contains("cdcp_gate") && line.contains(NAME)) {
            continue;
        }
        let short = shorten(line);
        if line.starts_with('#') {
            inert.push(format!("commented out: {short}"));
            continue;
        }
        let code = code_part(line).trim();
        if !(code.contains("cdcp_gate") && code.contains(NAME)) {
            inert.push(format!(
                "named only in a trailing comment, which the shell never runs: {short}"
            ));
            continue;
        }
        if code.starts_with("echo ") || code.starts_with("ok ") {
            inert.push(format!("a banner or receipt, not an invocation: {short}"));
            continue;
        }
        if code.starts_with(':') {
            inert.push(format!(
                "`:` is the shell no-op builtin — its argument is never executed: {short}"
            ));
            continue;
        }
        if let Some(op) = SWALLOWERS.iter().find(|s| code.contains(**s)) {
            inert.push(format!(
                "exit status discarded by `{op}` — the gate runs and its verdict is thrown away: {short}"
            ));
            continue;
        }
        live += 1;
    }

    if live > 0 {
        WiringEvidence::Unproven
    } else if inert.is_empty() {
        WiringEvidence::Absent
    } else {
        WiringEvidence::Inert(inert)
    }
}

/// Back-compatible boolean view of the text leg. `true` means only "no compiled-in
/// disqualifier matched" — it is an ABSENCE detector, not a certificate.
pub fn check_sh_wires_guard(text: &str) -> bool {
    check_sh_wiring(text) == WiringEvidence::Unproven
}

// ── reason honesty: a "load-bearing check.sh" claim must be an invoke ──────
//
// bd-allowlist-stale-load-bearing-seq9. The `reason` field is the justification
// that keeps an exemption alive. Six rows kept saying "Load-bearing check.sh
// gate … grandfathered pending the Rust port" after the port landed and
// check.sh stopped invoking them. A hand-edit of those strings without a
// tripwire will rot again.
//
// WHAT THIS DECIDES: whether a reason that CLAIMS check.sh runs the path is
// matched by an invocation derived from scripts/check.sh, and whether a
// reason that CLAIMS the path is off the check.sh path is contradicted by
// an invoke OR a `-f` presence check. Presence tests are not invocations.
//
// WHAT THIS DOES NOT DECIDE: whether the rest of the English is true. An
// authoring-helper reason that is lying about being an authoring helper is
// still prose.

/// Phrases that assert this path is still the live check.sh oracle.
/// Case-insensitive. "Differential oracle … Not a check.sh step" must not match.
pub fn reason_claims_check_sh_invoke(reason: &str) -> bool {
    let r = reason.to_ascii_lowercase();
    r.contains("load-bearing check.sh")
        || r.contains("check.sh invokes")
        || r.contains("check.sh hard-fails if")
        || r.contains("byte-exact oracle")
        || r.contains("oracle required")
}

/// Phrases that assert this path is *not* reachable from check.sh.
/// Checked against the transitive walk — the inverse of the load-bearing lie.
pub fn reason_claims_not_on_check_sh(reason: &str) -> bool {
    let r = reason.to_ascii_lowercase();
    r.contains("not a check.sh step") || r.contains("not on the check.sh path")
}

/// Executors whose next token is treated as an invoked path.
///
/// Longer names first so `python3` is not read as `python`, `nodejs` is not
/// read as `node`, and `bash`/`zsh` are not read as `sh`.
const INVOKE_EXECS: &[&str] = &["python3", "python", "nodejs", "node", "bash", "zsh", "sh"];

fn is_word_start(text: &str, i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let b = text.as_bytes()[i - 1];
    // `.` is not a word start: `voice-slop.sh >/dev/null` must not be
    // parsed as an `sh` invoke of `>/dev/null`.
    !b.is_ascii_alphanumeric() && b != b'_' && b != b'.'
}

fn next_shell_token(after: &str) -> Option<&str> {
    let s = after.trim_start();
    if s.is_empty() {
        return None;
    }
    if let Some(q) = s.chars().next().filter(|c| *c == '"' || *c == '\'') {
        let rest = &s[q.len_utf8()..];
        let end = rest.find(q)?;
        Some(&rest[..end])
    } else {
        let end = s
            .find(|c: char| c.is_ascii_whitespace() || matches!(c, ';' | '|' | '&' | ')'))
            .unwrap_or(s.len());
        Some(&s[..end])
    }
}

fn next_invoked_path(after_exec: &str) -> Option<String> {
    let tok = next_shell_token(after_exec)?.trim();
    if tok.is_empty()
        || tok.starts_with('-')
        || tok.starts_with('$')
        || tok.starts_with('>')
        || tok.starts_with('<')
    {
        return None;
    }
    if !(tok.contains('/')
        || tok.ends_with(".py")
        || tok.ends_with(".sh")
        || tok.ends_with(".mjs")
        || tok.ends_with(".js"))
    {
        return None;
    }
    Some(tok.strip_prefix("./").unwrap_or(tok).to_string())
}

fn extract_invoked_paths(code: &str, out: &mut BTreeSet<String>) {
    // Walk char boundaries: check.sh contains em-dashes, and a byte-index
    // walk panics on `&code[i..]` mid-character.
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) {
            continue;
        }
        let rest = &code[i..];
        let Some(exec) = INVOKE_EXECS.iter().copied().find(|e| rest.starts_with(e)) else {
            continue;
        };
        let after = &rest[exec.len()..];
        let boundary = match after.chars().next() {
            None => true,
            Some(c) => c.is_ascii_whitespace() || c == '"' || c == '\'',
        };
        if boundary {
            if let Some(path) = next_invoked_path(after) {
                out.insert(path);
            }
        }
    }
}

/// Paths `scripts/check.sh` actually invokes, derived from the file, never
/// hand-maintained. Comments and `[ -f path ]` presence tests do not count.
pub fn check_sh_invocation_set(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        extract_invoked_paths(code_part(line), &mut out);
    }
    out
}

pub fn check_sh_invokes_path(text: &str, path: &str) -> bool {
    check_sh_invocation_set(text).contains(path)
}

/// Anti-vacuous errors and lying-reason violations from one snapshot's rows
/// and that snapshot's check.sh text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HonestyFindings {
    pub errors: Vec<String>,
    pub violations: Vec<String>,
}

/// Check that every `reason` claiming a check.sh invoke is matched by one.
///
/// Anti-vacuous: zero `[[allow]]` rows is an ERROR (the scan judged nothing);
/// a missing or empty check.sh is an ERROR (the scan did not open the file
/// it claims to have read). Empty invocation set is an ANSWER, not an error
/// — a fixture check.sh that only runs this gate invokes nothing, and that
/// is how a planted "load-bearing check.sh" reason is shown to be a lie.
///
/// Pass `invoked_override` (the transitive walk) to judge invoke-claims against
/// what check.sh actually runs, not a grep of check.sh itself. Pass
/// `presence_override` so a "not on the check.sh path" reason is RED when
/// the walk only `[ -f path ]`s the file.
pub fn reason_honesty_findings(rows: &[Row], check_sh_text: Option<&str>) -> HonestyFindings {
    reason_honesty_with_set(rows, check_sh_text, None, None)
}

pub fn reason_honesty_with_set(
    rows: &[Row],
    check_sh_text: Option<&str>,
    invoked_override: Option<&BTreeSet<String>>,
    presence_override: Option<&BTreeSet<String>>,
) -> HonestyFindings {
    let mut out = HonestyFindings::default();
    if rows.is_empty() {
        out.errors.push(format!(
            "{REGISTRY_PATH}: zero [[allow]] rows — a reason-honesty scan over nothing is an ERROR, not a pass"
        ));
        return out;
    }
    let Some(text) = check_sh_text else {
        out.errors.push(format!(
            "{CHECK_SH_PATH} was not opened — a reason-honesty scan that does not read check.sh is an ERROR, not a pass"
        ));
        return out;
    };
    if text.is_empty() {
        out.errors.push(format!(
            "{CHECK_SH_PATH} is empty — the invocation set cannot be derived. ERROR, not a pass"
        ));
        return out;
    }
    let owned;
    let invoked = match invoked_override {
        Some(s) => s,
        None => {
            owned = check_sh_invocation_set(text);
            &owned
        }
    };
    for r in rows {
        let path = r.path.trim();
        if path.is_empty() {
            continue;
        }
        if reason_claims_check_sh_invoke(&r.reason) && !invoked.contains(path) {
            out.violations.push(format!(
                "[[allow]] {path}: reason claims this is a live check.sh oracle (\"load-bearing check.sh\" / \"check.sh invokes\" / \"check.sh hard-fails if\" / \"byte-exact oracle\" / \"oracle required\"), but {CHECK_SH_PATH} does not invoke that path"
            ));
        }
        let reached = invoked.contains(path) || presence_override.is_some_and(|p| p.contains(path));
        if reason_claims_not_on_check_sh(&r.reason) && reached {
            out.violations.push(format!(
                "[[allow]] {path}: reason claims this is not a check.sh step / not on the check.sh path, but the derived check.sh walk reaches that path (invoke or -f presence)"
            ));
        }
    }
    out
}

// ── remaining-oracle inventory (bd-retire-oracle-on-behaviour-change-gna0) ─
//
// After EXTRACT-THEN-DELETE the leftover scripts/{verify,validate,smoke}_*.py
// must be listed. An empty scan while the table exists is an ERROR (the scan
// judged nothing). A row whose file is gone is a stale ledger. An unlisted
// remaining oracle is a lie. Fixtures omit `[oracle_inventory]` and this
// function is not called.

const ORACLE_INVENTORY_PREFIXES: &[&str] = &["verify_", "validate_", "smoke_"];
const ORACLE_DISPOSITIONS: &[&str] = &[
    "live_selftest",
    "live_check_sh",
    "cargo_test_differential",
    "honesty_ledger",
];

pub fn is_inventoried_oracle_script(rel: &str) -> bool {
    let Some(name) = rel.strip_prefix("scripts/") else {
        return false;
    };
    if name.contains('/') {
        return false;
    }
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".py")
        && ORACLE_INVENTORY_PREFIXES
            .iter()
            .any(|p| lower.starts_with(p))
}

pub fn discover_oracle_scripts(scripts_dir: &Path) -> Result<BTreeSet<String>, String> {
    let mut out = BTreeSet::new();
    // ABSENT-OK: this helper returns an empty set; inventory_findings
    // errors if the snapshot claims an inventory (zero-scan is RED there).
    if !scripts_dir.is_dir() {
        return Ok(out);
    }
    let rd = std::fs::read_dir(scripts_dir).map_err(|e| format!("read scripts/: {e}"))?;
    for ent in rd {
        let ent = ent.map_err(|e| format!("scripts/ dirent: {e}"))?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        let rel = format!("scripts/{name}");
        // ABSENT-OK: type-filter; a non-file scripts/ entry is not an oracle.
        if is_inventoried_oracle_script(&rel) && ent.path().is_file() {
            out.insert(rel);
        }
    }
    Ok(out)
}

/// Anti-vacuous inventory of remaining verify/validate/smoke oracles.
///
/// `None` means the snapshot's registry does not claim to inventory them
/// (fixtures). `Some` with zero rows, or a scan that found nothing, is ERROR.
pub fn inventory_findings(
    inv: Option<&OracleInventory>,
    discovered: &BTreeSet<String>,
) -> HonestyFindings {
    let mut out = HonestyFindings::default();
    let Some(inv) = inv else {
        return out;
    };
    if inv.files.is_empty() {
        out.errors.push(format!(
            "{REGISTRY_PATH}: [oracle_inventory] has zero files — a scan that inventories nothing is an ERROR, not a pass"
        ));
        return out;
    }
    let mut registered = BTreeSet::new();
    for (i, f) in inv.files.iter().enumerate() {
        let path = f.path.trim();
        let disp = f.disposition.trim();
        let why = f.why.trim();
        if path.is_empty() {
            out.errors.push(format!(
                "{REGISTRY_PATH}: [oracle_inventory] file #{} has empty path",
                i + 1
            ));
            continue;
        }
        if !is_inventoried_oracle_script(path) {
            out.errors.push(format!(
                "{REGISTRY_PATH}: [oracle_inventory] {path} is not a scripts/{{verify,validate,smoke}}_*.py"
            ));
        }
        if disp.is_empty() || !ORACLE_DISPOSITIONS.contains(&disp) {
            out.errors.push(format!(
                "{REGISTRY_PATH}: [oracle_inventory] {path}: disposition {disp:?} is empty or unknown"
            ));
        }
        if why.is_empty() {
            out.errors.push(format!(
                "{REGISTRY_PATH}: [oracle_inventory] {path}: empty `why` — blank is never permissive"
            ));
        }
        if !registered.insert(path.to_string()) {
            out.errors.push(format!(
                "{REGISTRY_PATH}: [oracle_inventory] duplicate path {path}"
            ));
        }
    }
    if discovered.is_empty() {
        out.errors.push(format!(
            "{REGISTRY_PATH}: [oracle_inventory] scan found 0 scripts/{{verify,validate,smoke}}_*.py — a scan that judged nothing is an ERROR, not a pass"
        ));
    }
    for p in discovered.difference(&registered) {
        out.errors.push(format!(
            "{REGISTRY_PATH}: [oracle_inventory] uninventoried remaining oracle {p}"
        ));
    }
    for p in registered.difference(discovered) {
        out.errors.push(format!(
            "{REGISTRY_PATH}: [oracle_inventory] stale row {p} — the file is gone; delete the row in the same change that deleted the file"
        ));
    }
    out
}

// ── transitive invocation walk (bd-check-sh-transitive-invocation-gzvb) ────
//
// A grep of check.sh does not enumerate what check.sh runs. This walk follows
// `sh` / `source` / `.` children and resolves single-assignment `$VAR` targets
// for python3 / node / cargo run. `[ -f path ]` lands in `presence`, never
// `paths`. Comments are not either.

/// What `scripts/check.sh` transitively reaches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvocationWalk {
    /// Repo-relative script paths (`scripts/foo.py`, `tests/voice-slop.sh`, …).
    pub paths: BTreeSet<String>,
    /// `[ -f path ]` / `test -f path` targets. Distinct from invoke.
    pub presence: BTreeSet<String>,
    /// `cargo run -p <pkg> -- <cmd>` records. Not followed into Rust.
    pub cargo: BTreeSet<String>,
    /// Shell files whose bodies were opened. Cycle-breaking, not an inventory.
    pub followed: BTreeSet<String>,
}

impl InvocationWalk {
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty() && self.cargo.is_empty()
    }

    pub fn python(&self) -> Vec<&str> {
        self.paths
            .iter()
            .filter(|p| p.ends_with(".py"))
            .map(String::as_str)
            .collect()
    }
}

/// Empty inventory is an ERROR, not a pass. Check.sh always runs something.
pub fn require_nonempty_inventory(walk: &InvocationWalk) -> Result<(), String> {
    if walk.is_empty() {
        return Err(
            "transitive invocation inventory is empty — a scan that found nothing is an ERROR, not a pass"
                .into(),
        );
    }
    Ok(())
}

/// Floor derived from the tree: at least one walked path (invoke or presence)
/// must exist on disk. A walk whose every target is a ghost is an ERROR.
pub fn require_tree_derived_floor(
    walk: &InvocationWalk,
    exists: impl Fn(&str) -> bool,
) -> Result<usize, String> {
    let n = walk
        .paths
        .iter()
        .chain(walk.presence.iter())
        .filter(|p| exists(p))
        .count();
    if n == 0 {
        return Err(
            "tree-derived invocation/presence floor is 0 — a walk whose targets do not exist on disk is an ERROR, not a pass"
                .into(),
        );
    }
    Ok(n)
}

fn is_followable_shell(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".sh") || lower.ends_with(".bash") || lower.ends_with(".zsh")
}

fn normalize_repo_path(tok: &str) -> Option<String> {
    let t = tok.trim();
    let t = t.strip_prefix("./").unwrap_or(t);
    let t = t
        .strip_prefix("$ROOT/")
        .or_else(|| t.strip_prefix("${ROOT}/"))
        .unwrap_or(t);
    if t.is_empty() || t.starts_with('$') || t.starts_with('/') || t.contains("..") {
        return None;
    }
    if !(t.contains('/')
        || t.ends_with(".py")
        || t.ends_with(".sh")
        || t.ends_with(".mjs")
        || t.ends_with(".js")
        || t.ends_with(".bash")
        || t.ends_with(".zsh"))
    {
        return None;
    }
    Some(t.to_string())
}

fn var_name(tok: &str) -> Option<&str> {
    let t = tok
        .strip_prefix("${")
        .and_then(|s| s.strip_suffix('}'))
        .or_else(|| tok.strip_prefix('$'))?;
    if !t.is_empty() && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Some(t)
    } else {
        None
    }
}

fn resolve_invoke_token(tok: &str, vars: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    if let Some(name) = var_name(tok) {
        return vars
            .get(name)
            .into_iter()
            .flatten()
            .filter_map(|v| normalize_repo_path(v))
            .collect();
    }
    normalize_repo_path(tok).into_iter().collect()
}

fn collect_assignments(text: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut vars: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        let code = code_part(line);
        for (i, _) in code.char_indices() {
            if !is_word_start(code, i) {
                continue;
            }
            let rest = &code[i..];
            let Some(eq) = rest.find('=') else {
                continue;
            };
            let name = &rest[..eq];
            if name.is_empty()
                || !name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                continue;
            }
            let after = &rest[eq + 1..];
            if after.starts_with('$') || after.starts_with('`') {
                continue;
            }
            if let Some(val) = next_shell_token(after) {
                if !val.is_empty() && !val.contains('$') && !val.contains('`') {
                    vars.entry(name.to_string())
                        .or_default()
                        .insert(val.to_string());
                }
            }
        }
    }
    vars
}

fn collect_sourced_tokens(code: &str, out: &mut Vec<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) {
            continue;
        }
        let rest = &code[i..];
        let after = if rest.starts_with("source") {
            let tail = &rest["source".len()..];
            if tail.chars().next().is_some_and(|c| c.is_ascii_whitespace()) {
                Some(tail)
            } else {
                None
            }
        } else if rest.starts_with('.') {
            let tail = &rest[1..];
            if tail.chars().next().is_some_and(|c| c.is_ascii_whitespace()) {
                Some(tail)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(tail) = after {
            if let Some(tok) = next_shell_token(tail) {
                if !tok.is_empty() && !tok.starts_with('-') {
                    out.push(tok.to_string());
                }
            }
        }
    }
}

fn collect_exec_tokens(code: &str, out: &mut Vec<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) {
            continue;
        }
        let rest = &code[i..];
        let Some(exec) = INVOKE_EXECS.iter().copied().find(|e| rest.starts_with(e)) else {
            continue;
        };
        let after = &rest[exec.len()..];
        let boundary = match after.chars().next() {
            None => true,
            Some(c) => c.is_ascii_whitespace() || c == '"' || c == '\'',
        };
        if !boundary {
            continue;
        }
        if let Some(tok) = next_shell_token(after) {
            let tok = tok.trim();
            if !tok.is_empty()
                && !tok.starts_with('-')
                && !tok.starts_with('>')
                && !tok.starts_with('<')
            {
                out.push(tok.to_string());
            }
        }
    }
}

fn flag_value<'a>(s: &'a str, flag: &str) -> Option<&'a str> {
    let mut search = s;
    loop {
        let i = search.find(flag)?;
        if !is_word_start(search, i) {
            search = &search[i + flag.len()..];
            continue;
        }
        let after = &search[i + flag.len()..];
        if !after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_whitespace())
        {
            search = &search[i + flag.len()..];
            continue;
        }
        return next_shell_token(after);
    }
}

fn extract_cargo_runs(code: &str, out: &mut BTreeSet<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) {
            continue;
        }
        let rest = &code[i..];
        if !rest.starts_with("cargo") {
            continue;
        }
        let after = &rest["cargo".len()..];
        if !after
            .chars()
            .next()
            .map(|c| c.is_ascii_whitespace())
            .unwrap_or(true)
        {
            continue;
        }
        if !after.split_whitespace().any(|w| w == "run") {
            continue;
        }
        let pkg = flag_value(after, "-p").unwrap_or("");
        let cmd = after
            .split_once(" -- ")
            .and_then(|(_, c)| c.split_whitespace().next())
            .unwrap_or("");
        let rec = match (pkg.is_empty(), cmd.is_empty()) {
            (false, false) => format!("cargo run -p {pkg} -- {cmd}"),
            (false, true) => format!("cargo run -p {pkg}"),
            (true, false) => format!("cargo run -- {cmd}"),
            (true, true) => "cargo run".into(),
        };
        out.insert(rec);
    }
}

fn preceding_token(code: &str, at: usize) -> &str {
    let before = code[..at].trim_end();
    let bytes = before.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        let c = bytes[i - 1] as char;
        if c.is_ascii_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '`') {
            break;
        }
        i -= 1;
    }
    &before[i..]
}

/// `-f` is a presence test only inside `[` / `[[` / `test`. `rm -f` / `git add -f`
/// are force flags and must not mint a presence row.
fn collect_presence_tokens(code: &str, out: &mut Vec<String>) {
    for (i, _) in code.char_indices() {
        if !is_word_start(code, i) || !code[i..].starts_with("-f") {
            continue;
        }
        let after = &code[i + 2..];
        let boundary = match after.chars().next() {
            None => true,
            Some(c) => c.is_ascii_whitespace() || c == '"' || c == '\'',
        };
        if !boundary {
            continue;
        }
        if !matches!(
            preceding_token(code, i),
            "[" | "[[" | "test" | "!" | "-a" | "-o"
        ) {
            continue;
        }
        if let Some(tok) = next_shell_token(after) {
            let tok = tok.trim();
            if !tok.is_empty() && tok != "]" && tok != "]]" && !tok.starts_with('-') {
                out.push(tok.to_string());
            }
        }
    }
}

/// Derive the transitive invocation set from `entry_text` (`scripts/check.sh`).
///
/// `read` opens a child script. An invoked followable shell that cannot be
/// read is an ERROR — an incomplete walk must not report like a complete one.
/// Empty walk is `Ok` (fixtures); `require_nonempty_inventory` is the live gate.
pub fn walk_invocations(
    entry_text: &str,
    mut read: impl FnMut(&str) -> Option<String>,
) -> Result<InvocationWalk, String> {
    let mut walk = InvocationWalk::default();
    let mut queue: Vec<(String, String)> =
        vec![("scripts/check.sh".into(), entry_text.to_string())];
    let mut seen = BTreeSet::new();

    while let Some((from, text)) = queue.pop() {
        if !seen.insert(from.clone()) {
            continue;
        }
        walk.followed.insert(from.clone());
        let vars = collect_assignments(&text);
        for raw in text.lines() {
            let line = raw.trim();
            if line.starts_with('#') {
                continue;
            }
            let code = code_part(line);
            extract_cargo_runs(code, &mut walk.cargo);
            let mut tokens = Vec::new();
            collect_exec_tokens(code, &mut tokens);
            collect_sourced_tokens(code, &mut tokens);
            for tok in tokens {
                for path in resolve_invoke_token(&tok, &vars) {
                    walk.paths.insert(path.clone());
                    if is_followable_shell(&path) && !seen.contains(&path) {
                        match read(&path) {
                            Some(body) => queue.push((path, body)),
                            None => {
                                return Err(format!(
                                    "invoked {path} from {from} but could not read it — transitive inventory is incomplete. ERROR, not a pass"
                                ));
                            }
                        }
                    }
                }
            }
            let mut pres = Vec::new();
            collect_presence_tokens(code, &mut pres);
            for tok in pres {
                for path in resolve_invoke_token(&tok, &vars) {
                    walk.presence.insert(path);
                }
            }
        }
    }
    Ok(walk)
}

/// Non-comment body still names `python3 <path>` or assigns that path to a
/// variable later used as `python3 "$VAR"`. Used by the live tripwire so a
/// gna0 deletion of the call does not hard-fail this bead.
pub fn script_still_invokes_py(text: &str, path: &str) -> bool {
    let vars = collect_assignments(text);
    let names: BTreeSet<&str> = vars
        .iter()
        .filter(|(_, vs)| {
            vs.iter()
                .any(|v| v == path || normalize_repo_path(v).as_deref() == Some(path))
        })
        .map(|(k, _)| k.as_str())
        .collect();
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        let code = code_part(line);
        let mut tokens = Vec::new();
        collect_exec_tokens(code, &mut tokens);
        for tok in tokens {
            if normalize_repo_path(&tok).as_deref() == Some(path) {
                return true;
            }
            if let Some(name) = var_name(&tok) {
                if names.contains(name) {
                    return true;
                }
            }
        }
    }
    false
}

// ── the wiring BEHAVIOURAL leg ─────────────────────────────────────────────

/// Is the probe's plant genuinely a known-bad for the registry the snapshot
/// carries? `Ok(())` only when it is: unlisted, in scope, and scanned.
///
/// # WHY THIS PARSES INSTEAD OF SCANNING
///
/// Until 2026-08-14 this was `reg_text.contains(PROBE_PLANT)` — a raw substring
/// scan of the registry file. Exemption is not conferred by a byte sequence
/// appearing somewhere in a file; it is conferred by an `[[allow]]` row whose
/// `path` matches, which is exactly what `unlisted` tests. The scan therefore
/// answered a different question than the one that matters, and it answered it
/// wrong in both directions:
///
/// * FALSE POSITIVE, measured: the file's own comment warning nobody to add such
///   a row named the path, and took `scripts/check.sh` RED with ZERO `[[allow]]`
///   rows for it. Documenting the rule tripped the rule.
/// * FALSE NEGATIVE, by construction: TOML escapes (`"scripts/__…"`)
///   spell the same parsed string with different bytes, so a real exemption
///   could be written past a substring scan.
///
/// This function mirrors `unlisted`'s notion of "listed" EXACTLY — trimmed
/// `path`, compared for equality — so the parent's vacuity verdict and the
/// child gate's exemption decision cannot disagree.
///
/// # WHAT AN UNREADABLE REGISTRY MEANS HERE
///
/// A registry that does not parse is an ERROR, never a pass. That is the leg the
/// substring scan covered by accident: bytes are readable when rows are not, so
/// swapping in a parse without this branch would let a malformed registry make
/// the plant silently exempt with the gate saying nothing. Rows this function
/// cannot read are rows it cannot clear.
///
/// # WHAT IT CANNOT DECIDE
///
/// Whether the child `scripts/check.sh` propagates the verdict — that is the
/// probe's job, not this function's. It reads one snapshot's registry text and
/// says only whether planting the known-bad is still meaningful against it.
pub fn probe_plant_vacuity(reg_text: &str) -> Result<(), String> {
    let al = parse_allowlist(reg_text).map_err(|e| {
        format!(
            "the {REGISTRY_PATH} this probe would judge does not parse ({e}). The vacuity check reads PARSED [[allow]] rows, and rows it cannot read are rows it cannot clear — a malformed registry must never make the probe's own known-bad silently exempt. ERROR, not a pass"
        )
    })?;
    if let Some(i) = al.allow.iter().position(|r| r.path.trim() == PROBE_PLANT) {
        return Err(format!(
            "{REGISTRY_PATH} carries an [[allow]] row (#{}) whose `path` is {PROBE_PLANT}; the probe's own known-bad would be exempt and the run would be vacuous. ERROR, not a pass. (A COMMENT naming that path is not a row and does not trip this.)",
            i + 1
        ));
    }
    if !is_in_scope(PROBE_PLANT, &al.scan) || !has_scanned_extension(PROBE_PLANT, &al.scan) {
        return Err(format!(
            "{REGISTRY_PATH}'s [scan] block puts {PROBE_PLANT} outside the scanned surface (roots {:?}, extensions {:?}), so planting it would prove nothing. ERROR, not a pass",
            al.scan.roots, al.scan.extensions
        ));
    }
    Ok(())
}

/// What running `scripts/check.sh` against a planted known-bad showed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeVerdict {
    /// check.sh stopped, non-zero, on the guard's verdict about the plant.
    Propagates,
    /// The guard never reported on the plant at all: the step did not run.
    NeverRan,
    /// The guard reported RED and check.sh carried on or exited 0.
    Swallowed(String),
    /// Neither could be established. Never a pass.
    Unattributable(String),
}

fn describe_exit(code: Option<i32>) -> String {
    match code {
        Some(c) => format!("with exit {c}"),
        None => "without exiting on its own (the probe stopped it — either the transcript had already settled the question, or the timeout expired)".to_string(),
    }
}

/// Decide the probe's verdict from check.sh's own output and exit code.
///
/// Pure so it can be unit-tested against transcripts of all four shapes without
/// running anything.
///
/// SWEEP VERDICT (bd-ip10): the `.contains` calls here stay substring tests. A
/// build transcript has no schema to parse — it is the artifact under test, not a
/// registry — so attribution is a heuristic and is worded as one: the only PASS
/// (`Propagates`) additionally requires check.sh to have exited non-zero, and
/// everything the text leaves open lands in `Unattributable`, which is an ERROR
/// rather than a pass. What it cannot decide: a check.sh that printed the plant
/// and "FAIL" for its own reasons would be read as this gate's verdict.
pub fn classify_probe(log: &str, exit_code: Option<i32>, plant: &str) -> ProbeVerdict {
    let lines: Vec<&str> = log.lines().collect();
    let verdict_at = lines
        .iter()
        .position(|l| l.contains(plant) && l.contains("FAIL"));
    let banner_at = lines
        .iter()
        .position(|l| l.contains("==>") && l.contains(NAME));
    let ok_after = |from: usize| lines.iter().skip(from).any(|l| l.contains("check.sh: ok:"));

    match verdict_at {
        None => {
            if exit_code == Some(0) || banner_at.map(|b| ok_after(b + 1)).unwrap_or(false) {
                ProbeVerdict::NeverRan
            } else {
                ProbeVerdict::Unattributable(format!(
                    "check.sh ended {} without the guard ever reporting on {plant}; the failure cannot be attributed to the substrate step",
                    describe_exit(exit_code)
                ))
            }
        }
        Some(i) => {
            if ok_after(i + 1) {
                ProbeVerdict::Swallowed(
                    "check.sh reported a later step `ok` AFTER the guard had already failed"
                        .to_string(),
                )
            } else if exit_code == Some(0) {
                ProbeVerdict::Swallowed(
                    "check.sh exited 0 while the guard's verdict on the plant was RED".to_string(),
                )
            } else if exit_code.is_some() {
                ProbeVerdict::Propagates
            } else {
                ProbeVerdict::Unattributable(
                    "check.sh was still running at the probe timeout with the guard already RED"
                        .to_string(),
                )
            }
        }
    }
}

/// True once the transcript already settles the question, so the probe can stop
/// a check.sh that is going to run for minutes to tell us nothing new.
pub fn probe_can_stop_early(log: &str, plant: &str) -> bool {
    matches!(
        classify_probe(log, None, plant),
        ProbeVerdict::NeverRan | ProbeVerdict::Swallowed(_)
    )
}

fn probe_timeout() -> Duration {
    let secs = std::env::var(PROBE_TIMEOUT_ENV)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|s| *s > 0)
        .unwrap_or(PROBE_DEFAULT_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

#[allow(unused_variables)]
fn kill_tree(pid: u32, child: &mut std::process::Child) {
    // check.sh spawns cargo/python children; killing only `sh` would orphan them.
    #[cfg(unix)]
    {
        let _ = Command::new("/bin/kill")
            .arg("-KILL")
            .arg(format!("-{pid}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn tail(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let from = lines.len().saturating_sub(n);
    lines[from..].join("\n")
}

/// `--prove-wired`: the behavioural leg of bd-bo6i.
///
/// Materialise the INDEX (not the working tree — the wiring that matters is the
/// one the commit carries), plant an unlisted `.py`, run `scripts/check.sh` for
/// real, and require check.sh itself to stop non-zero on the guard's verdict.
fn prove_wired(ctx: &GateCtx) -> Result<(), GateError> {
    if std::env::var_os(PROBE_ENV).is_some() {
        return Err(GateError::error(format!(
            "{PROBE_ENV} is set: this run is already inside a behavioural probe, and a probe that re-enters itself would not terminate. ERROR, never a pass"
        )));
    }
    let root: &Path = &ctx.root;
    if !vcs::is_repo(root) {
        return Err(GateError::error(format!(
            "{} is not inside a git working tree; the probe judges the tree the index would commit",
            root.display()
        )));
    }

    let base = root.join(PROBE_DIR);
    let tree = base.join("tree");
    let _ = std::fs::remove_dir_all(&tree);
    std::fs::create_dir_all(&tree)
        .map_err(|e| GateError::error(format!("create {}: {e}", tree.display())))?;

    let engine = vcs::materialise_index(root, &tree).map_err(GateError::error)?;
    let check_sh = engine.join(CHECK_SH_PATH);
    if !check_sh.is_file() {
        return Err(GateError::error(format!(
            "the index carries no {CHECK_SH_PATH}; there is nothing to run. ERROR, not a pass"
        )));
    }
    let reg_text = std::fs::read_to_string(engine.join(REGISTRY_PATH)).map_err(|e| {
        GateError::error(format!(
            "the index carries no readable {REGISTRY_PATH} ({e}); the probe would be vacuous. ERROR, not a pass"
        ))
    })?;
    probe_plant_vacuity(&reg_text).map_err(GateError::error)?;

    let plant = engine.join(PROBE_PLANT);
    if let Some(parent) = plant.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| GateError::error(format!("create {}: {e}", parent.display())))?;
    }
    std::fs::write(
        &plant,
        "print(\"cdcp substrate probe: planted, unlisted, and expected to stop check.sh\")\n",
    )
    .map_err(|e| GateError::error(format!("plant {}: {e}", plant.display())))?;
    // The plant must be in the copy's INDEX: the gate scans git's view of a tree.
    vcs::init_and_stage_all(&tree).map_err(GateError::error)?;

    let log_path = base.join("check_sh.log");
    let log = std::fs::File::create(&log_path)
        .map_err(|e| GateError::error(format!("create {}: {e}", log_path.display())))?;
    // One descriptor, duplicated: both streams share an offset, so the transcript
    // keeps the ordering the verdict depends on.
    let log_err = log
        .try_clone()
        .map_err(|e| GateError::error(format!("clone log handle: {e}")))?;

    let mut cmd = Command::new("sh");
    cmd.arg(CHECK_SH_PATH)
        .current_dir(&engine)
        .env(PROBE_ENV, "1")
        .env("CARGO_TARGET_DIR", base.join("target"))
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| GateError::error(format!("spawn sh {CHECK_SH_PATH}: {e}")))?;
    let pid = child.id();
    let deadline = Instant::now() + probe_timeout();

    let status = loop {
        match child.try_wait() {
            Err(e) => {
                kill_tree(pid, &mut child);
                return Err(GateError::error(format!("wait for check.sh: {e}")));
            }
            Ok(Some(s)) => break Some(s),
            Ok(None) => {}
        }
        let so_far = std::fs::read_to_string(&log_path).unwrap_or_default();
        if probe_can_stop_early(&so_far, PROBE_PLANT) || Instant::now() >= deadline {
            kill_tree(pid, &mut child);
            break None;
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    let log_text = std::fs::read_to_string(&log_path).unwrap_or_default();
    let code = status.and_then(|s| s.code());
    let verdict = classify_probe(&log_text, code, PROBE_PLANT);
    let evidence = format!(
        "planted {PROBE_PLANT} in {}; check.sh ended {}; transcript {}\n--- last lines of check.sh ---\n{}",
        engine.display(),
        describe_exit(code),
        log_path.display(),
        tail(&log_text, 12)
    );

    match verdict {
        ProbeVerdict::Propagates => {
            if !ctx.has_flag("--quiet") {
                println!(
                    "{NAME}: ok: wiring PROVEN behaviourally — a planted unlisted .py made {CHECK_SH_PATH} exit {}",
                    code.unwrap_or(-1)
                );
                println!(
                    "{NAME}: this leg establishes one thing: a RED verdict from this gate stops check.sh. It says nothing about the other steps in check.sh, and nothing about files outside the index."
                );
                println!("{NAME}: {evidence}");
            }
            Ok(())
        }
        ProbeVerdict::NeverRan => Err(GateError::violation([format!(
            "{CHECK_SH_PATH} never invoked `cdcp_gate {NAME}`: an unlisted .py was planted and the gate never reported on it. A line that names the gate is not a step that runs it. {evidence}"
        )])),
        ProbeVerdict::Swallowed(why) => Err(GateError::violation([format!(
            "{CHECK_SH_PATH} runs `cdcp_gate {NAME}` and discards its verdict — {why}. A gate whose RED does not stop the build is decoration. {evidence}"
        )])),
        ProbeVerdict::Unattributable(why) => Err(GateError::error(format!(
            "the behavioural wiring leg could not be evaluated — {why}. ERROR, not a pass. {evidence}"
        ))),
    }
}

// ── wiring the pure logic to the tree ──────────────────────────────────────

/// Findings a snapshot's `[wiring]` block and check.sh text produce.
fn wiring_findings(
    al: &Allowlist,
    check_text: &str,
    head_status: Option<&str>,
    force: bool,
) -> (Vec<String>, Vec<String>, WiringEvidence) {
    let ev = check_sh_wiring(check_text);
    let mut hard = Vec::new();
    let mut soft = Vec::new();

    let msg = match &ev {
        WiringEvidence::Absent => Some(format!(
            "{CHECK_SH_PATH} does not invoke `cdcp_gate {NAME}` — BUILT != WIRED. Add: {} ({})",
            al.wiring.invocation.trim(),
            al.wiring.bead.trim()
        )),
        WiringEvidence::Inert(why) => Some(format!(
            "{CHECK_SH_PATH} names `cdcp_gate {NAME}` but every occurrence is inert — BUILT != WIRED: {}",
            why.join(" | ")
        )),
        WiringEvidence::Unproven => None,
    };
    if let Some(m) = msg {
        if force || al.wiring.status.trim() == "wired" {
            hard.push(m);
        } else {
            soft.push(m);
        }
    }
    // The ratchet is never soft: it is the edit that would MAKE the leg soft.
    if let Some(m) = check_wiring_ratchet(head_status, &al.wiring.status) {
        hard.push(m);
    }
    (hard, soft, ev)
}

/// Report a finding once when both snapshots agree, and name the snapshot when
/// they do not — that disagreement is the bug class bd-how names.
fn merge(worktree: Vec<String>, index: Vec<String>) -> Vec<String> {
    let in_index: BTreeSet<&str> = index.iter().map(String::as_str).collect();
    let in_worktree: BTreeSet<&str> = worktree.iter().map(String::as_str).collect();
    let mut out = Vec::new();
    for m in &worktree {
        if in_index.contains(m.as_str()) {
            out.push(m.clone());
        } else {
            out.push(format!("{} — {m}", Snapshot::Worktree.label()));
        }
    }
    for m in &index {
        if !in_worktree.contains(m.as_str()) {
            out.push(format!("{} — {m}", Snapshot::Index.label()));
        }
    }
    out
}

/// What the WORKING TREE says this path is, or `None` when it is not on disk.
///
/// `symlink_metadata`, never `metadata`: a symlink is the thing being classified
/// here, and following it would report the target's shape — including reporting
/// a dangling link as absent, which is how a symlink walks out of the candidate
/// list entirely.
fn worktree_mode(root: &Path, path: &str) -> Option<String> {
    let md = std::fs::symlink_metadata(root.join(path)).ok()?;
    let ft = md.file_type();
    if ft.is_symlink() {
        return Some(SYMLINK_MODE.to_string());
    }
    // ABSENT-OK: mode classifier; a non-directory is classified by the
    // branches below, never dropped.
    if ft.is_dir() {
        // git only ever tracks a directory as a submodule gitlink.
        return Some(GITLINK_MODE.to_string());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if md.permissions().mode() & 0o111 != 0 {
            return Some(EXECUTABLE_MODE.to_string());
        }
    }
    Some("100644".to_string())
}

/// First `n` bytes of a file, or empty when it cannot be read. Only ever called
/// for entries `needs_content_probe` selected, and only to ask whether the first
/// two bytes are `#!`.
fn head_bytes(p: &Path, n: usize) -> Vec<u8> {
    use std::io::Read;
    let mut buf = Vec::new();
    if let Ok(f) = std::fs::File::open(p) {
        let _ = f.take(n as u64).read_to_end(&mut buf);
    }
    buf
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(KNOWN_FLAGS)?;
    if ctx.has_flag("--prove-wired") {
        return prove_wired(ctx);
    }
    let quiet = ctx.has_flag("--quiet");
    let root: &Path = &ctx.root;

    // ── snapshot A: the working tree ────────────────────────────────────────
    let reg_path: PathBuf = root.join(REGISTRY_PATH);
    let wt_text = std::fs::read_to_string(&reg_path)
        .map_err(|e| GateError::error(format!("read {}: {e}", reg_path.display())))?;
    let wt_al = parse_allowlist(&wt_text).map_err(GateError::error)?;
    let mut schema = check_floor(&wt_al.scan);
    schema.extend(check_wiring_status(&wt_al.wiring));
    if !schema.is_empty() {
        return Err(GateError::Error(schema.join(" | ")));
    }

    if !vcs::is_repo(root) {
        return Err(GateError::error(format!(
            "{} is not inside a git working tree; this gate scans git's view of the tree",
            root.display()
        )));
    }

    // The index, WITH MODES. `tracked_files` throws the mode away, and the mode is
    // what tells a symlink from a file (bd-xmn5).
    let ix_raw = vcs::tracked_entries(root).map_err(GateError::error)?;
    let tracked: Vec<String> = ix_raw.iter().map(|e| e.path.clone()).collect();

    // ── anti-vacuous ────────────────────────────────────────────────────────
    // Zero files scanned is an ERROR. A never-scanned tree reports exactly like a
    // clean one; that is how a gate becomes decoration.
    if tracked.is_empty() {
        return Err(GateError::error(
            "scanned 0 files — a vacuous scan is an ERROR, not a pass",
        ));
    }
    let in_scope: Vec<&String> = tracked
        .iter()
        .filter(|p| is_in_scope(p, &wt_al.scan))
        .collect();
    if in_scope.is_empty() {
        // Since bd-xmn5 the scan is the whole tree, so reaching this needs every
        // tracked path to be malformed. It stays as the floor for the case where
        // `WHOLE_TREE_SCOPE` is ever turned back off: a scan whose roots resolve
        // to nothing must never report like a scan that found nothing wrong.
        return Err(GateError::error(format!(
            "0 files in scope out of {} tracked — the scanned surface resolves to nothing; ERROR, not a pass",
            tracked.len()
        )));
    }

    // ── snapshot B: the index — the tree this commit creates ────────────────
    let ix_text = vcs::index_text(root, REGISTRY_PATH)
        .map_err(GateError::error)?
        .ok_or_else(|| {
            GateError::error(format!(
                "{REGISTRY_PATH} is not in the index — this commit removes the policy the gate reads, so nothing in it is exempt. ERROR, not a pass"
            ))
        })?;
    let ix_al = parse_allowlist(&ix_text)
        .map_err(|e| GateError::error(format!("{} — {e}", Snapshot::Index.label())))?;
    let mut ix_schema = check_floor(&ix_al.scan);
    ix_schema.extend(check_wiring_status(&ix_al.wiring));
    if !ix_schema.is_empty() {
        return Err(GateError::Error(format!(
            "{} — {}",
            Snapshot::Index.label(),
            ix_schema.join(" | ")
        )));
    }

    // ── check.sh, from both snapshots ───────────────────────────────────────
    // A check.sh that cannot be read is an ERROR: the wiring leg was not
    // evaluated, and an unevaluated leg must never report like a passed one.
    let wt_check = std::fs::read_to_string(root.join(CHECK_SH_PATH)).map_err(|e| {
        GateError::error(format!(
            "read {CHECK_SH_PATH} from the working tree: {e} — the wiring leg cannot be evaluated. ERROR, not a pass"
        ))
    })?;
    let ix_check = vcs::index_text(root, CHECK_SH_PATH)
        .map_err(GateError::error)?
        .ok_or_else(|| {
            GateError::error(format!(
                "{CHECK_SH_PATH} is not in the index — this commit removes the file the wiring leg reads. ERROR, not a pass"
            ))
        })?;

    // HEAD only supplies the ratchet's floor; an unborn HEAD simply has none.
    let head_status: Option<String> = vcs::head_text(root, REGISTRY_PATH)
        .ok()
        .flatten()
        .and_then(|t| parse_allowlist(&t).ok())
        .map(|a| a.wiring.status.trim().to_string());

    // ── entries: the SUBJECT and the POLICY come from the same snapshot ─────
    //
    // Each snapshot supplies its OWN modes and its OWN bytes. The working tree's
    // modes come from `symlink_metadata` and its shebangs from disk; the index's
    // modes come from `git ls-files -s` and its shebangs from `git show :./…`.
    // Borrowing either across the boundary would reintroduce bd-how in a new
    // place: a file chmod +x on the desk but not staged, or a script whose
    // shebang was added after `git add`, would be judged against content no
    // commit ever had.
    let wt_entries: Vec<Entry> = tracked
        .iter()
        .filter_map(|p| {
            let mode = worktree_mode(root, p)?;
            let shebang = if needs_content_probe(p, &mode, &wt_al.scan) {
                shebang_line(&head_bytes(&root.join(p), 256))
            } else {
                None
            };
            Some(Entry {
                path: p.clone(),
                mode,
                shebang,
            })
        })
        .collect();

    let mut ix_entries: Vec<Entry> = Vec::with_capacity(ix_raw.len());
    for e in &ix_raw {
        let shebang = if needs_content_probe(&e.path, &e.mode, &ix_al.scan) {
            // A blob the index cannot produce is an ERROR, never a silent "not a
            // script": an unread file must not report like a read one.
            let bytes = vcs::index_bytes(root, &e.path).map_err(|err| {
                GateError::error(format!(
                    "{}: the index blob could not be read ({err}); the shebang leg was not evaluated for it. ERROR, not a pass",
                    e.path
                ))
            })?;
            bytes.as_deref().and_then(shebang_line)
        } else {
            None
        };
        ix_entries.push(Entry {
            path: e.path.clone(),
            mode: e.mode.clone(),
            shebang,
        });
    }

    // ── rows: each snapshot answers "does this file exist" for ITSELF ───────
    let today = date::today();
    let index_set: BTreeSet<&str> = tracked.iter().map(String::as_str).collect();
    // symlink_metadata, so a row for a DANGLING symlink is not reported as a row
    // for a file that is gone. The link is tracked; its target is not our business.
    let wt_exists = |p: &str| std::fs::symlink_metadata(root.join(p)).is_ok();
    let ix_exists = |p: &str| index_set.contains(p);

    let mut wt_schema = validate_rows(&wt_al.allow, &wt_al.scan, today, &wt_exists);
    wt_schema.extend(dead_rows(&wt_al.allow, &wt_entries, &wt_al.scan));
    let mut ix_schema_rows = validate_rows(&ix_al.allow, &ix_al.scan, today, &ix_exists);
    ix_schema_rows.extend(dead_rows(&ix_al.allow, &ix_entries, &ix_al.scan));
    let schema_errs = merge(wt_schema, ix_schema_rows);
    if !schema_errs.is_empty() {
        // Schema errors are ERROR-class: the registry could not be honestly read
        // as a set of exemptions, so no file is exempt on its strength.
        return Err(GateError::Error(format!(
            "{} schema error(s) in {REGISTRY_PATH}: {}",
            schema_errs.len(),
            schema_errs.join(" | ")
        )));
    }

    // ── transitive invocation walk (both snapshots, bd-how) ──────────────
    let wt_walk = walk_invocations(&wt_check, |p| std::fs::read_to_string(root.join(p)).ok())
        .map_err(GateError::error)?;
    let ix_walk = walk_invocations(&ix_check, |p| vcs::index_text(root, p).ok().flatten())
        .map_err(|e| GateError::error(format!("{} — {e}", Snapshot::Index.label())))?;

    // ── reason honesty: "load-bearing check.sh" must be an actual invoke ──
    // Each snapshot answers against ITS OWN check.sh AND the transitive
    // walk of that snapshot's children. A planted claim on the desk and a
    // clean index (or the reverse) is the bd-how shape for this field.
    let wt_honesty = reason_honesty_with_set(
        &wt_al.allow,
        Some(&wt_check),
        Some(&wt_walk.paths),
        Some(&wt_walk.presence),
    );
    let ix_honesty = reason_honesty_with_set(
        &ix_al.allow,
        Some(&ix_check),
        Some(&ix_walk.paths),
        Some(&ix_walk.presence),
    );
    let honesty_errs = merge(wt_honesty.errors, ix_honesty.errors);
    if !honesty_errs.is_empty() {
        return Err(GateError::error(format!(
            "{} reason-honesty error(s) in {REGISTRY_PATH}: {}",
            honesty_errs.len(),
            honesty_errs.join(" | ")
        )));
    }
    let honesty_viols = merge(wt_honesty.violations, ix_honesty.violations);

    // ── remaining-oracle inventory (absent table = skip; fixtures) ────────
    let wt_disc = discover_oracle_scripts(&root.join("scripts")).map_err(GateError::error)?;
    let ix_disc: BTreeSet<String> = tracked
        .iter()
        .filter(|p| is_inventoried_oracle_script(p))
        .cloned()
        .collect();
    let wt_inv = inventory_findings(wt_al.oracle_inventory.as_ref(), &wt_disc);
    let ix_inv = inventory_findings(ix_al.oracle_inventory.as_ref(), &ix_disc);
    let inv_errs = merge(wt_inv.errors, ix_inv.errors);
    if !inv_errs.is_empty() {
        return Err(GateError::error(format!(
            "{} oracle-inventory error(s) in {REGISTRY_PATH}: {}",
            inv_errs.len(),
            inv_errs.join(" | ")
        )));
    }

    // bd-yp9x: claiming to scan js/mjs with zero files of that extension is
    // ERROR. Fixtures omit [oracle_inventory]; they list mjs only so the
    // compiled-in floor cannot be dropped in header().
    let live_registry = wt_al.oracle_inventory.is_some() || ix_al.oracle_inventory.is_some();
    if live_registry {
        let js_errs = merge(
            empty_js_family_scan(&wt_al.scan, &wt_entries),
            empty_js_family_scan(&ix_al.scan, &ix_entries),
        );
        if !js_errs.is_empty() {
            return Err(GateError::error(format!(
                "{} js-family scan error(s): {}",
                js_errs.len(),
                js_errs.join(" | ")
            )));
        }
    }

    // ── presence ────────────────────────────────────────────────────────────
    let mut violations = merge(
        unlisted_entries(&wt_entries, &wt_al.allow, &wt_al.scan),
        unlisted_entries(&ix_entries, &ix_al.allow, &ix_al.scan),
    );

    // Staged leg: what THIS commit would add, phrased as such. Judged by the
    // index's allowlist AND the index's entry, because that is what the commit
    // carries.
    let mut staged_count = 0usize;
    if ctx.has_flag("--staged") {
        let staged = vcs::staged_additions(root).map_err(GateError::error)?;
        staged_count = staged.len();
        let by_path: std::collections::BTreeMap<&str, &Entry> =
            ix_entries.iter().map(|e| (e.path.as_str(), e)).collect();
        let staged_entries: Vec<Entry> = staged
            .iter()
            .map(|p| match by_path.get(p.as_str()) {
                Some(e) => (*e).clone(),
                None => Entry::plain(p),
            })
            .collect();
        for s in unlisted_entries(&staged_entries, &ix_al.allow, &ix_al.scan) {
            if !violations.iter().any(|v: &String| v.ends_with(&s)) {
                violations.push(format!("staged for commit — {s}"));
            }
        }
    }

    // ── wiring: BUILT != WIRED, in both snapshots ───────────────────────────
    let force = ctx.has_flag("--verify-wired");
    let (wt_hard, wt_soft, wt_ev) =
        wiring_findings(&wt_al, &wt_check, head_status.as_deref(), force);
    let (ix_hard, ix_soft, ix_ev) =
        wiring_findings(&ix_al, &ix_check, head_status.as_deref(), force);
    violations.extend(merge(wt_hard, ix_hard));
    violations.extend(honesty_viols);
    for m in merge(wt_soft, ix_soft) {
        eprintln!("{NAME}: PENDING WIRING: {m}");
    }

    if !violations.is_empty() {
        return Err(GateError::Violation(violations));
    }

    // Empty inventory is ERROR on the live orchestrator and on any check.sh
    // that names this gate (the cargo-run line must appear in the walk).
    // echo-nothing fixtures fail wiring first and never reach this.
    let live = wt_al.oracle_inventory.is_some() || ix_al.oracle_inventory.is_some();
    let claims_to_run =
        matches!(wt_ev, WiringEvidence::Unproven) || matches!(ix_ev, WiringEvidence::Unproven);
    if live || claims_to_run {
        if let Err(e) = require_nonempty_inventory(&wt_walk) {
            return Err(GateError::error(e));
        }
        if let Err(e) = require_nonempty_inventory(&ix_walk) {
            return Err(GateError::error(format!(
                "{} — {e}",
                Snapshot::Index.label()
            )));
        }
    }
    // Tree-derived floor is live-only: fixtures have no [oracle_inventory]
    // and their check.sh often invokes only this gate (cargo run, no scripts).
    if live {
        if let Err(e) = require_tree_derived_floor(&wt_walk, wt_exists) {
            return Err(GateError::error(e));
        }
        if let Err(e) = require_tree_derived_floor(&ix_walk, ix_exists) {
            return Err(GateError::error(format!(
                "{} — {e}",
                Snapshot::Index.label()
            )));
        }
    }

    if !quiet {
        let listed = ix_al.allow.len();
        let wiring = if wt_ev == ix_ev {
            wt_ev.tag().to_string()
        } else {
            format!("worktree={} index={}", wt_ev.tag(), ix_ev.tag())
        };
        let identified = ix_entries
            .iter()
            .filter(|e| scan_reason(e, &ix_al.scan).is_some())
            .count();
        println!(
            "{NAME}: ok: scanned={} in_scope={} identified_non_rust={identified} staged_adds={} exemptions={} wiring={wiring}",
            tracked.len(),
            in_scope.len(),
            staged_count,
            listed,
        );
        let py = wt_walk.python();
        println!(
            "{NAME}: invocation inventory (transitive): paths={} presence={} cargo_run={} python={}: {}",
            wt_walk.paths.len(),
            wt_walk.presence.len(),
            wt_walk.cargo.len(),
            py.len(),
            if py.is_empty() {
                "(none — remaining py would be listed here; do not read this as a retirement claim)"
                    .to_string()
            } else {
                py.join(", ")
            }
        );
        if wt_text != ix_text || wt_check != ix_check {
            println!(
                "{NAME}: note: the working tree and the index disagree about {REGISTRY_PATH} and/or {CHECK_SH_PATH}. Both snapshots were judged and both are clean."
            );
        }
        println!(
            "{NAME}: floor-raise: a row must exist, be dated, and not claim a check.sh invoke the orchestrator does not make. The rest of the reason is still prose. {listed} exemption(s) outstanding; target is 0."
        );
        println!(
            "{NAME}: the wiring leg above is TEXT ONLY — reading a shell line cannot establish that it executes. Run `cdcp_gate {NAME} --prove-wired` for the behavioural leg."
        );
        println!(
            "{NAME}: scope: this is a path-and-shebang policy over the whole engine tree. It cannot decide that a .rs file does not shell out to python3. The pre-commit hook covers `git commit` ONLY — merge, cherry-pick, rebase, `git am`, `commit-tree` and `--no-verify` create commits without it (measured, git 2.53.0); THIS presence scan (what check.sh runs) is the floor for those paths (bd-efm7)."
        );
    }
    Ok(())
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn scan() -> ScanCfg {
        ScanCfg {
            roots: vec!["scripts".into(), "crates".into()],
            extensions: FLOOR_EXTENSIONS.iter().map(|e| (*e).to_string()).collect(),
            include_engine_root_files: true,
        }
    }

    fn entry(path: &str, mode: &str, shebang: Option<&str>) -> Entry {
        Entry {
            path: path.into(),
            mode: mode.into(),
            shebang: shebang.map(str::to_string),
        }
    }

    fn row(path: &str) -> Row {
        Row {
            path: path.into(),
            reason: "Grandfathered check.sh gate; port tracked by the migration epic".into(),
            migration_bead: "bd-substrate-rust-migration-jhd.7".into(),
            expires: "2099-01-01".into(),
        }
    }

    fn always() -> impl Fn(&str) -> bool {
        |_: &str| true
    }

    const TODAY: Ymd = (2026, 8, 13);

    // ── regression: a filename is not a path traversal ────────────────────
    //
    // Adversarial review 2026-08-14 (codex, read-only) found `is_in_scope`
    // rejecting `path.contains("..")`, which put ORDINARY files out of scope.
    // Confirmed by injection before the fix: `scripts/payload..py` staged and
    // tracked returned exit 0 on BOTH the presence and staged legs. The dots are
    // in the filename; nothing traverses anywhere.

    #[test]
    fn a_double_dot_in_the_filename_is_still_in_scope() {
        for p in [
            "scripts/payload..py",
            "scripts/a..b..c.sh",
            "crates/x..y.py",
            "weird..name.py",
        ] {
            assert!(
                is_in_scope(p, &scan()),
                "{p} is an ordinary file in a mandatory root, not a traversal"
            );
        }
        let v = unlisted(&["scripts/payload..py".to_string()], &[], &scan());
        assert_eq!(v.len(), 1, "and it must actually go RED");
        assert!(v[0].contains("scripts/payload..py"), "must name it: {v:?}");
    }

    #[test]
    fn real_traversal_components_are_still_out_of_scope() {
        for p in [
            "../outside.py",
            "scripts/../../etc/passwd.sh",
            "scripts/./x.py",
            "/abs/path.py",
        ] {
            assert!(
                !is_in_scope(p, &scan()),
                "{p} contains a traversal COMPONENT and must stay out of scope"
            );
        }
    }

    // ── bd-n1aj: the two halves cannot disagree ───────────────────────────
    //
    // MEASURED 2026-08-14, before the fix, in a throwaway repo: `scripts/payload..py`
    // TRACKED with a well-formed [[allow]] row present ->
    //   substrate-guard: ERROR: 1 schema error(s) ... [[allow]] scripts/payload..py:
    //   `path` must be a normalised engine-root-relative path
    //   exit 4, on the presence leg AND on --staged.
    // The same at exit 4 for `scripts/a\b.py`. Without a row both were exit 2,
    // named — so the gate demanded a row it would then refuse to accept.

    /// Paths the two halves are asked about. Ordinary, adversarial, and the two
    /// filenames that were un-allowlistable.
    const AGREEMENT_CORPUS: &[&str] = &[
        // ordinary, in scope, scanned
        "scripts/verify_bank.py",
        "scripts/check.sh",
        "crates/cdcp_gate/gen.py",
        "stray.sh",
        // in scope and scanned, and formerly un-allowlistable
        "scripts/payload..py",
        "scripts/a..b..c.sh",
        "crates/x..y.py",
        "weird..name.py",
        "scripts/a\\b.py",
        "scripts/a\\\\b.sh",
        "back\\slash.py",
        // in scope and scanned (bd-yp9x: mjs is now a floor extension)
        "scripts/smoke.mjs",
        // in scope, not scanned — no row is demanded and none is wanted
        "scripts/README",
        "crates/cdcp_gate/src/main.rs",
        // out of scope
        "docs/a.py",
        "tests/a.sh",
        // traversal and absolute: out of scope, and a row is still malformed
        "../outside.py",
        "scripts/../../etc/passwd.sh",
        "scripts/./x.py",
        "./x.py",
        "..",
        "/abs/path.py",
        "/etc/passwd.sh",
        // degenerate
        "",
    ];

    /// THE bd-n1aj ASSERTION, and the reason this cannot recur quietly.
    ///
    /// The gate must never demand a row it would then reject. Stated over the
    /// corpus: if `unlisted` reports a path as needing an `[[allow]]` row, then a
    /// well-formed row for that exact path must produce no schema finding.
    ///
    /// This is asserted, rather than left to two implementations that look alike,
    /// because looking alike is what failed: the substring test and the component
    /// test read as the same rule right up until one of them was corrected.
    #[test]
    fn no_in_scope_path_can_be_un_allowlistable() {
        let s = scan();
        let mut demanded = 0usize;
        for p in AGREEMENT_CORPUS {
            let demands_row = !unlisted(&[(*p).to_string()], &[], &s).is_empty();
            if !demands_row {
                continue;
            }
            demanded += 1;
            let v = validate_rows(&[row(p)], &s, TODAY, &always());
            assert!(
                v.is_empty(),
                "{p}: the gate demands a row for this path and then rejects the row: {v:?}"
            );
        }
        // Anti-vacuous: a corpus that demands nothing asserts nothing.
        assert!(
            demanded >= 10,
            "only {demanded} corpus paths demanded a row — this test proved nothing"
        );
    }

    /// The other direction, so the widening stays bounded: a path the gate does
    /// NOT scan must not become quietly row-able. Traversal and absolute paths
    /// are still malformed, and out-of-scope rows are still dead weight.
    #[test]
    fn traversal_and_absolute_rows_are_still_rejected() {
        let s = scan();
        for p in [
            "../outside.py",
            "scripts/../../etc/passwd.sh",
            "scripts/./x.py",
            "/abs/path.py",
        ] {
            let v = validate_rows(&[row(p)], &s, TODAY, &always());
            assert!(
                v.iter()
                    .any(|m| m.contains("normalised engine-root-relative path")),
                "{p} must still be rejected as a malformed row: {v:?}"
            );
            assert!(
                !is_in_scope(p, &s),
                "{p} must also stay out of scope — the two answers are one answer"
            );
        }
    }

    /// The leg the fix is FOR, at the row level.
    #[test]
    fn a_dotted_or_backslashed_filename_can_be_allowlisted() {
        let s = scan();
        for p in ["scripts/payload..py", "scripts/a\\b.py", "weird..name.py"] {
            let v = validate_rows(&[row(p)], &s, TODAY, &always());
            assert!(
                v.is_empty(),
                "{p} is an ordinary file in a mandatory root; its row must be accepted: {v:?}"
            );
        }
    }

    /// Anti-vacuous, unchanged by the widening: nothing above turns a blank path
    /// into a row.
    #[test]
    fn an_empty_or_whitespace_path_row_is_still_a_schema_error() {
        for blank in ["", "   ", "\t\n"] {
            let mut r = row("scripts/a.py");
            r.path = blank.into();
            let v = validate_rows(&[r], &scan(), TODAY, &always());
            assert!(
                v.iter().any(|m| m.contains("empty `path`")),
                "{blank:?} must stay a schema ERROR: {v:?}"
            );
        }
        // A row with no `path` field at all lands the same way.
        let text = r#"
schema_version = 1
[scan]
roots = ["scripts", "crates"]
extensions = ["py", "sh"]
include_engine_root_files = true
[wiring]
status = "pending"
check_sh = "scripts/check.sh"
invocation = "cargo run -q -p cdcp_gate -- substrate-guard"
bead = "bd-substrate-rust-migration-jhd.1"
[[allow]]
reason = "Grandfathered load-bearing gate; port tracked by the migration epic"
migration_bead = "bd-x"
expires = "2099-01-01"
"#;
        let al = parse_allowlist(text).expect("parses; the field is missing, not malformed");
        let v = validate_rows(&al.allow, &al.scan, TODAY, &always());
        assert!(v.iter().any(|m| m.contains("empty `path`")), "{v:?}");
    }

    #[test]
    fn normalisation_defect_names_only_traversal_and_absolute() {
        assert!(normalisation_defect("scripts/payload..py").is_none());
        assert!(normalisation_defect("scripts/a\\b.py").is_none());
        assert!(normalisation_defect("a.py").is_none());
        assert!(normalisation_defect("").is_some());
        assert!(normalisation_defect("/abs.py").is_some());
        assert!(normalisation_defect("../a.py").is_some());
        assert!(normalisation_defect("a/./b.py").is_some());
        assert!(normalisation_defect("a/../b.py").is_some());
    }

    // ── the assertion this gate exists for ────────────────────────────────
    #[test]
    fn unlisted_py_is_red() {
        let v = unlisted(&["scripts/foo.py".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("scripts/foo.py"), "must name the file: {v:?}");
    }

    #[test]
    fn unlisted_sh_is_red() {
        let v = unlisted(&["scripts/foo.sh".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("scripts/foo.sh"));
    }

    #[test]
    fn unlisted_at_engine_root_is_red() {
        let v = unlisted(&["stray.sh".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn unlisted_under_crates_is_red() {
        let v = unlisted(&["crates/cdcp_core/gen.py".to_string()], &[], &scan());
        assert_eq!(v.len(), 1);
    }

    // ── known-good: the leg that keeps this gate from being routed around ──
    #[test]
    fn allowlisted_file_passes() {
        let v = unlisted(
            &["scripts/verify_bank.py".to_string()],
            &[row("scripts/verify_bank.py")],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn rust_files_pass_anywhere() {
        let v = unlisted(
            &[
                "crates/cdcp_gate/src/main.rs".to_string(),
                "scripts/whatever.rs".to_string(),
                "build.rs".to_string(),
            ],
            &[],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    // ── bd-xmn5: the scope is now the whole engine tree ───────────────────
    //
    // MEASURED 2026-08-14 against the built binary, in a clone, BEFORE the fix:
    //   docs/payload.py staged   -> exit 0
    //   tests/payload.sh staged  -> exit 0
    // and four real shell files had been living outside the floor since the gate
    // was written, two of them invoked as gates by scripts/check.sh itself.
    // This test is the inverse of the one that used to stand here and assert
    // that tests/ and docs/ PASS. It was not rewritten; the rule changed.
    #[test]
    fn every_directory_is_in_scope_now() {
        for p in [
            "tests/voice-slop.sh",
            "docs/x.py",
            ".flywheel/watchdog.sh",
            "web/assets/deploy.sh",
            "job-research/scrape.py",
        ] {
            let v = unlisted(&[p.to_string()], &[], &scan());
            assert_eq!(v.len(), 1, "{p} must demand a row now: {v:?}");
            assert!(v[0].contains(p), "must name it: {v:?}");
        }
    }

    /// `.js` STAYS OUT (bd-yp9x). The browser product surface is not this
    /// floor; folding it in would make the row count stop meaning
    /// python/shell/node-gate debt. `.mjs` is in the floor — see
    /// `unlisted_mjs_is_red`.
    #[test]
    fn non_script_files_still_pass_anywhere() {
        let v = unlisted(
            &[
                "docs/notes.md".to_string(),
                "web/data/x.json".to_string(),
                "web/assets/js/app.js".to_string(),
                "web/index.html".to_string(),
            ],
            &[],
            &scan(),
        );
        assert!(
            v.is_empty(),
            "the floor is py/sh/mjs, not a dragnet over .js: {v:?}"
        );
    }

    #[test]
    fn unlisted_mjs_is_red() {
        for p in ["scripts/smoke_srs.mjs", "scripts/payload.MJS"] {
            let v = unlisted(&[p.to_string()], &[], &scan());
            assert_eq!(v.len(), 1, "{p} must be RED: {v:?}");
            assert!(v[0].contains(p), "must name it: {v:?}");
        }
    }

    // ── bd-xmn5: extensions are case-insensitive, and are a FAMILY ────────
    //
    // MEASURED before the fix, staged in a clone: scripts/payload.PY -> exit 0,
    // scripts/payload.Py -> exit 0, scripts/payload.bash -> exit 0,
    // scripts/payload.zsh -> exit 0.
    #[test]
    fn an_upper_case_extension_is_the_same_extension() {
        for p in [
            "scripts/payload.PY",
            "scripts/payload.Py",
            "scripts/payload.pY",
            "stray.SH",
            "crates/x/y.Sh",
        ] {
            let v = unlisted(&[p.to_string()], &[], &scan());
            assert_eq!(v.len(), 1, "{p} must be RED: {v:?}");
        }
    }

    #[test]
    fn the_scanned_family_is_shell_python_and_node_gates_not_two_spellings() {
        for p in [
            "scripts/payload.bash",
            "scripts/payload.zsh",
            "scripts/payload.ksh",
            "scripts/payload.pyw",
            "scripts/payload.mjs",
        ] {
            let v = unlisted(&[p.to_string()], &[], &scan());
            assert_eq!(v.len(), 1, "{p} must be RED: {v:?}");
        }
    }

    #[test]
    fn extension_of_reads_the_basename_only() {
        assert_eq!(extension_of("a/b.c/d").as_deref(), None);
        assert_eq!(extension_of("scripts/x.PY").as_deref(), Some("py"));
        assert_eq!(extension_of("scripts/x").as_deref(), None);
        assert_eq!(extension_of("scripts/.hidden").as_deref(), Some("hidden"));
        assert!(has_no_extension("scripts/x"));
        assert!(!has_no_extension("scripts/.hidden"));
    }

    // ── bd-xmn5: the shebang leg ──────────────────────────────────────────
    //
    // MEASURED before the fix: scripts/payload, no extension, first line
    // `#!/usr/bin/env python3`, staged -> exit 0.
    #[test]
    fn an_extensionless_shebang_file_demands_a_row() {
        for (p, line) in [
            ("scripts/payload", "#!/usr/bin/env python3"),
            ("docs/payload", "#!/bin/sh"),
            ("tool", "#!/usr/bin/env bash"),
        ] {
            let e = entry(p, "100644", Some(line));
            let v = unlisted_entries(&[e], &[], &scan());
            assert_eq!(v.len(), 1, "{p} must be RED: {v:?}");
            assert!(v[0].contains("executable text script"), "{v:?}");
        }
    }

    #[test]
    fn an_extensionless_file_with_no_shebang_is_not_a_script() {
        let v = unlisted_entries(
            &[
                entry("LICENSE", "100644", None),
                entry(".flywheel/ALERT", "100644", None),
                entry("Makefile", "100644", None),
            ],
            &[],
            &scan(),
        );
        assert!(v.is_empty(), "a data file is not a script: {v:?}");
    }

    #[test]
    fn shebang_line_reads_bytes_and_survives_a_binary() {
        assert_eq!(
            shebang_line(b"#!/usr/bin/env python3\nprint(1)\n").as_deref(),
            Some("#!/usr/bin/env python3")
        );
        assert_eq!(shebang_line(b"print(1)\n"), None);
        assert_eq!(shebang_line(b""), None);
        // A tracked .wasm is mode 100755 in this very repo. Asking "are the first
        // two bytes #!" must not become an ERROR over a UTF-8 decode.
        assert_eq!(shebang_line(&[0x00, 0x61, 0x73, 0x6d, 0xff, 0xfe]), None);
        // Invalid UTF-8 AFTER a real shebang is still a shebang.
        let mut mixed = b"#!/bin/sh".to_vec();
        mixed.extend_from_slice(&[0xff, 0xfe, b'\n']);
        assert!(shebang_line(&mixed).is_some());
    }

    /// The probe is narrow on purpose: one `git show` per selected file per
    /// snapshot. Widening it to every tracked file is a cost decision, not a
    /// safety one, and the blind spot it leaves is named in the header.
    #[test]
    fn only_extensionless_or_executable_entries_are_content_probed() {
        let s = scan();
        assert!(needs_content_probe("scripts/payload", "100644", &s));
        assert!(needs_content_probe("docs/notes.txt", EXECUTABLE_MODE, &s));
        assert!(!needs_content_probe("docs/notes.txt", "100644", &s));
        assert!(
            !needs_content_probe("scripts/a.py", EXECUTABLE_MODE, &s),
            "the extension already decided it; do not pay for a blob read"
        );
    }

    // ── bd-xmn5: what the gate cannot see through ─────────────────────────
    //
    // MEASURED before the fix: a tracked directory symlink scripts/linkdir ->
    // an outside directory holding hidden.py was staged at exit 0, with
    // hidden.py readable through it on disk.
    #[test]
    fn a_symlink_demands_a_row_whatever_it_is_called() {
        for p in ["scripts/linkdir", "docs/linkfile", "notes.md"] {
            let v = unlisted_entries(&[entry(p, SYMLINK_MODE, None)], &[], &scan());
            assert_eq!(v.len(), 1, "{p} must be RED: {v:?}");
            assert!(v[0].contains("SYMLINK"), "{v:?}");
        }
    }

    #[test]
    fn a_submodule_gitlink_demands_a_row() {
        let v = unlisted_entries(&[entry("vendor/sub", GITLINK_MODE, None)], &[], &scan());
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("SUBMODULE"), "{v:?}");
    }

    /// A row silences the finding — including for the shapes the gate cannot see
    /// through. That is the whole bargain: a human says they looked, and the
    /// sentence rots.
    #[test]
    fn a_row_clears_every_identification_shape() {
        let s = scan();
        for e in [
            entry("scripts/a.PY", "100644", None),
            entry("scripts/tool", "100755", Some("#!/bin/sh")),
            entry("scripts/linkdir", SYMLINK_MODE, None),
            entry("vendor/sub", GITLINK_MODE, None),
        ] {
            let rows = [row(&e.path)];
            assert!(
                unlisted_entries(std::slice::from_ref(&e), &rows, &s).is_empty(),
                "{e:?} must be clearable by a row"
            );
        }
    }

    // ── bd-xmn5: dead_rows is the exact complement of unlisted_entries ─────
    //
    // The bd-n1aj failure mode, generalised: if the two halves can disagree, a
    // path can be simultaneously demanded and refused. They call one function.
    #[test]
    fn dead_rows_and_unlisted_entries_cannot_both_fire() {
        let s = scan();
        let entries = vec![
            entry("scripts/a.py", "100644", None),
            entry("scripts/tool", "100755", Some("#!/bin/sh")),
            entry("scripts/linkdir", SYMLINK_MODE, None),
            entry("vendor/sub", GITLINK_MODE, None),
            entry("README.md", "100644", None),
            entry("LICENSE", "100644", None),
            entry("crates/x/src/main.rs", "100644", None),
        ];
        let mut demanded = 0usize;
        let mut dead = 0usize;
        for e in &entries {
            let rows = [row(&e.path)];
            let demands = !unlisted_entries(std::slice::from_ref(e), &[], &s).is_empty();
            let is_dead = !dead_rows(&rows, std::slice::from_ref(e), &s).is_empty();
            assert!(
                !(demands && is_dead),
                "{}: demanded a row AND called the row dead weight",
                e.path
            );
            assert!(
                demands || is_dead,
                "{}: neither demanded nor rejected — the two are meant to partition",
                e.path
            );
            demanded += usize::from(demands);
            dead += usize::from(is_dead);
        }
        assert!(
            demanded >= 4 && dead >= 3,
            "demanded={demanded} dead={dead}"
        );
    }

    #[test]
    fn a_row_for_an_untracked_path_is_left_to_the_exists_leg() {
        // dead_rows must not double-report what `validate_rows` already says.
        let v = dead_rows(&[row("scripts/gone.py")], &[], &scan());
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn extensionless_and_other_extensions_pass() {
        let v = unlisted(
            &[
                "scripts/README".to_string(),
                "scripts/_module_page_template.html".to_string(),
                "web/assets/js/app.js".to_string(),
            ],
            &[],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    // ── known-bad: schema ────────────────────────────────────────────────
    #[test]
    fn empty_reason_is_a_schema_error_not_permission() {
        let mut r = row("scripts/a.py");
        r.reason = String::new();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(
            v.iter().any(|m| m.contains("empty `reason`")),
            "blank must never be permissive: {v:?}"
        );
    }

    #[test]
    fn whitespace_reason_is_a_schema_error() {
        let mut r = row("scripts/a.py");
        r.reason = "   \t ".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.iter().any(|m| m.contains("empty `reason`")), "{v:?}");
    }

    #[test]
    fn missing_reason_field_lands_as_a_schema_error() {
        let text = r#"
schema_version = 1
[scan]
roots = ["scripts", "crates"]
extensions = ["py", "sh"]
include_engine_root_files = true
[wiring]
status = "pending"
check_sh = "scripts/check.sh"
invocation = "cargo run -q -p cdcp_gate -- substrate-guard"
bead = "bd-substrate-rust-migration-jhd.1"
[[allow]]
path = "scripts/a.py"
migration_bead = "bd-x"
expires = "2099-01-01"
"#;
        let al = parse_allowlist(text).expect("parses; the field is missing, not malformed");
        let v = validate_rows(&al.allow, &al.scan, TODAY, &always());
        assert!(v.iter().any(|m| m.contains("`reason`")), "{v:?}");
    }

    #[test]
    fn token_reason_is_rejected() {
        let mut r = row("scripts/a.py");
        r.reason = "temp".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.iter().any(|m| m.contains("chars")), "{v:?}");
    }

    #[test]
    fn backdated_expires_is_red() {
        let mut r = row("scripts/a.py");
        r.expires = "2026-08-12".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.iter().any(|m| m.contains("EXPIRED")), "{v:?}");
    }

    #[test]
    fn expires_today_still_passes() {
        let mut r = row("scripts/a.py");
        r.expires = "2026-08-13".into();
        let v = validate_rows(&[r], &scan(), TODAY, &always());
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn missing_or_unparseable_expires_is_red() {
        for bad in ["", "never", "soon", "2026-13-01"] {
            let mut r = row("scripts/a.py");
            r.expires = bad.into();
            let v = validate_rows(&[r], &scan(), TODAY, &always());
            assert!(!v.is_empty(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn missing_or_bogus_migration_bead_is_red() {
        for bad in ["", "  ", "TODO", "issue-12"] {
            let mut r = row("scripts/a.py");
            r.migration_bead = bad.into();
            let v = validate_rows(&[r], &scan(), TODAY, &always());
            assert!(
                v.iter().any(|m| m.contains("migration_bead")),
                "{bad:?} -> {v:?}"
            );
        }
    }

    #[test]
    fn duplicate_rows_are_red() {
        let v = validate_rows(
            &[row("scripts/a.py"), row("scripts/a.py")],
            &scan(),
            TODAY,
            &always(),
        );
        assert!(v.iter().any(|m| m.contains("duplicate")), "{v:?}");
    }

    #[test]
    fn stale_row_for_a_deleted_file_is_red() {
        let v = validate_rows(&[row("scripts/gone.py")], &scan(), TODAY, &|_| false);
        assert!(
            v.iter().any(|m| m.contains("no file at this path")),
            "{v:?}"
        );
    }

    /// bd-xmn5 flipped this one: `docs/` is inside the scanned surface now, so a
    /// row for `docs/a.py` is an ORDINARY row and must be accepted. What stays
    /// outside is traversal, and that is asserted in
    /// `traversal_and_absolute_rows_are_still_rejected`.
    #[test]
    fn a_row_for_a_formerly_out_of_scope_directory_is_now_ordinary() {
        let v = validate_rows(&[row("docs/a.py")], &scan(), TODAY, &always());
        assert!(v.is_empty(), "{v:?}");
        let v = validate_rows(&[row("../outside.py")], &scan(), TODAY, &always());
        assert!(
            v.iter().any(|m| m.contains("outside the scanned surface")
                || m.contains("normalised engine-root-relative path")),
            "traversal must still be refused: {v:?}"
        );
    }

    #[test]
    fn good_row_is_clean() {
        let v = validate_rows(&[row("scripts/verify_bank.py")], &scan(), TODAY, &always());
        assert!(v.is_empty(), "{v:?}");
    }

    // ── known-bad: registry weakening ────────────────────────────────────
    #[test]
    fn registry_cannot_narrow_the_extension_floor() {
        let mut s = scan();
        s.extensions = vec!["sh".into()];
        let v = check_floor(&s);
        assert!(v.iter().any(|m| m.contains("\"py\"")), "{v:?}");
        assert!(
            v.iter().any(|m| m.contains("\"mjs\"")),
            "dropping mjs must be a floor-narrow: {v:?}"
        );
    }

    #[test]
    fn registry_cannot_drop_a_scan_root() {
        let mut s = scan();
        s.roots = vec!["scripts".into()];
        assert!(!check_floor(&s).is_empty());
    }

    #[test]
    fn registry_cannot_turn_off_engine_root_scanning() {
        let mut s = scan();
        s.include_engine_root_files = false;
        assert!(!check_floor(&s).is_empty());
    }

    #[test]
    fn registry_may_widen_the_floor() {
        let mut s = scan();
        // js is the deliberate omission (bd-yp9x). The registry may still
        // name it; that is a reviewable widening, not a one-word disable.
        s.extensions.push("js".into());
        s.roots.push("web".into());
        assert!(check_floor(&s).is_empty());
    }

    #[test]
    fn empty_js_or_mjs_scan_is_an_error_when_claimed() {
        let py_sh = ScanCfg {
            roots: vec!["scripts".into(), "crates".into()],
            extensions: vec!["py".into(), "sh".into()],
            include_engine_root_files: true,
        };
        let none = [Entry::plain("scripts/a.py")];
        assert!(
            empty_js_family_scan(&py_sh, &none).is_empty(),
            "py/sh-only claims do not invent a js scan"
        );

        let mut js_only = py_sh.clone();
        js_only.extensions.push("js".into());
        let v = empty_js_family_scan(&js_only, &none);
        assert!(
            v.iter()
                .any(|m| m.contains("claims \"js\"") && m.contains("0 .js")),
            "{v:?}"
        );
        let with_js = [Entry::plain("web/assets/js/app.js")];
        assert!(empty_js_family_scan(&js_only, &with_js).is_empty());

        let floor = scan();
        let v = empty_js_family_scan(&floor, &none);
        assert!(
            v.iter()
                .any(|m| m.contains("claims \"mjs\"") && m.contains("0 .mjs")),
            "{v:?}"
        );
        let with_mjs = [Entry::plain("scripts/smoke_srs.mjs")];
        assert!(empty_js_family_scan(&floor, &with_mjs).is_empty());
    }

    // ── known-bad: wiring status ─────────────────────────────────────────
    fn wiring(status: &str, check_sh: &str) -> Wiring {
        Wiring {
            status: status.into(),
            check_sh: check_sh.into(),
            invocation: "cargo run -q -p cdcp_gate -- substrate-guard || fail".into(),
            bead: "bd-x".into(),
        }
    }

    #[test]
    fn blank_wiring_status_is_a_schema_error() {
        assert!(check_wiring_status(&wiring("", CHECK_SH_PATH))
            .iter()
            .any(|m| m.contains("never permissive")));
    }

    #[test]
    fn unknown_wiring_status_is_a_schema_error() {
        assert!(!check_wiring_status(&wiring("skip", CHECK_SH_PATH)).is_empty());
    }

    // ── bd-bo6i: check_sh must be pinned ─────────────────────────────────
    #[test]
    fn check_sh_pointed_at_another_file_is_a_schema_error() {
        // Confirmed by injection 2026-08-14: pointing [wiring].check_sh at a file
        // holding a suitable string satisfied the wiring leg from a file nothing
        // runs, while the real check.sh had the step deleted. Exit was 0.
        for decoy in ["docs/decoy_wiring.txt", "scripts/check.sh.bak", "README.md"] {
            let v = check_wiring_status(&wiring("wired", decoy));
            assert!(
                v.iter().any(|m| m.contains("pinned")),
                "{decoy}: must be an ERROR: {v:?}"
            );
        }
        assert!(check_wiring_status(&wiring("wired", CHECK_SH_PATH)).is_empty());
        assert!(
            check_wiring_status(&wiring("wired", "  scripts/check.sh  ")).is_empty(),
            "surrounding whitespace is not a repoint"
        );
    }

    // ── bd-bo6i: the ratchet ─────────────────────────────────────────────
    #[test]
    fn wiring_status_is_a_ratchet_not_a_toggle() {
        assert!(check_wiring_ratchet(Some("wired"), "pending").is_some());
        assert!(check_wiring_ratchet(Some("wired"), "").is_some());
        assert!(check_wiring_ratchet(Some("wired"), "wired").is_none());
        // The first wiring commit, and any repo without history, have no floor.
        assert!(check_wiring_ratchet(Some("pending"), "pending").is_none());
        assert!(check_wiring_ratchet(None, "pending").is_none());
    }

    // ── bd-bo6i: the text leg subtracts, it never certifies ──────────────
    #[test]
    fn the_three_confirmed_inert_forms_are_not_wiring() {
        // All three were measured at `wired=yes`, exit 0, on 2026-08-14.
        for form in [
            ": \"cargo run -q -p cdcp_gate -- substrate-guard\"",
            "true # cargo run -q -p cdcp_gate -- substrate-guard",
            "cargo run -q -p cdcp_gate -- substrate-guard || true",
        ] {
            let ev = check_sh_wiring(&format!("#!/bin/sh\nset -eu\n{form}\n"));
            assert!(
                matches!(ev, WiringEvidence::Inert(_)),
                "{form:?} must be INERT, got {ev:?}"
            );
            assert!(!check_sh_wires_guard(&format!("{form}\n")), "{form:?}");
        }
    }

    #[test]
    fn other_status_discarding_forms_are_inert_too() {
        for form in [
            "cargo run -q -p cdcp_gate -- substrate-guard ||:",
            "cargo run -q -p cdcp_gate -- substrate-guard || :",
            "cargo run -q -p cdcp_gate -- substrate-guard || exit 0",
            "cargo run -q -p cdcp_gate -- substrate-guard ; true",
        ] {
            assert!(
                matches!(check_sh_wiring(form), WiringEvidence::Inert(_)),
                "{form:?}"
            );
        }
    }

    #[test]
    fn absent_and_unproven_are_distinct_answers() {
        assert_eq!(
            check_sh_wiring("echo hi\ncargo test --workspace\n"),
            WiringEvidence::Absent
        );
        assert_eq!(
            check_sh_wiring(
                "cargo run -q -p cdcp_gate -- substrate-guard || fail \"substrate guard\"\n"
            ),
            WiringEvidence::Unproven,
            "the real step is the strongest the TEXT can say, and that is still UNPROVEN"
        );
        assert_eq!(
            check_sh_wiring("run_cdcp_gate substrate-guard || fail \"substrate guard\"\n"),
            WiringEvidence::Unproven,
            "the built-binary wrapper is still a live invoke (bd-checksh-cargo-run-attribution-tebe)"
        );
    }

    #[test]
    fn banners_and_comments_are_not_invocations() {
        assert!(matches!(
            check_sh_wiring("# cargo run -p cdcp_gate -- substrate-guard\n"),
            WiringEvidence::Inert(_)
        ));
        assert!(matches!(
            check_sh_wiring("echo \"==> cdcp_gate substrate-guard (S0)\"\n"),
            WiringEvidence::Inert(_)
        ));
        assert!(matches!(
            check_sh_wiring("ok \"cdcp_gate substrate-guard floor\"\n"),
            WiringEvidence::Inert(_)
        ));
        assert_eq!(
            check_sh_wiring(
                "echo \"==> cdcp_gate substrate-guard\"\ncargo run -q -p cdcp_gate -- substrate-guard || fail \"x\"\nok \"substrate floor\"\n"
            ),
            WiringEvidence::Unproven,
            "the real three-line step must survive the disqualifiers"
        );
    }

    #[test]
    fn a_hash_inside_quotes_is_not_a_comment() {
        // The disqualifiers must not manufacture RED out of an ordinary message.
        assert_eq!(
            code_part("cargo run -p cdcp_gate -- substrate-guard || fail \"bad # here\""),
            "cargo run -p cdcp_gate -- substrate-guard || fail \"bad # here\""
        );
        assert_eq!(code_part("true # cargo run"), "true ");
        assert_eq!(code_part("echo 'a#b' # tail"), "echo 'a#b' ");
    }

    // ── bd-bo6i: the behavioural verdict ─────────────────────────────────
    #[test]
    fn probe_certifies_only_a_transcript_that_stops_on_the_plant() {
        let plant = PROBE_PLANT;
        let red = format!("substrate-guard: FAIL: {plant}: non-Rust file with no row");
        let banner = "==> cdcp_gate substrate-guard (S0 substrate floor)";

        // wired: the gate went RED and check.sh stopped there.
        let good = format!("{banner}\n{red}\ncheck.sh: FAIL: substrate guard\n");
        assert_eq!(
            classify_probe(&good, Some(2), plant),
            ProbeVerdict::Propagates
        );

        // `|| true`: the gate ran, and check.sh sailed on.
        let swallowed = format!("{banner}\n{red}\ncheck.sh: ok: S0 substrate floor\n");
        assert!(matches!(
            classify_probe(&swallowed, Some(0), plant),
            ProbeVerdict::Swallowed(_)
        ));
        assert!(
            matches!(
                classify_probe(&swallowed, None, plant),
                ProbeVerdict::Swallowed(_)
            ),
            "killed early on the same evidence is the same verdict"
        );

        // `:` / `true #`: the gate never ran at all.
        let never = format!("{banner}\ncheck.sh: ok: S0 substrate floor\n");
        assert_eq!(
            classify_probe(&never, Some(0), plant),
            ProbeVerdict::NeverRan
        );
        assert_eq!(classify_probe(&never, None, plant), ProbeVerdict::NeverRan);

        // A failure that is not this gate's must never be read as this gate's.
        let elsewhere = format!("{banner}\ncheck.sh: FAIL: missing docs/ORACLE-GAUNTLET.md\n");
        assert!(matches!(
            classify_probe(&elsewhere, Some(2), plant),
            ProbeVerdict::Unattributable(_)
        ));
        assert!(
            matches!(
                classify_probe("", None, plant),
                ProbeVerdict::Unattributable(_)
            ),
            "a timeout with no evidence is an ERROR, never a pass"
        );
    }

    #[test]
    fn probe_stops_early_only_once_the_answer_is_settled() {
        let plant = PROBE_PLANT;
        let red = format!("substrate-guard: FAIL: {plant}: no row");
        assert!(!probe_can_stop_early("", plant));
        assert!(!probe_can_stop_early(&red, plant));
        assert!(probe_can_stop_early(
            &format!("{red}\ncheck.sh: ok: next step\n"),
            plant
        ));
    }

    // ── snapshot labelling ───────────────────────────────────────────────
    #[test]
    fn merge_names_a_snapshot_only_when_they_disagree() {
        let both = merge(vec!["x".into()], vec!["x".into()]);
        assert_eq!(both, vec!["x".to_string()], "agreement is reported once");

        let index_only = merge(vec![], vec!["y".into()]);
        assert_eq!(index_only.len(), 1);
        assert!(
            index_only[0].contains("this commit creates") && index_only[0].ends_with("y"),
            "{index_only:?}"
        );

        let worktree_only = merge(vec!["z".into()], vec![]);
        assert!(
            worktree_only[0].contains("working tree only"),
            "{worktree_only:?}"
        );
    }

    #[test]
    fn scope_predicate() {
        let s = scan();
        assert!(is_in_scope("scripts/a.py", &s));
        assert!(is_in_scope("crates/x/y/a.sh", &s));
        assert!(is_in_scope("a.sh", &s));
        // bd-xmn5: docs/ used to be outside, and docs/payload.py was measured at
        // exit 0 because of it. The whole tree is in scope now.
        assert!(is_in_scope("docs/a.py", &s));
        assert!(is_in_scope(".flywheel/watchdog.sh", &s));
        // What stays out is traversal and absolutes, and only that.
        assert!(!is_in_scope("/etc/a.sh", &s));
        assert!(!is_in_scope("../a.sh", &s));
        assert!(!is_in_scope("scripts/./a.sh", &s));
    }

    #[test]
    fn bead_id_shape() {
        assert!(looks_like_bead_id("bd-substrate-rust-migration-jhd.7"));
        assert!(looks_like_bead_id("cp-123"));
        assert!(!looks_like_bead_id("bd-"));
        assert!(!looks_like_bead_id("xx-1"));
        assert!(!looks_like_bead_id(""));
    }

    // ── bd-ip10: the vacuity check reads ROWS, not bytes ──────────────────
    //
    // MEASURED 2026-08-14, before the fix: `reg_text.contains(PROBE_PLANT)` took
    // scripts/check.sh RED with ZERO [[allow]] rows for that path — the only
    // occurrence was the comment warning nobody to add one. Reproduction:
    //   ./target/debug/cdcp_gate --root . substrate-guard --prove-wired -> exit 4
    // with the clear comment in the judged snapshot.

    /// A registry body with `extra` spliced in after `[scan]`.
    fn reg(extra: &str) -> String {
        format!(
            "schema_version = 1\n\n\
             [scan]\n\
             roots = [\"scripts\", \"crates\"]\n\
             extensions = [\"py\", \"sh\"]\n\
             include_engine_root_files = true\n\n\
             {extra}\n\
             [wiring]\n\
             status = \"wired\"\n\
             check_sh = \"scripts/check.sh\"\n\
             invocation = \"cargo run -q -p cdcp_gate -- substrate-guard\"\n\
             bead = \"bd-substrate-rust-migration-jhd.1\"\n"
        )
    }

    fn plant_row(path: &str) -> String {
        format!(
            "[[allow]]\npath = {path:?}\nreason = \"Grandfathered load-bearing gate; port tracked by the migration epic\"\nmigration_bead = \"bd-substrate-rust-migration-jhd.7\"\nexpires = \"2099-12-31\"\n"
        )
    }

    /// THE bd-ip10 ASSERTION. A comment naming the plant is documentation, not
    /// an exemption, and the probe must run. Deleting this line is exactly what
    /// the CHARTER meta-test asks to be tried; with the check mutated back to a
    /// byte scan, this is the assertion that goes red.
    #[test]
    fn vacuity_a_comment_naming_the_plant_is_not_a_row() {
        let text = reg(&format!(
            "# NEVER add a row for {PROBE_PLANT} — that is the plant\n\
             # --prove-wired uses, and listing it makes the probe vacuous.\n"
        ));
        assert!(
            text.contains(PROBE_PLANT),
            "the fixture must actually name the path, or it tests nothing"
        );
        assert_eq!(
            probe_plant_vacuity(&text),
            Ok(()),
            "a comment is not an [[allow]] row; the gate must be describable in its own registry"
        );
    }

    /// Known-bad, unchanged from the substring era: a real row is a real exemption.
    #[test]
    fn vacuity_an_allow_row_for_the_plant_is_an_error() {
        let e = probe_plant_vacuity(&reg(&plant_row(PROBE_PLANT))).unwrap_err();
        assert!(e.contains(PROBE_PLANT), "{e}");
        assert!(e.contains("vacuous"), "{e}");
    }

    /// Known-bad the parse is STRONGER on: TOML escapes spell the same path in
    /// different bytes, so a substring scan could be written straight past.
    #[test]
    fn vacuity_a_row_whose_path_is_escaped_is_still_caught() {
        let escaped = "scripts/\\u005F\\u005Fcdcp_probe_unlisted\\u005F\\u005F.py";
        let body = reg(&format!(
            "[[allow]]\npath = \"{escaped}\"\nreason = \"Grandfathered load-bearing gate; port tracked by the migration epic\"\nmigration_bead = \"bd-x\"\nexpires = \"2099-12-31\"\n"
        ));
        assert!(
            !body.contains(PROBE_PLANT),
            "the fixture must NOT contain the plant as bytes, or it does not test the escape"
        );
        let e = probe_plant_vacuity(&body).unwrap_err();
        assert!(e.contains("vacuous"), "{e}");
    }

    /// Known-bad, NEW, and the reason this change is not a narrowing: bytes stay
    /// readable when rows do not. Swapping a byte scan for a parse without this
    /// branch would let a malformed registry exempt the plant in silence.
    #[test]
    fn vacuity_an_unparseable_registry_is_an_error_not_a_silent_pass() {
        for broken in [
            "schema_version = 1\n[scan\nroots = [",
            "this is not toml at all {{{",
            "",
            "schema_version = 2\n[scan]\nroots = [\"scripts\", \"crates\"]\nextensions = [\"py\", \"sh\"]\ninclude_engine_root_files = true\n[wiring]\nstatus = \"wired\"\ncheck_sh = \"scripts/check.sh\"\ninvocation = \"x\"\nbead = \"b\"\n",
        ] {
            let e = probe_plant_vacuity(broken)
                .expect_err("an unreadable registry must never clear the plant");
            assert!(
                e.contains("ERROR, not a pass"),
                "{broken:?} -> {e}: must say so in the words the rest of this gate uses"
            );
        }
    }

    /// The other way to make the plant harmless: leave the row out and put the
    /// path out of scope instead.
    /// bd-xmn5 note: the ROOTS half of this test is gone, because roots no longer
    /// bound the scan — re-rooting cannot push the plant out any more, so a test
    /// asserting that it does would have been asserting a rule that had stopped
    /// existing. Narrowing the EXTENSIONS still can, and still must be an ERROR.
    #[test]
    fn vacuity_a_scan_that_excludes_the_plant_is_an_error() {
        let narrowed = reg("").replace("\"py\",", "");
        assert!(
            !narrowed.contains("\"py\""),
            "the fixture must actually drop py, or it tests nothing"
        );
        let e = probe_plant_vacuity(&narrowed).unwrap_err();
        assert!(e.contains("outside the scanned surface"), "{e}");
    }

    #[test]
    fn vacuity_an_ordinary_registry_clears_the_plant() {
        assert_eq!(
            probe_plant_vacuity(&reg(&plant_row("scripts/other.py"))),
            Ok(())
        );
    }

    // ── bd-allowlist-stale-load-bearing-seq9: reason honesty ──────────────

    #[test]
    fn reason_claims_only_the_named_phrases() {
        assert!(reason_claims_check_sh_invoke(
            "Load-bearing check.sh gate, grandfathered pending the Rust port"
        ));
        assert!(reason_claims_check_sh_invoke(
            "Retained; check.sh hard-fails if it is absent"
        ));
        assert!(reason_claims_check_sh_invoke(
            "check.sh invokes this as a smoke"
        ));
        // "load-bearing" alone, or "check.sh" as the orchestrator's own row,
        // is not a claim that THIS path is a check.sh step.
        assert!(!reason_claims_check_sh_invoke(
            "Grandfathered load-bearing gate; port tracked by the migration epic"
        ));
        assert!(!reason_claims_check_sh_invoke(
            "THIS ONE LEGITIMATELY STAYS SHELL: check.sh is the thin orchestrator"
        ));
        assert!(!reason_claims_check_sh_invoke(
            "Differential oracle for cdcp_gate verify-orphans. Not a check.sh step."
        ));
        assert!(reason_claims_check_sh_invoke(
            "Retained as the byte-exact oracle for verify-orphans"
        ));
        assert!(reason_claims_check_sh_invoke(
            "check.sh hard-fails: orphan gate / oracle required"
        ));
        // "byte for byte" is the cargo-test differential, not a live-oracle claim.
        assert!(!reason_claims_check_sh_invoke(
            "compare both implementations byte for byte. Not a check.sh step."
        ));
        assert!(!reason_claims_check_sh_invoke(
            "Grandfathered pending the byte-exact Rust port."
        ));
        assert!(reason_claims_not_on_check_sh(
            "Manual authoring step. Not on the check.sh path; ports after the gates."
        ));
        assert!(reason_claims_not_on_check_sh(
            "Retained so tests can compare both implementations. Not a check.sh step."
        ));
        assert!(!reason_claims_not_on_check_sh(
            "Invoked transitively by selftest_orphan.sh"
        ));
    }

    #[test]
    fn invocation_set_counts_executors_and_ignores_presence_and_comments() {
        let sh = r#"
# python3 scripts/commented_out.py
[ -f scripts/validate_grounding.py ] || fail "missing scripts/validate_grounding.py"
echo "==> smoke_weak_links.py";  python3 scripts/smoke_weak_links.py || fail "x"
python3 "scripts/export_anki.py" --format apkg
run_selftest "orphan" sh scripts/selftest_orphan.sh
sh tests/voice-slop.sh >/dev/null
cp scripts/export_anki.py /tmp/copy.py
python3 "$_anki_plant/scripts/export_anki.py"
"#;
        let set = check_sh_invocation_set(sh);
        assert!(set.contains("scripts/smoke_weak_links.py"), "{set:?}");
        assert!(set.contains("scripts/export_anki.py"), "{set:?}");
        assert!(set.contains("scripts/selftest_orphan.sh"), "{set:?}");
        assert!(set.contains("tests/voice-slop.sh"), "{set:?}");
        assert!(
            !set.contains(">/dev/null"),
            "a redirect is not an invoke: {set:?}"
        );
        assert!(
            !set.contains("scripts/commented_out.py"),
            "comments are not invokes: {set:?}"
        );
        assert!(
            !set.contains("scripts/validate_grounding.py"),
            "presence tests are not invokes: {set:?}"
        );
    }

    #[test]
    fn a_load_bearing_claim_for_an_uninvoked_path_is_red() {
        let mut r = row("scripts/verify_orphans.py");
        r.reason = "Load-bearing check.sh gate, grandfathered pending the Rust port".into();
        let sh = "#!/bin/sh\npython3 scripts/export_anki.py --format apkg\n";
        let h = reason_honesty_findings(&[r, row("scripts/export_anki.py")], Some(sh));
        assert!(h.errors.is_empty(), "{h:?}");
        assert_eq!(h.violations.len(), 1, "{h:?}");
        assert!(
            h.violations[0].contains("scripts/verify_orphans.py"),
            "must name the file: {h:?}"
        );
        assert!(
            h.violations[0].contains("does not invoke"),
            "must name the lie: {h:?}"
        );
    }

    #[test]
    fn a_load_bearing_claim_for_an_invoked_path_is_quiet() {
        let mut r = row("scripts/export_anki.py");
        r.reason = "Load-bearing check.sh step: V11 Anki export".into();
        let sh = "#!/bin/sh\npython3 scripts/export_anki.py --format apkg\n";
        let h = reason_honesty_findings(&[r], Some(sh));
        assert!(h.errors.is_empty() && h.violations.is_empty(), "{h:?}");
    }

    #[test]
    fn zero_allowlist_rows_is_an_error_not_a_pass() {
        let h = reason_honesty_findings(&[], Some("#!/bin/sh\npython3 scripts/a.py\n"));
        assert!(
            h.errors.iter().any(|e| e.contains("zero [[allow]] rows")),
            "{h:?}"
        );
    }

    #[test]
    fn unread_or_empty_check_sh_is_an_error_not_a_pass() {
        let rows = [row("scripts/a.py")];
        let missing = reason_honesty_findings(&rows, None);
        assert!(
            missing.errors.iter().any(|e| e.contains("was not opened")),
            "{missing:?}"
        );
        let empty = reason_honesty_findings(&rows, Some(""));
        assert!(
            empty.errors.iter().any(|e| e.contains("is empty")),
            "{empty:?}"
        );
    }

    /// Anti-vacuous for the shallow parser: check.sh itself still names
    /// the remaining first-level scripts. Nested oracles belong to the
    /// transitive walk, not this grep.
    #[test]
    fn live_check_sh_names_the_first_level_scripts() {
        let root = crate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root");
        let text = std::fs::read_to_string(root.join(CHECK_SH_PATH)).expect("check.sh");
        assert!(
            !text.is_empty(),
            "a scan that did not open check.sh is an ERROR"
        );
        let set = check_sh_invocation_set(&text);
        assert!(
            !set.is_empty(),
            "check.sh always calls something; a parser that found none did not parse"
        );
        for must in ["tests/voice-slop.sh", "tests/publishability-bar.sh"] {
            assert!(set.contains(must), "missing invoke {must}: {set:?}");
        }
        for retired in [
            "scripts/export_anki.py",
            "scripts/smoke_learn_v2.py",
            "scripts/smoke_weak_links.py",
            "scripts/verify_content_lock.py",
            "scripts/verify_knowledge_paths.py",
            "scripts/verify_paraphrase_pairs.py",
        ] {
            assert!(
                !set.contains(retired),
                "{retired} is gone from check.sh itself: {set:?}"
            );
        }
        // jhd.21: last first-level python3 scripts/*.py is retired. Putting one back is RED.
        assert!(
            !set.iter().any(|p| p.ends_with(".py")),
            "first-level python3 scripts/*.py must stay GONE: {set:?}"
        );
    }

    #[test]
    fn live_allowlist_reasons_match_live_check_sh() {
        let root = crate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root");
        let al_text = std::fs::read_to_string(root.join(REGISTRY_PATH)).expect("allowlist");
        let check = std::fs::read_to_string(root.join(CHECK_SH_PATH)).expect("check.sh");
        let al = parse_allowlist(&al_text).expect("parses");
        assert!(
            !al.allow.is_empty(),
            "zero allowlist rows is an ERROR, not a pass"
        );
        let walk = walk_invocations(&check, |p| std::fs::read_to_string(root.join(p)).ok())
            .expect("live walk");
        require_nonempty_inventory(&walk).expect("empty inventory is ERROR");
        let h = reason_honesty_with_set(
            &al.allow,
            Some(&check),
            Some(&walk.paths),
            Some(&walk.presence),
        );
        assert!(
            h.errors.is_empty() && h.violations.is_empty(),
            "live allowlist reasons must be honest against the transitive walk: {h:?}"
        );
        require_tree_derived_floor(&walk, |p| root.join(p).is_file())
            .expect("tree-derived floor is 0");
    }

    #[test]
    fn a_byte_exact_oracle_claim_for_an_uninvoked_path_is_red() {
        let mut r = row("scripts/verify_orphans.py");
        r.reason = "Retained as the byte-exact oracle after the port".into();
        let sh = "#!/bin/sh\npython3 scripts/verify_paraphrase_pairs.py\n";
        let h = reason_honesty_findings(&[r], Some(sh));
        assert!(h.errors.is_empty(), "{h:?}");
        assert_eq!(h.violations.len(), 1, "{h:?}");
        assert!(
            h.violations[0].contains("scripts/verify_orphans.py"),
            "{h:?}"
        );
        assert!(h.violations[0].contains("does not invoke"), "{h:?}");
    }

    #[test]
    fn an_oracle_required_claim_for_an_uninvoked_path_is_red() {
        let mut r = row("scripts/validate_grounding.py");
        r.reason = "differential oracle required for validate-grounding".into();
        let sh = "#!/bin/sh\n[ -f scripts/validate_grounding.py ] || exit 2\n";
        let h = reason_honesty_findings(&[r], Some(sh));
        assert!(h.errors.is_empty(), "{h:?}");
        assert_eq!(h.violations.len(), 1, "{h:?}");
    }

    fn inv_row(path: &str, disp: &str, why: &str) -> OracleInventoryFile {
        OracleInventoryFile {
            path: path.into(),
            disposition: disp.into(),
            why: why.into(),
        }
    }

    #[test]
    fn inventory_absent_is_quiet() {
        let mut disc = BTreeSet::new();
        disc.insert("scripts/verify_orphans.py".into());
        let h = inventory_findings(None, &disc);
        assert!(h.errors.is_empty() && h.violations.is_empty(), "{h:?}");
    }

    #[test]
    fn inventory_of_zero_files_is_an_error() {
        let inv = OracleInventory { files: vec![] };
        let disc = BTreeSet::new();
        let h = inventory_findings(Some(&inv), &disc);
        assert!(h.errors.iter().any(|e| e.contains("zero files")), "{h:?}");
    }

    #[test]
    fn empty_oracle_scan_is_an_error_not_a_pass() {
        let inv = OracleInventory {
            files: vec![inv_row(
                "scripts/verify_orphans.py",
                "live_selftest",
                "selftest still runs it",
            )],
        };
        let h = inventory_findings(Some(&inv), &BTreeSet::new());
        assert!(h.errors.iter().any(|e| e.contains("scan found 0")), "{h:?}");
        assert!(h.errors.iter().any(|e| e.contains("stale row")), "{h:?}");
    }

    #[test]
    fn uninventoried_remaining_oracle_is_an_error() {
        let inv = OracleInventory {
            files: vec![inv_row(
                "scripts/verify_orphans.py",
                "live_selftest",
                "selftest still runs it",
            )],
        };
        let mut disc = BTreeSet::new();
        disc.insert("scripts/verify_orphans.py".into());
        disc.insert("scripts/verify_bank.py".into());
        let h = inventory_findings(Some(&inv), &disc);
        assert!(
            h.errors
                .iter()
                .any(|e| e.contains("uninventoried remaining oracle scripts/verify_bank.py")),
            "{h:?}"
        );
    }

    #[test]
    fn matching_inventory_is_quiet() {
        let inv = OracleInventory {
            files: vec![inv_row(
                "scripts/verify_orphans.py",
                "live_selftest",
                "selftest still runs it",
            )],
        };
        let mut disc = BTreeSet::new();
        disc.insert("scripts/verify_orphans.py".into());
        let h = inventory_findings(Some(&inv), &disc);
        assert!(h.errors.is_empty() && h.violations.is_empty(), "{h:?}");
    }

    #[test]
    fn empty_why_or_unknown_disposition_is_an_error() {
        let inv = OracleInventory {
            files: vec![inv_row("scripts/verify_orphans.py", "vibes", "")],
        };
        let mut disc = BTreeSet::new();
        disc.insert("scripts/verify_orphans.py".into());
        let h = inventory_findings(Some(&inv), &disc);
        assert!(h.errors.iter().any(|e| e.contains("disposition")), "{h:?}");
        assert!(h.errors.iter().any(|e| e.contains("empty `why`")), "{h:?}");
    }

    #[test]
    fn live_oracle_inventory_matches_the_scripts_dir() {
        let root = crate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root");
        let al_text = std::fs::read_to_string(root.join(REGISTRY_PATH)).expect("allowlist");
        let al = parse_allowlist(&al_text).expect("parses");
        let inv = al
            .oracle_inventory
            .as_ref()
            .expect("live allowlist must carry [oracle_inventory]");
        assert!(
            !inv.files.is_empty(),
            "live inventory of remaining oracles must not be empty"
        );
        let disc = discover_oracle_scripts(&root.join("scripts")).expect("scan scripts/");
        assert!(
            !disc.is_empty(),
            "a scan of remaining verify/validate/smoke .py that found nothing is an ERROR"
        );
        let h = inventory_findings(Some(inv), &disc);
        assert!(
            h.errors.is_empty() && h.violations.is_empty(),
            "live oracle inventory must match the scripts/ scan: {h:?}"
        );
        assert!(
            !disc.contains("scripts/verify_content_lock.py"),
            "retired content-lock oracle leaked back into the scan: {disc:?}"
        );
        assert!(
            !disc.contains("scripts/verify_knowledge_paths.py"),
            "retired knowledge-paths oracle leaked back into the scan: {disc:?}"
        );
        assert!(
            !disc.iter().any(|p| p.contains("smoke_")),
            "EXTRACT-THEN-DELETE left a smoke_*.py: {disc:?}"
        );
    }

    // ── bd-check-sh-transitive-invocation-gzvb ────────────────────────────

    #[test]
    fn empty_inventory_is_an_error_not_a_pass() {
        let w = walk_invocations("#!/bin/sh\ntrue\n", |_| None).expect("walk");
        assert!(w.is_empty(), "{w:?}");
        let e = require_nonempty_inventory(&w).unwrap_err();
        assert!(e.contains("empty"), "{e}");
    }

    #[test]
    fn plant_gzvb_behind_a_variable_in_a_temp_copy_must_appear() {
        // TEMP copy — not the live tree. Hide python3 scripts/plant_gzvb.py
        // behind $HIDDEN so a grep of the entry file cannot see the path.
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        files.insert(
            "scripts/check.sh".to_string(),
            "#!/bin/sh\nHIDDEN=\"scripts/plant_gzvb.py\"\npython3 \"$HIDDEN\"\nsh scripts/child_gzvb.sh\n"
                .to_string(),
        );
        files.insert(
            "scripts/child_gzvb.sh".to_string(),
            "#!/bin/sh\nCHECKER=\"scripts/nested_oracle.py\"\npython3 \"$CHECKER\"\nnode scripts/smoke.mjs\n[ -f scripts/presence_only.py ] || exit 2\n"
                .to_string(),
        );
        let walk = walk_invocations(files.get("scripts/check.sh").unwrap(), |p| {
            files.get(p).cloned()
        })
        .expect("walk");
        require_nonempty_inventory(&walk).expect("plant walk must not be empty");
        assert!(
            walk.paths.contains("scripts/plant_gzvb.py"),
            "plant RED: $HIDDEN hid plant_gzvb.py and the walk missed it: {walk:?}"
        );
        assert!(
            walk.paths.contains("scripts/nested_oracle.py"),
            "child $CHECKER missed: {walk:?}"
        );
        assert!(
            walk.paths.contains("scripts/child_gzvb.sh"),
            "child script missed: {walk:?}"
        );
        assert!(
            walk.paths.contains("scripts/smoke.mjs"),
            "node child missed: {walk:?}"
        );
        assert!(
            walk.presence.contains("scripts/presence_only.py"),
            "presence-only plant missed: {walk:?}"
        );
        assert!(
            !walk.paths.contains("scripts/presence_only.py"),
            "a [ -f ] is not an invoke: {walk:?}"
        );
    }

    #[test]
    fn unread_followable_child_is_an_error_not_a_silent_skip() {
        let e = walk_invocations("#!/bin/sh\nsh scripts/missing.sh\n", |_| None).unwrap_err();
        assert!(e.contains("scripts/missing.sh"), "{e}");
        assert!(e.contains("incomplete"), "{e}");
    }

    #[test]
    fn not_on_check_sh_path_claim_for_a_reached_path_is_red() {
        let mut r = row("scripts/verify_bank.py");
        r.reason = "Differential oracle. Not a check.sh step.".into();
        let mut invoked = BTreeSet::new();
        invoked.insert("scripts/verify_bank.py".into());
        let h = reason_honesty_with_set(
            &[r],
            Some("#!/bin/sh\nsh scripts/smoke_slo.sh\n"),
            Some(&invoked),
            None,
        );
        assert!(h.errors.is_empty(), "{h:?}");
        assert_eq!(h.violations.len(), 1, "{h:?}");
        assert!(h.violations[0].contains("scripts/verify_bank.py"), "{h:?}");
        assert!(
            h.violations[0].contains("not on the check.sh path")
                || h.violations[0].contains("not a check.sh step"),
            "{h:?}"
        );
    }

    // ── bd-transitive-invocation-blindspot-lcfj leftovers ─────────────────

    #[test]
    fn presence_only_file_is_reported_distinct_from_invoke() {
        let sh = "#!/bin/sh\n[ -f scripts/foo.py ] || exit 2\npython3 scripts/bar.py\nrm -f scripts/foo.py\n";
        let walk = walk_invocations(sh, |_| None).expect("walk");
        assert!(
            walk.presence.contains("scripts/foo.py"),
            "presence-only missed: {walk:?}"
        );
        assert!(
            !walk.paths.contains("scripts/foo.py"),
            "[ -f ] is not an invoke: {walk:?}"
        );
        assert!(walk.paths.contains("scripts/bar.py"), "{walk:?}");
    }

    #[test]
    fn rm_dash_f_is_not_a_presence_check() {
        let walk = walk_invocations(
            "#!/bin/sh\nrm -f scripts/foo.py\ngit add -f scripts/foo.py\npython3 scripts/bar.py\n",
            |_| None,
        )
        .expect("walk");
        assert!(
            !walk.presence.contains("scripts/foo.py"),
            "rm/git -f is force, not presence: {walk:?}"
        );
    }

    #[test]
    fn not_on_path_claim_for_a_presence_only_path_is_red() {
        let mut r = row("scripts/validate_grounding.py");
        r.reason = "Differential oracle. Not a check.sh step.".into();
        let sh =
            "#!/bin/sh\n[ -f scripts/validate_grounding.py ] || exit 2\npython3 scripts/other.py\n";
        let walk = walk_invocations(sh, |_| None).expect("walk");
        assert!(walk.presence.contains("scripts/validate_grounding.py"));
        assert!(!walk.paths.contains("scripts/validate_grounding.py"));
        let h = reason_honesty_with_set(&[r], Some(sh), Some(&walk.paths), Some(&walk.presence));
        assert!(h.errors.is_empty(), "{h:?}");
        assert_eq!(h.violations.len(), 1, "{h:?}");
        assert!(
            h.violations[0].contains("scripts/validate_grounding.py"),
            "{h:?}"
        );
    }

    #[test]
    fn nested_python3_invoke_of_an_oracle_only_row_is_red() {
        // Known-bad direction 1: python3 inside a nested shell for a file
        // whose row says oracle-only / not-on-path → RED. In-tree strings,
        // no git apply.
        let mut r = row("scripts/oracle_only.py");
        r.reason = "Retained as oracle-only. Not on the check.sh path.".into();
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        files.insert(
            "scripts/check.sh".into(),
            "#!/bin/sh\nsh scripts/nested_lcfj.sh\n".into(),
        );
        files.insert(
            "scripts/nested_lcfj.sh".into(),
            "#!/bin/sh\npython3 scripts/oracle_only.py\n".into(),
        );
        let walk = walk_invocations(files.get("scripts/check.sh").unwrap(), |p| {
            files.get(p).cloned()
        })
        .expect("walk");
        assert!(
            walk.paths.contains("scripts/oracle_only.py"),
            "nested python3 missed: {walk:?}"
        );
        let h = reason_honesty_with_set(
            &[r],
            files.get("scripts/check.sh").map(String::as_str),
            Some(&walk.paths),
            Some(&walk.presence),
        );
        assert!(h.errors.is_empty(), "{h:?}");
        assert_eq!(h.violations.len(), 1, "{h:?}");
        assert!(h.violations[0].contains("scripts/oracle_only.py"), "{h:?}");
    }

    #[test]
    fn load_bearing_row_for_a_file_nothing_reaches_is_red() {
        // Known-bad direction 2: mark a row load-bearing for a file nothing
        // reaches → RED. Inverse of the nested-invoke plant.
        let mut r = row("scripts/untouched.py");
        r.reason = "Load-bearing check.sh gate".into();
        let sh = "#!/bin/sh\n[ -f scripts/other.py ] || true\npython3 scripts/other.py\n";
        let walk = walk_invocations(sh, |_| None).expect("walk");
        assert!(!walk.paths.contains("scripts/untouched.py"));
        assert!(!walk.presence.contains("scripts/untouched.py"));
        let h = reason_honesty_with_set(&[r], Some(sh), Some(&walk.paths), Some(&walk.presence));
        assert!(h.errors.is_empty(), "{h:?}");
        assert_eq!(h.violations.len(), 1, "{h:?}");
        assert!(h.violations[0].contains("scripts/untouched.py"), "{h:?}");
        assert!(h.violations[0].contains("does not invoke"), "{h:?}");
    }

    #[test]
    fn tree_derived_floor_of_zero_is_an_error() {
        let walk =
            walk_invocations("#!/bin/sh\npython3 scripts/ghost.py\n", |_| None).expect("walk");
        require_nonempty_inventory(&walk).expect("invoke set is non-empty");
        let e = require_tree_derived_floor(&walk, |_| false).unwrap_err();
        assert!(e.contains("floor is 0"), "{e}");
    }

    #[test]
    fn live_transitive_inventory_names_remaining_py_and_conditional_oracles() {
        let root = crate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root");
        let check = std::fs::read_to_string(root.join(CHECK_SH_PATH)).expect("check.sh");
        let walk = walk_invocations(&check, |p| std::fs::read_to_string(root.join(p)).ok())
            .expect("live walk");
        require_nonempty_inventory(&walk).expect("empty inventory is ERROR");
        let py = walk.python();
        assert!(
            !py.contains(&"scripts/verify_paraphrase_pairs.py"),
            "paraphrase_pairs python invoke must stay GONE (jhd.21): {py:?}"
        );
        assert!(!py.is_empty(), "do not claim zero python: {py:?}");
        if let Ok(slo) = std::fs::read_to_string(root.join("scripts/smoke_slo.sh")) {
            if script_still_invokes_py(&slo, "scripts/verify_bank.py") {
                assert!(
                    walk.paths.contains("scripts/verify_bank.py"),
                    "smoke_slo.sh still calls verify_bank.py but inventory missed it: {py:?}"
                );
            }
        }
        for script in [
            "scripts/selftest_doc_consistency.sh",
            "scripts/selftest_injection_count.sh",
        ] {
            let Ok(text) = std::fs::read_to_string(root.join(script)) else {
                continue;
            };
            let vars = collect_assignments(&text);
            if let Some(vals) = vars.get("CHECKER") {
                for v in vals {
                    if let Some(p) = normalize_repo_path(v) {
                        if p.ends_with(".py") {
                            assert!(
                                walk.paths.contains(&p),
                                "{script} $CHECKER={p} missing from inventory: {py:?}"
                            );
                        }
                    }
                }
            }
        }
        for retired in [
            "scripts/export_anki.py",
            "scripts/verify_content_lock.py",
            "scripts/verify_knowledge_paths.py",
            "scripts/smoke_learn_v2.py",
        ] {
            assert!(
                !walk.paths.contains(retired),
                "{retired} is retired and must not reappear: {py:?}"
            );
        }
        assert!(
            walk.paths.contains("scripts/smoke_slo.sh"),
            "must follow sh scripts/smoke_slo.sh: {:?}",
            walk.paths
        );
        require_tree_derived_floor(&walk, |p| root.join(p).is_file())
            .expect("tree-derived floor is 0");
        assert!(
            walk.presence.contains("scripts/validate_grounding.py"),
            "validate_grounding.py is [ -f ] only; must appear as presence: {:?}",
            walk.presence
        );
        assert!(
            !walk.paths.contains("scripts/validate_grounding.py"),
            "validate_grounding.py is not invoked: {:?}",
            walk.paths
        );
        // Tree-derived mjs floor: on-disk scripts/*.mjs ∩ walk. Do not pull
        // js into the rust allowlist; do inventory node/mjs (bd-yp9x / lcfj).
        let mut mjs_on_disk = BTreeSet::new();
        if let Ok(rd) = std::fs::read_dir(root.join("scripts")) {
            for ent in rd.flatten() {
                let name = ent.file_name();
                let name = name.to_string_lossy();
                if name.ends_with(".mjs") {
                    mjs_on_disk.insert(format!("scripts/{name}"));
                }
            }
        }
        assert!(
            !mjs_on_disk.is_empty(),
            "tree-derived mjs floor: scripts/*.mjs is empty"
        );
        let mjs_hit: BTreeSet<_> = mjs_on_disk.intersection(&walk.paths).cloned().collect();
        assert!(
            !mjs_hit.is_empty(),
            "node gates on the path must appear in the inventory: on_disk={mjs_on_disk:?} walk={:?}",
            walk.paths
        );
        for must in [
            "scripts/smoke_srs.mjs",
            "scripts/smoke_mastery.mjs",
            "scripts/smoke_hub_mastery.mjs",
            "scripts/smoke_quiz_approved.mjs",
            "scripts/smoke_results_wasm.mjs",
        ] {
            if mjs_on_disk.contains(must) {
                assert!(
                    walk.paths.contains(must),
                    "node {must} on disk and named from a followed shell: {mjs_hit:?}"
                );
            }
        }
        // bd-checksh-cargo-run-attribution-tebe: check.sh compiles once, then
        // invokes ./target/debug/cdcp_gate (via run_cdcp_gate). cargo run of
        // this gate is the attribution leak this bead closes; the inventory
        // walker still only extracts `cargo run`, so the live contract is the
        // script text.
        let live_bin = check.lines().any(|l| {
            let code = code_part(l).trim();
            code.contains("cdcp_gate")
                && code.contains(NAME)
                && !code.contains("cargo run")
                && !code.starts_with("echo ")
                && !code.starts_with("ok ")
                && !code.starts_with(':')
        });
        assert!(
            live_bin,
            "substrate-guard must be invoked via the built binary, not cargo run"
        );
        assert!(
            !check.lines().any(|l| {
                let code = code_part(l).trim();
                !code.is_empty()
                    && (code.contains("cargo run -") || code.contains("cargo run -p"))
                    && code.contains("cdcp_gate")
            }),
            "cdcp_gate must not be invoked via cargo run (sibling rustc output is a gate-attribution leak)"
        );
    }

    // ── the header's own honesty ─────────────────────────────────────────
    #[test]
    fn header_states_a_floor_raise_and_overclaims_nothing() {
        let src = include_str!("substrate_guard.rs");
        let header: String = src
            .lines()
            .take_while(|l| l.starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            header.contains("FLOOR-RAISE"),
            "header must state the claim class"
        );
        assert!(
            header.contains("CANNOT"),
            "header must state what the gate cannot decide"
        );
        for banned in ["guarantee", "proves", "makes impossible", "impossible"] {
            assert!(
                !header.to_lowercase().contains(banned),
                "header overclaims with {banned:?}"
            );
        }
    }
}
