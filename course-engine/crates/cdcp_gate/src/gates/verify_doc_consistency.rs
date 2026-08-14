//! `verify-doc-consistency` — the roadmap-truth gate, ported byte-for-byte from
//! `scripts/verify_doc_consistency.py`.
//!
//! # Claim class: FLOOR-RAISE
//!
//! This gate raises the floor under the repo's load-bearing prose by two
//! machine-checkable assertions, and only those two:
//!
//! 1. **Milestone-status agreement.** The milestone tables in `CHARTER.md`,
//!    `README.md` and `course-engine/docs/PHASE-NEXT.md` are parsed into
//!    `(milestone id -> status)` rows. RED when one id carries conflicting
//!    statuses across or within docs, when one id appears twice in a single
//!    table, when a status cell uses vocabulary the gate cannot read
//!    (fail-closed: an unreadable status is not a passing status), when a
//!    cell asserts DONE and OPEN at once, or when a row in a table that HAS a
//!    Status column is too short to reach it (see the DECISION below).
//! 2. **Publication truth.** The repository is public (`REPO_PUBLIC`). RED when
//!    any scanned markdown still asserts that publication is pending, blocked,
//!    deferred, or awaiting a human.
//!
//! # What this gate CANNOT decide
//!
//! - Whether a status is *true*. It compares declarations against each other; a
//!   milestone every doc calls DONE that shipped nothing stays green here. Only
//!   a loop-#3 external signal settles that.
//! - Whether the roadmap is *complete*. A milestone nobody wrote down is
//!   invisible to a gate that reads what was written down.
//! - Prose outside milestone tables and the five publication patterns. Vague,
//!   stale, or contradictory narrative text is out of scope by construction.
//! - Any claim in a doc it never read: a markdown file that is not valid UTF-8
//!   is reported as unreadable and refused, never silently skipped.
//!
//! # DECISION (bd-hw3, 2026-08-14): a row shorter than its Status column is RED
//!
//! Both implementations used to fall back to the section heading's status when a
//! data row had fewer cells than the Status column index. When the heading was
//! not itself a status word that fallback was `None`; the oracle then printed
//! `PASS`, most of the summary, and died with a `TypeError` in
//! `",".join(sorted({r["status"] for r in rows}))`, while this port completed the
//! report and rendered the missing status as the literal string `None`. Both
//! output shapes were wrong, in opposite directions.
//!
//! The resolution is to fail closed, in both, for three reasons:
//!
//! - Every OTHER unreadable status here already fails closed — empty cell,
//!   unrecognised word, DONE-and-OPEN at once. A status cell that is ABSENT
//!   ENTIRELY is strictly less readable than one present and unrecognised, so it
//!   cannot be the single case that passes. The old behaviour was fail-OPEN by
//!   accident, not by design.
//! - It corrupted the anti-vacuous counters: the row still counted toward
//!   `milestone_rows` and `milestone_ids`, so the gate reported having read a row
//!   it could not read.
//! - Rendering `None` MINTS A THIRD STATUS VALUE. A milestone DONE in one doc and
//!   ragged in another would surface as a cross-doc conflict `…=DONE · …=None`,
//!   which names the wrong defect: the docs do not disagree, one row is malformed.
//!
//! Kept deliberately: the heading-supplied status. A milestone table with NO
//! Status column at all, under a status-bearing heading (the PHASE-NEXT shape),
//! still takes its status from the heading — a table-level declaration, and
//! legitimate. The defect was conflating that with a row-level SHORTFALL inside a
//! table that does declare a Status column. Separating the two is the whole fix,
//! and it is why `Row::status` is now a plain `&'static str`: after the change no
//! row can carry an absent status, so the summary join has nothing to render.
//!
//! # Anti-vacuous
//!
//! An empty input set is an ERROR, not a pass. Zero markdown files, a missing
//! roadmap doc, or a roadmap doc yielding zero milestone rows all exit non-zero.
//! A doc that was never parsed must never report like one that agreed.
//!
//! # Byte-exactness contract with the Python original
//!
//! `crates/cdcp_gate/tests/diff_verify_doc_consistency.rs` runs both
//! implementations on identical inputs and asserts stdout, stderr, AND exit
//! code match byte for byte on every case. The stdout stream here is the
//! Python's stdout character for character — including the `!r` repr forms, the
//! `sorted()` orderings, and the 40-item report cap.
//!
//! To hold that contract the failure report is written to **stdout with exit
//! status 1**, not routed through `GateError`: the dispatcher's `report()`
//! writes to stderr and maps to exit 2 or 4, and the oracle produces neither, so
//! routing through it would make the two sides differ on every RED case — which
//! would blind the one mechanism that can tell a port BUG from an intended FIX.
//! `crate::exit` is therefore deliberately NOT used for this gate's failure
//! paths. That is a knowing, single-file deviation from the shared convention,
//! recorded here for review rather than made quietly, and it lasts exactly as
//! long as the oracle does (see `crate_exit_code` and bd-2m9). Invocation errors
//! still go through `GateError::Usage`, because the oracle's argparse surface is
//! not a verdict on the tree and is not part of the differential.
//!
//! Every input class the suite exercises now has a byte-exact target. The one
//! that did not — the ragged milestone row, which made the oracle print `PASS`
//! and then die with a `TypeError` traceback — was repaired on both sides under
//! bd-hw3; see the DECISION above, and `ragged_row_is_red_in_both` for the pin
//! that replaced the recorded divergence.

use crate::registry::{GateCtx, GateError};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub const NAME: &str = "verify-doc-consistency";
pub const SUMMARY: &str =
    "roadmap docs must not contradict each other (milestone status + publication truth)";

/// Roadmap docs whose milestone tables must agree. Paths are root-relative.
const MILESTONE_DOCS: [&str; 3] = [
    "CHARTER.md",
    "README.md",
    "course-engine/docs/PHASE-NEXT.md",
];

