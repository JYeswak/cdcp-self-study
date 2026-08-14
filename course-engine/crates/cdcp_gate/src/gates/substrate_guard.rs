//! substrate-guard — S0 of the Rust migration (bd-substrate-rust-migration-jhd.1).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises a floor. It enforces exactly one property: **no unreasoned
//! non-Rust source file enters the tree.** Concretely — a `.py` or `.sh` file
//! tracked or staged under `scripts/`, `crates/`, or the engine root must have a
//! row in `registries/substrate_allowlist.toml` carrying a non-empty `reason`, a
//! bead id, and an `expires` date that has not passed.
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
//! It cannot decide whether a stated `reason` is honest. It cannot tell a real
//! migration bead from a plausible-looking id. It cannot tell an achievable
//! `expires` from a date chosen to be far away. It reads none of the scripts it
//! permits, so it says nothing about what they do. An author who wants a script
//! in this tree can still get one in by writing a sentence — the change is that
//! the sentence is now dated, attributed, reviewable in a diff, and it rots.
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
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const NAME: &str = "substrate-guard";
pub const SUMMARY: &str =
    "no .py/.sh may enter scripts//crates//root without a reasoned, dated, bead-linked allowlist row";

/// Where the registry lives, relative to the engine root.
pub const REGISTRY_PATH: &str = "registries/substrate_allowlist.toml";

/// The ONE file whose wiring this gate judges.
///
/// `[wiring].check_sh` used to be free text, so pointing it at any file holding a
/// suitable string satisfied the wiring leg from a file nothing runs (bd-bo6i).
/// It is now pinned; a different value is a schema ERROR.
pub const CHECK_SH_PATH: &str = "scripts/check.sh";

