//! capability-maturity — B1 of milestone B (bd-hardening-b-ledgers-gvm.1).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises exactly one floor: **a capability claim in this repo is
//! ATTRIBUTED, DATED, and points at something that exists.** Concretely, every
//! `[[capability]]` row in `registries/capability-maturity.toml` must carry a
//! level drawn from a compiled-in lattice, a non-empty owner, a `last_review`
//! date inside the policy window, a one-sentence claim, and evidence entries
//! that resolve — a `contract` to a readable file, a `test`/`known_bad` to a
//! named function that is actually defined in a file in this tree, a `commit` to
//! an object git can name. Where a row quotes a CHARTER cell, the cell must
//! still read the way the row records, and a cell that says "wired" must be
//! backed by a level at or above `gate-wired`.
//!
//! # WHAT THIS GATE CANNOT DECIDE — read this before quoting it
//!
//! This gate is unusually easy to overclaim for, so the ceiling is stated first.
//!
//! It **cannot decide that a level is honest.** Nothing here can tell
//! `proven-to-trip` from `test-backed` on the merits; it can only insist that
//! whichever was written carries the evidence shape that level requires. An
//! author who wants a higher level can still reach it by naming a test that
//! exists — the change is that a test must exist, be named, be greppable in a
//! diff, and the row rots after the policy window.
//!
//! It **cannot decide that a cited test asserts anything meaningful.** It reads
//! the function's name, never its body. `fn known_bad_x_is_red() {}` resolves
//! exactly like a real injection does. Whether the assertion bites is L4's job
//! (`selftest_*` suites and the per-gate known-bad legs), not this gate's.
//!
//! It **cannot decide that the `claim` sentence is true**, that the `owner` is a
//! person who will answer, or that a `last_review` date reflects a review that
//! happened rather than a date that was bumped.
//!
//! It **cannot decide that a level matches reality.** `gate-wired` says a step
//! in `scripts/check.sh` fails the build; this gate does not run check.sh and
//! does not read it for wiring — `substrate-guard --prove-wired` is the only
//! thing in this tree that settles wiring behaviourally, and it settles it for
//! one gate.
//!
//! The `charter_claim` leg reads a markdown table as text, and text is all it
//! can be: it locates a row by its first cell and compares the second cell
//! verbatim. A cell reworded for good reasons trips it exactly like a cell
//! quietly inflated, which is why the finding is worded as DRIFT and points at
//! both sides rather than declaring which one is wrong. What it does settle is
//! narrow and worth having: a maturity claim published in prose and a maturity
//! level recorded in a registry can no longer disagree in silence.
//!
//! The floor moves from *a maturity claim is a sentence somebody typed* to *a
//! maturity claim is attributed, dated, expiring, and pointed at a file, a named
//! test function or a commit that exists*. That is the whole of it.
//!
//! # WHY THE ROWS ARE ALLOWED TO BE RED
//!
//! `L3 External oracle | YES · wired` survived for months because a maturity
//! claim living in a prose table has no mechanical falsifier. The cure is not a
//! ledger tuned until it is green — that is the same defect with a schema. The
//! cure is a ledger that records the honest level and lets the gate refuse the
//! rows where the published claim outruns the evidence. Two rows are RED the day
//! this landed, and both are findings about the repo rather than defects in the
//! file: `l2.slo-as-code` (published "YES · wired"; no named test asserts any
//! budget) and `l5.fuzz-crash-floor` (published "YES · wired"; nothing runs the
//! fuzz targets).
//!
//! # ANTI-VACUOUS (L4)
//!
//! Zero capability rows is an ERROR. Zero evidence entries is an ERROR. A cited
//! CHARTER file that yields zero markdown table rows is an ERROR, because a
//! claim file that could not be read as a table must never report the way one
//! with no conflicts does. A registry that loosens `staleness_days` past the
//! compiled-in ceiling, or drops a compiled-in REQUIRED row, is an ERROR: the
//! registry configures the check, so the registry is itself an attack surface,
//! and it may tighten the policy but never widen it.

#![forbid(unsafe_code)]

use crate::date::{self, Ymd};
use crate::registry::{GateCtx, GateError};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const NAME: &str = "capability-maturity";
pub const SUMMARY: &str =
    "capability claims are attributed, dated, expiring, and pointed at a test/file that exists";

/// Where the ledger lives, relative to the engine root.
pub const REGISTRY_PATH: &str = "registries/capability-maturity.toml";

/// The longest review window the registry may configure. It may tighten this;
/// widening it would disable the expiry with a one-number diff, which is the
/// bypass the whole ledger exists to close.
pub const MAX_STALENESS_DAYS: i64 = 90;

/// Rows whose absence is an ERROR. Deleting the row that records the failure
/// this ledger was built for is not a way to pass it.
pub const REQUIRED_IDS: &[&str] = &["l3.external-oracle-factual"];