/// A table is a MILESTONE table when its first column header is one of these.
/// Excluded on purpose, with the reason (an exclusion without a reason is a
/// schema error): a table keyed by "Epic" records BEAD lifecycle, not milestone
/// status — a tracking bead may legitimately stay open for a follow-up task
/// after the milestone itself is green, so bead state is not a roadmap claim.
const MILESTONE_KEY_HEADERS: [&str; 4] = ["id", "wave", "milestone", "phase"];

const DONE_WORDS: [&str; 8] = [
    "done",
    "green",
    "closed",
    "complete",
    "completed",
    "shipped",
    "delivered",
    "landed",
];

const OPEN_WORDS: [&str; 13] = [
    "open",
    "pending",
    "planned",
    "blocked",
    "ongoing",
    "todo",
    "deferred",
    "wip",
    "unstarted",
    "in progress",
    "in-progress",
    "not started",
    "not yet",
];

// ── Publication truth ───────────────────────────────────────────────────────
// Declared fact, not a guess. If the repository is ever made private again this
// constant must be flipped in the same commit — that is the point of pinning it
// here rather than inferring it from a git remote (private repos have remotes
// too) or from a doc (which is the thing under test).
const REPO_PUBLIC: bool = true;
const REPO_PUBLIC_SINCE: &str = "2026-08-12";
const REPO_PUBLIC_EVIDENCE: &str = "github.com/JYeswak/cdcp-self-study";

/// `_FLIP` in the original: the publication-subject alternation. Deliberately
/// not word-anchored — the Python's alternation carries no `\b` either.
const FLIP_ALTS: [&str; 5] = [
    "visibility flip",
    "publication",
    "publishing",
    "going public",
    "public release",
];

/// `_STUCK` in the original: the not-done alternation, word-anchored on both
/// sides in every pattern that uses it.
const STUCK_ALTS: [&str; 7] = [
    "pending",
    "blocked",
    "not performed",
    "deferred",
    "awaiting",
    "not yet done",
    "remains open",
];

const WHY_NOT_DONE: &str = "publication described as not done";
const WHY_AUDIT_NO: &str = "audit says the repo is not public";
const WHY_AWAITING_JOSH: &str = "work parked on a human that already happened";
const WHY_HUMAN_CALL: &str = "visibility flip described as still to come";

const MAX_REPORT: usize = 40;

/// The gap the `[^.\n]{0,60}?` bridge may span, in characters.
const BRIDGE: usize = 60;

// ═══════════════════════════ Python primitives ═════════════════════════════

/// Python's `\w` for `str` patterns: alphanumeric plus underscore.
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `\b` at character index `pos` of `cs`.
fn at_word_boundary(cs: &[char], pos: usize) -> bool {
    let before = pos > 0 && is_word_char(cs[pos - 1]);
    let after = pos < cs.len() && is_word_char(cs[pos]);
    before != after
}

/// Python's `str.splitlines()`: splits on the full CPython line-boundary set,
/// not just `\n` / `\r\n`, and drops the empty tail after a final boundary.
fn py_splitlines(s: &str) -> Vec<&str> {
    fn is_break(c: char) -> bool {
        matches!(
            c,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        )
    }
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut it = s.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        if !is_break(c) {
            continue;
        }
        out.push(&s[start..i]);
        let mut next = i + c.len_utf8();
        if c == '\r' {
            if let Some(&(j, '\n')) = it.peek() {
                it.next();
                next = j + 1;
            }
        }
        start = next;
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

/// CPython's `repr()` for `str`, which is what an f-string `!r` emits.
///
/// Non-ASCII is emitted literally, matching CPython for every printable
/// codepoint; the explicitly-listed non-printable ranges below are escaped the
/// way CPython escapes them. Codepoints outside those ranges that CPython would
/// still consider non-printable are a documented approximation — resolving them
/// exactly needs a Unicode category table this crate does not carry.
fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        let u = c as u32;
        if c == quote || c == '\\' {
            out.push('\\');
            out.push(c);
        } else if c == '\t' {
            out.push_str("\\t");
        } else if c == '\n' {
            out.push_str("\\n");
        } else if c == '\r' {
            out.push_str("\\r");
        } else if u < 0x20 || u == 0x7f {
            out.push_str(&format!("\\x{u:02x}"));
        } else if u < 0x7f {
            out.push(c);
        } else if (0x80..=0x9f).contains(&u) || u == 0xa0 || u == 0xad {
            out.push_str(&format!("\\x{u:02x}"));
        } else if u == 0x1680
            || (0x2000..=0x200f).contains(&u)
            || (0x2028..=0x202e).contains(&u)
            || u == 0x205f
            || (0x2060..=0x206f).contains(&u)
            || u == 0x3000
            || u == 0xfeff
        {
            out.push_str(&format!("\\u{u:04x}"));
        } else {
            out.push(c);
        }
    }
    out.push(quote);
    out
}