/// Extensions the registry may WIDEN but may never narrow below.
pub const FLOOR_EXTENSIONS: &[&str] = &["py", "sh"];
/// Directories the registry may ADD to but may never drop.
pub const FLOOR_ROOTS: &[&str] = &["scripts", "crates"];
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
        if !scan.extensions.iter().any(|e| e == ext) {
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
pub fn is_in_scope(path: &str, scan: &ScanCfg) -> bool {
    if normalisation_defect(path).is_some() {
        return false;
    }
    match path.split_once('/') {
        // Engine-root file: no directory component.
        None => scan.include_engine_root_files,
        Some((head, _)) => scan.roots.iter().any(|r| r == head),
    }
}

pub fn has_scanned_extension(path: &str, scan: &ScanCfg) -> bool {
    let Some((_, ext)) = path.rsplit_once('.') else {
        return false;
    };
    !ext.contains('/') && scan.extensions.iter().any(|e| e == ext)
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
        if !has_scanned_extension(path, scan) {
            v.push(format!(
                "{where_}: extension is not one this gate scans ({}) — delete the row",
                scan.extensions.join(", ")
            ));
        }

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

/// Scanned files with no row. `rows` is assumed already schema-checked.
pub fn unlisted(candidates: &[String], rows: &[Row], scan: &ScanCfg) -> Vec<String> {
    let listed: BTreeSet<&str> = rows.iter().map(|r| r.path.trim()).collect();
    let mut out = Vec::new();
    for c in candidates {
        if !is_in_scope(c, scan) || !has_scanned_extension(c, scan) {
            continue;
        }
        if !listed.contains(c.as_str()) {
            out.push(format!(
                "{c}: non-Rust file with no row in {REGISTRY_PATH}. Port it to Rust (see epic bd-substrate-rust-migration-jhd), or add a row with a real `reason`, a `migration_bead`, and an `expires` date"
            ));
        }
    }
    out
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

    let tracked = vcs::tracked_files(root).map_err(GateError::error)?;

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
        return Err(GateError::error(format!(
            "0 files in scope under {:?} (+ engine-root files) out of {} tracked — the scan roots resolve to nothing; ERROR, not a pass",
            wt_al.scan.roots,
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

    // ── rows: each snapshot answers "does this file exist" for ITSELF ───────
    let today = date::today();
    let index_set: BTreeSet<&str> = tracked.iter().map(String::as_str).collect();
    let wt_exists = |p: &str| root.join(p).exists();
    let ix_exists = |p: &str| index_set.contains(p);

    let schema_errs = merge(
        validate_rows(&wt_al.allow, &wt_al.scan, today, &wt_exists),
        validate_rows(&ix_al.allow, &ix_al.scan, today, &ix_exists),
    );
    if !schema_errs.is_empty() {
        // Schema errors are ERROR-class: the registry could not be honestly read
        // as a set of exemptions, so no file is exempt on its strength.
        return Err(GateError::Error(format!(
            "{} schema error(s) in {REGISTRY_PATH}: {}",
            schema_errs.len(),
            schema_errs.join(" | ")
        )));
    }

    // ── presence: the SUBJECT and the POLICY come from the same snapshot ────
    let wt_candidates: Vec<String> = tracked
        .iter()
        .filter(|p| root.join(p).exists())
        .cloned()
        .collect();
    let mut violations = merge(
        unlisted(&wt_candidates, &wt_al.allow, &wt_al.scan),
        unlisted(&tracked, &ix_al.allow, &ix_al.scan),
    );

    // Staged leg: what THIS commit would add, phrased as such. Judged by the
    // index's allowlist, because that is the allowlist the commit carries.
    let mut staged_count = 0usize;
    if ctx.has_flag("--staged") {
        let staged = vcs::staged_additions(root).map_err(GateError::error)?;
        staged_count = staged.len();
        for s in unlisted(&staged, &ix_al.allow, &ix_al.scan) {
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
    for m in merge(wt_soft, ix_soft) {
        eprintln!("{NAME}: PENDING WIRING: {m}");
    }

    if !violations.is_empty() {
        return Err(GateError::Violation(violations));
    }

    if !quiet {
        let listed = ix_al.allow.len();
        let wiring = if wt_ev == ix_ev {
            wt_ev.tag().to_string()
        } else {
            format!("worktree={} index={}", wt_ev.tag(), ix_ev.tag())
        };
        println!(
            "{NAME}: ok: scanned={} in_scope={} staged_adds={} exemptions={} wiring={wiring}",
            tracked.len(),
            in_scope.len(),
            staged_count,
            listed,
        );
        if wt_text != ix_text || wt_check != ix_check {
            println!(
                "{NAME}: note: the working tree and the index disagree about {REGISTRY_PATH} and/or {CHECK_SH_PATH}. Both snapshots were judged and both are clean."
            );
        }
        println!(
            "{NAME}: floor-raise only: a row records that a reason was WRITTEN, not that it is true. {listed} exemption(s) outstanding; target is 0."
        );
        println!(
            "{NAME}: the wiring leg above is TEXT ONLY — reading a shell line cannot establish that it executes. Run `cdcp_gate {NAME} --prove-wired` for the behavioural leg."
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
            extensions: vec!["py".into(), "sh".into()],
            include_engine_root_files: true,
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
        // in scope, not scanned — no row is demanded and none is wanted
        "scripts/README",
        "scripts/smoke.mjs",
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

    #[test]
    fn files_outside_the_scanned_surface_pass() {
        // tests/ and docs/ are not in scope; the gate is a floor, not a dragnet.
        let v = unlisted(
            &["tests/voice-slop.sh".to_string(), "docs/x.py".to_string()],
            &[],
            &scan(),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn extensionless_and_other_extensions_pass() {
        let v = unlisted(
            &[
                "scripts/README".to_string(),
                "scripts/smoke_srs.mjs".to_string(),
                "scripts/_module_page_template.html".to_string(),
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

    #[test]
    fn row_outside_scope_is_red() {
        let v = validate_rows(&[row("docs/a.py")], &scan(), TODAY, &always());
        assert!(
            v.iter().any(|m| m.contains("outside the scanned surface")),
            "{v:?}"
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
        s.extensions.push("mjs".into());
        s.roots.push("web".into());
        assert!(check_floor(&s).is_empty());
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
        assert!(!is_in_scope("docs/a.py", &s));
        assert!(!is_in_scope("/etc/a.sh", &s));
        assert!(!is_in_scope("../a.sh", &s));
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
    #[test]
    fn vacuity_a_scan_that_excludes_the_plant_is_an_error() {
        let narrowed = reg("").replace("extensions = [\"py\", \"sh\"]", "extensions = [\"sh\"]");
        let e = probe_plant_vacuity(&narrowed).unwrap_err();
        assert!(e.contains("outside the scanned surface"), "{e}");

        let rerooted = reg("").replace("roots = [\"scripts\", \"crates\"]", "roots = [\"crates\"]");
        let e = probe_plant_vacuity(&rerooted).unwrap_err();
        assert!(e.contains("outside the scanned surface"), "{e}");
    }

    #[test]
    fn vacuity_an_ordinary_registry_clears_the_plant() {
        assert_eq!(
            probe_plant_vacuity(&reg(&plant_row("scripts/other.py"))),
            Ok(())
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