/// A claim sentence shorter than this says nothing a reviewer can disagree with.
pub const MIN_CLAIM_LEN: usize = 40;

/// At least this many rows must cross-check a published CHARTER cell. Zero would
/// leave the prose free to drift again, which is the defect class, not a style.
pub const MIN_CHARTER_CLAIMS: usize = 1;

const KNOWN_FLAGS: &[&str] = &["--quiet"];

/// The maturity lattice, weakest first. Compiled in: the registry may not add,
/// rename or reorder levels, because the level is what selects the obligation.
///
/// * `absent`         — the capability does not exist here.
/// * `declared`       — artifacts exist; nothing runs them automatically.
/// * `smoke-checked`  — a script or shell step checks it; no named test.
/// * `test-backed`    — at least one named test function exercises it.
/// * `gate-wired`     — test-backed, and a step in `scripts/check.sh` fails red.
/// * `proven-to-trip` — plus a known-bad injection asserted to reach red.
pub const LEVELS: &[&str] = &[
    "absent",
    "declared",
    "smoke-checked",
    "test-backed",
    "gate-wired",
    "proven-to-trip",
];

/// The lowest rank a published "wired" claim can honestly rest on: `gate-wired`.
pub const WIRED_MIN_LEVEL: &str = "gate-wired";

/// The evidence kinds a row may cite.
pub const KINDS: &[&str] = &["contract", "test", "known_bad", "commit"];

// ── registry schema ────────────────────────────────────────────────────────

