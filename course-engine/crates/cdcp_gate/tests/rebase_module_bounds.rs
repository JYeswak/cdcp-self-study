//! bd-lt7 / bd-ggs7 — gates must not encode a known defect as an invariant.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Three things are held here, and nothing more:
//!
//!   1. `scripts/verify_coverage.py`, `cdcp_learn::units`,
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
//! # WHAT THE COUNT COVERS — FOUR SHAPES, AND WHY THERE ARE FOUR
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
//! So `Shape` is now explicit and the sweep runs four detectors:
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
//!   - `Shape::NamedBound` — a bare 13–16 BOUND TO A NAME, and the whole line
//!     is that binding: `MIN_TERMS = 15`, `pub const MAX: usize = 14;`. Added
//!     for bd-8mjs, below.
//!
//!   - `Shape::ComputedBound` — a module-ceiling name bound to something other
//!     than a bare integer (`MAX_MODULE = len(x) - 1`,
//!     `MAX_MODULE = os.environ.get(...)`, `MAX_MODULE = cfg["max"]`), or a
//!     Python `range(1, NAME)` over a simple identifier. Added for bd-ob8i.
//!     The name test is the narrowness: `last = xs.len() - 1` is last-index
//!     arithmetic, not a module bound, and this tree is full of it.
//!
//! # THE THIRD SHAPE, AND WHY IT IS A DETECTOR AND NOT A DISCLAIMER (bd-8mjs)
//!
//! On 2026-08-14 a sibling rebased `scripts/build_glossary_json.py`'s floor
//! from `if len(terms) < 15:` onto `MIN_TERMS = 15`. The floor did not move.
//! The LITERAL moved behind a named constant, the `NumericBound` row stopped
//! matching, and the row was deleted — the ledger shrank while nothing was
//! fixed. Harmless for a glossary term count; not harmless as a mechanism,
//! because it means any live module bound can be made invisible to this sweep
//! by naming it, and the open count would fall on its own.
//!
//! The choice was between DETECTING the shape and NARROWING the claim in
//! writing at the count. The detector won on one measured fact: widening
//! surfaced `pub const MIN_TERMS: usize = 15;` in the Rust port of the same
//! gate, and `const REPR_FIXED_MAX: i32 = 16;` in `validate_grounding.rs` —
//! neither of which anyone had in mind when the blind spot was written up. A
//! narrowed CLAIM would have been accurate about the line that prompted it and
//! silent about the two it did not know existed. A written narrowing describes
//! the hole; only a detector finds what is in it.
//!
//! The cost is the over-match risk, and it is the real one: this tree is full
//! of legitimate 12–16 constants (a 15-minute reading-time floor, a 15-term
//! glossary floor, `>>> 15` in a mulberry32 PRNG, a 16-character hash
//! truncation, `default=15` on an argparse flag, `module = 15` inside dozens of
//! fixture strings). An over-strict gate gets routed around, which is a slower
//! death than no gate. The whole-line-is-the-binding rule is what buys the
//! narrowness, every excluded shape above has a known-GOOD leg, and the
//! measured result over the three scanned trees is three hits, all three
//! inventoried, none of them a module bound.
//!
//! # WHAT THIS TEST STILL CANNOT DECIDE
//!
//! The inventory is a text scan. It cannot tell a live bound from one quoted in
//! a docstring — it only insists that every match was looked at once and given
//! a verdict — and it will not see a module bound spelled some way the patterns
//! below do not match. After bd-8mjs the named-constant hole is closed for the
//! form `NAME = 15`. After bd-ob8i a module-ceiling name bound to a non-literal
//! (computed, env, config, tuple) is closed, and so is `range(1, NAME)`. What
//! remains uncovered, and is stated at the count itself as well as here: a
//! one-letter or non-ceiling name bound to a computed value then compared
//! (`n = len(x) - 1`; `if m <= n` — that is data-flow), a table split across
//! four files with three modules each, and every tree outside SCANNED. It says
//! nothing about whether the registries themselves are right: if `domains.toml`
//! omits a module the course teaches, every gate downstream of it is
//! confidently wrong together, and no assertion here would notice.
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
    /// A bare integer 13–16 BOUND TO A NAME: `MIN_TERMS = 15`,
    /// `pub const MAX: usize = 14;`, `let n = 13;`.
    NamedBound,
    /// A module-ceiling name bound to a non-literal, or `range(1, NAME)`.
    ComputedBound,
}

