//! bd-lt7 / bd-ggs7 — gates must not encode a known defect as an invariant.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Three things are held here, and nothing more:
//!
//!   1. `scripts/verify_coverage.py`, `scripts/build_units.py`,
//!      `scripts/smoke_feedback_links.py` and `scripts/smoke_weak_links.py`
//!      derive their module set from a registry (`knowledge/domains.toml`,
//!      `web/data/modules_index.json`, and `knowledge/domains.toml` twice more)
//!      rather than from a frozen literal, and each still trips on a real
//!      defect after the rebase.
//!   2. None of them refuses legitimate work — every rebased gate has a
//!      known-GOOD leg here, because an attack-only suite ships an over-strict
//!      gate, and over-strict gates get routed around.
//!   3. Every hand-frozen module fact left in the scanned trees is INVENTORIED
//!      below with a verdict. A match that is not in the inventory fails this
//!      test with its file, line and text; an inventory row whose line no
//!      longer exists fails it too. The inventory is the "justified in place"
//!      half of bd-lt7's acceptance: a literal may stay, but only if someone
//!      wrote down why.
//!
//! # THE CLASS THIS EXISTS FOR
//!
//! Module 15 was assessed but never taught. Three gates had, over time, written
//! that defect down as a RULE — the hub must NOT link to module 15, module > 14
//! needs no Learn page, module 15 keys are "unexpected" — so the correct fix
//! failed three gates for being correct. The defect was never "someone
//! hardcoded 14"; it was that the assertion came from OBSERVED STATE instead of
//! from a stated contract. A gate rebased onto a registry can still be wrong,
//! but it is wrong in a way the registry can correct.
//!
//! # WHAT THE COUNT COVERS — TWO SHAPES, AND WHY THERE ARE TWO (bd-ggs7)
//!
//! Until 2026-08-14 this sweep saw ONE shape: a numeric comparison or range
//! against 13–16. It reported "0 open instances" while
//! `scripts/smoke_weak_links.py` sat in the tree holding a hand-written
//! `{1: "01-…", … 15: "15-…"}` module→slug table. Same defect class — state
//! observed once and frozen into a literal, so the gate encodes the tree's
//! current shape as a requirement — wearing a form with no numeric bound in it
//! at all. A detector keyed on syntax finds instances that share syntax, not
//! instances that share the defect, and "0 open" meant only "no numeric bound
//! outside the inventory".
//!
//! So `Shape` is now explicit and the sweep runs two detectors:
//!
//!   - `Shape::NumericBound` — a comparison or range against 13–16.
//!   - `Shape::FrozenTable` — four or more CONSECUTIVE module ids enumerated as
//!     literals in one file, in either spelling the tree uses (`"06-power"`, or
//!     a numeric key mapping to a quoted string). Four is the threshold and it
//!     is deliberate: a worked example needs at most three modules (a first, a
//!     middle, an edge), so a fourth consecutive id means the file is
//!     transcribing the module sequence rather than illustrating it. Measured
//!     2026-08-14 across `scripts/`, `crates/` and `web/assets/js/`: every
//!     fixture in the tree stops at a run of three; the only runs of four or
//!     more were the two real module tables.
//!
//! # WHAT THIS TEST STILL CANNOT DECIDE
//!
//! The inventory is a text scan. It cannot tell a live bound from one quoted in
//! a docstring — it only insists that every match was looked at once and given
//! a verdict — and it will not see a module bound spelled some way the patterns
//! below do not match (a named constant, a value read from config, arithmetic,
//! a table split across four files with three modules each). It says nothing
//! about whether the registries themselves are right: if `domains.toml` omits a
//! module the course teaches, every gate downstream of it is confidently wrong
//! together, and no assertion here would notice.
//!
//! The scanned set is `scripts/`, `crates/` and `web/assets/js/` — named in
//! `SCANNED`, each with its own anti-vacuous file floor, so that a scan which
//! lost an entire tree cannot report like one that read it and found it clean.
//! Everything outside those three is unscanned and this file's count says
//! nothing about it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

#[derive(Debug)]
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn capture(cmd: &mut Command) -> Run {
    let out = cmd.output().unwrap_or_else(|e| panic!("spawn failed: {e}"));
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

// ── 1. the inventory of hand-frozen module facts ───────────────────────────

/// Which detector found the line. Carried on every inventory row so that a
/// detector which stops matching ANYTHING is caught: a dead detector reports
/// exactly like a clean tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// A comparison or range against 13–16.
    NumericBound,
    /// Four or more consecutive module ids enumerated as literals in one file.
    FrozenTable,
}

impl Shape {
    fn name(self) -> &'static str {
        match self {
            Shape::NumericBound => "numeric-bound",
            Shape::FrozenTable => "frozen-module-table",
        }
    }
}

/// Why a matched line is allowed to stay in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// Not a bound in effect: prose, a docstring, or a comment recording the
    /// history of a bound that has already been removed.
    Prose,
    /// A live comparison that is not a module bound at all — a report
    /// truncation, a loop counter, a term count, a float exponent.
    NotAModuleBound,
    /// A live module bound that is deliberately kept, with the reason.
    Justified,
    /// A live module bound that is a genuine bd-lt7 instance in a file this
    /// bead's owner does not own. Recorded so the sweep is honest about what it
    /// found rather than reporting a clean tree.
    OpenInstanceNotMine,
}