/// `line.strip()[:120]` — a CHARACTER slice, not a byte slice.
fn first_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// `re.search(rf"\b{re.escape(w)}\b", text)` for an all-word-character needle.
fn has_word(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    for (i, _) in text.match_indices(word) {
        let before_ok = text[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !is_word_char(c));
        let after_ok = text[i + word.len()..]
            .chars()
            .next()
            .is_none_or(|c| !is_word_char(c));
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

// ═══════════════════════════ cell / row parsing ════════════════════════════

/// `_EMPH.sub("", cell).strip()`.
fn strip_md(cell: &str) -> String {
    cell.chars()
        .filter(|c| !matches!(c, '*' | '`' | '_'))
        .collect::<String>()
        .trim()
        .to_string()
}

struct RangeMatch {
    text: String,
    prefix: char,
    lo: Option<u128>,
    hi: Option<u128>,
}

fn digits_at(cs: &[char], mut i: usize) -> (usize, String) {
    let mut s = String::new();
    while i < cs.len() && cs[i].is_ascii_digit() {
        s.push(cs[i]);
        i += 1;
    }
    (i, s)
}

fn skip_ws(cs: &[char], mut i: usize) -> usize {
    while i < cs.len() && cs[i].is_whitespace() {
        i += 1;
    }
    i
}

/// `_RANGE = r"\b([MV])(\d+)\s*[-–—]\s*(?:[MV])?(\d+)\b"`, non-overlapping,
/// left to right.
fn find_ranges(s: &str) -> Vec<RangeMatch> {
    let cs: Vec<char> = s.chars().collect();
    let n = cs.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < n {
        if let Some((end, prefix, lo, hi)) = try_range(&cs, i) {
            out.push(RangeMatch {
                text: cs[i..end].iter().collect(),
                prefix,
                lo: lo.parse::<u128>().ok(),
                hi: hi.parse::<u128>().ok(),
            });
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

fn try_range(cs: &[char], i: usize) -> Option<(usize, char, String, String)> {
    if !at_word_boundary(cs, i) {
        return None;
    }
    let prefix = *cs.get(i)?;
    if prefix != 'M' && prefix != 'V' {
        return None;
    }
    let (j, lo) = digits_at(cs, i + 1);
    if lo.is_empty() {
        return None;
    }
    let k = skip_ws(cs, j);
    let dash = *cs.get(k)?;
    if dash != '-' && dash != '\u{2013}' && dash != '\u{2014}' {
        return None;
    }
    let p = skip_ws(cs, k + 1);
    // `(?:[MV])?` is greedy: try consuming it first, then the empty match.
    let starts: [usize; 2] = match cs.get(p) {
        Some('M') | Some('V') => [p + 1, p],
        _ => [p, p],
    };
    for start in starts {
        let (end, hi) = digits_at(cs, start);
        if hi.is_empty() {
            continue;
        }
        if end < cs.len() && is_word_char(cs[end]) {
            continue; // trailing \b fails; a shorter \d+ cannot rescue it
        }
        return Some((end, prefix, lo, hi));
    }
    None
}

struct TokenMatch {
    end: usize,
    base: String,
    suffix: String,
}

/// `_TOKEN = r"\b([MV]\d+)((?:-S\d+)(?:/S\d+)*)?\b"`, non-overlapping, left to
/// right, with the optional group's greedy backtracking preserved.
fn find_tokens(s: &str) -> Vec<TokenMatch> {
    let cs: Vec<char> = s.chars().collect();
    let n = cs.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < n {
        if let Some(t) = try_token(&cs, i) {
            i = t.end;
            out.push(t);
        } else {
            i += 1;
        }
    }
    out
}

fn try_token(cs: &[char], i: usize) -> Option<TokenMatch> {
    if !at_word_boundary(cs, i) {
        return None;
    }
    let prefix = *cs.get(i)?;
    if prefix != 'M' && prefix != 'V' {
        return None;
    }
    let (base_end, digits) = digits_at(cs, i + 1);
    if digits.is_empty() {
        return None;
    }

    // `(?:-S\d+)(?:/S\d+)*` — collect every reachable end, shortest first.
    let mut reps: Vec<usize> = Vec::new();
    if cs.get(base_end) == Some(&'-') && cs.get(base_end + 1) == Some(&'S') {
        let (mut p, d) = digits_at(cs, base_end + 2);
        if !d.is_empty() {
            reps.push(p);
            loop {
                if cs.get(p) == Some(&'/') && cs.get(p + 1) == Some(&'S') {
                    let (q, d2) = digits_at(cs, p + 2);
                    if d2.is_empty() {
                        break;
                    }
                    p = q;
                    reps.push(p);
                } else {
                    break;
                }
            }
        }
    }

    // Greedy: longest group-2 first, then successively shorter, then absent.
    let mut candidates: Vec<usize> = reps.into_iter().rev().collect();
    candidates.push(base_end);
    for end in candidates {
        if end == cs.len() || !is_word_char(cs[end]) {
            return Some(TokenMatch {
                end,
                base: cs[i..base_end].iter().collect(),
                suffix: cs[base_end..end].iter().collect(),
            });
        }
    }
    None
}

/// Extract milestone ids from a table's first cell. Handles ranges (`M0–M7` ->
/// M0..M7) and sub-milestone runs (`M9-S1/S2` -> M9-S1, M9-S2). Returns an empty
/// vec when the cell names no milestone, which is how non-milestone rows are
/// skipped.
fn milestone_ids(cell: &str) -> Vec<String> {
    let text = strip_md(cell);
    let mut out: Vec<String> = Vec::new();
    let mut consumed = text.clone();
    for m in find_ranges(&text) {
        let (Some(lo), Some(hi)) = (m.lo, m.hi) else {
            continue;
        };
        if lo <= hi && hi - lo <= 64 {
            for i in lo..=hi {
                out.push(format!("{}{}", m.prefix, i));
            }
            consumed = consumed.replace(&m.text, " ");
        }
    }
    for t in find_tokens(&consumed) {
        if t.suffix.is_empty() {
            out.push(t.base);
            continue;
        }
        for part in t.suffix.split('/') {
            let num = part.replace("-S", "").replace('S', "");
            out.push(format!("{}-S{}", t.base, num));
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for i in out {
        if !seen.contains(&i) {
            seen.push(i);
        }
    }
    seen
}

/// `(status, error)`. `status` is `DONE` or `OPEN`.
fn classify_status(cell: &str) -> (Option<&'static str>, Option<String>) {
    let text = strip_md(cell).to_lowercase();
    if text.is_empty() {
        return (None, Some("empty status cell".to_string()));
    }
    let is_done = DONE_WORDS.iter().any(|w| has_word(&text, w));
    let is_open = OPEN_WORDS.iter().any(|w| has_word(&text, w));
    if is_done && is_open {
        return (
            None,
            Some(format!(
                "status asserts DONE and OPEN at once: {}",
                py_repr(cell.trim())
            )),
        );
    }
    if is_done {
        return (Some("DONE"), None);
    }
    if is_open {
        return (Some("OPEN"), None);
    }
    (
        None,
        Some(format!(
            "unrecognised status vocabulary: {}",
            py_repr(cell.trim())
        )),
    )
}

fn split_row(line: &str) -> Vec<String> {
    let mut body = line.trim();
    if let Some(rest) = body.strip_prefix('|') {
        body = rest;
    }
    if let Some(rest) = body.strip_suffix('|') {
        body = rest;
    }
    body.split('|').map(|c| c.trim().to_string()).collect()
}

/// `re.fullmatch(r":?-{2,}:?", cell)` for every cell.
fn is_separator(line: &str) -> bool {
    let cells = split_row(line);
    if cells.is_empty() {
        return false;
    }
    cells.iter().all(|c| {
        let cs: Vec<char> = c.chars().collect();
        let mut i = 0;
        if cs.first() == Some(&':') {
            i += 1;
        }
        let start = i;
        while i < cs.len() && cs[i] == '-' {
            i += 1;
        }
        if i - start < 2 {
            return false;
        }
        if cs.get(i) == Some(&':') {
            i += 1;
        }
        i == cs.len()
    })
}

// ═════════════════════════════ doc parsing ═════════════════════════════════

/// One finding. `vacuous` marks the class the Python conflates into exit 1 but
/// this crate's `exit` module keeps separate: the gate could not honestly
/// evaluate, as opposed to evaluating and disagreeing.
pub struct Finding {
    pub msg: String,
    /// True when the gate could not honestly evaluate — zero markdown files, a
    /// missing roadmap doc, a doc yielding zero rows, an unreadable file. The
    /// oracle's own ANTI-VACUOUS section names exactly this set.
    pub vacuous: bool,
}

impl Finding {
    fn violation(msg: String) -> Self {
        Self {
            msg,
            vacuous: false,
        }
    }
    fn vacuous(msg: String) -> Self {
        Self { msg, vacuous: true }
    }
}

/// bd-2m9: **THE 0/2/3/4 MAPPING LIVES HERE — do not delete it.**
///
/// While the Python oracle is still in the tree, `run` reproduces the oracle's
/// exit 1 with an empty stderr, because byte-exactness is the only mechanism
/// that can distinguish a port BUG from an intended FIX, and an exit-code delta
/// on every RED case blinds it (controller decision, 2026-08-14). `check.sh`
/// invokes every gate as `cmd || fail "..."`, which reads only zero vs
/// non-zero, so nothing downstream can observe the difference today.
///
/// This function is the classification that decision defers, kept live and
/// tested so the later commit has something to switch to rather than something
/// to re-derive: once the oracles are deleted, `run`'s
/// `std::process::exit(1)` becomes `Err(GateError::violation(..))` or
/// `Err(GateError::error(..))` exactly as this returns.
///
/// The split matters because `crate::exit` reserves 4 for "a deliverable that
/// was never checked" — the class the oracle collapses into the same 1 it uses
/// for "checked, and the docs disagree".
pub fn crate_exit_code(findings: &[Finding]) -> u8 {
    if findings.is_empty() {
        crate::exit::OK
    } else if findings.iter().any(|f| f.vacuous) {
        crate::exit::ERROR
    } else {
        crate::exit::VIOLATION
    }
}

struct Row {
    id: String,
    /// Never absent. A row whose status could not be read is an error, not a
    /// row — see the DECISION in the module header (bd-hw3).
    status: &'static str,
    doc: &'static str,
    line: usize,
}

/// The status a milestone row declares, or the reason it declares none.
///
/// Keeping these three cases apart IS the bd-hw3 fix. The old code had two
/// branches and let the third fall through the second:
///
/// - the table has a Status column and this row REACHES it → classify the cell;
/// - the table has a Status column and this row is TOO SHORT → RED. The row
///   declares no status, and borrowing the heading's would be a guess;
/// - the table has NO Status column → the status-bearing heading declares it for
///   every row (the PHASE-NEXT shape). The guard in `parse_doc` has already
///   `continue`d the table when the heading is not a status either, so the
///   `ok_or_else` below is unreachable — and fails closed if that ever changes.
fn row_status(
    cells: &[String],
    status_col: Option<usize>,
    heading_status: Option<&'static str>,
    raw_line: &str,
) -> Result<&'static str, String> {
    match status_col {
        Some(c) if c < cells.len() => match classify_status(&cells[c]) {
            (Some(s), _) => Ok(s),
            (None, Some(e)) => Err(e),
            (None, None) => Err("unclassifiable status cell".to_string()),
        },
        Some(c) => Err(format!(
            "row is shorter than its Status column (has {} cell(s), Status is column {}): {}",
            cells.len(),
            c + 1,
            py_repr(raw_line.trim())
        )),
        None => heading_status.ok_or_else(|| "table declares no status".to_string()),
    }
}

fn parse_doc(path: &Path, rel: &'static str) -> (Vec<Row>, Vec<Finding>) {
    let mut rows: Vec<Row> = Vec::new();
    let mut errors: Vec<Finding> = Vec::new();
    if !path.is_file() {
        errors.push(Finding::vacuous(format!(
            "{rel}: roadmap doc missing (cannot verify agreement)"
        )));
        return (rows, errors);
    }
    let raw = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            // The Python has no handler here either; it would abort. Surface it
            // as an honest ERROR rather than pretending the doc agreed.
            errors.push(Finding::vacuous(format!("{rel}: unreadable ({e})")));
            return (rows, errors);
        }
    };
    let text = match String::from_utf8(raw) {
        Ok(t) => t,
        Err(e) => {
            errors.push(Finding::vacuous(format!(
                "{rel}: not valid UTF-8 ({}) — refusing to pass unparsed",
                py_utf8_error(e.as_bytes())
            )));
            return (rows, errors);
        }
    };
    let lines = py_splitlines(&text);

    let mut heading = String::new();
    let mut i = 0usize;
    let mut n_tables = 0usize;
    while i < lines.len() {
        let line = lines[i];
        if line.trim_start().starts_with('#') {
            heading = line.trim_start_matches('#').trim().to_string();
            i += 1;
            continue;
        }
        if !line.trim_start().starts_with('|') {
            i += 1;
            continue;
        }

        let mut block: Vec<(usize, &str)> = Vec::new();
        while i < lines.len() && lines[i].trim_start().starts_with('|') {
            block.push((i + 1, lines[i]));
            i += 1;
        }
        if block.len() < 3 || !is_separator(block[1].1) {
            continue;
        }

        let header: Vec<String> = split_row(block[0].1)
            .iter()
            .map(|h| h.to_lowercase().trim().to_string())
            .collect();
        if header.is_empty() || !MILESTONE_KEY_HEADERS.contains(&header[0].as_str()) {
            continue;
        }

        let status_col = header.iter().position(|h| h == "status");
        let (heading_status, _) = classify_status(&heading);
        if status_col.is_none() && heading_status.is_none() {
            // Milestone-keyed table with no status column and no status-bearing
            // heading: nothing to compare. Record it so a table that silently
            // stopped declaring status is visible in the report.
            errors.push(Finding::violation(format!(
                "{rel}:{}: milestone table under heading {} declares no status (no Status column, heading is not a status)",
                block[0].0,
                py_repr(&heading)
            )));
            continue;
        }

        n_tables += 1;
        let mut seen_in_table: Vec<(String, usize)> = Vec::new();
        for (lineno, raw_line) in block.iter().skip(2) {
            let cells = split_row(raw_line);
            if cells.is_empty() {
                continue;
            }
            let ids = milestone_ids(&cells[0]);
            if ids.is_empty() {
                continue;
            }
            let status = match row_status(&cells, status_col, heading_status, raw_line) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(Finding::violation(format!("{rel}:{lineno}: {e}")));
                    continue;
                }
            };
            for mid in ids {
                if let Some((_, first)) = seen_in_table.iter().find(|(k, _)| *k == mid) {
                    errors.push(Finding::violation(format!(
                        "{rel}:{lineno}: milestone {mid} appears twice in the same table (first at line {first}) — a table cannot state two truths about one milestone"
                    )));
                    continue;
                }
                seen_in_table.push((mid.clone(), *lineno));
                rows.push(Row {
                    id: mid,
                    status,
                    doc: rel,
                    line: *lineno,
                });
            }
        }
    }

    if n_tables == 0 {
        errors.push(Finding::vacuous(format!(
            "{rel}: zero milestone tables parsed (empty scan set is an ERROR, not a pass)"
        )));
    } else if rows.is_empty() {
        errors.push(Finding::vacuous(format!(
            "{rel}: milestone tables yielded zero rows (vacuous)"
        )));
    }
    (rows, errors)
}

// ═══════════════════════════ markdown discovery ════════════════════════════

/// CPython `PurePosixPath.__lt__` compares the parts tuple, NOT the joined
/// string — `a/b.md` sorts before `a-b/c.md`. Sorting by the raw path bytes
/// would reorder the publication findings.
fn parts_key(p: &Path) -> Vec<String> {
    p.components()
        .map(|c| match c {
            Component::RootDir => "/".to_string(),
            Component::Prefix(pf) => pf.as_os_str().to_string_lossy().into_owned(),
            Component::CurDir => ".".to_string(),
            Component::ParentDir => "..".to_string(),
            Component::Normal(s) => s.to_string_lossy().into_owned(),
        })
        .collect()
}

fn sort_paths(mut v: Vec<PathBuf>) -> Vec<PathBuf> {
    v.sort_by_key(|a| parts_key(a));
    v
}

fn dedupe_paths(v: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen: Vec<PathBuf> = Vec::new();
    for p in v {
        if !seen.contains(&p) {
            seen.push(p);
        }
    }
    seen
}

fn git_md_paths(root: &Path) -> Option<Vec<PathBuf>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "*.md",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    Some(
        py_splitlines(&text)
            .into_iter()
            .filter(|p| !p.trim().is_empty())
            .map(|p| root.join(p))
            .collect(),
    )
}

fn rglob_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let Ok(md) = std::fs::symlink_metadata(&p) else {
            continue;
        };
        if md.is_dir() {
            // `**` does not descend through symlinked directories.
            rglob_md(&p, out);
            continue;
        }
        let matches = p
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n.ends_with(".md"));
        if matches && p.is_file() {
            out.push(p);
        }
    }
}