/// Every field is `#[serde(default)]` so a MISSING field arrives as an empty
/// value and is reported as the schema error it is, rather than as an opaque
/// TOML parse failure that names a line and not a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Ledger {
    pub schema_version: u32,
    #[serde(default)]
    pub policy: Policy,
    #[serde(default)]
    pub capability: Vec<Row>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Policy {
    /// Absent lands as 0, which fails the floor check below. Blank is never
    /// permissive: a missing window must not read as "never expires".
    #[serde(default)]
    pub staleness_days: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Row {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub last_review: String,
    #[serde(default)]
    pub claim: String,
    #[serde(default)]
    pub charter_claim: Option<CharterClaim>,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

/// The published cell this row tracks. Present only where the repo makes a
/// public maturity claim about the capability.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CharterClaim {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub row: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Evidence {
    #[serde(default)]
    pub kind: String,
    /// `ref` is a Rust keyword; the TOML key is `ref`.
    #[serde(default, rename = "ref")]
    pub reference: String,
}

// ── the lattice ────────────────────────────────────────────────────────────

/// Rank of a level, or `None` when the level is not in the lattice.
pub fn rank(level: &str) -> Option<usize> {
    LEVELS.iter().position(|l| *l == level.trim())
}

/// `(min_test, min_contract, min_known_bad)` — what a level obliges its row to
/// carry. Minimums, not maximums: a row may always cite more than it must.
pub fn obligation(level_rank: usize) -> (usize, usize, usize) {
    match level_rank {
        // absent · declared · smoke-checked — a file or script, no named test.
        0..=2 => (0, 1, 0),
        // test-backed — a named test function, nothing else required.
        3 => (1, 0, 0),
        // gate-wired — the test, plus the artifact that carries the wiring.
        4 => (1, 1, 0),
        // proven-to-trip — plus an injection asserted to reach red.
        _ => (1, 1, 1),
    }
}

// ── civil-date arithmetic ──────────────────────────────────────────────────
// `crate::date` answers "what is today" and "is this before that"; staleness
// needs an AGE IN DAYS, and this gate is not allowed to reach into a shared
// module another agent owns mid-migration. Howard Hinnant's `days_from_civil`,
// the exact inverse of the one `crate::date` uses.

/// Days since 1970-01-01 for a proleptic-Gregorian civil date.
pub fn days_from_civil((y, m, d): Ymd) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Whole days from `from` to `to`; negative when `to` precedes `from`.
pub fn days_between(from: Ymd, to: Ymd) -> i64 {
    days_from_civil(to) - days_from_civil(from)
}

// ── pure schema validation ─────────────────────────────────────────────────

pub fn parse_ledger(text: &str) -> Result<Ledger, String> {
    let l: Ledger = toml::from_str(text).map_err(|e| format!("parse {REGISTRY_PATH}: {e}"))?;
    if l.schema_version != 1 {
        return Err(format!(
            "{REGISTRY_PATH}: schema_version {} unsupported (expected 1)",
            l.schema_version
        ));
    }
    Ok(l)
}

/// The registry configures its own review window, so it may only TIGHTEN the
/// compiled-in ceiling.
pub fn check_policy(p: &Policy) -> Vec<String> {
    let mut v = Vec::new();
    if p.staleness_days <= 0 {
        v.push(format!(
            "{REGISTRY_PATH}: [policy].staleness_days is {} — a window that cannot expire is no window; blank and zero are never permissive",
            p.staleness_days
        ));
    } else if p.staleness_days > MAX_STALENESS_DAYS {
        v.push(format!(
            "{REGISTRY_PATH}: [policy].staleness_days {} exceeds the compiled-in ceiling of {MAX_STALENESS_DAYS} — the registry may tighten the window, never widen it",
            p.staleness_days
        ));
    }
    v
}

/// Is this a normalised engine-root-relative path? A ref that can climb out of
/// the tree, or name an absolute location, is refused rather than resolved.
pub fn is_clean_relative_path(p: &str) -> bool {
    let p = p.trim();
    if p.is_empty() || p.starts_with('/') || p.contains('\\') {
        return false;
    }
    !p.split('/').any(|c| c.is_empty() || c == "." || c == "..")
}

/// A Rust identifier, as a test function name must be.
pub fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A short git rev: 7–40 lowercase hex characters.
pub fn looks_like_rev(s: &str) -> bool {
    let s = s.trim();
    (7..=40).contains(&s.len())
        && s.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

/// Split a `test`/`known_bad` ref into `(path, function name)`.
pub fn split_test_ref(r: &str) -> Result<(&str, &str), String> {
    let r = r.trim();
    let mut parts = r.split("::");
    let (Some(path), Some(func)) = (parts.next(), parts.next()) else {
        return Err(format!(
            "{r:?} is not <path>::<test_fn> — a test reference must name the file AND the function"
        ));
    };
    if parts.next().is_some() {
        return Err(format!("{r:?} holds more than one `::` separator"));
    }
    if !is_clean_relative_path(path) {
        return Err(format!(
            "{r:?}: {path:?} is not a normalised engine-root-relative path"
        ));
    }
    if !is_ident(func) {
        return Err(format!("{r:?}: {func:?} is not a function name"));
    }
    Ok((path, func))
}

/// Everything decidable without touching the filesystem. These are SCHEMA
/// errors: a ledger that cannot be read as a set of claims exempts nothing.
pub fn schema_errors(l: &Ledger) -> Vec<String> {
    let mut v = check_policy(&l.policy);
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut evidence_total = 0usize;
    let mut charter_rows = 0usize;

    // Anti-vacuous: a ledger with nothing in it reports exactly like one whose
    // every claim held up.
    if l.capability.is_empty() {
        v.push(format!(
            "{REGISTRY_PATH}: zero [[capability]] rows — a vacuous ledger is an ERROR, not a pass"
        ));
    }

    for (i, r) in l.capability.iter().enumerate() {
        let id = r.id.trim();
        let where_ = if id.is_empty() {
            format!("[[capability]] #{}", i + 1)
        } else {
            format!("[[capability]] {id}")
        };

        if id.is_empty() {
            v.push(format!("{where_}: missing or empty `id`"));
        } else if !seen.insert(id) {
            v.push(format!("{where_}: duplicate `id`"));
        }
        if r.title.trim().is_empty() {
            v.push(format!("{where_}: missing or empty `title`"));
        }
        if r.owner.trim().is_empty() {
            v.push(format!(
                "{where_}: missing or empty `owner` — a capability claim nobody owns is nobody's to defend"
            ));
        }
        let claim = r.claim.trim();
        if claim.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `claim` — a level with no sentence saying what it means is a label, not a claim"
            ));
        } else if claim.len() < MIN_CLAIM_LEN {
            v.push(format!(
                "{where_}: `claim` is {} chars; at least {MIN_CLAIM_LEN} are needed to say something a reviewer can disagree with",
                claim.len()
            ));
        }

        let level = r.level.trim();
        if level.is_empty() {
            v.push(format!(
                "{where_}: missing or empty `level` — blank is never permissive; use one of: {}",
                LEVELS.join(", ")
            ));
        } else if rank(level).is_none() {
            v.push(format!(
                "{where_}: `level` {level:?} is not in the lattice ({})",
                LEVELS.join(" < ")
            ));
        }

        if r.last_review.trim().is_empty() {
            v.push(format!(
                "{where_}: missing or empty `last_review` — a claim that cannot go stale is permanent by another name"
            ));
        } else if let Err(e) = date::parse_ymd(r.last_review.trim()) {
            v.push(format!("{where_}: `last_review` {e}"));
        }

        if r.evidence.is_empty() {
            v.push(format!(
                "{where_}: empty `evidence` — a claim citing nothing is the prose table this ledger replaced; empty is a SCHEMA ERROR, never a pass"
            ));
        }
        evidence_total += r.evidence.len();

        for (j, e) in r.evidence.iter().enumerate() {
            let kind = e.kind.trim();
            let reference = e.reference.trim();
            let at = format!("{where_} evidence #{}", j + 1);
            if kind.is_empty() {
                v.push(format!("{at}: missing or empty `kind`"));
            } else if !KINDS.contains(&kind) {
                v.push(format!(
                    "{at}: `kind` {kind:?} is not one of: {}",
                    KINDS.join(", ")
                ));
            }
            if reference.is_empty() {
                v.push(format!("{at}: missing or empty `ref`"));
                continue;
            }
            match kind {
                "test" | "known_bad" => {
                    if let Err(e) = split_test_ref(reference) {
                        v.push(format!("{at}: {e}"));
                    }
                }
                "contract" => {
                    if !is_clean_relative_path(reference) {
                        v.push(format!(
                            "{at}: {reference:?} is not a normalised engine-root-relative path"
                        ));
                    }
                }
                "commit" if !looks_like_rev(reference) => {
                    v.push(format!(
                        "{at}: {reference:?} is not a 7–40 character lowercase hex git rev"
                    ));
                }
                _ => {}
            }
        }

        if let Some(c) = &r.charter_claim {
            charter_rows += 1;
            if c.file.trim().is_empty() {
                v.push(format!("{where_}: `charter_claim.file` is empty"));
            } else if !is_clean_relative_path(c.file.trim()) {
                v.push(format!(
                    "{where_}: `charter_claim.file` {:?} is not a normalised relative path",
                    c.file.trim()
                ));
            }
            if c.row.trim().is_empty() {
                v.push(format!("{where_}: `charter_claim.row` is empty"));
            }
            if c.status.trim().is_empty() {
                v.push(format!(
                    "{where_}: `charter_claim.status` is empty — an unquoted cell cannot drift, which is the point of quoting it"
                ));
            }
        }
    }

    if !l.capability.is_empty() && evidence_total == 0 {
        v.push(format!(
            "{REGISTRY_PATH}: zero evidence entries across {} row(s) — a ledger that cites nothing is an ERROR, not a pass",
            l.capability.len()
        ));
    }
    if !l.capability.is_empty() && charter_rows < MIN_CHARTER_CLAIMS {
        v.push(format!(
            "{REGISTRY_PATH}: {charter_rows} row(s) quote a published CHARTER cell; at least {MIN_CHARTER_CLAIMS} must. Dropping every cross-check would leave the prose free to drift again"
        ));
    }
    for req in REQUIRED_IDS {
        if !l.capability.iter().any(|r| r.id.trim() == *req) {
            v.push(format!(
                "{REGISTRY_PATH}: required row {req:?} is missing — deleting the record of a known absence is not a way to pass this gate"
            ));
        }
    }
    v
}