/// `(file, exact trimmed source line, verdict, reason)`.
///
/// Keyed by text and not by line number so that moving code does not fail this
/// test, while introducing a NEW bound anywhere does.
const INVENTORY: &[(&str, &str, Shape, Verdict, &str)] = &[
    // ── the two gates rebased by bd-lt7: only history remains ──────────────
    (
        "scripts/verify_coverage.py",
        "## Why the derivation, and not `range(1, 15)` (bd-lt7)",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring heading recording the bound this gate no longer has",
    ),
    (
        "scripts/verify_coverage.py",
        "Until 2026-08-14 this gate read `PRIMARY_MODULES = range(1, 15)` and module 15",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring recording the removed bound",
    ),
    // ── bd-qqwc / bd-engine-not-gate-ar39.2: three build_units.py rows
    //    RETIRED WITH THEIR FILE. The compiler is now cdcp_learn::units. ──
    // ── bd-lt7: ZERO open instances as of 2026-08-14 ──────────────────────
    //
    // All five OpenInstanceNotMine rows that lived here are gone: two agents
    // closed verify_objectives.py, smoke_feedback_links.py and
    // smoke_hub_mastery.mjs in the same wave. Every bound below is now PROSE in
    // a docstring recording what was removed and why — which is exactly the
    // shape this inventory is for, because a grep for the old literal still
    // hits and a reader must be able to tell documentation from live code.
    // (The controller grepped these five and briefly read them as unfixed.)
    (
        "scripts/verify_objectives.py",
        "## Why the derivation, and not `range(1, 15)` (bd-lt7)",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring heading recording the removed PRIMARY_MODULES literal",
    ),
    (
        "scripts/verify_objectives.py",
        "Until 2026-08-14 this gate read `PRIMARY_MODULES = range(1, 15)` and skipped",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring recording the removed literal and the domains.toml derivation",
    ),
    (
        "scripts/verify_objectives.py",
        "The old `domains_listed < 14` soft warning went with it. It was a FLOOR, so it",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring recording the removed soft warning and WHY removing it was safe: \
         it was a floor whose comparand was the same observed count, so once the \
         module set derives from domains.toml the check compares the registry \
         against itself",
    ),
    (
        "scripts/smoke_feedback_links.py",
        "`for n in range(1, 15)` report loop. The table happened to be right; the loop",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring recording the removed report-loop bound",
    ),
    // The suite that now ASSERTS the verify_objectives rebase has to name the
    // literal it protects against, so the sweep sees it too. Both rows are
    // header prose in the selftest, not a bound in effect anywhere.
    (
        "scripts/selftest_l7_objectives.sh",
        "# knowledge/domains.toml instead of `range(1, 15)`. (e) is the regression",
        Shape::NumericBound,
        Verdict::Prose,
        "selftest header naming the removed literal its case (e) guards against",
    ),
    (
        "scripts/selftest_l7_objectives.sh",
        "# THE bd-lt7 regression. Under `PRIMARY_MODULES = range(1, 15)` this exact",
        Shape::NumericBound,
        Verdict::Prose,
        "comment at case (e), recording the bound under which that exact fixture \
         tree was GREEN",
    ),
    (
        "scripts/smoke_hub_mastery.mjs",
        "* Until 2026-08-14 this gate asserted `MODULE_CATALOG.length === 14` and swept",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring recording the removed catalog-length bound",
    ),
    (
        "scripts/smoke_hub_mastery.mjs",
        "* modules with two `m <= 14` loops. Module 15 is assessed AND taught, so those",
        Shape::NumericBound,
        Verdict::Prose,
        "docstring recording the two removed sweep bounds",
    ),
    // ── live comparisons that are not module bounds ────────────────────────
    // scripts/build_glossary_json.py's `if len(terms) < 15:` row was DELETED on
    // 2026-08-14: a sibling rebased that floor onto `MIN_TERMS = 15`, and a
    // named constant is one of the spellings the header says this sweep cannot
    // see. The line is genuinely gone, so the row had to go with it — but the
    // FLOOR did not go anywhere, and the sweep can no longer hold a verdict on
    // it. That is the blind spot, not a clean-up; bd-8mjs tracks it. It was
    // the stricter "every row matched something" assertion below that noticed,
    // mid-wave, hours after the old count-based assertion had gone green over
    // the same tree.
    (
        "scripts/smoke_learn_v2.py",
        "if (g.get(\"term_count\") or 0) < 15:",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "glossary term-count floor",
    ),
    (
        "scripts/smoke_learn_v2.py",
        "fail(\"glossary term_count < 15\")",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "message text for the term-count floor",
    ),
    (
        "scripts/smoke_srs.mjs",
        "for (let i = 0; i < 15; i++) {",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "loop counter over a synthetic SRS queue",
    ),
    (
        "scripts/smoke_feedback_links.py",
        "if len(missing_module) > 15:",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "report truncation after 15 failure lines",
    ),
    (
        "scripts/smoke_feedback_links.py",
        "if len(unmapped_modules) > 15:",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "report truncation after 15 failure lines",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_bank.rs",
        "let body = if decpt <= -4 || decpt > 16 {",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "CPython float repr: the exponent-notation cutoff",
    ),
    (
        "crates/cdcp_gate/tests/diff_verify_content_lock.rs",
        "assert!(removed >= 15, \"emptied only {removed} files\");",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "fixture file count in an unrelated differential case",
    ),
    // ── live module bounds deliberately kept ──────────────────────────────
    (
        "crates/cdcp_gate/tests/diff_verify_knowledge_paths.rs",
        "for bound in [\"range(1, 15)\", \"range(1,15)\", \"<= 14\", \"< 15\"] {",
        Shape::NumericBound,
        Verdict::Justified,
        "a sibling gate's own detector for these bounds — the literals are the \
         thing being searched for, not a bound in effect",
    ),
    (
        "crates/cdcp_gate/src/gates/verify_knowledge_paths.rs",
        "//! bd-lt7 tracks gates that hardcode a module bound (`range(1, 15)`, `<= 14`) and",
        Shape::NumericBound,
        Verdict::Prose,
        "module header cross-referencing this bead",
    ),
    (
        "crates/cdcp_assemble/tests/learn_surface_coverage.rs",
        "compared >= 14,",
        Shape::NumericBound,
        Verdict::Justified,
        "a FLOOR, not an exclusion: ≥14 modules must be compared, which 15 \
         satisfies. It cannot hold a module out; it can only notice a collapse.",
    ),
    (
        "crates/cdcp_assemble/tests/learn_surface_coverage.rs",
        "let m15 = rows.iter().find(|r| r.order == 15);",
        Shape::NumericBound,
        Verdict::Justified,
        "the C5 decision stated as an assertion: module 15 specifically must be \
         taught because it is assessed. Naming the module IS the point here.",
    ),
    // ── bd-ggs7: the floor that replaced smoke_weak_links' frozen table ────
    (
        "crates/cdcp_learn/src/weak_links.rs",
        "} else if declared.len() < 14 {",
        Shape::NumericBound,
        Verdict::Justified,
        "a FLOOR, not an exclusion: the fourteen public EPI CDCP domains, so a \
         collapsed registry cannot make that gate agree with itself and pass \
         green. It cannot hold module 15 or any later module out. The literal \
         is spelled here rather than hidden behind a named constant precisely \
         so this sweep can see it and hold a verdict on it. Moved with \
         bd-substrate-rust-migration-jhd.16 from the deleted Python.",
    ),
    // ── bd-ggs7: web/assets/js entered the scan with this bead ─────────────
    (
        "web/assets/js/learn_chrome.js",
        "if (min < 15) min = 15;",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "a minutes floor on a reading-time estimate",
    ),
    (
        "web/assets/js/quiz.js",
        "let r = Math.imul(t ^ (t >>> 15), 1 | t);",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "mulberry32 PRNG: a bit shift, matched because `>>>15` contains `>15`",
    ),
    (
        "web/assets/js/quiz.js",
        "return ((r ^ (r >>> 14)) >>> 0) / 4294967296;",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "mulberry32 PRNG: a bit shift, matched because `>>>14` contains `>14`",
    ),
    (
        "web/assets/js/results.js",
        "return h.length > 16 ? h.slice(0, 12) + \"…\" : h;",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "hash string truncation for display",
    ),
    // ── bd-ggs7: the frozen module table the widened detector found ────────
    (
        "web/assets/js/results.js",
        "1: \"01-mission-critical\",",
        Shape::FrozenTable,
        Verdict::Justified,
        "MODULE_LEARN_SLUGS: modules 1–15 frozen as a literal, and the one place \
         in the tree where that is correct. It is the PRODUCT, not a gate — the \
         browser has no TOML reader, so the shipped map has to be a literal, and \
         a literal in the product cannot fail a correct change the way a literal \
         in a gate can. It is not allowed to rot silently either: \
         smoke_feedback_links.py and smoke_weak_links.py both compare it against \
         knowledge/domains.toml in BOTH directions — a declared module missing \
         from the map and a mapped module the registry does not declare are each \
         RED, naming the module. bd-we5a tracks generating it from the registry \
         at build time so the drift guard has nothing left to guard.",
    ),
];

/// The trees this sweep reads, their extensions, and a DELIBERATE per-tree file
/// floor. Per-tree and not a single total, because a total large enough to
/// survive ordinary churn is still reachable after an entire small tree has
/// vanished from the scan — measured 2026-08-14 the three trees held 45, 55 and
/// 16 files, so a total floor of 90 would not have noticed `web/assets/js`
/// disappearing. Each floor is set a deliberate margin under today's count:
/// enough room for files to be retired, not enough to be met by a tree that is
/// missing, misspelled, or filtered out by a broken extension list.
const SCANNED: &[(&str, &[&str], usize)] = &[
    ("scripts", &["py", "mjs", "js", "sh"], 40),
    ("crates", &["rs"], 45),
    ("web/assets/js", &["js", "mjs"], 12),
];

/// Four consecutive module ids in one file is a transcription of the module
/// sequence; three is a worked example. See the module header for the argument.
const TABLE_RUN: usize = 4;

#[derive(Debug, Clone)]
struct Hit {
    rel: String,
    line: usize,
    text: String,
    shape: Shape,
    detail: String,
}