/// Tracked + untracked-not-ignored `*.md` under `root`; filesystem fallback.
fn markdown_files(root: &Path) -> Vec<PathBuf> {
    if let Some(paths) = git_md_paths(root) {
        let paths: Vec<PathBuf> = paths.into_iter().filter(|p| p.is_file()).collect();
        if !paths.is_empty() {
            return sort_paths(dedupe_paths(paths));
        }
    }
    let mut v = Vec::new();
    rglob_md(root, &mut v);
    sort_paths(v)
}

// ══════════════════════════ publication scanning ═══════════════════════════

fn matches_alt_at(cs: &[char], pos: usize, alt: &str) -> Option<usize> {
    let a: Vec<char> = alt.chars().collect();
    if pos + a.len() > cs.len() {
        return None;
    }
    if cs[pos..pos + a.len()] == a[..] {
        Some(pos + a.len())
    } else {
        None
    }
}

/// `\b<alt>\b` where `<alt>` starts and ends with word characters.
fn matches_word_alt_at(cs: &[char], pos: usize, alt: &str) -> Option<usize> {
    if !at_word_boundary(cs, pos) {
        return None;
    }
    let end = matches_alt_at(cs, pos, alt)?;
    if end == cs.len() || !is_word_char(cs[end]) {
        Some(end)
    } else {
        None
    }
}