impl Shape {
    fn name(self) -> &'static str {
        match self {
            Shape::NumericBound => "numeric-bound",
            Shape::FrozenTable => "frozen-module-table",
            Shape::NamedBound => "named-bound",
            Shape::ComputedBound => "computed-bound",
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
    // ── bd-qqwc: three build_units.py rows RETIRED WITH THEIR FILE ────────
    //
    // `scripts/build_units.py` carried three Prose instances of the bd-lt7
    // bound (`int(m[:2]) <= 14`) in its docstring and at the rebased site. The
    // oracle was retired on 2026-08-14 once `tests/diff_build_units.rs` stopped
    // being a differential, so the file those rows named no longer exists and
    // the rows went with it — a row whose file is gone fails the `stale` leg
    // below, which is exactly the mechanism working.
    //
    // THE HISTORY DID NOT GO WITH THEM. `crates/cdcp_gate/src/gates/build_units.rs`
    // carries the same finding under the heading "Why the derivation, and not a
    // two-digit ceiling (bd-lt7)". It PARAPHRASES the literal deliberately:
    // spelling the old bound out in a scanned `.rs` would trip the
    // numeric-bound detector and need a row of its own, which is ceremony for a
    // bound that no longer exists anywhere in the tree.
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
    // RETIRED 2026-08-15 with scripts/smoke_feedback_links.py (EXTRACT-THEN-DELETE
    // into cdcp_learn::feedback). The docstring that recorded the old
    // report-loop bound left with the file. The product crate's truncation
    // ceiling is outside this sweep's window, so no replacement row.
    //
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
    // ── bd-8mjs: the floor that walked out of the ledger, back under a shape
    //             that can hold a verdict on it ──────────────────────────────
    //
    // On 2026-08-14 `scripts/build_glossary_json.py` had `if len(terms) < 15:`
    // rebased onto `MIN_TERMS = 15`. The FLOOR did not move; the LITERAL moved
    // behind a name, the `NumericBound` row stopped matching, and the row was
    // deleted — a ledger shrinking with nothing fixed. The `NamedBound`
    // detector exists so that the same floor is inventoried again, in the
    // spelling it now wears, in BOTH implementations of the gate.
    (
        "crates/cdcp_learn/src/glossary.rs",
        "pub const MIN_TERMS: usize = 15;",
        Shape::NamedBound,
        Verdict::NotAModuleBound,
        "the glossary term-count floor, now in cdcp_learn (product). The Python \
         oracle and the gate copy were deleted with bd-engine-not-gate-ar39.2. \
         Not a module bound: it counts TERMS, and a floor cannot hold a module out.",
    ),
    (
        "crates/cdcp_learn/src/learn_v2.rs",
        "pub const MIN_GLOSSARY_TERMS: i64 = 15;",
        Shape::NamedBound,
        Verdict::NotAModuleBound,
        "the glossary term-count floor in cdcp_learn::learn_v2 (product). \
         EXTRACT-THEN-DELETE (d037827) landed here; there was never a cdcp_gate \
         dump to inventory. Not a module bound: it counts TERMS, and it is a \
         floor, so it cannot hold any module out.",
    ),
    // RETIRED 2026-08-15 with crates/cdcp_gate/src/gates/export_anki.rs
    // (bd-substrate-rust-migration-jhd.13 EXTRACT-THEN-DELETE). The MT19937
    // tempering constant left with the byte-exact gate port. The product
    // crate does not carry it.
    (
        "crates/cdcp_gate/src/gates/validate_grounding.rs",
        "const REPR_FIXED_MAX: i32 = 16;",
        Shape::NamedBound,
        Verdict::NotAModuleBound,
        "CPython float repr: the exponent-notation cutoff, named. The same \
         constant is inventoried unnamed as `decpt > 16` in verify_bank.rs \
         below — the pair is what a `NumericBound`-only sweep saw half of.",
    ),
    (
        "crates/cdcp_learn/src/chrome.rs",
        "pub const MIN_CHECKS: usize = 15;",
        Shape::NamedBound,
        Verdict::NotAModuleBound,
        "chrome smoke check-count floor so an emptied list cannot go green. \
         Counts CHECKS, not modules, and a floor cannot hold a module out.",
    ),
    (
        "crates/cdcp_learn/tests/stale_contract_prose.rs",
        "order = 15",
        Shape::NamedBound,
        Verdict::Prose,
        "a planted domains.toml fixture inside a raw string. named_bound cannot \
         see that the line is quoted; it is not a live module bound. Found when \
         bd-ob8i re-ran the sweep; the line has been in the tree since bd-smvb.",
    ),
    // ── live comparisons that are not module bounds ────────────────────────
    // The two NumericBound rows for learn_v2 retired when the floor moved
    // behind MIN_GLOSSARY_TERMS (bd-inventory-row-smoke-learn-v2-42hk). The
    // NamedBound row above is the live verdict. Deriving the message from
    // the constant is not evasion: NumericBound no longer has a literal to
    // match, which is the point of naming the floor.
    (
        "crates/cdcp_anki/src/lib.rs",
        "let ext_attr: u32 = 0o644 << 16;",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "ZIP central-directory external attributes: Unix mode shifted into the \
         high word. Matched because the shift contains a comparison glyph next \
         to the bit width. Not a module bound.",
    ),
    (
        "crates/cdcp_learn/tests/chrome_smoke.rs",
        "assert!(MIN_CHECKS >= 15);",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "anti-vacuous assertion on the chrome check-count floor. Not a module bound.",
    ),
    (
        "scripts/smoke_srs.mjs",
        "for (let i = 0; i < 15; i++) {",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "loop counter over a synthetic SRS queue",
    ),
    // RETIRED 2026-08-15 with scripts/smoke_feedback_links.py. The two
    // report-truncation comparisons left with the file. cdcp_learn::feedback
    // truncates via a ceiling outside this sweep's window.
    (
        "crates/cdcp_gate/src/gates/verify_bank.rs",
        "let body = if decpt <= -4 || decpt > 16 {",
        Shape::NumericBound,
        Verdict::NotAModuleBound,
        "CPython float repr: the exponent-notation cutoff",
    ),
    // RETIRED 2026-08-15 with the file (bd-retire-oracle-on-behaviour-change-gna0):
    // diff_verify_content_lock.rs (fixture `removed >= 15`). A row whose file
    // is gone fails the stale leg.
    // RETIRED 2026-08-15 with the file (bd-substrate-rust-migration-jhd.33):
    // diff_verify_knowledge_paths.rs (bound-literal detector). A row whose
    // file is gone fails the stale leg.
    // ── live module bounds deliberately kept ──────────────────────────────
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
    // RETIRED 2026-08-15 with bd-we5a: web/assets/js/results.js no longer
    // holds a hand-frozen MODULE_LEARN_SLUGS table. The map is generated
    // from knowledge/domains.toml by `cdcp build-learn-slugs` into
    // web/data/module_learn_slugs.js (outside SCANNED). A literal map
    // copied back into results.js will trip FrozenTable with no inventory
    // row and fail this sweep — that is the point of deleting the row.
];

/// The trees this sweep reads, their extensions, and a DELIBERATE per-tree file
/// floor. Per-tree and not a single total, because a total large enough to
/// survive ordinary churn is still reachable after an entire small tree has
/// vanished from the scan — measured 2026-08-14 the three trees held 45, 63 and
/// 16 files, so a total floor of 90 would not have noticed `web/assets/js`
/// disappearing. (`crates/` was 55 earlier the same day; it is the tree that
/// churns, which is exactly why its floor is not pinned to its count.) The scan
/// SET is unchanged by bd-8mjs / bd-ob8i — those beads added detectors, not a
/// fourth tree — so the floors below are re-measured, not re-derived. Each is set a
/// deliberate margin under today's count:
/// enough room for files to be retired, not enough to be met by a tree that is
/// missing, misspelled, or filtered out by a broken extension list.
const SCANNED: &[(&str, &[&str], usize)] = &[
    ("scripts", &["py", "mjs", "js", "sh"], 29),
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

// ── the third detector: a bound BOUND TO A NAME (bd-8mjs) ──────────────────
//
// `bound_hits` keys on an OPERATOR next to the number, so it sees `< 15` and is
// blind to `MIN_TERMS = 15` two lines above the comparison. That blindness is
// not theoretical: on 2026-08-14 `scripts/build_glossary_json.py` had
// `if len(terms) < 15:` rebased onto `MIN_TERMS = 15`, its INVENTORY row went
// stale, and the sweep's open count would have shrunk with nothing fixed. The
// evasion needs no intent — it is indistinguishable from ordinary refactoring
// hygiene, which is exactly why a detector has to carry it rather than a rule.
//
// THE WHOLE LINE MUST BE THE BINDING. That single constraint is what keeps this
// detector from becoming the over-strict gate that gets routed around, and it
// is worth naming what it excludes, because each of these is real and live in
// this tree today:
//
//   ap.add_argument("--sample-report", type=int, default=15)   a call argument
//   "[[domain_min]]\nmodule = 15\nmin_items = 16\n"            a fixture string
//   //! `primary_notes_checked=15`. Nothing was fixed here     a comment
//   if (min < 15) min = 15;                                    a guarded floor
//   return h.length > 16 ? h.slice(0, 12) + "…" : h;           a truncation
//   let r = Math.imul(t ^ (t >>> 15), 1 | t);                  a PRNG shift
//
// A leading comment marker (`#`, `//`, `//!`, `*`) is also not a binding: a
// commented-out constant is not live code, and reading prose as live code is
// the error this suite has now made three times.
//
// Measured 2026-08-14 across `scripts/`, `crates/` and `web/assets/js/`: this
// detector finds exactly three lines, all three inventoried below, none of them
// a module bound. That ratio is the point — a detector that fired on the six
// shapes above would be a detector nobody could keep green.

/// The bound this line binds to a name, if the line is nothing but that
/// binding. Handles the spellings this tree uses: Python `NAME = 15`, Rust
/// `pub const NAME: usize = 15;`, JS/TS `const name = 14;`, shell `NAME=13`.
fn named_bound(line: &str) -> Option<u32> {
    let t = line.trim();
    let b = t.as_bytes();
    // Comment and prose lines are not bindings.
    if t.is_empty() || t.starts_with('#') || t.starts_with("//") || t.starts_with('*') {
        return None;
    }

    let mut i = 0usize;
    // Declaration keywords, any number of them, in any of the four languages.
    loop {
        let rest = &t[i..];
        let word_len = rest
            .bytes()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'(' || *c == b')')
            .count();
        let word = &rest[..word_len];
        if matches!(
            word,
            "pub"
                | "pub(crate)"
                | "const"
                | "static"
                | "let"
                | "mut"
                | "var"
                | "final"
                | "readonly"
                | "export"
                | "declare"
                | "local"
                | "my"
        ) {
            i += word_len;
            i += t[i..].bytes().take_while(|c| *c == b' ').count();
            if i >= t.len() {
                return None;
            }
            continue;
        }
        break;
    }

    // The name. Dotted so `self.max_module = 14` is a binding too; no `(` or
    // `[`, because `f(x) = …` and `d["k"] = …` are not name bindings.
    let start = i;
    if !matches!(b.get(i), Some(c) if c.is_ascii_alphabetic() || *c == b'_') {
        return None;
    }
    while matches!(b.get(i), Some(c) if c.is_ascii_alphanumeric() || *c == b'_' || *c == b'.') {
        i += 1;
    }
    if i == start {
        return None;
    }
    let skip_ws = |i: &mut usize| {
        while matches!(b.get(*i), Some(b' ') | Some(b'\t')) {
            *i += 1;
        }
    };
    skip_ws(&mut i);

    // An optional `: TYPE` annotation (Rust, TypeScript). `::` inside a path is
    // fine; anything outside the type alphabet is not an annotation.
    if b.get(i) == Some(&b':') {
        i += 1;
        while matches!(b.get(i), Some(c)
            if c.is_ascii_alphanumeric()
                || matches!(c, b'_' | b':' | b'<' | b'>' | b',' | b' ' | b'\'' | b'&'))
        {
            i += 1;
        }
    }
    skip_ws(&mut i);

    // A plain `=`. Not `==`, not `=>`, and not the tail of `<=`/`!=`/`+=`,
    // which the name scan above would already have refused.
    if b.get(i) != Some(&b'=') {
        return None;
    }
    if matches!(b.get(i + 1), Some(b'=') | Some(b'>')) {
        return None;
    }
    i += 1;
    skip_ws(&mut i);

    // Exactly the two digits, and nothing that makes them part of a longer
    // number: `150` and `15.5` are not this bound.
    let ds = i;
    while matches!(b.get(i), Some(c) if c.is_ascii_digit()) {
        i += 1;
    }
    if i - ds != 2 || b.get(i) == Some(&b'.') {
        return None;
    }
    let n: u32 = t[ds..i].parse().ok()?;
    if !(13..=16).contains(&n) {
        return None;
    }

    // The binding must END here: end of line, a terminator, or a comment.
    let tail = t[i..].trim();
    let ends = tail.is_empty()
        || tail.starts_with(';')
        || tail.starts_with(',')
        || tail.starts_with("//")
        || tail.starts_with('#')
        || tail.starts_with("/*");
    ends.then_some(n)
}

// ── the fourth detector: a computed / external module ceiling (bd-ob8i) ──
//
// `named_bound` keys on a BARE INTEGER. `MAX_MODULE = 14` is visible;
// `MAX_MODULE = len(declared) - 1` is not, nor is `range(1, MAX)`, nor is
// `MAX_MODULE = os.environ.get("MAX_MODULE")`. Those three are the shapes
// bd-ob8i named. They are not something a refactor does by accident — they
// are deliberate — but a detector that can see them is still the 8mjs
// lesson: a written narrowing describes the hole; only a detector finds
// what is in it.
//
// NARROWING is the shippable half. This tree is full of `len(x) - 1` that
// is last-index arithmetic (`let last = items.len() - 1`) and of env reads
// that are not module bounds (`CDCP_ENGINE_ROOT`, `UPDATE_GOLDENS`). A
// detector that fired on those would be the over-strict gate that gets
// routed around. Two constraints buy the narrowness:
//
//   1. A binding is a hit only when the NAME looks like a module ceiling
//      (`max`/`last`/`ceil`/`bound`/`limit` AND `module`) AND the RHS is
//      not a bare integer (the integer case is NamedBound's).
//   2. `range(1, NAME)` is a hit only for a simple identifier, not
//      `range(1, 15)` (NumericBound) and not `range(1, max + 1)` (the
//      correct exclusive end). Rust `1..=count` is inclusive over a
//      derived count and is not this shape.
//
// Comments are not bindings, same rule as named_bound. Measured against
// the three scanned trees at landing: zero live hits. The plants below
// are what prove the detector trips; an empty live set is not a pass by
// itself.

/// Last path segment of a binding name (`self.max_module` → `max_module`).
fn binding_basename(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// `MAX_MODULE`, `max_module`, `MODULE_MAX`, `CDCP_MAX_MODULE`.
/// Not `module_count`, not `min_modules`, not `MIN_TERMS`, not `primary_modules`.
fn is_module_ceiling_name(name: &str) -> bool {
    let base = binding_basename(name);
    let n: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let has_module = n.contains("module");
    let has_ceiling = n.contains("max")
        || n.contains("last")
        || n.contains("ceil")
        || n.contains("bound")
        || n.contains("limit");
    has_module && has_ceiling
}

/// RHS after stripping a trailing terminator or comment. A bare integer is
/// NamedBound's job even when it is outside 13–16.
fn rhs_is_bare_integer(rhs: &str) -> bool {
    let mut t = rhs.trim();
    if let Some(i) = t.find("//") {
        t = t[..i].trim();
    }
    if let Some(i) = t.find('#') {
        t = t[..i].trim();
    }
    t = t.trim_end_matches([';', ',']).trim();
    !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit())
}

/// The name and RHS of a whole-line binding, if the line is one. Same
/// keyword / name / `=` shape as [`named_bound`], but the RHS is returned
/// rather than required to be two digits.
fn binding_name_and_rhs(line: &str) -> Option<(String, String)> {
    let t = line.trim();
    let b = t.as_bytes();
    if t.is_empty() || t.starts_with('#') || t.starts_with("//") || t.starts_with('*') {
        return None;
    }

    let mut i = 0usize;
    loop {
        let rest = &t[i..];
        let word_len = rest
            .bytes()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'(' || *c == b')')
            .count();
        let word = &rest[..word_len];
        if matches!(
            word,
            "pub"
                | "pub(crate)"
                | "const"
                | "static"
                | "let"
                | "mut"
                | "var"
                | "final"
                | "readonly"
                | "export"
                | "declare"
                | "local"
                | "my"
        ) {
            i += word_len;
            i += t[i..].bytes().take_while(|c| *c == b' ').count();
            if i >= t.len() {
                return None;
            }
            continue;
        }
        break;
    }