/// The bound shapes this sweep can see. Deliberately narrow: comparisons and
/// ranges against 13–16, which is where a "modules 1..14" assumption lands.
fn bound_hits(line: &str) -> bool {
    let l = line.replace(' ', "");
    for n in ["13", "14", "15", "16"] {
        for op in ["<=", ">=", "===", "!==", "==", "!=", "<", ">"] {
            if let Some(rest) = find_after(&l, &format!("{op}{n}")) {
                if !rest.starts_with(|c: char| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
        if l.contains(&format!("range(1,{n})")) {
            return true;
        }
    }
    false
}

fn find_after<'a>(hay: &'a str, needle: &str) -> Option<&'a str> {
    hay.find(needle).map(|i| &hay[i + needle.len()..])
}

// ── the second detector: hand-frozen module TABLES (bd-ggs7) ───────────────
//
// A frozen module table has to spell the module ids out; that is the only thing
// every spelling of it has in common. Two spellings occur in this tree:
//
//   "06-power"                 a module-id string literal
//   6: "06-power"   6 => "…"   a numeric key mapping to a quoted string
//
// The second is kept because a table could name modules only by number
// (`{1: pageOne, 2: pageTwo, …}`), and the first because a table could be a
// bare list of slugs with no keys at all.

/// The module number a string literal names, if it is a module id: one or two
/// digits, a hyphen, then a lowercase slug. `"m02-standards"` is a bank item id
/// and not a module id; `"26-08-13"` is a date.
fn module_id_from_literal(s: &str) -> Option<u32> {
    let b = s.as_bytes();
    let digits = b.iter().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || digits > 2 || b.get(digits) != Some(&b'-') {
        return None;
    }
    let rest = &b[digits + 1..];
    if !matches!(rest.first(), Some(c) if c.is_ascii_lowercase()) {
        return None;
    }
    if !rest
        .iter()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
    {
        return None;
    }
    s[..digits].parse().ok()
}

/// Every quoted run on a line. Quotes are ASCII, so the byte offsets are char
/// boundaries even when the contents are not.
fn quoted_spans(line: &str) -> Vec<&str> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let q = b[i];
        if q == b'"' || q == b'\'' {
            let start = i + 1;
            let mut j = start;
            while j < b.len() && b[j] != q {
                j += 1;
            }
            if j >= b.len() {
                break;
            }
            out.push(&line[start..j]);
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// A leading `N:` / `"N":` / `N =>` whose value is a quoted string.
fn numeric_key_to_string(line: &str) -> Option<u32> {
    let t = line.trim_start();
    let b = t.as_bytes();
    let open = matches!(b.first(), Some(b'"') | Some(b'\''));
    let ds = usize::from(open);
    let mut i = ds;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == ds || i - ds > 2 {
        return None;
    }
    let n: u32 = t[ds..i].parse().ok()?;
    if open {
        if b.get(i) != Some(&b[0]) {
            return None;
        }
        i += 1;
    }
    let skip_ws = |b: &[u8], mut i: usize| {
        while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
            i += 1;
        }
        i
    };
    i = skip_ws(b, i);
    if b.get(i) == Some(&b':') {
        i += 1;
    } else if b.get(i) == Some(&b'=') && b.get(i + 1) == Some(&b'>') {
        i += 2;
    } else {
        return None;
    }
    i = skip_ws(b, i);
    matches!(b.get(i), Some(b'"') | Some(b'\'')).then_some(n)
}

/// Every module id named on a single line, in either spelling.
fn module_ids_on_line(line: &str) -> Vec<u32> {
    let mut out: Vec<u32> = quoted_spans(line)
        .into_iter()
        .filter_map(module_id_from_literal)
        .collect();
    if let Some(n) = numeric_key_to_string(line) {
        out.push(n);
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// The longest run of CONSECUTIVE module ids enumerated in one file, anchored at
/// the line where the run's first member appears. `None` below `TABLE_RUN`.
fn frozen_table_run(text: &str) -> Option<(usize, String, usize, u32, u32)> {
    let mut first_seen: std::collections::BTreeMap<u32, (usize, String)> =
        std::collections::BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        for n in module_ids_on_line(line) {
            first_seen
                .entry(n)
                .or_insert_with(|| (i + 1, line.trim().to_string()));
        }
    }
    let ids: Vec<u32> = first_seen.keys().copied().collect();
    let (mut best_start, mut best_len) = (0usize, 0usize);
    let (mut cur_start, mut cur_len) = (0usize, 0usize);
    for (k, id) in ids.iter().enumerate() {
        if k > 0 && *id == ids[k - 1] + 1 {
            cur_len += 1;
        } else {
            cur_start = k;
            cur_len = 1;
        }
        if cur_len > best_len {
            best_len = cur_len;
            best_start = cur_start;
        }
    }
    if best_len < TABLE_RUN {
        return None;
    }
    let first = ids[best_start];
    let last = ids[best_start + best_len - 1];
    let (line, text) = first_seen[&first].clone();
    Some((line, text, best_len, first, last))
}

/// Run both detectors over one root. Returns the per-tree file counts in
/// `SCANNED` order alongside every hit, so the caller decides which floors to
/// enforce — the live tree enforces them, a fixture cannot.
fn sweep(root: &Path) -> (Vec<usize>, Vec<Hit>) {
    let mut counts = Vec::new();
    let mut hits: Vec<Hit> = Vec::new();
    for (dir, exts, _) in SCANNED {
        let mut files = Vec::new();
        scan_files(&root.join(dir), exts, &mut files);
        // This test file's own INVENTORY quotes every match in the tree, so it
        // would match itself on every row. It is the ledger, not a subject.
        files.retain(|p| !p.ends_with("rebase_module_bounds.rs"));
        files.sort();
        counts.push(files.len());
        for f in &files {
            let rel = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .into_owned();
            let Ok(text) = std::fs::read_to_string(f) else {
                continue;
            };
            for (i, line) in text.lines().enumerate() {
                if bound_hits(line) {
                    hits.push(Hit {
                        rel: rel.clone(),
                        line: i + 1,
                        text: line.trim().to_string(),
                        shape: Shape::NumericBound,
                        detail: String::new(),
                    });
                }
            }
            if let Some((line, text, len, first, last)) = frozen_table_run(&text) {
                hits.push(Hit {
                    rel: rel.clone(),
                    line,
                    text,
                    shape: Shape::FrozenTable,
                    detail: format!(" [{len} consecutive module ids, {first}..{last}]"),
                });
            }
        }
    }
    (counts, hits)
}

fn scan_files(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned());
        if p.is_dir() {
            if matches!(name.as_deref(), Some("target") | Some("__pycache__")) {
                continue;
            }
            scan_files(&p, exts, out);
        } else if p
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| exts.contains(&e))
            .unwrap_or(false)
        {
            out.push(p);
        }
    }
}