/// `[^.\n]{0,60}?` — the newly-bridged character must not be `.` or a newline.
fn bridge_ok(cs: &[char], from: usize, k: usize) -> bool {
    if k == 0 {
        return true;
    }
    let c = cs[from + k - 1];
    c != '.' && c != '\n'
}

/// Which of the five `PENDING_PUBLICATION_PATTERNS` (if any) fires on this
/// line, in the original's order — the Python `break`s on the first hit.
///
/// `re.IGNORECASE` is honoured for ASCII only; every alternation in the original
/// is pure ASCII, so the difference is confined to inputs where a non-ASCII
/// codepoint case-folds onto an ASCII one (`K`, `ſ`).
fn publication_hit(line: &str) -> Option<&'static str> {
    let low = line.to_ascii_lowercase();
    let cs: Vec<char> = low.chars().collect();
    let n = cs.len();

    // (1) FLIP ... STUCK
    for i in 0..n {
        for alt in FLIP_ALTS {
            let Some(e) = matches_alt_at(&cs, i, alt) else {
                continue;
            };
            for k in 0..=BRIDGE {
                if e + k > n {
                    break;
                }
                if !bridge_ok(&cs, e, k) {
                    break;
                }
                if STUCK_ALTS
                    .iter()
                    .any(|s| matches_word_alt_at(&cs, e + k, s).is_some())
                {
                    return Some(WHY_NOT_DONE);
                }
            }
        }
    }

    // (2) STUCK ... FLIP
    for i in 0..n {
        for alt in STUCK_ALTS {
            let Some(e) = matches_word_alt_at(&cs, i, alt) else {
                continue;
            };
            for k in 0..=BRIDGE {
                if e + k > n {
                    break;
                }
                if !bridge_ok(&cs, e, k) {
                    break;
                }
                if FLIP_ALTS
                    .iter()
                    .any(|f| matches_alt_at(&cs, e + k, f).is_some())
                {
                    return Some(WHY_NOT_DONE);
                }
            }
        }
    }

    // (3) \bpublic repo:\s*\**\s*no\b
    for i in 0..n {
        if !at_word_boundary(&cs, i) {
            continue;
        }
        let Some(mut p) = matches_alt_at(&cs, i, "public repo:") else {
            continue;
        };
        while p < n && cs[p].is_whitespace() {
            p += 1;
        }
        while p < n && cs[p] == '*' {
            p += 1;
        }
        while p < n && cs[p].is_whitespace() {
            p += 1;
        }
        let Some(end) = matches_alt_at(&cs, p, "no") else {
            continue;
        };
        if end == n || !is_word_char(cs[end]) {
            return Some(WHY_AUDIT_NO);
        }
    }

    // (4) \bawaiting josh\b
    for i in 0..n {
        if matches_word_alt_at(&cs, i, "awaiting josh").is_some() {
            return Some(WHY_AWAITING_JOSH);
        }
    }

    // (5) \bflip is a human call\b
    for i in 0..n {
        if matches_word_alt_at(&cs, i, "flip is a human call").is_some() {
            return Some(WHY_HUMAN_CALL);
        }
    }

    None
}