// ── markdown table reading (text, and worded as text) ──────────────────────

/// One `| label | status | … |` row of a markdown table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    pub label: String,
    pub status: String,
}

/// Strip the emphasis and code decoration a status cell carries so that
/// `**YES · wired**` and `YES · wired` compare equal.
pub fn undecorate(cell: &str) -> String {
    cell.replace(['*', '`'], "").trim().to_string()
}

/// Every two-or-more-column markdown table row in `text`, alignment rows
/// dropped. Deliberately a text scan: no parse of a document settles what a
/// maturity cell means, only whether its characters still read the same way.
pub fn parse_table_rows(text: &str) -> Vec<TableRow> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() < 2 {
            continue;
        }
        // The `|---|:--|` alignment row is not a claim.
        if cells
            .iter()
            .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }
        out.push(TableRow {
            label: undecorate(cells[0]),
            status: undecorate(cells[1]),
        });
    }
    out
}

// ── filesystem- and git-facing evaluation ──────────────────────────────────

/// Does `text` define a function called `name`? Matches `fn name`, `pub fn
/// name`, `async fn name` and generic forms alike, because all of them spell
/// `fn` then the identifier.
pub fn defines_fn(text: &str, name: &str) -> bool {
    let bytes = text.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find("fn ") {
        let at = from + rel;
        from = at + 3;
        // `fn` must be its own token, not the tail of `defn`.
        if at > 0 {
            let prev = bytes[at - 1] as char;
            if prev.is_ascii_alphanumeric() || prev == '_' {
                continue;
            }
        }
        let rest = text[at + 3..].trim_start();
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if &rest[..end] == name {
            return true;
        }
    }
    false
}

/// Resolve a ledger path the way `verify_content_lock` resolves a pinned one:
/// engine root first, then the parent corpus that holds `CHARTER.md` and
/// `modules/`. The two gates agreeing on what "a repo-relative path" means is
/// worth more than either being clever.
pub fn resolve_ref(root: &Path, rel: &str) -> PathBuf {
    let cand = root.join(rel);
    if cand.exists() {
        return cand;
    }
    root.parent().unwrap_or(root).join(rel)
}

/// Everything the evaluation needs from the outside world, injected so the rules
/// stay testable without a filesystem and so no leg can quietly borrow another's
/// answer.
pub struct World<'a> {
    /// File contents, or `None` when the path names nothing readable.
    pub read: &'a dyn Fn(&str) -> Option<String>,
    /// Whether git can name this rev as a commit; `Err` when git could not be
    /// consulted at all, which is an ERROR and never a pass.
    pub commit: &'a dyn Fn(&str) -> Result<bool, String>,
}