#[test]
fn every_hand_frozen_module_bound_or_table_in_the_tree_is_inventoried() {
    let root = engine_root();
    let (counts, hits) = sweep(&root);

    // Anti-vacuous: an empty scan set is an ERROR. A sweep that read nothing
    // reports exactly like one that read everything and found it clean. The
    // floors are per-tree and deliberate — see SCANNED.
    for (i, (dir, _, floor)) in SCANNED.iter().enumerate() {
        assert!(
            counts[i] >= *floor,
            "scanned only {} files under {dir}/ (floor {floor}) — a vacuous sweep \
             is an ERROR, not a pass",
            counts[i]
        );
    }

    let mut unexpected: Vec<String> = Vec::new();
    let mut matched = vec![0usize; INVENTORY.len()];
    for h in &hits {
        match INVENTORY.iter().position(|(file, src, shape, _, _)| {
            *file == h.rel && *src == h.text && *shape == h.shape
        }) {
            Some(idx) => matched[idx] += 1,
            None => unexpected.push(format!(
                "{}:{}: [{}] {}{}",
                h.rel,
                h.line,
                h.shape.name(),
                h.text,
                h.detail
            )),
        }
    }

    assert!(
        unexpected.is_empty(),
        "{} hand-frozen module fact(s) are not in the INVENTORY in this file. Add \
         a row with a shape, a verdict and a reason, or derive the value from a \
         registry:\n  {}",
        unexpected.len(),
        unexpected.join("\n  ")
    );

    // Anti-vacuous from the other side, and stricter than a count: EVERY row
    // must have been matched by the detector it names. A row whose line no
    // longer exists is a stale ledger entry, and a shape with no live rows left
    // is a detector nobody is exercising — both report exactly like a clean
    // tree if nobody checks.
    let stale: Vec<String> = INVENTORY
        .iter()
        .zip(&matched)
        .filter(|(_, n)| **n == 0)
        .map(|((file, src, shape, _, _), _)| format!("{file}: [{}] {src}", shape.name()))
        .collect();
    assert!(
        stale.is_empty(),
        "{} INVENTORY row(s) matched nothing — the detector or the inventory has \
         drifted. Delete the row deliberately if the line is gone:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}

/// The sweep found open instances, and says so out loud rather than reporting a
/// clean tree. Reducing this list is the follow-on work; it may never grow
/// silently.
#[test]
fn the_open_instances_are_named_and_counted() {
    let open: Vec<&str> = INVENTORY
        .iter()
        .filter(|(_, _, _, v, _)| *v == Verdict::OpenInstanceNotMine)
        .map(|(f, _, _, _, _)| *f)
        .collect();
    assert_eq!(
        open.len(),
        0,
        "open bd-lt7/bd-ggs7 instances changed: {open:?} — update this count \
         deliberately. It reached 0 on 2026-08-14 for numeric bounds and stayed \
         at 0 on 2026-08-14 when the frozen-table detector was added and its two \
         finds were adjudicated (smoke_weak_links.py derived; results.js \
         justified as product with a bidirectional drift guard). The count now \
         covers BOTH shapes in SCANNED; a NEW hardcoded module bound or module \
         table there must be inventoried with a verdict, not silently added \
         here."
    );
}

// ── 1b. the detectors, proven to trip on planted input ─────────────────────
//
// The live tree is not allowed to be the only proof that these fire. If the
// tree were ever clean of both shapes, every assertion above would pass while
// the detectors sat dead — the exact failure this file exists to prevent. These
// four legs feed the detectors known-bad and known-good text directly.

#[test]
fn the_table_detector_sees_a_planted_frozen_table() {
    let planted = "MODULES = {\n  1: \"01-alpha\",\n  2: \"02-beta\",\n  \
                   3: \"03-gamma\",\n  4: \"04-delta\",\n}\n";
    let (line, text, len, first, last) =
        frozen_table_run(planted).expect("a four-module frozen table must be seen");
    assert_eq!((len, first, last), (4, 1, 4));
    assert_eq!(line, 2, "the hit must anchor at the first row of the run");
    assert_eq!(text, "1: \"01-alpha\",");
}

#[test]
fn the_table_detector_sees_a_bare_list_of_slugs_with_no_keys() {
    let planted = "SLUGS = [\n  \"06-power\",\n  \"07-emf\",\n  \"08-racks\",\n  \
                   \"09-cooling\",\n]\n";
    let (_, _, len, first, last) =
        frozen_table_run(planted).expect("a keyless slug list is still a frozen table");
    assert_eq!((len, first, last), (4, 6, 9));
}

#[test]
fn the_table_detector_does_not_fire_on_a_worked_example() {
    // Three modules is an example: a first, a middle, an edge. Four is a
    // transcription. This is the whole content of TABLE_RUN, asserted.
    let example = "# e.g. \"01-mission-critical\", \"02-standards\", \"03-site-building\"\n";
    assert!(frozen_table_run(example).is_none());
    // Non-consecutive ids are not a sequence being transcribed, however many.
    let scattered =
        "\"01-a\", \"06-power\", \"09-cooling\", \"14-auxiliary\", \"15-ops-adjacent\"\n";
    assert!(frozen_table_run(scattered).is_none());
}

#[test]
fn the_table_detector_does_not_confuse_ids_that_merely_start_with_digits() {
    // Bank item ids, dates, and numeric keys whose value is not a string are
    // the three near-misses that live in this tree today.
    let near = "\"m02-standards\", \"m03-x\", \"m04-y\", \"m05-z\"\n\"26-08-13\"\n\
                0 => ChoiceLetter::A,\n1 => ChoiceLetter::B,\n2 => ChoiceLetter::C,\n\
                3 => ChoiceLetter::D,\n";
    assert!(
        frozen_table_run(near).is_none(),
        "bank ids, dates and non-string match arms are not module tables"
    );
}

#[test]
fn the_numeric_bound_detector_still_trips_and_still_discriminates() {
    assert!(bound_hits("PRIMARY_MODULES = range(1, 15)"));
    assert!(bound_hits("if int(m[:2]) <= 14:"));
    assert!(bound_hits("MODULE_CATALOG.length === 14"));
    // A longer number that merely starts with one of the four is not a hit.
    assert!(!bound_hits("if x > 150:"));
    assert!(!bound_hits("total < 1400"));
}

// ── 2. verify_coverage.py: known-bad and known-GOOD ───────────────────────

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        Fixture { _dir: dir, root }
    }
    fn write(&self, rel: &str, body: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }
}

/// One bank item, as a single-item TOML file.
fn item(id: &str, module: u32, topic: &str) -> String {
    format!(
        "id = {id:?}\nmodule = {module}\nstem = \"stem for {id}\"\n\
         choices = [\"alpha\", \"beta\", \"gamma\", \"delta\"]\ncorrect = \"A\"\n\
         explanation = \"an explanation of sufficient length\"\ntopic_ids = [{topic:?}]\n\
         bloom = \"apply\"\nsource_class = \"original\"\n\
         quantity_evidence = \"qualitative_only\"\nstatus = \"approved\"\n"
    )
}

fn domains_registry(orders: &[u32]) -> String {
    let mut s = String::from("schema_version = 1\n");
    for o in orders {
        s.push_str(&format!(
            "\n[[domain]]\nid = \"{o:02}-fixture\"\norder = {o}\n\
             epi_heading = \"Fixture domain {o}\"\n"
        ));
    }
    s
}

fn coverage(f: &Fixture, bank: &str, domains: &str, policy: Option<&str>) -> Run {
    let root = engine_root();
    let mut cmd = Command::new("python3");
    cmd.arg(root.join("scripts/verify_coverage.py"))
        .arg("--bank")
        .arg(f.path(bank))
        .arg("--domains")
        .arg(f.path(domains));
    if let Some(p) = policy {
        cmd.arg("--policy").arg(f.path(p));
    }
    capture(&mut cmd)
}

#[test]
fn python3_is_present_because_a_skipped_leg_is_a_fooled_certificate() {
    let out = Command::new("python3")
        .arg("--version")
        .output()
        .expect("python3 must be installed: these legs cannot be skipped");
    assert!(out.status.success());
}

/// Known-GOOD. The live tree passes, and module 15 is now INSIDE the required
/// set rather than listed as an optional extra.
#[test]
fn verify_coverage_known_good_the_live_tree_passes_with_module_15_required() {
    let root = engine_root();
    let run = capture(Command::new("python3").arg(root.join("scripts/verify_coverage.py")));
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("modules (15 required, derived from the domain registry)"),
        "the required set must be derived, and must include module 15:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("    m15: "),
        "module 15 must be listed among the required modules:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("(optional)"),
        "module 15 must no longer be reported as an optional extra:\n{}",
        run.stdout
    );
}