    let start = i;
    if !matches!(b.get(i), Some(c) if c.is_ascii_alphabetic() || *c == b'_') {
        return None;
    }
    while matches!(b.get(i), Some(c) if c.is_ascii_alphanumeric() || *c == b'_' || *c == b'.') {
        i += 1;
    }
    if i == start {
        return None;
    }
    let name = t[start..i].to_string();
    let skip_ws = |i: &mut usize| {
        while matches!(b.get(*i), Some(b' ') | Some(b'\t')) {
            *i += 1;
        }
    };
    skip_ws(&mut i);
    if b.get(i) == Some(&b':') {
        i += 1;
        while matches!(b.get(i), Some(c)
            if c.is_ascii_alphanumeric()
                || matches!(c, b'_' | b':' | b'<' | b'>' | b',' | b' ' | b'\'' | b'&'))
        {
            i += 1;
        }
    }
    skip_ws(&mut i);
    if b.get(i) != Some(&b'=') {
        return None;
    }
    if matches!(b.get(i + 1), Some(b'=') | Some(b'>')) {
        return None;
    }
    i += 1;
    let rhs = t[i..].trim();
    if rhs.is_empty() {
        return None;
    }
    Some((name, rhs.to_string()))
}

/// `range(1, NAME)` / `range(1,NAME)` where NAME is a simple identifier, not
/// a literal and not an expression (`range(1, max+1)` is the correct end).
fn range_over_simple_name(compact: &str) -> bool {
    let Some(rest) = compact.split_once("range(1,").map(|(_, r)| r) else {
        return false;
    };
    let ident_len = rest
        .bytes()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
        .count();
    if ident_len == 0 {
        return false;
    }
    let ident = &rest[..ident_len];
    if ident.bytes().all(|c| c.is_ascii_digit()) {
        return false;
    }
    matches!(rest.as_bytes().get(ident_len), Some(b')') | Some(b','))
}