/// CPython's `UnicodeDecodeError` message text for a utf-8 decode failure.
fn py_utf8_error(bytes: &[u8]) -> String {
    let err = match std::str::from_utf8(bytes) {
        Ok(_) => return String::new(),
        Err(e) => e,
    };
    let start = err.valid_up_to();
    match err.error_len() {
        Some(len) => {
            let b = bytes[start];
            let reason = if (0xc2..=0xf4).contains(&b) {
                "invalid continuation byte"
            } else {
                "invalid start byte"
            };
            if len == 1 {
                format!("'utf-8' codec can't decode byte 0x{b:02x} in position {start}: {reason}")
            } else {
                format!(
                    "'utf-8' codec can't decode bytes in position {start}-{}: {reason}",
                    start + len - 1
                )
            }
        }
        None => {
            let len = bytes.len() - start;
            if len == 1 {
                format!(
                    "'utf-8' codec can't decode byte 0x{:02x} in position {start}: unexpected end of data",
                    bytes[start]
                )
            } else {
                format!(
                    "'utf-8' codec can't decode bytes in position {start}-{}: unexpected end of data",
                    bytes.len() - 1
                )
            }
        }
    }
}

fn scan_publication(root: &Path) -> (usize, Vec<Finding>) {
    let mut errors: Vec<Finding> = Vec::new();
    let files = markdown_files(root);
    if files.is_empty() {
        return (
            0,
            vec![Finding::vacuous(
                "zero markdown files scanned for publication truth (empty scan set is an ERROR, not a pass)"
                    .to_string(),
            )],
        );
    }
    if !REPO_PUBLIC {
        return (files.len(), errors);
    }
    for path in &files {
        let text = match std::fs::read(path) {
            Ok(b) => match String::from_utf8(b) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(Finding::vacuous(format!(
                        "{}: unreadable ({}) — refusing to pass unscanned",
                        path.display(),
                        py_utf8_error(e.as_bytes())
                    )));
                    continue;
                }
            },
            Err(e) => {
                errors.push(Finding::vacuous(format!(
                    "{}: unreadable ({e}) — refusing to pass unscanned",
                    path.display()
                )));
                continue;
            }
        };
        let rel = path.strip_prefix(root).unwrap_or(path);
        for (idx, line) in py_splitlines(&text).into_iter().enumerate() {
            if let Some(why) = publication_hit(line) {
                errors.push(Finding::violation(format!(
                    "{}:{}: {why} — repo has been public since {REPO_PUBLIC_SINCE} ({REPO_PUBLIC_EVIDENCE}): {}",
                    rel.display(),
                    idx + 1,
                    py_repr(&first_chars(line.trim(), 120))
                )));
            }
        }
    }
    (files.len(), errors)
}