/// Known-BAD, the bd-lt7 regression itself. A registry that declares module 15
/// while the bank holds nothing for it must go RED. Under `range(1, 15)` this
/// tree was GREEN.
#[test]
fn verify_coverage_known_bad_a_declared_module_with_no_items_trips() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1, 2, 15]));
    for m in [1u32, 2] {
        f.write(
            &format!("bank/m{m:02}.toml"),
            &item(&format!("i-{m}"), m, "t"),
        );
    }
    let run = coverage(&f, "bank", "domains.toml", None);
    assert_ne!(
        run.code, 0,
        "a starved declared module passed:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("module 15: 0 items < min 1"),
        "the finding must name the module:\n{}",
        run.stdout
    );
}

/// Known-BAD. Anti-vacuous: a registry that declares nothing is an ERROR, not a
/// green run over an empty required set.
#[test]
fn verify_coverage_known_bad_an_empty_registry_is_an_error() {
    let f = Fixture::new();
    f.write("domains.toml", "schema_version = 1\n");
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", None);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("domain registry declares zero modules (vacuous coverage is ERROR)"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("zero required modules after exemptions"),
        "{}",
        run.stdout
    );
}

/// Known-BAD. A missing registry is an ERROR, not a silent skip.
#[test]
fn verify_coverage_known_bad_a_missing_registry_is_an_error() {
    let f = Fixture::new();
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", None);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains("domain registry missing"),
        "{}",
        run.stdout
    );
}

/// Known-BAD. The escape hatch may not be quieter than the rule: an exemption
/// without a reason is a schema error, not an exemption.
#[test]
fn verify_coverage_known_bad_an_exemption_without_a_reason_is_an_error() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1, 15]));
    f.write("policy.toml", "[[coverage_exempt]]\nmodule = 15\n");
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", Some("policy.toml"));
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "coverage_exempt module 15 has no reason (an exemption without a reason is a \
             schema error)"
        ),
        "{}",
        run.stdout
    );
    // And the module stays required, so the shortfall is still reported.
    assert!(
        run.stdout.contains("module 15: 0 items < min 1"),
        "a rejected exemption must not silently hold the module out:\n{}",
        run.stdout
    );
}

/// Known-GOOD, the escape hatch working. A RECORDED exemption with a reason
/// holds a module out of the floor and is printed, so the hole is visible.
#[test]
fn verify_coverage_known_good_a_recorded_exemption_with_a_reason_is_honoured() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1, 15]));
    f.write(
        "policy.toml",
        "[[coverage_exempt]]\nmodule = 15\nreason = \"fixture: not yet authored\"\n",
    );
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", Some("policy.toml"));
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("m15: 0 — exempt: fixture: not yet authored"),
        "an exemption must be printed, not silent:\n{}",
        run.stdout
    );
}

/// Known-BAD. The two registries disagreeing about which modules exist is the
/// drift that produced this bead in the first place.
#[test]
fn verify_coverage_known_bad_a_floor_for_an_undeclared_module_is_drift() {
    let f = Fixture::new();
    f.write("domains.toml", &domains_registry(&[1]));
    f.write(
        "policy.toml",
        "[[domain_min]]\nmodule = 1\nmin_items = 1\n\
         [[domain_min]]\nmodule = 15\nmin_items = 16\n",
    );
    f.write("bank/m01.toml", &item("i-1", 1, "t"));
    let run = coverage(&f, "bank", "domains.toml", Some("policy.toml"));
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("[[domain_min]] module 15 is not a required module"),
        "{}",
        run.stdout
    );
}

// ── 3. build_units.py: known-bad and known-GOOD ───────────────────────────

/// A markdown module with `n` `##` units, each long enough to survive the
/// 40-word floor in `split_h2_units`.
fn module_markdown(title: &str, n: usize) -> String {
    // `split_h2_units` drops any section under forty words, so the filler has
    // to clear that floor or the fixture silently builds zero units and every
    // leg below would be measuring an empty tree instead of the rule.
    let filler = "This unit describes the fixture content in enough words that the \
                  unit survives the forty word floor applied by the splitter, which \
                  drops any section shorter than that unless it names objectives. \
                  The prose here carries no meaning of its own and exists only to \
                  push the counted word total past the threshold that the splitter \
                  applies to every section it emits, so that the fixture measures \
                  the check floor rather than the word floor."
        .to_string();
    let mut s = format!("# {title}\n\n");
    for i in 1..=n {
        s.push_str(&format!("## Unit {i} of {title}\n\n{filler}\n\n"));
    }
    s
}

/// A tree `cdcp_learn::units` can run in: a Learn index, module markdown, and a bank.
/// The Python oracle is gone (bd-qqwc / bd-engine-not-gate-ar39.2).
fn units_fixture(modules: &[(&str, usize)], bank_modules: &[u32]) -> Fixture {
    let f = Fixture::new();
    f.write("knowledge/topics.toml", "# no topics in this fixture\n");

    let rows: Vec<String> = modules
        .iter()
        .enumerate()
        .map(|(i, (id, _))| format!("{{\"id\": {id:?}, \"order\": {}, \"empty\": false}}", i + 1))
        .collect();
    f.write(
        "web/data/modules_index.json",
        &format!("{{\"modules\": [{}]}}\n", rows.join(", ")),
    );
    for (id, units) in modules {
        f.write(
            &format!("web/content/modules/{id}.md"),
            &module_markdown(id, *units),
        );
    }
    for m in bank_modules {
        for i in 0..6 {
            f.write(
                &format!("bank/items/m{m:02}-{i}.toml"),
                &item(&format!("i-{m:02}-{i}"), *m, "t"),
            );
        }
    }
    f
}

fn build_units(f: &Fixture) -> Run {
    let outcome = cdcp_learn::units::write_units(&f.path("")).expect("write_units");
    Run {
        code: outcome.code,
        stdout: outcome.stdout,
        stderr: String::new(),
    }
}

/// Known-GOOD. A copy of the live inputs passes, and its check floor covers
/// all 15 modules rather than the 14 that `int(m[:2]) <= 14` admitted.
/// NEVER run the builder against the live tree (it mutates units_index.json).
#[test]
fn build_units_known_good_the_live_tree_passes_over_all_declared_modules() {
    let root = engine_root();
    let f = Fixture::new();
    let copy = |rel: &str| {
        let dst = f.path(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(root.join(rel), &dst)
            .unwrap_or_else(|e| panic!("copy {rel}: {e}"));
    };
    copy("knowledge/topics.toml");
    copy("web/data/modules_index.json");
    copy("web/data/bank_items_seed42.json");
    std::fs::create_dir_all(f.path("web/content/modules")).unwrap();
    for e in std::fs::read_dir(root.join("web/content/modules"))
        .unwrap()
        .flatten()
    {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("md") {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            std::fs::copy(&p, f.path(&format!("web/content/modules/{name}"))).unwrap();
        }
    }
    let run = build_units(&f);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.starts_with("PASS: build_units"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("15-ops-adjacent"),
        "module 15 must be inside the check floor, not exempt from it:\n{}",
        run.stdout
    );
}

/// Known-GOOD, on a fixture: a tree where every module has bank items passes.
#[test]
fn build_units_known_good_a_well_stocked_fixture_passes() {
    let f = units_fixture(
        &[("01-mission-critical", 4), ("06-power", 3), ("15-ops", 3)],
        &[1, 6, 15],
    );
    let run = build_units(&f);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.starts_with("PASS: build_units"),
        "{}",
        run.stdout
    );
}

/// Known-BAD, the bd-lt7 regression itself. Module 15 in the Learn index with
/// no bank items behind it must go RED. Under `int(m[:2]) <= 14` its units were
/// excluded from the denominator and this tree was GREEN.
#[test]
fn build_units_known_bad_a_starved_module_15_now_trips_the_check_floor() {
    let f = units_fixture(
        &[("01-mission-critical", 4), ("06-power", 3), ("15-ops", 4)],
        &[1, 6], // nothing for module 15
    );
    let run = build_units(&f);
    assert_ne!(
        run.code, 0,
        "a starved module 15 passed the check floor:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("have ≥2 checks"),
        "the finding must name the check floor:\n{}",
        run.stdout
    );
    // The verdict is the head of the report, and it is not a PASS.
    assert!(
        run.stdout.starts_with("FAIL: build_units"),
        "verdict must lead the report and must be FAIL:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("PASS"),
        "no PASS may appear anywhere on a failing run:\n{}",
        run.stdout
    );
}