/// The computed / external ceiling this line is, if any.
fn computed_bound(line: &str) -> Option<&'static str> {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') || t.starts_with("//") || t.starts_with('*') {
        return None;
    }
    let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    if range_over_simple_name(&compact) {
        return Some("range-over-name");
    }
    if let Some((name, rhs)) = binding_name_and_rhs(t) {
        if is_module_ceiling_name(&name) && !rhs_is_bare_integer(&rhs) {
            return Some("non-literal-ceiling");
        }
    }
    None
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

/// Run the four detectors over one root. Returns the per-tree file counts in
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
                if let Some(n) = named_bound(line) {
                    hits.push(Hit {
                        rel: rel.clone(),
                        line: i + 1,
                        text: line.trim().to_string(),
                        shape: Shape::NamedBound,
                        detail: format!(" [binds {n} to a name]"),
                    });
                }
                if let Some(kind) = computed_bound(line) {
                    hits.push(Hit {
                        rel: rel.clone(),
                        line: i + 1,
                        text: line.trim().to_string(),
                        shape: Shape::ComputedBound,
                        detail: format!(" [{kind}]"),
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
        "open bd-lt7/bd-ggs7/bd-8mjs instances changed: {open:?} — update this \
         count deliberately.\n\
         \n\
         HISTORY. It reached 0 on 2026-08-14 for numeric bounds; stayed at 0 \
         when the frozen-table detector was added and its two finds were \
         adjudicated (smoke_weak_links.py derived; results.js justified as \
         product with a bidirectional drift guard); and stayed at 0 again when \
         the NAMED-BOUND detector was added for bd-8mjs and its three finds \
         were adjudicated (build_glossary_json.py and its Rust port both bind \
         a glossary TERM-count floor; validate_grounding.rs binds the CPython \
         float-repr cutoff). None of the three is a module bound. Stayed at 0 \
         again when COMPUTED-BOUND was added for bd-ob8i: the live trees had \
         zero ceiling-name-to-non-literal bindings and zero `range(1, NAME)` \
         (the plants below are what prove that detector trips).\n\
         \n\
         WHAT THIS ZERO COVERS, stated here and not only in the header, because \
         a count reads broader than its detectors: the four shapes \
         (numeric-bound, frozen-module-table, named-bound, computed-bound) over \
         the three trees in SCANNED. A NEW hardcoded module bound or module \
         table there must be inventoried with a verdict, not silently added.\n\
         \n\
         WHAT THIS ZERO DOES NOT COVER: a one-letter or non-ceiling name bound \
         to a computed value then compared (`n = len(x) - 1`; `if m <= n` — \
         that is data-flow), a module table split across files at three ids \
         each, and every tree outside SCANNED. Those can move without this \
         number changing. bd-ob8i closed the ceiling-name-to-non-literal and \
         `range(1, NAME)` spellings; it did not close data-flow."
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

#[test]
fn the_named_bound_detector_sees_every_spelling_a_bound_can_hide_behind() {
    // The line that produced bd-8mjs, and the port the widening found.
    assert_eq!(named_bound("MIN_TERMS = 15"), Some(15));
    assert_eq!(named_bound("pub const MIN_TERMS: usize = 15;"), Some(15));
    assert_eq!(named_bound("    const REPR_FIXED_MAX: i32 = 16;"), Some(16));
    // The evasion this detector exists to make impossible, in each language
    // the scanned trees are written in.
    assert_eq!(named_bound("MAX_MODULE = 14"), Some(14));
    assert_eq!(named_bound("const MAX_MODULE = 14;"), Some(14));
    assert_eq!(named_bound("let maxModule = 13;"), Some(13));
    assert_eq!(named_bound("static MAX_MODULE: usize = 14;"), Some(14));
    assert_eq!(
        named_bound("MAX_MODULE=14"),
        Some(14),
        "shell has no spaces"
    );
    assert_eq!(named_bound("readonly MAX_MODULE=14"), Some(14));
    // Indentation, an attribute target, and a trailing comment are all still
    // the same binding.
    assert_eq!(named_bound("        self.max_module = 14"), Some(14));
    assert_eq!(
        named_bound("MAX_MODULE = 14  # the public EPI domains"),
        Some(14)
    );
    assert_eq!(named_bound("MAX_MODULE = 14,"), Some(14));
}

/// Known-GOOD, and the leg that decides whether this detector is shippable.
/// Every case here is a REAL line from this tree, and every one of them would
/// make the sweep an over-strict gate that gets routed around.
#[test]
fn the_named_bound_detector_does_not_fire_on_the_legitimate_constants_in_this_tree() {
    for (line, why) in [
        // web/assets/js/learn_chrome.js — a reading-time floor in minutes.
        ("if (min < 15) min = 15;", "a guarded floor, not a binding"),
        // web/assets/js/quiz.js — mulberry32.
        (
            "let r = Math.imul(t ^ (t >>> 15), 1 | t);",
            "a PRNG bit shift",
        ),
        (
            "return ((r ^ (r >>> 14)) >>> 0) / 4294967296;",
            "a PRNG bit shift",
        ),
        // web/assets/js/results.js — display truncation of a hash.
        (
            "return h.length > 16 ? h.slice(0, 12) + \"…\" : h;",
            "a string truncation",
        ),
        // scripts/validate_grounding.py — an argparse default.
        (
            "ap.add_argument(\"--sample-report\", type=int, default=15)",
            "a call argument, not a line-level binding",
        ),
        // scripts/smoke_learn_v2.py — the term-count floor, unnamed.
        (
            "if (g.get(\"term_count\") or 0) < 15:",
            "a comparison; NumericBound already holds it",
        ),
        // crates/…/diff_verify_objectives.rs — a TOML fixture inside a Rust
        // string literal. Dozens of these exist; every one is a false positive.
        (
            "\"[[domain_min]]\\nmodule = 15\\nmin_items = 16\\n\",",
            "a quoted fixture, not code",
        ),
        (
            "        \"[[coverage_exempt]]\\nmodule = 15\\nreason = \\\"\\\"\\n\",",
            "a quoted fixture, not code",
        ),
        // PROSE IS NOT LIVE CODE — the error this suite has now made three
        // times. A commented-out constant is not a bound in effect.
        ("# MIN_TERMS = 15", "a Python comment"),
        ("// MAX_MODULE = 14;", "a Rust comment"),
        (
            "//! `primary_notes_checked=15`. Nothing was fixed here",
            "a Rust doc comment",
        ),
        ("* modules = 15 in the report loop", "a JSDoc continuation"),
        // Comparisons and match arms are not bindings.
        ("if x == 15:", "an equality test"),
        (
            "assert!(removed >= 15, \"emptied only {removed} files\");",
            "a macro call",
        ),
        ("15 => ChoiceLetter::P,", "a match arm"),
        (
            "let body = if decpt <= -4 || decpt > 16 {",
            "a bound expression",
        ),
        // A number that merely starts with the digits is not the bound.
        ("SAMPLE = 150", "a longer number"),
        ("RATIO = 15.5", "a float"),
        ("TOTAL = 1400", "a longer number"),
        // Structure, not assignment.
        ("line: 14,", "a struct field"),
        (
            "(\"web/assets/js\", &[\"js\", \"mjs\"], 12),",
            "a tuple element",
        ),
    ] {
        assert_eq!(
            named_bound(line),
            None,
            "over-match on {line:?} — {why}. An over-strict sweep gets routed \
             around, which is slower than no sweep at all."
        );
    }
}

#[test]
fn the_computed_bound_detector_sees_the_three_spellings_ob8i_named() {
    assert_eq!(
        computed_bound("MAX_MODULE = len(declared) - 1"),
        Some("non-literal-ceiling")
    );
    assert_eq!(
        computed_bound("let max_module = declared.len() - 1;"),
        Some("non-literal-ceiling")
    );
    assert_eq!(
        computed_bound("self.max_module = os.environ.get(\"MAX_MODULE\")"),
        Some("non-literal-ceiling")
    );
    assert_eq!(
        computed_bound("MAX_MODULE = cfg[\"max\"]"),
        Some("non-literal-ceiling")
    );
    assert_eq!(
        computed_bound("MAX_MODULE = (14,)"),
        Some("non-literal-ceiling")
    );
    assert_eq!(
        computed_bound("for n in range(1, MAX):"),
        Some("range-over-name")
    );
    assert_eq!(
        computed_bound("PRIMARY_MODULES = range(1, n)"),
        Some("range-over-name")
    );
    assert_eq!(
        computed_bound("MAX_MODULE=14"),
        None,
        "a bare integer is NamedBound's, not this shape"
    );
    assert_eq!(named_bound("MAX_MODULE=14"), Some(14));
}

/// Known-GOOD: every case is a REAL line (or the correct exclusive-range
/// spelling). A detector that fired here would be over-strict.
#[test]
fn the_computed_bound_detector_does_not_fire_on_the_legitimate_expressions_in_this_tree() {
    for (line, why) in [
        (
            "let last = sorted_terms.len() - 1;",
            "glossary last-index, name is not a module ceiling",
        ),
        (
            "let last = items.len() - 1;",
            "units last-index, name is not a module ceiling",
        ),
        (
            "let mut line_len = PREFIX.len() - 1;",
            "prefix length, name is not a module ceiling",
        ),
        (
            "Score::new(earned, (key.len() - 1) as u64)",
            "adjacent-pairs denominator, not an assignment",
        ),
        (
            "root_env = os.environ.get(\"CDCP_ENGINE_ROOT\", \"\").strip()",
            "engine-root env, name is not a module ceiling",
        ),
        (
            "if std::env::var(\"UPDATE_GOLDENS\").ok().as_deref() != Some(\"1\") {",
            "goldens env, not a module ceiling",
        ),
        (
            "if (g.get(\"term_count\") or 0) < 15:",
            "term-count floor; NumericBound already holds it",
        ),
        (
            "PRIMARY_MODULES = range(1, 15)",
            "literal end is NumericBound, not range-over-name",
        ),
        ("for u in range(1, 10):", "literal end is not a name"),
        (
            "for n in range(1, max_module + 1):",
            "the correct exclusive end is an expression, not a name",
        ),
        (
            "for n in 1..=count {",
            "Rust inclusive range over a derived count is not Python range(1, NAME)",
        ),
        ("module_count = len(modules)", "a count, not a ceiling name"),
        ("min_modules: 8,", "a floor, not a ceiling binding"),
        (
            "(\"primary_modules\".into(), ints(required)),",
            "a report key, not a ceiling binding",
        ),
        ("# MAX_MODULE = len(x) - 1", "a Python comment"),
        ("// let max_module = n - 1;", "a Rust comment"),
        (
            "for n in sorted(declared):",
            "a derived iteration, not a range over a name",
        ),
    ] {
        assert_eq!(
            computed_bound(line),
            None,
            "over-match on {line:?} — {why}. An over-strict sweep gets routed \
             around, which is slower than no sweep at all."
        );
    }
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
        // The floor here comes from the LIVE bank_policy.toml, so the needle
        // names the module and the approved count and leaves the floor to the
        // policy — a test that wrote today's floor down would rot with it.
        run.stdout.contains("module 15: 0 approved < min "),
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
        run.stdout
            .contains("module 15: 0 approved < min 1 (0 scanned, 0 not approved)"),
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
            .contains("m15: 0 approved of 0 scanned — exempt: fixture: not yet authored"),
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

/// A tree `cdcp_gate build-units` can run in: a Learn index, module markdown,
/// and a bank.
///
/// Until 2026-08-14 this also copied `scripts/build_units.py` in, because the
/// gate under test WAS that script. The oracle was retired with bd-qqwc; the
/// Rust takes its root from `--root`, so the fixture no longer needs to host a
/// copy of the implementation.
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

/// Run the Learn compiler against a fixture root.
///
/// Library call, not `cdcp_gate build-units`: the compiler left the gate
/// (bd-engine-not-gate-ar39.2). write_units writes only on GREEN, matching
/// the old process contract the cases below assert.
fn build_units(f: &Fixture) -> Run {
    let outcome = cdcp_learn::units::write_units(&f.path("")).expect("write_units");
    Run {
        code: outcome.code,
        stdout: outcome.stdout,
        stderr: String::new(),
    }
}

/// Byte-copy every input `build-units` resolves off its root, into a fixture.
/// The gate derives all six paths from the root it is handed, so `--root
/// <fixture>` is what keeps its WRITE inside the fixture.
fn live_units_fixture() -> Fixture {
    let root = engine_root();
    let f = Fixture::new();
    let copy = |rel: &str| {
        let dst = f.path(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(root.join(rel), &dst)
            .unwrap_or_else(|e| panic!("copy {rel} into the fixture: {e}"));
    };
    copy("knowledge/topics.toml");
    copy("web/data/modules_index.json");
    copy("web/data/bank_items_seed42.json");

    let mut copied = 0usize;
    std::fs::create_dir_all(f.path("web/content/modules")).unwrap();
    for e in std::fs::read_dir(root.join("web/content/modules"))
        .unwrap()
        .flatten()
    {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("md") {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            std::fs::copy(&p, f.path(&format!("web/content/modules/{name}"))).unwrap();
            copied += 1;
        }
    }
    // Anti-vacuous: a fixture that copied no module would build zero units and
    // still exit 0 on the wrong thing. The floor is the fifteen declared
    // modules with a module of slack, deliberately BELOW today's count and not
    // equal to it, so retiring one module does not fail this for the wrong
    // reason while an empty copy still cannot pass.
    assert!(
        copied >= 14,
        "fixture copied only {copied} module files — a vacuous fixture would let \
         every assertion below pass over an empty tree"
    );
    f
}

/// Known-GOOD, and bd-rebase-bounds-live-tree-write-ohgr's fix in one test.
///
/// The claim is the same as before — the check floor now covers all 15 modules
/// rather than the 14 that `int(m[:2]) <= 14` admitted — but it is now made
/// against a TREE COPY, and it is STRONGER for it.
///
/// Until 2026-08-14 this ran `python3 scripts/build_units.py` against the LIVE
/// tree. `build_units.py` is a BUILDER: on the green path it writes
/// `web/data/units_index.json`, a TRACKED artifact. Three things follow, and
/// none of them is hypothetical in a repo that runs six agents at once:
/// `git status` becomes a function of whether you ran the tests; a stale
/// committed artifact is silently refreshed instead of reported; and every
/// concurrent reader and writer of `web/data/` is raced.
///
/// MEASURED 2026-08-14, and the reason the fix is not "assert the tree stayed
/// clean": on a clean HEAD clone the whole suite left `git status --porcelain`
/// EMPTY, while the mtime of `web/data/units_index.json` moved. The write
/// happened; it was invisible because the artifact was already current. A
/// tree-cleanliness assertion cannot see this class of write, and the write is
/// no less a race for being idempotent today.
///
/// The last assertion is what makes the read-only form strictly stronger than
/// the live one. Running live MAKES the artifact current and therefore proves
/// nothing about it; comparing the fixture's bytes to the tracked file PROVES
/// the committed artifact is current, without touching it. (`diff_build_units`
/// holds the same tie-back for the Python/Rust pair; here it is what turns
/// "the gate ran green" into "the gate ran green over the fifteen modules that
/// are actually committed".)
#[test]
fn build_units_known_good_a_copy_of_the_live_tree_passes_over_all_declared_modules() {
    let f = live_units_fixture();
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

    let produced = std::fs::read(f.path("web/data/units_index.json"))
        .expect("the green path must have written the artifact into the FIXTURE");
    assert!(
        !produced.is_empty(),
        "a zero-byte artifact is an ERROR, not a pass"
    );
    let tracked = std::fs::read(engine_root().join("web/data/units_index.json"))
        .expect("the tracked web/data/units_index.json must exist");
    assert_eq!(
        produced, tracked,
        "the live inputs do not reproduce the tracked web/data/units_index.json, \
         so the committed artifact is stale. Regenerate it deliberately — this \
         test will NOT regenerate it for you, which is the whole point of the \
         bead that removed the live-tree run."
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
// The Python is DELETED (EXTRACT-THEN-DELETE into cdcp_learn::feedback).
// The reader takes a root, so the fixture is a tree of the product surfaces
// it reads — copied, not synthesised, because a hand-written stand-in would
// let this leg pass while the shipped ones diverged. The reader writes
// nothing, so pointing it at a fixture cannot dirty the live tree.

/// A tree `cdcp_learn::feedback` can run in: the registry and the product
/// surfaces it checks against.
fn feedback_fixture() -> Fixture {
    let root = engine_root();
    let f = Fixture::new();

    let copy = |rel: &str| {
        let dst = f.path(rel);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(root.join(rel), &dst)
            .unwrap_or_else(|e| panic!("copy {rel} into the fixture: {e}"));
    };
    copy("knowledge/domains.toml");
    copy("knowledge/topics.toml");
    copy("web/assets/js/results.js");
    copy("web/data/module_learn_slugs.js");
    copy("web/assets/js/learn_md.js");
    copy("web/data/keys_seed42.json");
    copy("web/data/bank_items_seed42.json");
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
    let o = cdcp_learn::feedback::run(&f.root);
    Run {
        code: o.code,
        stdout: o.stdout,
        stderr: String::new(),
    }
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
    let js_path = f.path("web/data/module_learn_slugs.js");
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
    copy("web/data/module_learn_slugs.js");
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

/// Drop one `MODULE_LEARN_SLUGS` row from the fixture's generated map.
fn drop_slug_row(f: &Fixture, order: u32, slug: &str) {
    let path = f.path("web/data/module_learn_slugs.js");
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

/// Known-BAD for the sweep, bd-8mjs's own regression: plant the EXACT evasion
/// — a live module bound moved behind a named constant, with the comparison
/// left reading against the name — and the sweep must name the FILE and the
/// LINE. Under the two-shape sweep this file was invisible: the comparison
/// carries no literal and the binding carries no operator.
#[test]
fn the_sweep_goes_red_on_a_module_bound_hidden_behind_a_name() {
    let f = Fixture::new();
    f.write(
        "scripts/planted_named_bound.py",
        "#!/usr/bin/env python3\n\
         \"\"\"a gate that froze the module ceiling behind a name.\"\"\"\n\
         MAX_MODULE = 14\n\
         \n\
         for m in modules:\n\
         \x20   if int(m[:2]) <= MAX_MODULE:\n\
         \x20       check(m)\n",
    );
    let (counts, hits) = sweep(&f.root);
    assert_eq!(counts[0], 1, "the fixture's scripts/ tree holds one file");

    // The two-shape sweep saw NOTHING here, and that is the whole bead.
    assert!(
        !hits.iter().any(|h| h.shape == Shape::NumericBound),
        "the planted file carries no numeric literal to match: {hits:?}"
    );
    let named: Vec<&Hit> = hits
        .iter()
        .filter(|h| h.shape == Shape::NamedBound)
        .collect();
    assert_eq!(
        named.len(),
        1,
        "the planted binding must produce exactly one hit, got {named:?}"
    );
    let h = named[0];
    assert_eq!(h.rel, "scripts/planted_named_bound.py");
    assert_eq!(h.line, 3, "the hit must name the line the binding is on");
    assert_eq!(h.text, "MAX_MODULE = 14");
    assert!(
        h.detail.contains("binds 14 to a name"),
        "the finding must say what it saw: {}",
        h.detail
    );
    assert!(
        !INVENTORY
            .iter()
            .any(|(file, src, shape, _, _)| *file == h.rel && *src == h.text && *shape == h.shape),
        "a planted binding must be un-inventoried"
    );
}

/// Known-BAD for the sweep, bd-ob8i's own regression: plant the EXACT evasion
/// — a module ceiling computed from a length, plus `range(1, NAME)` — and
/// the sweep must name the FILE and the LINE. Under the three-shape sweep
/// this file was invisible: there is no 13–16 literal and no name-to-integer
/// binding.
#[test]
fn the_sweep_goes_red_on_a_module_bound_computed_or_read_from_config() {
    let f = Fixture::new();
    f.write(
        "scripts/planted_computed_bound.py",
        "#!/usr/bin/env python3\n\
         \"\"\"a gate that computed the module ceiling instead of iterating the registry.\"\"\"\n\
         MAX_MODULE = len(declared) - 1\n\
         \n\
         for n in range(1, MAX_MODULE):\n\
         \x20   check(n)\n",
    );
    let (counts, hits) = sweep(&f.root);
    assert_eq!(counts[0], 1, "the fixture's scripts/ tree holds one file");

    assert!(
        !hits.iter().any(|h| h.shape == Shape::NumericBound),
        "the planted file carries no numeric literal to match: {hits:?}"
    );
    assert!(
        !hits.iter().any(|h| h.shape == Shape::NamedBound),
        "the planted file binds no bare integer: {hits:?}"
    );
    let computed: Vec<&Hit> = hits
        .iter()
        .filter(|h| h.shape == Shape::ComputedBound)
        .collect();
    assert_eq!(
        computed.len(),
        2,
        "len-minus-one binding AND range-over-name must both hit, got {computed:?}"
    );
    assert_eq!(computed[0].rel, "scripts/planted_computed_bound.py");
    assert_eq!(computed[0].text, "MAX_MODULE = len(declared) - 1");
    assert!(
        computed[0].detail.contains("non-literal-ceiling"),
        "the binding must say what it saw: {}",
        computed[0].detail
    );
    assert_eq!(computed[1].text, "for n in range(1, MAX_MODULE):");
    assert!(
        computed[1].detail.contains("range-over-name"),
        "the range must say what it saw: {}",
        computed[1].detail
    );
    assert!(
        computed
            .iter()
            .all(|h| !INVENTORY.iter().any(|(file, src, shape, _, _)| {
                *file == h.rel && *src == h.text && *shape == h.shape
            })),
        "a planted computed bound must be un-inventoried"
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

// ── 7. the live-tree runs, ledgered (bd-rebase-bounds-live-tree-write-ohgr) ─
//
// A live-tree run is not forbidden here, because forbidding it outright would
// be over-strict: three of the legs above are CHECKERS, which read a tree and
// write nothing, and running them against the shipped tree is the strongest
// form of those legs. What is forbidden is an UNARGUED one. The instance this
// bead removed — `python3 scripts/build_units.py` against the engine root —
// entered this file without anyone writing down that build_units is a BUILDER
// and writes `web/data/units_index.json` on its green path.
//
// So every live run is ledgered with the answer to one question: DOES IT WRITE?
// A new live run cannot be added without answering it, which is precisely the
// step that was skipped.
//
// WHAT THIS LEDGER DOES NOT COVER, stated because a ledger reads broader than
// its scan: it sees THIS FILE, and it sees a script path spelled as
// `.join("scripts/…")` inside a command. A run reached through a helper that
// hides the path (`python_script()` in `diff_verify_doc_consistency.rs` is a
// real example, in a file this bead does not own) is invisible to it. The
// suite-wide property — a full run leaves the tree clean — is not a unit test
// and is not asserted here; it was MEASURED on a clean HEAD clone instead, and
// the measurement is recorded on the build_units leg above, including the part
// a `git status` assertion cannot see.

/// `(script, writes?, why the live run is safe)`.
const LIVE_RUNS: &[(&str, bool, &str)] = &[(
    "scripts/verify_coverage.py",
    false,
    "a CHECKER. Its only write is behind `--write-json`, an explicit flag \
         neither call site passes; without it the script prints and exits.",
)];

/// Every command in this file that runs something resolved off the ENGINE root
/// is in `LIVE_RUNS`, with the write question answered.
#[test]
fn every_live_tree_run_in_this_file_is_ledgered_with_the_write_question_answered() {
    // Compile-time, so the scan cannot drift from the file it claims to read.
    let src = include_str!("rebase_module_bounds.rs");

    let mut found: Vec<String> = Vec::new();
    let mut at = 0usize;
    while let Some(i) = src[at..].find("Command::new(") {
        let start = at + i;
        let end = (start + 300).min(src.len());
        // Byte offsets from `find` are char boundaries; walk `end` back to one.
        let mut end = end;
        while !src.is_char_boundary(end) {
            end -= 1;
        }
        let window = &src[start..end];
        let mut w = 0usize;
        while let Some(j) = window[w..].find(".join(\"scripts/") {
            let s = w + j + ".join(\"".len();
            let Some(q) = window[s..].find('"') else {
                break;
            };
            found.push(window[s..s + q].to_string());
            w = s + q;
        }
        at = start + "Command::new(".len();
    }
    found.sort();
    found.dedup();

    // Anti-vacuous from both sides. A scan that matched nothing reports exactly
    // like a file with no live runs, and this file has some.
    assert!(
        found.len() >= 1,
        "the live-run scan found {} command(s) — it has gone dead, or the file \
         was reformatted past the shapes it reads. A scan that finds nothing is \
         an ERROR, not a clean bill: {found:?}",
        found.len()
    );

    let unledgered: Vec<&String> = found
        .iter()
        .filter(|s| !LIVE_RUNS.iter().any(|(script, _, _)| *script == s.as_str()))
        .collect();
    assert!(
        unledgered.is_empty(),
        "{} live-tree run(s) are not in LIVE_RUNS: {unledgered:?}. Add a row \
         answering whether the script WRITES — and if it does, run it in a tree \
         copy instead, the shape `live_units_fixture` and \
         `tests/diff_build_units.rs` both use.",
        unledgered.len()
    );

    let stale: Vec<&str> = LIVE_RUNS
        .iter()
        .map(|(s, _, _)| *s)
        .filter(|s| !found.iter().any(|f| f == s))
        .collect();
    assert!(
        stale.is_empty(),
        "{} LIVE_RUNS row(s) matched no command — the run is gone (delete the \
         row) or the scan is broken: {stale:?}",
        stale.len()
    );

    // The ledger's whole content: nothing that writes may run live.
    let writers: Vec<&str> = LIVE_RUNS
        .iter()
        .filter(|(_, writes, _)| *writes)
        .map(|(s, _, _)| *s)
        .collect();
    assert!(
        writers.is_empty(),
        "{writers:?} are ledgered as WRITING and are still run against the live \
         tree. A test that writes a tracked artifact makes `git status` a \
         function of whether you ran the tests, refreshes a stale artifact \
         instead of reporting it, and races every concurrent agent."
    );
}