/// What one pass over the ledger observed.
#[derive(Debug, Default, Clone)]
pub struct Report {
    pub violations: Vec<String>,
    pub errors: Vec<String>,
    pub evidence_checked: usize,
    pub charter_checked: usize,
    pub oldest_review: Option<String>,
}

/// The verdict pass. Assumes `schema_errors` already ran clean, so every field
/// is present and every ref is well formed; what is left is whether the world
/// agrees with the ledger.
pub fn evaluate(rows: &[Row], today: Ymd, staleness_days: i64, w: &World<'_>) -> Report {
    let mut rep = Report::default();
    let mut oldest: Option<(i64, String)> = None;

    for r in rows {
        let id = r.id.trim();
        let where_ = format!("[[capability]] {id}");
        let level = r.level.trim();
        let lr = rank(level).unwrap_or(0);

        // ── staleness ──────────────────────────────────────────────────────
        if let Ok(d) = date::parse_ymd(r.last_review.trim()) {
            let age = days_between(d, today);
            if age > staleness_days {
                rep.violations.push(format!(
                    "{where_}: `last_review` {} is {age} days old (window {staleness_days}) — EXPIRED. Re-read the evidence, then re-date the row; a date bumped without a review is the same defect in a newer coat",
                    r.last_review.trim()
                ));
            }
            match &oldest {
                Some((a, _)) if *a >= age => {}
                _ => oldest = Some((age, r.last_review.trim().to_string())),
            }
        }

        // ── evidence resolution ────────────────────────────────────────────
        let mut have_test = 0usize;
        let mut have_contract = 0usize;
        let mut have_known_bad = 0usize;

        for e in &r.evidence {
            let kind = e.kind.trim();
            let reference = e.reference.trim();
            rep.evidence_checked += 1;
            match kind {
                "contract" => {
                    if (w.read)(reference).is_some() {
                        have_contract += 1;
                    } else {
                        rep.violations.push(format!(
                            "{where_}: contract evidence {reference:?} names nothing readable in this tree — a claim pointed at a file that is not here is the prose claim again"
                        ));
                    }
                }
                "test" | "known_bad" => {
                    let Ok((path, func)) = split_test_ref(reference) else {
                        continue;
                    };
                    match (w.read)(path) {
                        None => rep.violations.push(format!(
                            "{where_}: {kind} evidence {reference:?} names {path:?}, which is not a readable file in this tree"
                        )),
                        Some(text) if !defines_fn(&text, func) => rep.violations.push(format!(
                            "{where_}: {kind} evidence {reference:?} names test function {func:?}, which {path} does not define"
                        )),
                        Some(_) => {
                            if kind == "test" {
                                have_test += 1;
                            } else {
                                have_known_bad += 1;
                            }
                        }
                    }
                }
                "commit" => match (w.commit)(reference) {
                    Err(e) => rep.errors.push(format!(
                        "{where_}: commit evidence {reference:?} could not be checked: {e} — an unevaluated leg must not report the way a passed one does"
                    )),
                    Ok(false) => rep.violations.push(format!(
                        "{where_}: commit evidence {reference:?} is not a commit this repository can name"
                    )),
                    Ok(true) => {}
                },
                _ => {}
            }
        }

        // ── the level's obligation ─────────────────────────────────────────
        let (min_test, min_contract, min_known_bad) = obligation(lr);
        if have_test < min_test {
            rep.violations.push(format!(
                "{where_}: level {level:?} obliges at least {min_test} resolvable `test` reference(s); {have_test} resolved. A capability at this level must be able to NAME the function that would go red"
            ));
        }
        if have_contract < min_contract {
            rep.violations.push(format!(
                "{where_}: level {level:?} obliges at least {min_contract} resolvable `contract` reference(s); {have_contract} resolved"
            ));
        }
        if have_known_bad < min_known_bad {
            rep.violations.push(format!(
                "{where_}: level {level:?} obliges at least {min_known_bad} resolvable `known_bad` reference(s); {have_known_bad} resolved. Without a named injection this is `gate-wired`, not `proven-to-trip`"
            ));
        }

        // ── the published cell ─────────────────────────────────────────────
        if let Some(c) = &r.charter_claim {
            rep.charter_checked += 1;
            let file = c.file.trim();
            let want_row = c.row.trim();
            let want_status = c.status.trim();
            let Some(text) = (w.read)(file) else {
                rep.errors.push(format!(
                    "{where_}: charter_claim names {file:?}, which is not readable — the published-claim leg could not be evaluated. ERROR, not a pass"
                ));
                continue;
            };
            let table = parse_table_rows(&text);
            if table.is_empty() {
                rep.errors.push(format!(
                    "{where_}: {file} yielded zero markdown table rows — a claim file that could not be read as a table must not report the way one with no conflicts does. ERROR, not a pass"
                ));
                continue;
            }
            let Some(found) = table.iter().find(|t| t.label == want_row) else {
                rep.violations.push(format!(
                    "{where_}: {file} has no table row labelled {want_row:?} — the ledger is tracking a claim that has moved or been renamed"
                ));
                continue;
            };
            if found.status != want_status {
                rep.violations.push(format!(
                    "{where_}: {file} row {want_row:?} now reads {:?}; the ledger records {want_status:?}. One of the two moved without the other — DRIFT, and this gate does not decide which side is wrong",
                    found.status
                ));
                continue;
            }
            if found.status.to_lowercase().contains("wired")
                && lr < rank(WIRED_MIN_LEVEL).unwrap_or(usize::MAX)
            {
                rep.violations.push(format!(
                    "{where_}: {file} publishes {want_status:?} for {want_row:?} while the ledger can only evidence level {level:?}. A published \"wired\" needs level {WIRED_MIN_LEVEL} or better — raise the capability, or correct the cell"
                ));
            }
        }
    }

    rep.oldest_review = oldest.map(|(_, d)| d);
    rep
}