/// Known-BAD. Anti-vacuous: no modules at all is an ERROR, not a green run over
/// an empty set.
#[test]
fn build_units_known_bad_zero_modules_is_an_error() {
    let f = units_fixture(&[], &[1]);
    std::fs::create_dir_all(f.path("web/content/modules")).unwrap();
    let run = build_units(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("zero modules discovered (vacuous unit build is ERROR)"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: build_units"),
        "{}",
        run.stdout
    );
}

/// bd-hw3's shape rule applied to this gate: it used to print
/// "PASS: build_units …" and then emit "FAIL: …" underneath on its way to
/// returning 1. stdout and CI must never disagree.
#[test]
fn build_units_never_writes_a_verdict_it_then_contradicts() {
    let f = units_fixture(&[("01-mission-critical", 2), ("06-power", 3)], &[1, 6]);
    let run = build_units(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("01-mission-critical has 2 units, need ≥4"),
        "{}",
        run.stdout
    );
    let first = run.stdout.lines().next().unwrap_or("");
    assert!(
        first.starts_with("FAIL: build_units"),
        "the first line written was {first:?}"
    );
    assert!(run.stderr.is_empty(), "{}", run.stderr);
}

// ── 4. smoke_feedback_links.py: known-bad and known-GOOD ──────────────────
//
// This gate takes NO path flags — every input is `Path(__file__).parents[1] /
// …` — so the only way to inject a known-bad into it is to give it a whole
// tree of its own. That is what `feedback_fixture` builds. The alternative
// considered and rejected was adding `--domains` / `--root` to the script:
// the gate's Python is correct, and widening its argument surface to make it
// testable would be changing the thing under test in order to test it.
//
// The fixture also has to exist for a second reason. The script WRITES
// `web/data/topic_anchors.json` on every run; pointed at the live tree it
// would dirty the working copy, and a leg that dirties the tree cannot be a
// CI leg. Inside the fixture the write lands on the copy.

/// A tree `smoke_feedback_links.py` can run in: a copy of the script and of
/// every input it resolves off its own location. Copied, not synthesised —
/// results.js, the Learn pages and the seed42 packs are the real product
/// surfaces, and a hand-written stand-in would let this leg pass while the
/// shipped ones diverged.
fn feedback_fixture() -> Fixture {
    let root = engine_root();
    let f = Fixture::new();

    let copy = |rel: &str| {
        let dst = f.path(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(root.join(rel), &dst)
            .unwrap_or_else(|e| panic!("copy {rel} into the fixture: {e}"));
    };
    // The gate, and the module it imports at runtime to rebuild topic anchors.
    copy("scripts/smoke_feedback_links.py");
    copy("scripts/build_learn.py");
    // The registry under test, and the topic registry the anchor builder reads.
    copy("knowledge/domains.toml");
    copy("knowledge/topics.toml");
    // The product surfaces the gate checks the registry against.
    copy("web/assets/js/results.js");
    copy("web/assets/js/learn_md.js");
    copy("web/data/keys_seed42.json");
    copy("web/data/bank_items_seed42.json");
    // A pre-existing anchor map, so a fixture in which the rebuild cannot run
    // degrades to the gate's documented fallback instead of to a spurious
    // failure that would look like the injection firing.
    copy("web/data/topic_anchors.json");

    let mut copied = 0usize;
    for (dir, ext) in [("web/learn", "html"), ("web/content/modules", "md")] {
        std::fs::create_dir_all(f.path(dir)).unwrap();
        for e in std::fs::read_dir(root.join(dir)).unwrap().flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some(ext) {
                let name = p.file_name().unwrap().to_string_lossy().into_owned();
                std::fs::copy(&p, f.path(&format!("{dir}/{name}"))).unwrap();
                copied += 1;
            }
        }
    }
    // Anti-vacuous: a fixture that copied no Learn surface would make every
    // "missing learn page" finding below fire for the wrong reason.
    assert!(
        copied >= 30,
        "fixture copied only {copied} Learn/content files — a vacuous fixture \
         would make every injection below fire for the wrong reason"
    );
    f
}

fn feedback_links(f: &Fixture) -> Run {
    capture(Command::new("python3").arg(f.path("scripts/smoke_feedback_links.py")))
}

/// Rewrite the fixture's domain registry, dropping every `[[domain]]` block
/// whose `order` is in `drop`. Text surgery rather than a TOML round-trip
/// because the point is to change ONE fact and leave the file otherwise as
/// shipped.
fn drop_domains(f: &Fixture, drop: &[u32]) {
    let path = f.path("knowledge/domains.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut out = String::new();
    let mut removed = 0usize;
    for (i, chunk) in text.split("[[domain]]").enumerate() {
        if i == 0 {
            out.push_str(chunk);
            continue;
        }
        if drop
            .iter()
            .any(|o| chunk.lines().any(|l| l.trim() == format!("order = {o}")))
        {
            removed += 1;
            continue;
        }
        out.push_str("[[domain]]");
        out.push_str(chunk);
    }
    assert_eq!(
        removed,
        drop.len(),
        "the fixture registry did not contain every module this case removes — \
         the injection would not have applied"
    );
    std::fs::write(path, out).unwrap();
}

/// Known-GOOD. The shipped tree passes, and module 15 is INSIDE the swept set:
/// under `for n in range(1, 15)` the report printed M01–M14 and module 15 was
/// simply absent from the surface this gate describes.
#[test]
fn feedback_links_known_good_the_shipped_tree_passes_with_module_15_reported() {
    let f = feedback_fixture();
    let run = feedback_links(&f);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.starts_with("PASS: smoke_feedback_links"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("modules=15 (derived from knowledge/domains.toml)"),
        "the module count must be derived, and must be 15:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("M15 → learn/15-ops-adjacent.html"),
        "module 15 must appear in the report loop, not stop at M14:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("untaught_module_items=0 (must be 0)"),
        "{}",
        run.stdout
    );
}

/// Known-BAD, the bd-lt7 regression from the product side. Retire module 15
/// from the registry while `results.js` still links it and the bank still
/// assesses it: the gate must report the drift in BOTH directions — a Learn
/// link for a module nobody declares, and items on a real form whose module has
/// no Learn surface.
#[test]
fn feedback_links_known_bad_a_module_the_registry_stops_declaring_is_drift() {
    let f = feedback_fixture();
    drop_domains(&f, &[15]);
    let run = feedback_links(&f);
    assert_ne!(
        run.code, 0,
        "a Learn link for an undeclared module passed:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains(
            "module 15: results.js maps '15-ops-adjacent' but knowledge/domains.toml \
             does not declare that module"
        ),
        "the product→registry direction must name the module:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("assessed but untaught: ")
            && run
                .stdout
                .contains("module 15 is not declared in knowledge/domains.toml"),
        "an item on a real form with no Learn surface is the C5 defect and must \
         be named, never a silently skipped row:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: smoke_feedback_links"),
        "the verdict must lead the report:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("PASS"),
        "no PASS may appear anywhere on a failing run:\n{}",
        run.stdout
    );
}

/// Known-BAD, the registry→product direction. A module the registry declares
/// with no Learn surface behind it must go RED naming the module. `range(1,
/// 15)` could not have seen this for module 15 at all.
#[test]
fn feedback_links_known_bad_a_declared_module_with_no_learn_surface_trips() {
    let f = feedback_fixture();
    let path = f.path("knowledge/domains.toml");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str(
        "\n[[domain]]\nid = \"16-fixture-only\"\norder = 16\n\
         epi_heading = \"Fixture domain with no Learn surface\"\n\
         exam_weight_unknown = true\n",
    );
    std::fs::write(&path, text).unwrap();

    let run = feedback_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "module 16: results.js slug map None != '16-fixture-only' \
             (knowledge/domains.toml)"
        ),
        "the slug-map gap must name the module:\n{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("missing learn page web/learn/16-fixture-only.html"),
        "the missing Learn page must be named:\n{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("missing content web/content/modules/16-fixture-only.md"),
        "the missing content file must be named:\n{}",
        run.stdout
    );
}