// ════════════════════════════════ the gate ═════════════════════════════════

/// Python's `Path.resolve()` (non-strict): canonicalise where possible, else
/// absolutise and collapse `.` / `..` textually.
fn py_resolve(p: &Path) -> PathBuf {
    if let Ok(c) = p.canonicalize() {
        return c;
    }
    let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        base.join(p)
    };
    let mut out = PathBuf::new();
    for c in joined.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn scan_root(ctx: &GateCtx) -> Result<PathBuf, GateError> {
    let mut explicit: Option<PathBuf> = None;
    let mut i = 0usize;
    while i < ctx.args.len() {
        match ctx.args[i].as_str() {
            "--repo-root" => {
                let Some(v) = ctx.args.get(i + 1) else {
                    return Err(GateError::usage("--repo-root needs a directory"));
                };
                explicit = Some(PathBuf::from(v));
                i += 2;
            }
            other => {
                return Err(GateError::usage(format!(
                    "unknown argument {other:?}; known: --repo-root <dir>"
                )));
            }
        }
    }
    // The Python's DEFAULT_ROOT is the ENGINE's PARENT — the git repo root —
    // because two of the three roadmap docs live above the engine directory.
    Ok(match explicit {
        Some(p) => py_resolve(&p),
        None => ctx
            .root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| ctx.root.clone()),
    })
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let root = scan_root(ctx)?;

    let mut errors: Vec<Finding> = Vec::new();
    let mut all_rows: Vec<Row> = Vec::new();
    for rel in MILESTONE_DOCS {
        let (rows, errs) = parse_doc(&root.join(rel), rel);
        all_rows.extend(rows);
        errors.extend(errs);
    }

    if all_rows.is_empty() {
        errors.push(Finding::vacuous(
            "zero milestone rows parsed across all roadmap docs (empty scan set is an ERROR, not a pass)"
                .to_string(),
        ));
    }

    // Ordered by id, matching the Python's `sorted(by_id.items())`; the row
    // vectors keep first-seen order, matching dict-of-list insertion order.
    let mut by_id: std::collections::BTreeMap<String, Vec<&Row>> =
        std::collections::BTreeMap::new();
    for row in &all_rows {
        by_id.entry(row.id.clone()).or_default().push(row);
    }

    let mut conflicts = 0usize;
    for (mid, rows) in &by_id {
        let mut statuses: Vec<&str> = rows.iter().map(|r| r.status).collect();
        statuses.sort_unstable();
        statuses.dedup();
        if statuses.len() > 1 {
            conflicts += 1;
            let where_: Vec<String> = rows
                .iter()
                .map(|r| format!("{}:{}={}", r.doc, r.line, r.status))
                .collect();
            errors.push(Finding::violation(format!(
                "milestone {mid} has conflicting status across the roadmap docs: {}",
                where_.join(" · ")
            )));
        }
    }

    let (n_md, pub_errors) = scan_publication(&root);
    errors.extend(pub_errors);

    let status = if errors.is_empty() { "PASS" } else { "FAIL" };
    let mut out = String::new();
    out.push_str(status);
    out.push('\n');
    out.push_str(&format!("  root={}\n", root.display()));
    out.push_str(&format!("  roadmap_docs={}\n", MILESTONE_DOCS.len()));
    out.push_str(&format!("  milestone_rows={}\n", all_rows.len()));
    out.push_str(&format!("  milestone_ids={}\n", by_id.len()));
    out.push_str(&format!("  conflicts={conflicts}\n"));
    out.push_str(&format!("  markdown_scanned={n_md}\n"));
    out.push_str(&format!(
        "  repo_public={} since {REPO_PUBLIC_SINCE}\n",
        if REPO_PUBLIC { "True" } else { "False" }
    ));
    for (mid, rows) in &by_id {
        let mut seen: Vec<&str> = rows.iter().map(|r| r.status).collect();
        seen.sort_unstable();
        seen.dedup();
        out.push_str(&format!(
            "    {mid}: {} ({} row(s))\n",
            seen.join(","),
            rows.len()
        ));
    }

    if !errors.is_empty() {
        out.push_str("  failures:\n");
        for e in errors.iter().take(MAX_REPORT) {
            out.push_str(&format!("    - {}\n", e.msg));
        }
        if errors.len() > MAX_REPORT {
            out.push_str(&format!("    ... +{} more\n", errors.len() - MAX_REPORT));
        }
        print!("{out}");
        // `process::exit` runs no destructors, so the buffer must go out first.
        let _ = std::io::stdout().flush();

        // bd-2m9: the oracle exits 1 and writes NOTHING to stderr; this port
        // reproduces exactly that while the oracle is the differential
        // reference. `crate_exit_code(&errors)` above is the 2-vs-4 split this
        // line becomes once the oracle is deleted — that is the one place to
        // edit, and it is unit-tested so the behaviour cannot rot meanwhile.
        debug_assert!(matches!(
            crate_exit_code(&errors),
            crate::exit::VIOLATION | crate::exit::ERROR
        ));
        std::process::exit(1);
    }

    out.push_str("  roadmap GREEN (milestone status agrees; publication truth holds)\n");
    print!("{out}");
    let _ = std::io::stdout().flush();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repr_matches_cpython_for_the_shapes_this_gate_emits() {
        assert_eq!(py_repr("mostly there"), "'mostly there'");
        assert_eq!(py_repr("it's"), "\"it's\"");
        assert_eq!(py_repr("it's \"q\""), "'it\\'s \"q\"'");
        assert_eq!(py_repr("a\\b"), "'a\\\\b'");
        assert_eq!(py_repr("a\tb"), "'a\\tb'");
        assert_eq!(py_repr("\u{7f}"), "'\\x7f'");
        assert_eq!(py_repr("M0–M2 · scaffold"), "'M0–M2 · scaffold'");
    }

    #[test]
    fn ranges_and_sub_milestones() {
        assert_eq!(milestone_ids("**M0–M2**"), vec!["M0", "M1", "M2"]);
        assert_eq!(milestone_ids("**M9-S1/S2**"), vec!["M9-S1", "M9-S2"]);
        assert_eq!(milestone_ids("**V11**"), vec!["V11"]);
        assert_eq!(milestone_ids("Learn v2"), Vec::<String>::new());
        assert_eq!(milestone_ids("M8"), vec!["M8"]);
        assert_eq!(milestone_ids("M1-M2 and M5"), vec!["M1", "M2", "M5"]);
    }

    #[test]
    fn status_vocabulary_is_fail_closed() {
        assert_eq!(classify_status("**DONE**").0, Some("DONE"));
        assert_eq!(classify_status("**open**").0, Some("OPEN"));
        assert!(classify_status("mostly there").1.is_some());
        assert!(classify_status("").1.is_some());
        assert!(classify_status("done but blocked").1.is_some());
    }

    /// bd-hw3: the three row-status cases, kept apart. The middle one is the
    /// repaired defect — it used to silently borrow the heading's status (or
    /// `None`, when the heading was not a status word) and record a row the gate
    /// had not read.
    #[test]
    fn a_row_too_short_for_its_status_column_is_red() {
        let cells = ["M2".to_string(), "ragged row".to_string()];
        let raw = "| M2 | ragged row |";

        // Reaches the Status column: classified normally.
        let full = ["M1".to_string(), "a".to_string(), "DONE".to_string()];
        assert_eq!(
            row_status(&full, Some(2), None, "| M1 | a | DONE |"),
            Ok("DONE")
        );

        // Too short, non-status heading — the crash input. Names the shortfall
        // and the row, and does NOT fall back to the heading.
        let err = row_status(&cells, Some(2), None, raw).unwrap_err();
        assert_eq!(
            err,
            "row is shorter than its Status column (has 2 cell(s), Status is column 3): \
             '| M2 | ragged row |'"
        );

        // Too short under a status-BEARING heading is equally RED: the table
        // declares a Status column, so the row owes one. Borrowing is a guess.
        assert!(row_status(&cells, Some(2), Some("DONE"), raw).is_err());

        // No Status column at all: the heading declares it. Preserved.
        assert_eq!(row_status(&cells, None, Some("DONE"), raw), Ok("DONE"));

        // Unreachable in `parse_doc`, but fail-closed if the guard ever moves.
        assert!(row_status(&cells, None, None, raw).is_err());
    }

    #[test]
    fn splitlines_follows_cpython() {
        assert_eq!(py_splitlines("a\nb\n"), vec!["a", "b"]);
        assert_eq!(py_splitlines("a\r\nb"), vec!["a", "b"]);
        assert_eq!(py_splitlines("a\u{b}b"), vec!["a", "b"]);
        assert_eq!(py_splitlines(""), Vec::<&str>::new());
    }

    #[test]
    fn publication_patterns_fire_where_the_original_does() {
        assert_eq!(
            publication_hit("The visibility flip is blocked pending a human decision."),
            Some(WHY_NOT_DONE)
        );
        assert_eq!(publication_hit("Public repo: **no**"), Some(WHY_AUDIT_NO));
        assert_eq!(
            publication_hit("Awaiting Josh on this"),
            Some(WHY_AWAITING_JOSH)
        );
        assert_eq!(
            publication_hit("the flip is a human call"),
            Some(WHY_HUMAN_CALL)
        );
        assert_eq!(publication_hit("the repo is public and that is that"), None);
    }

    /// bd-2m9: the deferred classification, kept honest while `run` still exits
    /// 1 for byte-exactness with the oracle. If this drifts, the later commit
    /// that flips the crate to 0/2/3/4 inherits a wrong mapping.
    #[test]
    fn crate_exit_code_splits_vacuous_from_violation() {
        assert_eq!(crate_exit_code(&[]), crate::exit::OK);
        assert_eq!(
            crate_exit_code(&[Finding::violation("docs disagree".into())]),
            crate::exit::VIOLATION
        );
        assert_eq!(
            crate_exit_code(&[Finding::vacuous("zero markdown files scanned".into())]),
            crate::exit::ERROR
        );
        // One unreadable input taints the whole run: a scan that could not
        // honestly complete must not report as a mere disagreement.
        assert_eq!(
            crate_exit_code(&[
                Finding::violation("docs disagree".into()),
                Finding::vacuous("roadmap doc missing".into()),
            ]),
            crate::exit::ERROR
        );
    }

    /// Every anti-vacuous message the gate can emit is classified as such. A
    /// finding that says "empty scan set is an ERROR" while being filed as an
    /// ordinary violation is the exact confusion `crate::exit` exists to stop.
    #[test]
    fn anti_vacuous_messages_are_filed_as_vacuous() {
        for msg in [
            "CHARTER.md: roadmap doc missing (cannot verify agreement)",
            "README.md: zero milestone tables parsed (empty scan set is an ERROR, not a pass)",
            "README.md: milestone tables yielded zero rows (vacuous)",
            "zero markdown files scanned for publication truth",
            "/x/broken.md: unreadable (...) — refusing to pass unscanned",
        ] {
            assert_eq!(
                crate_exit_code(&[Finding::vacuous(msg.to_string())]),
                crate::exit::ERROR,
                "{msg}"
            );
        }
    }

    #[test]
    fn path_sort_uses_parts_not_bytes() {
        let v = sort_paths(vec![
            PathBuf::from("/r/a-b/c.md"),
            PathBuf::from("/r/a/b.md"),
            PathBuf::from("/r/a.md"),
        ]);
        // CPython 3.14 `sorted()` on these three PosixPaths returns exactly this.
        assert_eq!(
            v,
            vec![
                PathBuf::from("/r/a/b.md"),
                PathBuf::from("/r/a-b/c.md"),
                PathBuf::from("/r/a.md"),
            ]
        );
    }
}