// ── the gate ───────────────────────────────────────────────────────────────

fn git_commit_exists(root: &Path, rev: &str) -> Result<bool, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{rev}^{{commit}}"),
        ])
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    match out.status.code() {
        Some(0) => Ok(true),
        // `--quiet` turns "no such object" into exit 1 with no output.
        Some(1) => Ok(false),
        other => Err(format!(
            "git rev-parse exited {} : {}",
            other.map_or_else(|| "by signal".to_string(), |c| c.to_string()),
            String::from_utf8_lossy(&out.stderr).trim()
        )),
    }
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(KNOWN_FLAGS)?;
    let quiet = ctx.has_flag("--quiet");
    let root: &Path = &ctx.root;

    let reg = root.join(REGISTRY_PATH);
    let text = std::fs::read_to_string(&reg)
        .map_err(|e| GateError::error(format!("read {}: {e}", reg.display())))?;
    let ledger = parse_ledger(&text).map_err(GateError::error)?;

    let schema = schema_errors(&ledger);
    if !schema.is_empty() {
        return Err(GateError::Error(format!(
            "{} schema error(s) in {REGISTRY_PATH}: {}",
            schema.len(),
            schema.join(" | ")
        )));
    }

    let read =
        |rel: &str| -> Option<String> { std::fs::read_to_string(resolve_ref(root, rel)).ok() };
    let commit = |rev: &str| -> Result<bool, String> { git_commit_exists(root, rev) };
    let world = World {
        read: &read,
        commit: &commit,
    };

    let rep = evaluate(
        &ledger.capability,
        date::today(),
        ledger.policy.staleness_days,
        &world,
    );

    // Anti-vacuous, from the other side: rows that exist but were never really
    // looked at must not report the way looked-at ones do.
    if rep.evidence_checked == 0 {
        return Err(GateError::error(format!(
            "checked 0 evidence entries across {} row(s) — a vacuous ledger scan is an ERROR, not a pass",
            ledger.capability.len()
        )));
    }
    if !rep.errors.is_empty() {
        return Err(GateError::Error(rep.errors.join(" | ")));
    }
    if !rep.violations.is_empty() {
        return Err(GateError::Violation(rep.violations));
    }

    if !quiet {
        println!(
            "{NAME}: ok: rows={} evidence={} charter_checked={} staleness_days={} oldest_review={}",
            ledger.capability.len(),
            rep.evidence_checked,
            rep.charter_checked,
            ledger.policy.staleness_days,
            rep.oldest_review.as_deref().unwrap_or("none")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(level: &str, evidence: Vec<(&str, &str)>) -> Row {
        Row {
            id: "x.y".into(),
            title: "t".into(),
            level: level.into(),
            owner: "josh".into(),
            last_review: "2026-08-14".into(),
            claim: "a sentence long enough to say something a reviewer can disagree with".into(),
            charter_claim: None,
            evidence: evidence
                .into_iter()
                .map(|(k, r)| Evidence {
                    kind: k.into(),
                    reference: r.into(),
                })
                .collect(),
        }
    }

    fn world_of<'a>(
        read: &'a dyn Fn(&str) -> Option<String>,
        commit: &'a dyn Fn(&str) -> Result<bool, String>,
    ) -> World<'a> {
        World { read, commit }
    }

    #[test]
    fn days_arithmetic_round_trips() {
        assert_eq!(days_from_civil((1970, 1, 1)), 0);
        assert_eq!(days_between((2026, 8, 14), (2026, 8, 14)), 0);
        assert_eq!(days_between((2026, 5, 16), (2026, 8, 14)), 90);
        assert_eq!(days_between((2026, 5, 15), (2026, 8, 14)), 91);
        assert_eq!(days_between((2026, 8, 14), (2026, 8, 13)), -1);
        // leap day is counted
        assert_eq!(days_between((2024, 2, 28), (2024, 3, 1)), 2);
    }

    #[test]
    fn the_lattice_is_ordered_and_the_wired_floor_is_in_it() {
        assert_eq!(rank("absent"), Some(0));
        assert!(rank("proven-to-trip") > rank("gate-wired"));
        assert!(rank(WIRED_MIN_LEVEL).is_some());
        assert_eq!(rank("shipped"), None);
    }

    #[test]
    fn obligations_rise_with_the_level() {
        assert_eq!(obligation(0), (0, 1, 0));
        assert_eq!(obligation(3), (1, 0, 0));
        assert_eq!(obligation(4), (1, 1, 0));
        assert_eq!(obligation(5), (1, 1, 1));
    }

    #[test]
    fn refs_that_could_climb_out_of_the_tree_are_refused() {
        assert!(is_clean_relative_path("scripts/check.sh"));
        assert!(!is_clean_relative_path("../CHARTER.md"));
        assert!(!is_clean_relative_path("/etc/passwd"));
        assert!(!is_clean_relative_path("a//b"));
        assert!(!is_clean_relative_path("a/./b"));
        assert!(!is_clean_relative_path(""));
        // A filename with two dots is NOT a traversal — the substrate guard was
        // bitten by exactly this substring test.
        assert!(is_clean_relative_path("scripts/payload..py"));
    }

    #[test]
    fn fn_detection_needs_a_whole_token() {
        assert!(defines_fn("#[test]\nfn alpha() {}", "alpha"));
        assert!(defines_fn("    pub async fn beta<T>(x: T) {}", "beta"));
        assert!(!defines_fn("fn alphabet() {}", "alpha"));
        assert!(!defines_fn("// mentions alpha in a comment", "alpha"));
        assert!(!defines_fn("defn alpha", "alpha"));
    }

    #[test]
    fn table_rows_drop_the_alignment_row_and_undecorate() {
        let md = "| Layer | Applies? |\n|---|:--|\n| **L2 SLO** | **YES · wired** |\n";
        let rows = parse_table_rows(md);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].label, "L2 SLO");
        assert_eq!(rows[1].status, "YES · wired");
    }

    #[test]
    fn a_blank_field_is_a_schema_error_never_permission() {
        let mut l = Ledger {
            schema_version: 1,
            policy: Policy { staleness_days: 90 },
            capability: vec![row("absent", vec![("contract", "a.md")])],
        };
        l.capability[0].id = "l3.external-oracle-factual".into();
        l.capability[0].charter_claim = Some(CharterClaim {
            file: "CHARTER.md".into(),
            row: "L3".into(),
            status: "NO".into(),
        });
        assert!(schema_errors(&l).is_empty(), "{:?}", schema_errors(&l));

        l.capability[0].owner = "  ".into();
        let errs = schema_errors(&l);
        assert!(errs.iter().any(|e| e.contains("`owner`")), "{errs:?}");
    }

    #[test]
    fn an_empty_evidence_list_is_a_schema_error() {
        let l = Ledger {
            schema_version: 1,
            policy: Policy { staleness_days: 90 },
            capability: vec![Row {
                id: "l3.external-oracle-factual".into(),
                charter_claim: Some(CharterClaim {
                    file: "CHARTER.md".into(),
                    row: "L3".into(),
                    status: "NO".into(),
                }),
                ..row("absent", vec![])
            }],
        };
        let errs = schema_errors(&l);
        assert!(
            errs.iter().any(|e| e.contains("empty `evidence`")),
            "{errs:?}"
        );
    }

    #[test]
    fn zero_rows_is_an_error_not_a_pass() {
        let l = Ledger {
            schema_version: 1,
            policy: Policy { staleness_days: 90 },
            capability: vec![],
        };
        let errs = schema_errors(&l);
        assert!(
            errs.iter().any(|e| e.contains("vacuous ledger")),
            "{errs:?}"
        );
    }

    #[test]
    fn a_widened_window_is_refused_and_a_tightened_one_is_not() {
        assert!(check_policy(&Policy { staleness_days: 30 }).is_empty());
        assert!(!check_policy(&Policy {
            staleness_days: MAX_STALENESS_DAYS + 1
        })
        .is_empty());
        assert!(!check_policy(&Policy { staleness_days: 0 }).is_empty());
    }

    #[test]
    fn an_expired_row_is_red_and_a_fresh_one_is_not() {
        let read = |_: &str| Some("fn t() {}".to_string());
        let commit = |_: &str| Ok(true);
        let w = world_of(&read, &commit);

        let mut r = row("absent", vec![("contract", "a.md")]);
        r.last_review = "2026-01-01".into();
        let rep = evaluate(std::slice::from_ref(&r), (2026, 8, 14), 90, &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("EXPIRED")),
            "{:?}",
            rep.violations
        );

        r.last_review = "2026-08-01".into();
        let rep = evaluate(&[r], (2026, 8, 14), 90, &w);
        assert!(rep.violations.is_empty(), "{:?}", rep.violations);
    }

    #[test]
    fn a_dangling_test_ref_is_red_and_a_resolving_one_is_not() {
        let commit = |_: &str| Ok(true);
        let present = |_: &str| Some("#[test]\nfn real_one() {}\n".to_string());
        let w = world_of(&present, &commit);
        let good = row("test-backed", vec![("test", "t/a.rs::real_one")]);
        assert!(evaluate(&[good], (2026, 8, 14), 90, &w)
            .violations
            .is_empty());

        let bad = row("test-backed", vec![("test", "t/a.rs::imaginary_one")]);
        let rep = evaluate(&[bad], (2026, 8, 14), 90, &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("imaginary_one")),
            "{:?}",
            rep.violations
        );

        let missing = |_: &str| None;
        let w2 = world_of(&missing, &commit);
        let gone = row("test-backed", vec![("test", "t/a.rs::real_one")]);
        let rep = evaluate(&[gone], (2026, 8, 14), 90, &w2);
        assert!(
            rep.violations.iter().any(|v| v.contains("readable file")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn a_level_whose_obligation_is_unmet_is_red() {
        let read = |_: &str| Some("fn t() {}".to_string());
        let commit = |_: &str| Ok(true);
        let w = world_of(&read, &commit);
        // gate-wired with no test reference at all.
        let r = row("gate-wired", vec![("contract", "scripts/check.sh")]);
        let rep = evaluate(&[r], (2026, 8, 14), 90, &w);
        assert!(
            rep.violations
                .iter()
                .any(|v| v.contains("`test` reference")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn a_published_wired_cell_over_a_weak_level_is_red_and_drift_is_named() {
        let table = "| Layer | Applies? |\n|---|---|\n| **L2 SLO-as-code** | **YES · wired** |\n";
        let read = move |p: &str| {
            if p == "CHARTER.md" {
                Some(table.to_string())
            } else {
                Some("fn t() {}".to_string())
            }
        };
        let commit = |_: &str| Ok(true);
        let w = world_of(&read, &commit);

        let mut r = row("smoke-checked", vec![("contract", "slo.toml")]);
        r.charter_claim = Some(CharterClaim {
            file: "CHARTER.md".into(),
            row: "L2 SLO-as-code".into(),
            status: "YES · wired".into(),
        });
        let rep = evaluate(std::slice::from_ref(&r), (2026, 8, 14), 90, &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("gate-wired or better")
                || v.contains("needs level gate-wired")),
            "{:?}",
            rep.violations
        );

        // Known-GOOD: the same cell over a level that carries it.
        let mut ok = row(
            "gate-wired",
            vec![("test", "t/a.rs::t"), ("contract", "scripts/check.sh")],
        );
        ok.charter_claim = r.charter_claim.clone();
        assert!(evaluate(&[ok], (2026, 8, 14), 90, &w).violations.is_empty());

        // Drift: the ledger quotes a status the document no longer carries.
        let mut moved = row("gate-wired", vec![("test", "t/a.rs::t"), ("contract", "c")]);
        moved.charter_claim = Some(CharterClaim {
            file: "CHARTER.md".into(),
            row: "L2 SLO-as-code".into(),
            status: "NO".into(),
        });
        let rep = evaluate(&[moved], (2026, 8, 14), 90, &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("DRIFT")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn a_charter_file_with_no_table_is_an_error_not_a_pass() {
        let read = |p: &str| {
            if p == "CHARTER.md" {
                Some("no tables here\n".to_string())
            } else {
                Some("fn t() {}".to_string())
            }
        };
        let commit = |_: &str| Ok(true);
        let w = world_of(&read, &commit);
        let mut r = row("absent", vec![("contract", "a.md")]);
        r.charter_claim = Some(CharterClaim {
            file: "CHARTER.md".into(),
            row: "L6 Formal".into(),
            status: "NO".into(),
        });
        let rep = evaluate(&[r], (2026, 8, 14), 90, &w);
        assert!(
            rep.errors
                .iter()
                .any(|e| e.contains("zero markdown table rows")),
            "{:?}",
            rep.errors
        );
    }

    #[test]
    fn an_unnameable_commit_is_red_and_an_unusable_git_is_an_error() {
        let read = |_: &str| Some("fn t() {}".to_string());
        let absent = |_: &str| Ok(false);
        let w = world_of(&read, &absent);
        let r = row("absent", vec![("contract", "a.md"), ("commit", "deadbee")]);
        let rep = evaluate(std::slice::from_ref(&r), (2026, 8, 14), 90, &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("not a commit")),
            "{:?}",
            rep.violations
        );

        let broken = |_: &str| Err("could not run git".to_string());
        let w2 = world_of(&read, &broken);
        let rep = evaluate(&[r], (2026, 8, 14), 90, &w2);
        assert!(
            !rep.errors.is_empty(),
            "an unevaluated leg must be an ERROR"
        );
    }
}