/// Known-BAD. Anti-vacuous: a registry that declares nothing is an ERROR, not a
/// green run over an empty module set. This is the failure that reports exactly
/// like a clean one if nobody writes the check.
#[test]
fn feedback_links_known_bad_an_empty_registry_is_an_error() {
    let f = feedback_fixture();
    std::fs::write(f.path("knowledge/domains.toml"), "schema_version = 1\n").unwrap();
    let run = feedback_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("domain registry declares zero modules (vacuous link check is ERROR)"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: smoke_feedback_links"),
        "{}",
        run.stdout
    );
}

/// Known-GOOD, the other half of the drift rule. Retiring a module from BOTH
/// sources at once is a legitimate edit, not drift, and must stay green — an
/// attack-only suite would make the registry uneditable.
#[test]
fn feedback_links_known_good_retiring_a_module_from_both_sources_is_not_drift() {
    let f = feedback_fixture();
    // 14 is chosen over 15 only because the bank has items in both; the point
    // is that the two sources agree after the edit, whichever module it is.
    drop_domains(&f, &[14]);
    let js_path = f.path("web/assets/js/results.js");
    let js = std::fs::read_to_string(&js_path).unwrap();
    let stripped: String = js
        .lines()
        .filter(|l| !l.contains("14: '14-auxiliary'") && !l.contains("14: \"14-auxiliary\""))
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(
        stripped.len(),
        js.len(),
        "the fixture's slug map did not contain module 14 on its own line — this \
         case would otherwise assert nothing"
    );
    std::fs::write(&js_path, stripped + "\n").unwrap();

    let run = feedback_links(&f);
    // The two sources now agree about module 14, so neither drift direction
    // fires. Items still assessed in module 14 are the C5 defect and are
    // REPORTED — this leg asserts only that the drift rules stayed quiet.
    assert!(
        !run.stdout.contains("module 14: results.js maps"),
        "an agreed retirement must not be reported as product→registry drift:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("module 14: results.js slug map"),
        "an agreed retirement must not be reported as registry→product drift:\n{}",
        run.stdout
    );
}

// ── 5. smoke_weak_links.py: known-bad and known-GOOD (bd-ggs7) ────────────
//
// Same fixture argument as section 4: this gate resolves every input off
// `Path(__file__).parents[1]`, so the only honest way to inject a known-bad is
// to give it a whole tree. The Learn pages and results.js are COPIED rather
// than synthesised, because a hand-written stand-in would let this leg stay
// green while the shipped surfaces diverged.

fn weak_links_fixture() -> Fixture {
    let root = engine_root();
    let f = Fixture::new();
    let copy = |rel: &str| {
        let dst = f.path(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(root.join(rel), &dst)
            .unwrap_or_else(|e| panic!("copy {rel} into the fixture: {e}"));
    };
    copy("knowledge/domains.toml");
    copy("web/assets/js/results.js");
    copy("web/data/modules_index.json");

    let mut copied = 0usize;
    std::fs::create_dir_all(f.path("web/learn")).unwrap();
    for e in std::fs::read_dir(root.join("web/learn")).unwrap().flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("html") {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            std::fs::copy(&p, f.path(&format!("web/learn/{name}"))).unwrap();
            copied += 1;
        }
    }
    // Anti-vacuous, and the same deliberate number as the gate's own floor: the
    // fourteen public EPI domains. A fixture that copied fewer Learn pages
    // would make every "missing Learn page" finding below fire for the wrong
    // reason.
    assert!(
        copied >= 14,
        "fixture copied only {copied} Learn pages — a vacuous fixture would make \
         every injection below fire for the wrong reason"
    );
    f
}

fn weak_links(f: &Fixture) -> Run {
    let o = cdcp_learn::weak_links::run(&f.root);
    Run {
        code: o.code,
        stdout: o.stdout,
        stderr: String::new(),
    }
}

/// Drop one `MODULE_LEARN_SLUGS` row from the fixture's results.js.
fn drop_slug_row(f: &Fixture, order: u32, slug: &str) {
    let path = f.path("web/assets/js/results.js");
    let js = std::fs::read_to_string(&path).unwrap();
    let needles = [format!("{order}: \"{slug}\""), format!("{order}: '{slug}'")];
    let stripped: String = js
        .lines()
        .filter(|l| !needles.iter().any(|n| l.contains(n.as_str())))
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(
        stripped.len(),
        js.len(),
        "the fixture's slug map did not contain module {order} on its own line — \
         this case would otherwise assert nothing"
    );
    std::fs::write(&path, stripped + "\n").unwrap();
}

/// Drop one module from the fixture's Learn index, asserting it was there.
fn drop_index_module(f: &Fixture, order: u32) {
    let run = capture(
        Command::new("python3")
            .arg("-c")
            .arg(
                "import json,sys\n\
                 p, o = sys.argv[1], int(sys.argv[2])\n\
                 d = json.load(open(p))\n\
                 before = len(d['modules'])\n\
                 d['modules'] = [m for m in d['modules'] if m.get('order') != o]\n\
                 assert len(d['modules']) == before - 1, 'index lacked that module'\n\
                 json.dump(d, open(p, 'w'), indent=2)\n",
            )
            .arg(f.path("web/data/modules_index.json"))
            .arg(order.to_string()),
    );
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
}

/// Known-GOOD, the live tree. The module set is DERIVED and it is 15 — under
/// the frozen table this printed the same number from a literal, which is
/// exactly why the number alone never proved anything.
#[test]
fn weak_links_known_good_the_live_tree_passes_with_a_derived_module_set() {
    let root = engine_root();
    let o = cdcp_learn::weak_links::run(&root);
    let run = Run {
        code: o.code,
        stdout: o.stdout,
        stderr: String::new(),
    };
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("modules=15 (derived from knowledge/domains.toml)"),
        "the module set must be derived, and must be 15:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("M15 → learn/15-ops-adjacent.html"),
        "module 15 must be inside the swept set:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("PASS: smoke_weak_links"),
        "{}",
        run.stdout
    );
}

/// Known-GOOD, the leg the frozen table could not have passed. A tree that
/// legitimately teaches fourteen modules — the registry, the shipped slug map
/// and the Learn index all agreeing — is not a defect. The old
/// `EXPECTED_SLUGS` table would have failed it with "module 15: missing from
/// MODULE_LEARN_SLUGS", i.e. refused a correct change for being correct.
#[test]
fn weak_links_known_good_a_legitimate_14_module_tree_passes() {
    let f = weak_links_fixture();
    drop_domains(&f, &[15]);
    drop_slug_row(&f, 15, "15-ops-adjacent");
    drop_index_module(&f, 15);
    let run = weak_links(&f);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("modules=14 (derived from knowledge/domains.toml)"),
        "a 14-module tree must be reported as 14, not rejected:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("M15"),
        "a retired module must not be swept:\n{}",
        run.stdout
    );
}

/// Known-BAD, the registry→product direction. A module the registry declares
/// with no Learn surface behind it must go RED naming the module. The frozen
/// table could not see module 16 at all: it was simply not in `EXPECTED_SLUGS`,
/// so nothing was checked and the run was green.
#[test]
fn weak_links_known_bad_a_declared_module_with_no_learn_surface_trips() {
    let f = weak_links_fixture();
    let path = f.path("knowledge/domains.toml");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str(
        "\n[[domain]]\nid = \"16-fixture-only\"\norder = 16\n\
         epi_heading = \"Fixture domain with no Learn surface\"\n\
         exam_weight_unknown = true\n",
    );
    std::fs::write(&path, text).unwrap();

    let run = weak_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "module 16: knowledge/domains.toml declares '16-fixture-only' but \
             MODULE_LEARN_SLUGS has no entry"
        ),
        "the slug-map gap must name the module:\n{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("module 16: declared slug has no Learn page web/learn/16-fixture-only.html"),
        "the missing Learn page must be named:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: smoke_weak_links"),
        "the verdict must lead the report:\n{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("PASS"),
        "no PASS may appear anywhere on a failing run:\n{}",
        run.stdout
    );
}

/// Known-BAD, the product→registry direction. Retiring a module from the
/// registry alone, while the shipped map still links it, is drift and must be
/// named. This is the assertion that stands in for the deleted table.
#[test]
fn weak_links_known_bad_a_mapped_module_the_registry_drops_is_drift() {
    let f = weak_links_fixture();
    drop_domains(&f, &[15]);
    let run = weak_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "module 15: results.js maps '15-ops-adjacent' but knowledge/domains.toml \
             does not declare that module"
        ),
        "the drift must name the module:\n{}",
        run.stdout
    );
}

/// Known-BAD, the derivation's SOURCE broken. A registry collapsed below the
/// certification's fourteen public domains is an ERROR — without this floor the
/// gate would happily derive a three-module set, check it against a slug map it
/// then reports as full of extras, and in the degenerate case where the product
/// collapsed too, agree with itself and pass.
#[test]
fn weak_links_known_bad_a_collapsed_registry_is_an_error() {
    let f = weak_links_fixture();
    drop_domains(&f, &[13, 14, 15]);
    let run = weak_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "domain registry declares only 12 modules; the CDCP course has fourteen \
             public EPI domains at minimum"
        ),
        "the floor must name the count it saw and the number it wanted:\n{}",
        run.stdout
    );
    assert!(
        run.stdout.starts_with("FAIL: smoke_weak_links"),
        "{}",
        run.stdout
    );
}

/// Known-BAD. Anti-vacuous: a registry that declares nothing is an ERROR, not a
/// green run over an empty module set.
#[test]
fn weak_links_known_bad_an_empty_registry_is_an_error() {
    let f = weak_links_fixture();
    std::fs::write(f.path("knowledge/domains.toml"), "schema_version = 1\n").unwrap();
    let run = weak_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("domain registry declares zero modules (vacuous weak-link check is ERROR)"),
        "{}",
        run.stdout
    );
}

/// Known-BAD. A missing registry is an ERROR, not a silent skip — the failure
/// mode where a gate points at a path nobody writes any more and stays green.
#[test]
fn weak_links_known_bad_a_missing_registry_is_an_error() {
    let f = weak_links_fixture();
    std::fs::remove_file(f.path("knowledge/domains.toml")).unwrap();
    let run = weak_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains("domain registry missing"),
        "{}",
        run.stdout
    );
}

/// Known-BAD. An unparseable registry fails closed. "Could not read the
/// contract" is never "the contract is satisfied".
#[test]
fn weak_links_known_bad_an_unparseable_registry_is_an_error() {
    let f = weak_links_fixture();
    std::fs::write(
        f.path("knowledge/domains.toml"),
        "schema_version = 1\n[[domain]\nid = \"01-broken\"\n",
    )
    .unwrap();
    let run = weak_links(&f);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(
        run.stdout.contains("domain registry parse error"),
        "{}",
        run.stdout
    );
}

// ── 6. the widened sweep, proven to trip on a planted tree ────────────────

/// Known-BAD for the sweep itself: plant a hand-frozen module table of the
/// newly-visible shape into a tree and the sweep must name the FILE and the
/// LINE. Planted into a fixture rather than into `scripts/`, because a sweep
/// you can only test by dirtying the repo is a sweep nobody runs.
#[test]
fn the_sweep_goes_red_on_a_planted_frozen_table_naming_file_and_line() {
    let f = Fixture::new();
    f.write(
        "scripts/planted_frozen_table.py",
        "#!/usr/bin/env python3\n\
         \"\"\"a gate that froze the module set instead of reading the registry.\"\"\"\n\
         EXPECTED = {\n\
         \x20   1: \"01-mission-critical\",\n\
         \x20   2: \"02-standards\",\n\
         \x20   3: \"03-site-building\",\n\
         \x20   4: \"04-floor-ceiling\",\n\
         }\n",
    );
    let (counts, hits) = sweep(&f.root);
    assert_eq!(counts[0], 1, "the fixture's scripts/ tree holds one file");

    let table: Vec<&Hit> = hits
        .iter()
        .filter(|h| h.shape == Shape::FrozenTable)
        .collect();
    assert_eq!(
        table.len(),
        1,
        "the planted table must produce exactly one hit, got {table:?}"
    );
    let h = table[0];
    assert_eq!(h.rel, "scripts/planted_frozen_table.py");
    assert_eq!(h.line, 4, "the hit must name the line the run starts on");
    assert_eq!(h.text, "1: \"01-mission-critical\",");
    assert!(
        h.detail.contains("4 consecutive module ids, 1..4"),
        "the finding must say what it saw: {}",
        h.detail
    );
    // And it is NOT in the inventory, so the live assertion would fire.
    assert!(
        !INVENTORY
            .iter()
            .any(|(file, src, shape, _, _)| *file == h.rel && *src == h.text && *shape == h.shape),
        "a planted table must be un-inventoried"
    );
}

/// Known-GOOD for the sweep: an ordinary gate that DERIVES its module set is
/// not a hit, however many times it mentions a module. An over-strict sweep
/// gets routed around.
#[test]
fn the_sweep_stays_quiet_on_a_gate_that_derives_its_module_set() {
    let f = Fixture::new();
    f.write(
        "scripts/derived_gate.py",
        "#!/usr/bin/env python3\n\
         \"\"\"derives the module set from knowledge/domains.toml.\n\
         \n\
         e.g. \"01-mission-critical\" -> web/learn/01-mission-critical.html, and\n\
         \"06-power\" -> web/learn/06-power.html.\n\
         \"\"\"\n\
         declared = load_declared_modules(DOMAINS_TOML)\n\
         for n in sorted(declared):\n\
         \x20   check(n, declared[n])\n",
    );
    let (_, hits) = sweep(&f.root);
    assert!(
        hits.is_empty(),
        "a derived gate must not be a hit: {hits:?}"
    );
}

/// The anti-vacuous floors, proven to trip. Without this leg the floors in
/// `SCANNED` are numbers nobody has ever seen fail: a scan that lost a tree
/// would be caught only if the failure happened for real, in CI, once. Here a
/// root with none of the three trees is swept and every floor is shown to
/// reject it — including `web/assets/js`, the small tree a single total floor
/// could not have protected.
#[test]
fn the_per_tree_file_floors_reject_a_tree_that_vanished() {
    let f = Fixture::new();
    f.write(
        "unrelated/readme.md",
        "no scanned tree exists under this root\n",
    );
    let (counts, hits) = sweep(&f.root);
    assert!(hits.is_empty(), "nothing to find: {hits:?}");
    for (i, (dir, _, floor)) in SCANNED.iter().enumerate() {
        assert_eq!(counts[i], 0, "{dir}/ does not exist in this fixture");
        assert!(
            counts[i] < *floor,
            "the {dir}/ floor of {floor} would have accepted a vanished tree"
        );
    }
    // And the floors are floors, not ceilings: the live tree clears every one.
    let (live, _) = sweep(&engine_root());
    for (i, (dir, _, floor)) in SCANNED.iter().enumerate() {
        assert!(
            live[i] >= *floor,
            "the live {dir}/ holds {} files, under its own floor of {floor} — the \
             floor is set above the tree it is supposed to measure",
            live[i]
        );
    }
}
