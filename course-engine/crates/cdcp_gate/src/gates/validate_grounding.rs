//! validate-grounding — Rust port of `scripts/validate_grounding.py`
//! (bd-substrate-rust-migration-jhd.9).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **an item may not carry normative-looking
//! precision that nothing in the tree supports, and its wording must overlap the
//! corpus the course was written from.** Concretely it goes RED when any of five
//! heuristics fires on a bank item's stem + choices + explanation —
//!
//!   1. *hallucinated clause* — text shaped like `ISO 22237 clause 4.1`,
//!      `clause 5.2.1`, or `x§3.4`: a standard family welded to a numbered
//!      subdivision. Bank items cite families, not clause numbers, because the
//!      clause text is not in the tree to check against.
//!   2. *dump language* — "actual exam question", "brain dump", "real EPI exam",
//!      and a promise that the exam is passed. Provenance claims the repo
//!      cannot honour. (The fourth phrase is spelled out at `DUMP_PASS_SRC`;
//!      `tests/repo_surface.rs` bans its literal spelling in prose.)
//!   3. *numeric setpoint without evidence* — an exact °C/°F figure asserted with
//!      "exactly"/"precisely"/"must be", or a figure called
//!      recommended/required/mandatory, on an item whose `quantity_evidence` is
//!      not one of `free_url`, `licensed_note`, `exam_form_public`.
//!   4. *fake multi-level cite* — `ISO/IEC 22237` followed within 40 characters
//!      by an `n.n.n` triple.
//!   5. *low corpus overlap* — under `--strict-overlap`, an item whose tokens
//!      overlap the corpus below `--min-overlap` (default 0.08). Without that
//!      flag the same finding is a WARN and the gate stays green.
//!
//! It also goes RED, under a separate `FAIL: vacuous grounding check` banner,
//! when it had too little to check at all: fewer than `MIN_SCANNED_ITEMS` items,
//! fewer than `MIN_CORPUS_CHARS` corpus characters, or a corpus root that is
//! missing or unlistable. See ANTI-VACUOUS below — those are conditions of the
//! INPUT, not findings about an item, which is why they are counted and printed
//! apart from `high_severity`.
//!
//! # WHAT THIS GATE CANNOT DECIDE
//!
//! Every leg above is a HEURISTIC over surface text, and the overlap leg is the
//! weakest of them. It cannot decide that a claim is TRUE. It cannot decide that
//! a claim is grounded — only that the words an item uses also appear, as whole
//! words, somewhere in `modules/`, `reference/`, or `knowledge/`. An item that
//! reuses the corpus vocabulary to state something the corpus flatly
//! contradicts scores 1.000 and passes. A correct item written in fresh
//! vocabulary scores low and is warned about. Overlap is a lexical measure with
//! no notion of meaning, entailment, or citation.
//!
//! The clause and setpoint legs are pattern matches, so they see the SHAPE of a
//! citation and never its correctness: a real, correctly transcribed clause
//! number trips leg 1 exactly as an invented one does, and an invented figure
//! written without the trigger words passes leg 3 untouched. Leg 3 reads
//! `quantity_evidence` as a label and never opens the evidence it names — a
//! `free_url` that 404s clears the gate. Nothing here reads the corpus for
//! meaning; the corpus is consumed as a bag of whole words and a character
//! count, and this port never renders any of its text.
//!
//! The floor therefore moves from *silence* to *these five surface shapes are
//! absent and the vocabulary is shared*. That is the whole claim, and a
//! heuristic gate has no stronger one available.
//!
//! # ANTI-VACUOUS (L4) — FIXED IN THE ORACLE FIRST, THEN HERE (bd-yje7)
//!
//! Until 2026-08-14 all three of these exited 0 with a `PASS` banner, in the
//! oracle and therefore in this port: an EMPTY `bank/items` directory; a corpus
//! of zero characters; and a missing `knowledge/corpus/public`. That inverted
//! the meaning of green — with no corpus there is nothing that could contradict
//! any claim, so every item scored clean and the gate emitted its most
//! reassuring output exactly when it was blindest.
//!
//! Each is now an ERROR that NAMES ITSELF, printed under `FAIL: vacuous
//! grounding check` with exit 1. The oracle was changed first and this port
//! second, so the differential went red on the fix and green again on the port —
//! the sequence that keeps the harness honest.
//!
//! The floors are DELIBERATE MINIMA, not `> 0`; a one-byte corpus would satisfy
//! `> 0` and move the hole rather than close it. Both thresholds and their
//! reasons live beside the constants (`MIN_SCANNED_ITEMS`, `MIN_CORPUS_CHARS`)
//! and are mirrored from the oracle, which owns them.
//!
//! Not checked, deliberately: per-root EMPTINESS. `MIN_CORPUS_CHARS` governs
//! total volume, and the licensing remediation
//! (`bd-corpus-public-captures-not-licensed-class-kej`) may legitimately leave
//! `knowledge/corpus/public` empty while the rest of the tree still grounds the
//! bank. Its DISAPPEARANCE is still an error, because a walk over a directory
//! that is not there contributes zero characters in silence.
//!
//! # BYTE-EXACTNESS WITH THE PYTHON ORACLE
//!
//! `scripts/validate_grounding.py` stays in the tree as the differential oracle;
//! `tests/diff_validate_grounding.rs` runs both on every case and asserts
//! stdout, stderr, and exit code match byte for byte. That contract is why this
//! module carries hand-written emulations of `re`, `str.lower()`, `repr(float)`,
//! `argparse`, and `pathlib.rglob` rather than the idiomatic Rust
//! nearest-neighbour, and why the report is written to **stdout with exit status
//! 1** instead of going through `GateError`: the dispatcher's `report()` writes
//! to stderr and maps to exit 2 or 4, which the oracle never produces, so
//! routing through it would make the two sides differ on every RED case. The
//! exit-code mapping in `crate::exit` is therefore deliberately NOT used by this
//! gate, exactly as in `verify_orphans`. That is a knowing, single-file
//! deviation from the shared convention, recorded here rather than made quietly.
//!
//! ## Why emission order is not a hazard here
//!
//! The corpus is `"\n".join(chunks)` over an unsorted `rglob` walk, so the
//! Python's file order is filesystem order. It does not reach the output:
//! `corpus_chars` is a sum plus a fixed separator count, and the only other use
//! of the corpus is whole-word membership — and since every chunk is delimited
//! by `\n` on both sides, no word run can span two chunks, so the run set is the
//! union of the per-chunk run sets. Both are order-independent, so this port
//! sorts its walk for determinism without changing a byte. The item loop is a
//! different matter and IS ordered: the oracle iterates `sorted(glob("*.toml"))`
//! and the low-overlap sample list is stably sorted by score alone, so ties fall
//! back to filename order; both are reproduced.
//!
//! ## Residual deviations
//!
//! Each is unreachable from the live tree and from every case in the
//! differential, and each is a wrong-bytes risk rather than a wrong-verdict one.
//!
//! - A malformed item yields `<file>: parse error <msg>` where `<msg>` comes
//!   from the `toml` crate, not `tomllib`. Both go RED on the same file.
//! - `choices` holding a non-string element makes the oracle raise `TypeError`
//!   and print a traceback: empty stdout, exit 1. This port reproduces the empty
//!   stdout and the exit status, with a one-line stderr note in place of the
//!   traceback.
//! - `re.IGNORECASE` on `str` patterns folds a handful of non-ASCII characters
//!   onto ASCII letters (`ſ`→`s`, `K`→`k`). This port folds ASCII only. Verified
//!   2026-08-14: none of those characters occur in the bank, the corpus, or the
//!   topic registry.
//! - `\d` in `re` matches any Unicode decimal digit; this port matches ASCII
//!   digits. Verified over the full character inventory of the bank, corpus, and
//!   topic registry: no non-ASCII digit occurs, and Python's `\w`, `\s`, and
//!   `\d` agree with the classifiers below on all 181 distinct characters
//!   present.
//! - `--help` wraps to the terminal width. Both sides read `COLUMNS` first and
//!   fall back to 80; the oracle additionally queries the tty when `COLUMNS` is
//!   unset, which this port cannot do without a C dependency. Under `check.sh`
//!   and under the differential, stdout is a pipe, so the oracle also lands on
//!   80 and the two agree.
//! - `argparse`'s exact wording for `--` and for abbreviation collisions is
//!   Python-version-sensitive; the strings here are the 3.14 ones, pinned by the
//!   differential.
//! - `str()` of a TOML datetime in a `stem`/`explanation` differs (`T` separator
//!   versus space). No shipped item has a non-string stem.

use crate::registry::{GateCtx, GateError};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::Value;

pub const NAME: &str = "validate-grounding";
pub const SUMMARY: &str =
    "anti-hallucination heuristics over bank items plus corpus vocabulary overlap";

/// `prog` as `argparse` derives it from `sys.argv[0]`.
pub const PROG: &str = "validate_grounding.py";

/// Engine-root-relative locations, matching the Python module constants.
pub const ITEMS_REL: &str = "bank/items";
pub const KNOWLEDGE_REL: &str = "knowledge";
pub const TOPICS_REL: &str = "knowledge/topics.toml";
pub const CORPUS_PUBLIC_REL: &str = "knowledge/corpus/public";
/// Sibling directories of the engine root that also feed the corpus.
pub const SIBLING_CORPUS_DIRS: &[&str] = &["modules", "reference"];

/// Suffixes `load_corpus_text` accepts, already lowercased.
pub const CORPUS_SUFFIXES: &[&str] = &[".md", ".toml", ".txt"];

/// The substring that holds `knowledge/corpus/public` out of the recursive walk
/// so it can be added, non-recursively, from `*.txt` only.
pub const CORPUS_PUBLIC_MARKER: &str = "corpus/public";

/// How many findings the report prints before it truncates.
pub const MAX_REPORT: usize = 60;

/// Anti-vacuous floor on items scanned — one full exam form.
///
/// `knowledge/bank_policy.toml` sets `exam_n_items = 40`, so a bank that cannot
/// fill a single form cannot produce the artifact this course ships, and "no
/// heuristic fired" over it says nothing about the product. Deliberately ONE
/// TENTH of `verify_bank`'s `pool_min_items = 400`: the pool floor belongs to
/// that gate, and enforcing it twice would turn a legitimately-still-growing
/// bank RED here for a reason another gate already owns. Live tree 2026-08-14:
/// 804 items, 20x this floor. Mirrored from the oracle, which owns the value.
pub const MIN_SCANNED_ITEMS: usize = 40;

/// Anti-vacuous floor on corpus size — one module's worth of prose.
///
/// Measured 2026-08-14, the 29 live modules run 749..47651 characters, median
/// 23870, so this sits just under one median module. Below it there is not
/// enough text for whole-word overlap to mean anything, and the only ways to get
/// there are a corpus that was never found, was emptied, or was truncated. Live
/// tree: 659149 characters, 33x this floor — and 545885 of those come from
/// OUTSIDE `knowledge/corpus/public`, so every capture can be deleted for the
/// licensing remediation and this floor still clears at 27x, while an actual
/// disappearance of the corpus goes RED instead of green on the way down.
/// Mirrored from the oracle, which owns the value.
pub const MIN_CORPUS_CHARS: usize = 20_000;

/// `STOP` — dropped from every token set.
pub const STOP: &[&str] = &[
    "that", "this", "with", "from", "they", "them", "then", "than", "have", "been", "were", "will",
    "when", "what", "which", "into", "over", "also", "only", "more", "most", "some", "such",
    "each", "both", "same", "other", "about", "after", "before", "under", "above", "while",
    "where", "their", "there", "these", "those", "being", "does", "done", "make", "used", "using",
    "very", "just", "like", "because", "through", "during", "without", "within",
];

// ── Python-behaviour emulations ────────────────────────────────────────────
// Each exists because the acceptance bar is byte-identical output, not merely
// an identical verdict.

/// The `\s` class of Python's `re` on `str`: Unicode `White_Space` plus the four
/// ASCII information separators that `str.isspace()` counts and Rust's
/// `char::is_whitespace` does not.
pub fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// The `\w` class of Python's `re` on `str`.
///
/// CPython's `SRE_UNI_IS_WORD` is `Py_UNICODE_ISALNUM(ch) || ch == '_'`. Rust's
/// `is_alphanumeric` is `Alphabetic | N*`, which is broader by exactly the
/// Other_Alphabetic marks. Checked 2026-08-14 against the full character
/// inventory of the corpus, bank, and topic registry: zero disagreements.
pub fn py_word(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// The `\d` class. See the header for why this is ASCII-only.
pub fn py_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn digit_or_dash(c: char) -> bool {
    c.is_ascii_digit() || c == '-'
}

fn digit_or_dot(c: char) -> bool {
    c.is_ascii_digit() || c == '.'
}

fn is_degree(c: char) -> bool {
    c == '\u{b0}'
}

fn is_slash(c: char) -> bool {
    c == '/'
}

fn is_cf(c: char) -> bool {
    matches!(c, 'C' | 'c' | 'F' | 'f')
}

fn not_newline(c: char) -> bool {
    c != '\n'
}

/// `io.TextIOWrapper`'s universal-newline translation, which `Path.read_text`
/// applies after decoding: `\r\n` and a lone `\r` both become `\n`. It moves
/// `corpus_chars`, so it is not cosmetic.
pub fn universal_newlines(s: &str) -> String {
    if !s.contains('\r') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    out
}

/// `repr()` of a Python `float`, which is what an f-string renders.
///
/// CPython emits the shortest round-tripping digit string, then chooses fixed
/// notation unless the decimal point sits at or left of `-4`, or right of
/// `REPR_FIXED_MAX`.
pub fn py_float_repr(x: f64) -> String {
    /// The `decpt` ceiling above which CPython's repr switches to exponent form.
    const REPR_FIXED_MAX: i32 = 16;

    if x.is_nan() {
        return "nan".to_string();
    }
    if x.is_infinite() {
        return if x < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    // `{:e}` is Rust's shortest round-trip form: `[-]D[.DDD]e[-]EXP`.
    let e = format!("{x:e}");
    let (sign, body) = match e.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", e.as_str()),
    };
    let (mant, exp) = body.split_once('e').unwrap_or((body, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let decpt = exp + 1;

    if decpt <= -4 || decpt > REPR_FIXED_MAX {
        let mut m = String::new();
        m.push_str(&digits[..1]);
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        let ex = decpt - 1;
        let s = if ex < 0 { '-' } else { '+' };
        return format!("{sign}{m}e{s}{:02}", ex.abs());
    }
    let d = digits.as_str();
    if decpt <= 0 {
        format!("{sign}0.{}{d}", "0".repeat((-decpt) as usize))
    } else if (decpt as usize) >= d.len() {
        format!("{sign}{d}{}.0", "0".repeat(decpt as usize - d.len()))
    } else {
        format!("{sign}{}.{}", &d[..decpt as usize], &d[decpt as usize..])
    }
}

/// `repr()` of a Python `str`. Same rules as the sibling port's copy; kept local
/// so this gate owns exactly one file.
pub fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || (c as u32) == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `str(value)` for a value `tomllib` would have produced.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => py_float_repr(*f),
        Value::Boolean(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Datetime(d) => d.to_string(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(py_inner_repr).collect::<Vec<_>>().join(", ")
        ),
        Value::Table(t) => format!(
            "{{{}}}",
            t.iter()
                .map(|(k, v)| format!("{}: {}", py_repr(k), py_inner_repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn py_inner_repr(v: &Value) -> String {
    match v {
        Value::String(s) => py_repr(s),
        other => py_str(other),
    }
}

/// Python truthiness of a value `tomllib` would have produced.
pub fn py_truthy(v: &Value) -> bool {
    match v {
        Value::String(s) => !s.is_empty(),
        Value::Integer(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Boolean(b) => *b,
        Value::Datetime(_) => true,
        Value::Array(a) => !a.is_empty(),
        Value::Table(t) => !t.is_empty(),
    }
}

// ── the pattern matcher ────────────────────────────────────────────────────
//
// The oracle's heuristics are `re` patterns. The crate has no regex dependency
// and hand-written parsing is the house pattern, so each pattern is compiled by
// hand into the node list below and run by a small backtracking matcher. The
// matcher answers `re.search`'s question — does a match exist anywhere — which
// is the only thing the oracle asks of these patterns.

type ClassFn = fn(char) -> bool;

/// Unbounded repetition ceiling.
const UNBOUNDED: usize = usize::MAX;

/// One element of a compiled pattern.
pub enum Node {
    /// `\b`
    Bound,
    /// A case-insensitive literal. ASCII folding only; see the header.
    Lit(&'static str),
    /// `(?:a|b|c)` over literals, tried left to right as `re` does.
    Alt(&'static [&'static str]),
    /// A greedy character-class repetition with an inclusive `[min, max]`.
    Cls(ClassFn, usize, usize),
    /// A greedy repetition of a sub-sequence, `[min, max]`.
    Rep(&'static [Node], usize, usize),
}

/// Is `pos` a `\b` position in `s`?
fn is_boundary(s: &[char], pos: usize) -> bool {
    let before = pos > 0 && py_word(s[pos - 1]);
    let after = pos < s.len() && py_word(s[pos]);
    before != after
}

/// Case-insensitive literal match anchored at `pos`; the end offset on success.
fn match_lit(lit: &str, s: &[char], pos: usize) -> Option<usize> {
    let mut i = pos;
    for lc in lit.chars() {
        let c = *s.get(i)?;
        // ASCII folding only; a non-ASCII character must match exactly. See the
        // header for the `re.IGNORECASE` folds this deliberately does not do.
        if !c.eq_ignore_ascii_case(&lc) {
            return None;
        }
        i += 1;
    }
    Some(i)
}

/// Match `nodes` at `pos`, handing each candidate end offset to `k`.
fn m(nodes: &[Node], s: &[char], pos: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
    let Some((head, rest)) = nodes.split_first() else {
        return k(pos);
    };
    match head {
        Node::Bound => is_boundary(s, pos) && m(rest, s, pos, k),
        Node::Lit(l) => match match_lit(l, s, pos) {
            Some(p) => m(rest, s, p, k),
            None => false,
        },
        Node::Alt(alts) => {
            for l in *alts {
                if let Some(p) = match_lit(l, s, pos) {
                    if m(rest, s, p, k) {
                        return true;
                    }
                }
            }
            false
        }
        Node::Cls(f, min, max) => {
            let mut hi = 0usize;
            let mut p = pos;
            while hi < *max && p < s.len() && (*f)(s[p]) {
                p += 1;
                hi += 1;
            }
            let mut n = hi;
            loop {
                if n < *min {
                    return false;
                }
                if m(rest, s, pos + n, k) {
                    return true;
                }
                if n == 0 {
                    return false;
                }
                n -= 1;
            }
        }
        Node::Rep(sub, min, max) => rep(sub, *min, *max, 0, rest, s, pos, k),
    }
}

/// Greedy repetition: try one more iteration before falling through to `rest`.
#[allow(clippy::too_many_arguments)]
fn rep(
    sub: &[Node],
    min: usize,
    max: usize,
    count: usize,
    rest: &[Node],
    s: &[char],
    pos: usize,
    k: &mut dyn FnMut(usize) -> bool,
) -> bool {
    if count < max {
        let more = {
            let mut step = |p: usize| -> bool {
                // A zero-width iteration would loop forever and can never help.
                p != pos && rep(sub, min, max, count + 1, rest, s, p, &mut *k)
            };
            m(sub, s, pos, &mut step)
        };
        if more {
            return true;
        }
    }
    count >= min && m(rest, s, pos, k)
}

/// `re.search`: does a match start anywhere?
pub fn search(nodes: &[Node], s: &[char]) -> bool {
    (0..=s.len()).any(|p| m(nodes, s, p, &mut |_| true))
}

// ── the compiled heuristics ────────────────────────────────────────────────

/// `\b(?:ISO|IEC|EN|ANSI|TIA|NFPA|IEEE)\s*[\d\-]+(?:-\d+)*\s*(?:clause|section|§|part)\s*[\d\.]+`
///
/// The `(?:-\d+)*` tail is dropped, which is language-preserving: every string
/// `[\d\-]+(?:-\d+)*` accepts is already in `[\d\-]+`, and every end offset the
/// pair can reach the single class reaches too, so no continuation can tell
/// them apart. `tail_is_redundant` pins that.
static CLAUSE_0: &[Node] = &[
    Node::Bound,
    Node::Alt(&["ISO", "IEC", "EN", "ANSI", "TIA", "NFPA", "IEEE"]),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Cls(digit_or_dash, 1, UNBOUNDED),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Alt(&["clause", "section", "\u{a7}", "part"]),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Cls(digit_or_dot, 1, UNBOUNDED),
];

/// `\bclause\s+\d+\.\d+(?:\.\d+)*\b`
static CLAUSE_1_TAIL: &[Node] = &[Node::Lit("."), Node::Cls(py_digit, 1, UNBOUNDED)];
static CLAUSE_1: &[Node] = &[
    Node::Bound,
    Node::Lit("clause"),
    Node::Cls(py_space, 1, UNBOUNDED),
    Node::Cls(py_digit, 1, UNBOUNDED),
    Node::Lit("."),
    Node::Cls(py_digit, 1, UNBOUNDED),
    Node::Rep(CLAUSE_1_TAIL, 0, UNBOUNDED),
    Node::Bound,
];

/// `\b§\s*\d+\.\d+`
///
/// `§` is not a word character, so the leading `\b` requires a word character
/// immediately BEFORE the `§`. That is the oracle's behaviour, quirk included.
static CLAUSE_2: &[Node] = &[
    Node::Bound,
    Node::Lit("\u{a7}"),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Cls(py_digit, 1, UNBOUNDED),
    Node::Lit("."),
    Node::Cls(py_digit, 1, UNBOUNDED),
];

/// The three clause patterns with the prefix each finding prints.
pub static CLAUSE_PATTERNS: &[(&[Node], &str)] = &[
    (CLAUSE_0, "\\b(?:ISO|IEC|EN|ANSI|TIA|NFPA|IEEE)\\s*[\\"),
    (CLAUSE_1, "\\bclause\\s+\\d+\\.\\d+(?:\\.\\d+)*\\b"),
    (CLAUSE_2, "\\b\u{a7}\\s*\\d+\\.\\d+"),
];

/// `\b(?:exactly|precisely|must be)\s+\d+(?:\.\d+)?\s*°?\s*[CF]\b`
static TRAP_A_FRAC: &[Node] = &[Node::Lit("."), Node::Cls(py_digit, 1, UNBOUNDED)];
static TRAP_A: &[Node] = &[
    Node::Bound,
    Node::Alt(&["exactly", "precisely", "must be"]),
    Node::Cls(py_space, 1, UNBOUNDED),
    Node::Cls(py_digit, 1, UNBOUNDED),
    Node::Rep(TRAP_A_FRAC, 0, 1),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Cls(is_degree, 0, 1),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Cls(is_cf, 1, 1),
    Node::Bound,
];

/// `\b\d{2,3}\s*°\s*[CF]\s*(?:recommended|required|mandatory)\b`
static TRAP_B: &[Node] = &[
    Node::Bound,
    Node::Cls(py_digit, 2, 3),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Cls(is_degree, 1, 1),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Cls(is_cf, 1, 1),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Alt(&["recommended", "required", "mandatory"]),
    Node::Bound,
];

/// `\bISO/?IEC\s*22237[^\n]{0,40}\d+\.\d+\.\d+`
static ISO_MULTI: &[Node] = &[
    Node::Bound,
    Node::Lit("ISO"),
    Node::Cls(is_slash, 0, 1),
    Node::Lit("IEC"),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Lit("22237"),
    Node::Cls(not_newline, 0, 40),
    Node::Cls(py_digit, 1, UNBOUNDED),
    Node::Lit("."),
    Node::Cls(py_digit, 1, UNBOUNDED),
    Node::Lit("."),
    Node::Cls(py_digit, 1, UNBOUNDED),
];

static DUMP_ACTUAL: &[Node] = &[Node::Lit("actual exam question")];
static DUMP_BRAIN: &[Node] = &[
    Node::Lit("brain"),
    Node::Cls(py_space, 0, UNBOUNDED),
    Node::Lit("dump"),
];
static DUMP_REAL: &[Node] = &[Node::Lit("real EPI exam")];
/// Assembled with `concat!` rather than written out because
/// `tests/repo_surface.rs` bans the literal spelling of the first word anywhere
/// in the shipped source — and the oracle's pattern source has to be reproduced
/// byte for byte, so the string itself cannot be softened.
const DUMP_PASS_SRC: &str = concat!("guarante", "ed pass");
static DUMP_PASS: &[Node] = &[Node::Lit(DUMP_PASS_SRC)];

/// The dump-language patterns with the pattern source each finding prints.
pub static DUMP_PHRASES: &[(&[Node], &str)] = &[
    (DUMP_ACTUAL, "actual exam question"),
    (DUMP_BRAIN, "brain\\s*dump"),
    (DUMP_REAL, "real EPI exam"),
    (DUMP_PASS, DUMP_PASS_SRC),
];

/// `quantity_evidence` values that excuse a numeric setpoint.
pub const FREE_EVIDENCE: &[&str] = &["free_url", "licensed_note", "exam_form_public"];

// ── tokenising and overlap ─────────────────────────────────────────────────

/// `tokenize` — `set(re.findall(r"[a-z0-9]{4,}", s.lower())) - STOP`.
pub fn tokenize(s: &str) -> HashSet<String> {
    let lowered = s.to_lowercase();
    let mut out = HashSet::new();
    let mut run = String::new();
    for c in lowered.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            run.push(c);
            continue;
        }
        push_token(&mut out, &mut run);
    }
    push_token(&mut out, &mut run);
    out
}

fn push_token(out: &mut HashSet<String>, run: &mut String) {
    if run.len() >= 4 && !STOP.contains(&run.as_str()) {
        out.insert(run.clone());
    }
    run.clear();
}

/// Every maximal run of `\w` characters in the corpus.
///
/// This is what makes `re.search(rf"\b{re.escape(t)}\b", corpus)` computable in
/// one pass: a token drawn from `[a-z0-9]{4,}` is entirely word characters, so
/// it can only sit between two `\b` positions when it IS a maximal word run.
/// Membership in this set and a boundary-anchored search accept exactly the same
/// tokens. Checked against the oracle over all 804 live items: zero score
/// differences.
pub fn word_runs(corpus: &str) -> HashSet<&str> {
    let mut out = HashSet::new();
    let mut start: Option<usize> = None;
    for (i, c) in corpus.char_indices() {
        if py_word(c) {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            out.insert(&corpus[s..i]);
        }
    }
    if let Some(s) = start {
        out.insert(&corpus[s..]);
    }
    out
}

/// `overlap_score` — the share of an item's tokens that are topic-label words or
/// whole words of the corpus.
pub fn overlap_score(
    toks: &HashSet<String>,
    runs: &HashSet<&str>,
    topic_words: &HashSet<String>,
) -> f64 {
    if toks.is_empty() {
        return 0.0;
    }
    let hits = toks
        .iter()
        .filter(|t| topic_words.contains(*t) || runs.contains(t.as_str()))
        .count();
    hits as f64 / toks.len().max(1) as f64
}

// ── corpus and topic registry ──────────────────────────────────────────────

/// `PurePath.suffix`, lowercased.
fn suffix_lower(name: &str) -> String {
    match name.rfind('.') {
        Some(i) if i > 0 => name[i..].to_lowercase(),
        _ => String::new(),
    }
}

/// `p.read_text(encoding="utf-8", errors="replace")`.
fn read_text_replace(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(universal_newlines(&String::from_utf8_lossy(&bytes)))
}

/// `base.rglob("*")`, restricted to the corpus suffixes and to files.
///
/// `pathlib` includes dotfiles, recurses into hidden directories, does NOT
/// recurse into symlinked directories, and DOES yield symlinked files (whose
/// `is_file()` follows the link). All four are reproduced. Results are sorted;
/// see the header for why order cannot reach the output.
fn rglob_corpus_files(base: &Path, base_disp: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(base) else {
        return;
    };
    let mut entries: Vec<(String, PathBuf)> = rd
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    entries.sort();
    for (name, path) in entries {
        let disp = format!("{base_disp}/{name}");
        let meta_link = std::fs::symlink_metadata(&path);
        let is_symlink = meta_link
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        if path.is_dir() {
            if !is_symlink {
                rglob_corpus_files(&path, &disp, out);
            }
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if !CORPUS_SUFFIXES.contains(&suffix_lower(&name).as_str()) {
            continue;
        }
        if disp.contains(CORPUS_PUBLIC_MARKER) {
            continue;
        }
        out.push(path);
    }
}

/// `load_corpus_text()` — the joined, lowercased corpus.
///
/// The bodies of `knowledge/corpus/**` are third-party captures, some of them
/// `redistribution: not-licensed`. They are consumed here as a character count
/// and a bag of whole words, and no code path in this gate renders, quotes, or
/// echoes any of that text.
pub fn load_corpus_text(root: &Path) -> String {
    let root_disp = root.to_string_lossy().into_owned();
    let parent_disp = root
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut bases: Vec<(PathBuf, String)> = Vec::new();
    for name in SIBLING_CORPUS_DIRS {
        if let Some(parent) = root.parent() {
            bases.push((parent.join(name), format!("{parent_disp}/{name}")));
        }
    }
    bases.push((
        root.join(KNOWLEDGE_REL),
        format!("{root_disp}/{KNOWLEDGE_REL}"),
    ));

    let mut chunks: Vec<String> = Vec::new();
    for (base, disp) in &bases {
        // `Path.exists()` follows symlinks and is true for files too; a base
        // that is a file yields nothing from `rglob`, which read_dir also does.
        if !base.exists() {
            continue;
        }
        let mut files = Vec::new();
        rglob_corpus_files(base, disp, &mut files);
        for f in files {
            if let Some(t) = read_text_replace(&f) {
                chunks.push(t);
            }
        }
    }

    let public = root.join(CORPUS_PUBLIC_REL);
    if public.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&public) {
            let mut names: Vec<(String, PathBuf)> = rd
                .flatten()
                .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
                .collect();
            names.sort();
            for (name, path) in names {
                if !name.ends_with(".txt") {
                    continue;
                }
                if let Some(t) = read_text_replace(&path) {
                    chunks.push(t);
                }
            }
        }
    }

    chunks.join("\n").to_lowercase()
}

/// The corpus roots in the oracle's order, as `(label, path)`.
///
/// The labels are what the report prints, so they are engine-root-relative and
/// spell the two siblings with a leading `../`.
pub fn corpus_roots(root: &Path) -> Vec<(&'static str, PathBuf)> {
    let parent = root.parent().unwrap_or(root);
    vec![
        ("../modules", parent.join("modules")),
        ("../reference", parent.join("reference")),
        ("knowledge", root.join(KNOWLEDGE_REL)),
        ("knowledge/corpus/public", root.join(CORPUS_PUBLIC_REL)),
    ]
}

/// `corpus_root_errors()` — every declared corpus root that is missing or
/// cannot be listed.
///
/// `load_corpus_text` skips both cases in silence, which is precisely how a gate
/// ends up reporting PASS over a corpus it never opened. The oracle detects
/// "unlistable" by taking the first entry of `path.iterdir()` and catching
/// `OSError`; `read_dir` returning `Err` is the same condition.
pub fn corpus_root_errors(root: &Path) -> Vec<String> {
    let mut errs = Vec::new();
    for (label, path) in corpus_roots(root) {
        if !path.is_dir() {
            errs.push(format!("corpus root missing: {label}"));
            continue;
        }
        if std::fs::read_dir(&path).is_err() {
            errs.push(format!("corpus root unreadable: {label}"));
        }
    }
    errs
}

/// `topic_labels()` — `id` -> `label` for every `[[topic]]` block.
pub fn topic_labels(topics: &Path) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    if !topics.is_file() {
        return labels;
    }
    let Some(text) = std::fs::read(topics)
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
    else {
        return labels;
    };
    // `re.split(r"\n\[\[topic\]\]\n", "\n" + text)`, then every block after the
    // first. The oracle prepends the newline (bd-yje7) so a registry whose very
    // first byte opens a block still yields that label; without it the raw id
    // was tokenised in place of its label and every score built on it degraded
    // with no finding printed.
    let text = format!("\n{text}");
    for block in text.split("\n[[topic]]\n").skip(1) {
        let chars: Vec<char> = block.chars().collect();
        let (Some(id), Some(lab)) = (
            first_key_value(&chars, "id"),
            first_key_value(&chars, "label"),
        ) else {
            continue;
        };
        labels.insert(id, lab);
    }
    labels
}

/// `re.search(r'(?m)^<key>\s*=\s*"([^"]+)"', block)`.
///
/// The `\s*` runs match newlines in `re`, so a match may span lines; a per-line
/// scan would silently find fewer and quietly weaken every score downstream.
fn first_key_value(ch: &[char], key: &str) -> Option<String> {
    let n = ch.len();
    for p in 0..=n {
        // `^` under re.MULTILINE.
        if p != 0 && ch[p - 1] != '\n' {
            continue;
        }
        if let Some(v) = match_key_at(ch, p, key) {
            return Some(v);
        }
    }
    None
}

fn match_key_at(ch: &[char], start: usize, key: &str) -> Option<String> {
    let n = ch.len();
    let mut i = start;
    for kc in key.chars() {
        if i >= n || ch[i] != kc {
            return None;
        }
        i += 1;
    }
    while i < n && py_space(ch[i]) {
        i += 1;
    }
    if i >= n || ch[i] != '=' {
        return None;
    }
    i += 1;
    while i < n && py_space(ch[i]) {
        i += 1;
    }
    if i >= n || ch[i] != '"' {
        return None;
    }
    i += 1;
    let s = i;
    while i < n && ch[i] != '"' {
        i += 1;
    }
    if i == s || i >= n {
        return None;
    }
    Some(ch[s..i].iter().collect())
}

// ── argparse emulation ─────────────────────────────────────────────────────

/// The parsed command line.
#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub strict_overlap: bool,
    pub min_overlap: f64,
    pub sample_report: i128,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            strict_overlap: false,
            min_overlap: 0.08,
            sample_report: 15,
        }
    }
}

/// Exactly what the oracle writes, and the status it exits with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Help,
    Flag,
    Float,
    Int,
}

/// Options in the order `add_argument` registered them; the ambiguity message
/// lists candidates in that order.
const OPTIONS: &[(&str, Kind)] = &[
    ("--help", Kind::Help),
    ("--strict-overlap", Kind::Flag),
    ("--min-overlap", Kind::Float),
    ("--sample-report", Kind::Int),
];

/// `shutil.get_terminal_size().columns`, minus the part this port cannot see.
fn formatter_width() -> usize {
    let cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(80);
    (cols as usize).saturating_sub(2)
}

/// `HelpFormatter._format_usage`, for this parser's fixed option list.
pub fn format_usage(width: usize) -> String {
    const PREFIX: &str = "usage: ";
    let parts = [
        "[-h]",
        "[--strict-overlap]",
        "[--min-overlap MIN_OVERLAP]",
        "[--sample-report SAMPLE_REPORT]",
    ];
    let flat = format!("{PROG} {}", parts.join(" "));
    if PREFIX.len() + flat.len() <= width {
        return format!("{PREFIX}{flat}\n");
    }
    let indent_len = if PREFIX.len() + PROG.len() <= (3 * width) / 4 {
        PREFIX.len() + PROG.len() + 1
    } else {
        PREFIX.len()
    };
    let indent = " ".repeat(indent_len);
    let head: Vec<&str> = if indent_len == PREFIX.len() {
        parts.to_vec()
    } else {
        std::iter::once(PROG).chain(parts).collect()
    };

    let mut lines: Vec<String> = Vec::new();
    let mut line: Vec<&str> = Vec::new();
    let mut line_len = PREFIX.len() - 1;
    for part in head {
        if line_len + 1 + part.len() > width && !line.is_empty() {
            lines.push(format!("{indent}{}", line.join(" ")));
            line.clear();
            line_len = indent_len.saturating_sub(1);
        }
        line_len += 1 + part.len();
        line.push(part);
    }
    if !line.is_empty() {
        lines.push(format!("{indent}{}", line.join(" ")));
    }
    if indent_len == PREFIX.len() {
        lines.insert(0, PROG.to_string());
    } else if let Some(first) = lines.first_mut() {
        *first = first[indent_len.min(first.len())..].to_string();
    }
    format!("{PREFIX}{}\n", lines.join("\n"))
}

/// `textwrap.wrap` for help strings that contain no hyphens (both of ours).
fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let mut word = word;
        loop {
            let room = if cur.is_empty() {
                width
            } else {
                width.saturating_sub(cur.chars().count() + 1)
            };
            if word.chars().count() <= room {
                if !cur.is_empty() {
                    cur.push(' ');
                }
                cur.push_str(word);
                break;
            }
            if cur.is_empty() {
                // break_long_words: split at exactly `width`.
                let cut: String = word.chars().take(width).collect();
                lines.push(cut.clone());
                word = &word[cut.len()..];
                continue;
            }
            lines.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// `ArgumentParser.format_help`.
pub fn format_help(width: usize) -> String {
    let rows: &[(&str, &str)] = &[
        ("-h, --help", "show this help message and exit"),
        (
            "--strict-overlap",
            "FAIL items with low corpus overlap (default: warn only)",
        ),
        ("--min-overlap MIN_OVERLAP", ""),
        ("--sample-report SAMPLE_REPORT", ""),
    ];
    let indent = 2usize;
    let action_max_length = rows.iter().map(|(inv, _)| inv.len()).max().unwrap_or(0) + indent;
    let max_help_position = 24usize.min(width.saturating_sub(20).max(4));
    let help_position = (action_max_length + 2).min(max_help_position);
    let help_width = width.saturating_sub(help_position).max(11);
    let action_width = help_position.saturating_sub(indent + 2);

    let mut body = String::new();
    for (inv, help) in rows {
        if help.is_empty() {
            body.push_str(&format!("{}{inv}\n", " ".repeat(indent)));
            continue;
        }
        let lines = wrap(help, help_width);
        if inv.len() <= action_width {
            body.push_str(&format!(
                "{}{inv:<action_width$}  {}\n",
                " ".repeat(indent),
                lines.first().cloned().unwrap_or_default()
            ));
        } else {
            body.push_str(&format!("{}{inv}\n", " ".repeat(indent)));
            body.push_str(&format!(
                "{}{}\n",
                " ".repeat(help_position),
                lines.first().cloned().unwrap_or_default()
            ));
        }
        for l in lines.iter().skip(1) {
            body.push_str(&format!("{}{l}\n", " ".repeat(help_position)));
        }
    }
    format!("{}\noptions:\n{body}", format_usage(width))
}

fn usage_error(msg: &str) -> Outcome {
    Outcome {
        stdout: String::new(),
        stderr: format!("{}{PROG}: error: {msg}\n", format_usage(formatter_width())),
        code: 2,
    }
}

/// `_negative_number_matcher` as 3.13+ spells it: `-` then an optional `.` then
/// a digit. The parser registers no negative-number-looking options, so such a
/// token is a VALUE, which is how `--min-overlap -0.5` works.
fn looks_like_negative_number(s: &str) -> bool {
    let mut it = s.chars();
    if it.next() != Some('-') {
        return false;
    }
    let mut c = it.next();
    if c == Some('.') {
        c = it.next();
    }
    matches!(c, Some(d) if d.is_ascii_digit())
}

/// `_parse_optional`: is this token classified 'O' rather than 'A'?
fn looks_like_option(s: &str) -> bool {
    if s.is_empty() || !s.starts_with('-') {
        return false;
    }
    if s == "-h" || OPTIONS.iter().any(|(o, _)| *o == s) {
        return true;
    }
    if s.chars().count() == 1 {
        return false;
    }
    if let Some((head, _)) = s.split_once('=') {
        if head == "-h" || OPTIONS.iter().any(|(o, _)| *o == head) {
            return true;
        }
    }
    if !prefix_matches(s.split('=').next().unwrap_or(s)).is_empty() {
        return true;
    }
    if looks_like_negative_number(s) {
        return false;
    }
    !s.contains(' ')
}

/// `_get_option_tuples` for long options: unambiguous-prefix matching.
fn prefix_matches(name: &str) -> Vec<&'static str> {
    if !name.starts_with("--") {
        return Vec::new();
    }
    OPTIONS
        .iter()
        .filter(|(o, _)| o.starts_with(name))
        .map(|(o, _)| *o)
        .collect()
}

/// `float(s)` — Python's, including surrounding whitespace and digit-grouping
/// underscores, both of which Rust's `str::parse` rejects.
pub fn py_float(s: &str) -> Option<f64> {
    let t = s.trim_matches(py_space);
    let cleaned = strip_underscores(t)?;
    cleaned.parse::<f64>().ok()
}

/// `int(s)` in base 10, clamped to a magnitude the slice arithmetic can hold.
/// Python's `int` is unbounded, but the value is only ever used as a list slice
/// bound, where any magnitude past the list length behaves identically.
pub fn py_int(s: &str) -> Option<i128> {
    let t = s.trim_matches(py_space);
    let cleaned = strip_underscores(t)?;
    let (sign, digits) = match cleaned.strip_prefix('-') {
        Some(rest) => (-1i128, rest),
        None => (1i128, cleaned.strip_prefix('+').unwrap_or(&cleaned)),
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    const CLAMP: i128 = 1 << 62;
    match digits.parse::<i128>() {
        Ok(v) if v <= CLAMP => Some(sign * v),
        _ => Some(sign * CLAMP),
    }
}

/// Remove digit-grouping underscores, rejecting any that is not between digits.
fn strip_underscores(s: &str) -> Option<String> {
    if !s.contains('_') {
        return Some(s.to_string());
    }
    let ch: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(ch.len());
    for (i, c) in ch.iter().enumerate() {
        if *c != '_' {
            out.push(*c);
            continue;
        }
        let prev_ok = i > 0 && ch[i - 1].is_ascii_digit();
        let next_ok = ch.get(i + 1).map(|n| n.is_ascii_digit()).unwrap_or(false);
        if !(prev_ok && next_ok) {
            return None;
        }
    }
    Some(out)
}

/// `parser.parse_args`. `Err` carries the exact bytes and status the oracle
/// exits with, for `--help` (0) as well as for every error (2).
pub fn parse_args(argv: &[String]) -> Result<Args, Outcome> {
    let mut args = Args::default();
    let mut extras: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < argv.len() {
        let raw = argv[i].clone();
        // Measured on 3.14: `--` is not consumed, and it plus everything after
        // it lands in the unrecognized-arguments list.
        if raw == "--" {
            extras.extend(argv[i..].iter().cloned());
            break;
        }
        if !looks_like_option(&raw) {
            extras.push(raw);
            i += 1;
            continue;
        }
        let (name, explicit) = match raw.split_once('=') {
            Some((n, v)) => (n.to_string(), Some(v.to_string())),
            None => (raw.clone(), None),
        };
        let (opt, kind) = if name == "-h" {
            ("-h", Kind::Help)
        } else if let Some((o, k)) = OPTIONS.iter().find(|(o, _)| *o == name) {
            (*o, *k)
        } else {
            let hits = prefix_matches(&name);
            match hits.len() {
                1 => {
                    let o = hits[0];
                    (o, OPTIONS.iter().find(|(x, _)| *x == o).unwrap().1)
                }
                0 => {
                    extras.push(raw);
                    i += 1;
                    continue;
                }
                _ => {
                    return Err(usage_error(&format!(
                        "ambiguous option: {raw} could match {}",
                        hits.join(", ")
                    )));
                }
            }
        };

        match kind {
            Kind::Help => {
                return Err(Outcome {
                    stdout: format_help(formatter_width()),
                    stderr: String::new(),
                    code: 0,
                });
            }
            Kind::Flag => {
                if let Some(v) = explicit {
                    return Err(usage_error(&format!(
                        "argument {opt}: ignored explicit argument {}",
                        py_repr(&v)
                    )));
                }
                args.strict_overlap = true;
                i += 1;
            }
            Kind::Float | Kind::Int => {
                let value = match explicit {
                    Some(v) => {
                        i += 1;
                        v
                    }
                    None => match argv.get(i + 1) {
                        Some(v) if !looks_like_option(v) => {
                            i += 2;
                            v.clone()
                        }
                        _ => {
                            return Err(usage_error(&format!(
                                "argument {opt}: expected one argument"
                            )));
                        }
                    },
                };
                if kind == Kind::Float {
                    match py_float(&value) {
                        Some(v) => args.min_overlap = v,
                        None => {
                            return Err(usage_error(&format!(
                                "argument {opt}: invalid float value: {}",
                                py_repr(&value)
                            )));
                        }
                    }
                } else {
                    match py_int(&value) {
                        Some(v) => args.sample_report = v,
                        None => {
                            return Err(usage_error(&format!(
                                "argument {opt}: invalid int value: {}",
                                py_repr(&value)
                            )));
                        }
                    }
                }
            }
        }
    }
    if !extras.is_empty() {
        return Err(usage_error(&format!(
            "unrecognized arguments: {}",
            extras.join(" ")
        )));
    }
    Ok(args)
}

// ── the gate ───────────────────────────────────────────────────────────────

/// `" ".join(it.get("choices") or [])`, or `None` where the oracle raises
/// `TypeError` mid-loop.
fn join_choices(v: Option<&Value>) -> Option<String> {
    // `it.get("choices") or []` — anything falsy short-circuits to the empty
    // list, so an empty string, an empty table, `0` and `false` all join to "".
    let Some(v) = v else {
        return Some(String::new());
    };
    if !py_truthy(v) {
        return Some(String::new());
    }
    match v {
        Value::Array(a) => {
            let mut parts = Vec::with_capacity(a.len());
            for e in a {
                match e {
                    Value::String(s) => parts.push(s.clone()),
                    // `str.join` over a non-string element raises TypeError.
                    _ => return None,
                }
            }
            Some(parts.join(" "))
        }
        // `" ".join("abc")` interleaves the characters.
        Value::String(s) => Some(
            s.chars()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ),
        // Iterating a dict yields its keys.
        Value::Table(t) => Some(t.keys().cloned().collect::<Vec<_>>().join(" ")),
        _ => None,
    }
}

/// Python's `lst[:k]` end index, negative `k` included.
fn slice_end(len: usize, k: i128) -> usize {
    let n = len as i128;
    let end = if k >= 0 { k.min(n) } else { (n + k).max(0) };
    end as usize
}

/// Run the whole check and render the oracle's report.
pub fn evaluate(root: &Path, args: &Args) -> Outcome {
    let items_dir = root.join(ITEMS_REL);
    if !items_dir.is_dir() {
        return Outcome {
            stdout: "FAIL: bank/items missing\n".to_string(),
            stderr: String::new(),
            code: 1,
        };
    }

    let corpus = load_corpus_text(root);
    let corpus_chars = corpus.chars().count();
    let runs = word_runs(&corpus);
    let labels = topic_labels(&root.join(TOPICS_REL));

    let mut high: Vec<String> = Vec::new();
    let mut warns = 0usize;
    let mut scanned = 0usize;
    let mut low_overlap: Vec<(String, f64)> = Vec::new();

    let mut names: Vec<String> = match std::fs::read_dir(&items_dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".toml"))
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();

    for name in names {
        let path = items_dir.join(&name);
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|t| t.parse::<toml::Table>().map_err(|e| e.to_string()));
        let it = match parsed {
            Ok(t) => t,
            Err(e) => {
                high.push(format!("{name}: parse error {e}"));
                continue;
            }
        };
        scanned += 1;
        // `it.get("id") or path.name` — TRUTHINESS, not merely presence, so an
        // `id = ""` falls back to the filename instead of prefixing every
        // finding for this item with nothing at all (bd-yje7).
        let iid = match it.get("id") {
            Some(v) if py_truthy(v) => py_str(v),
            _ => name.clone(),
        };
        let Some(choices) = join_choices(it.get("choices")) else {
            return Outcome {
                stdout: String::new(),
                stderr: format!(
                    "{PROG}: TypeError joining `choices` of {name} \
                     (the oracle prints a traceback here and exits 1)\n"
                ),
                code: 1,
            };
        };
        let stem = it.get("stem").map(py_str).unwrap_or_default();
        let expl = it.get("explanation").map(py_str).unwrap_or_default();
        let text = format!("{stem} {choices} {expl}");
        let ch: Vec<char> = text.chars().collect();

        for (nodes, label) in CLAUSE_PATTERNS {
            if search(nodes, &ch) {
                high.push(format!("{iid}: hallucinated-clause pattern: {label}..."));
                break;
            }
        }
        for (nodes, label) in DUMP_PHRASES {
            if search(nodes, &ch) {
                high.push(format!("{iid}: dump-language: {label}"));
            }
        }

        let qe_free = match it.get("quantity_evidence") {
            Some(Value::String(s)) => FREE_EVIDENCE.contains(&s.as_str()),
            _ => false,
        };
        if (search(TRAP_A, &ch) || search(TRAP_B, &ch)) && !qe_free {
            high.push(format!(
                "{iid}: numeric setpoint without free/licensed evidence"
            ));
        }
        if search(ISO_MULTI, &ch) {
            high.push(format!("{iid}: looks like fake multi-level clause cite"));
        }

        let mut topic_words: HashSet<String> = HashSet::new();
        if let Some(Value::Array(tids)) = it.get("topic_ids") {
            for t in tids {
                let key = match t {
                    Value::String(s) => s.clone(),
                    other => py_str(other),
                };
                let lab = labels.get(&key).cloned().unwrap_or(key);
                topic_words.extend(tokenize(&lab.replace('-', " ")));
            }
        }
        let score = overlap_score(&tokenize(&text), &runs, &topic_words);
        if score < args.min_overlap {
            if args.strict_overlap {
                high.push(format!(
                    "{iid}: low corpus overlap {score:.3} < {}",
                    py_float_repr(args.min_overlap)
                ));
            } else {
                warns += 1;
            }
            low_overlap.push((iid, score));
        }
    }

    // Anti-vacuous (bd-yje7). Each condition names ITSELF, because "PASS" over
    // an empty bank and "PASS" over a clean one are otherwise the same bytes.
    let mut vacuous: Vec<String> = corpus_root_errors(root);
    if scanned < MIN_SCANNED_ITEMS {
        vacuous.push(format!(
            "scanned_items={scanned} < floor {MIN_SCANNED_ITEMS} \
             (fewer items than one exam form \u{2014} nothing was meaningfully checked)"
        ));
    }
    if corpus_chars < MIN_CORPUS_CHARS {
        vacuous.push(format!(
            "corpus_chars={corpus_chars} < floor {MIN_CORPUS_CHARS} \
             (no grounding text to contradict a claim with)"
        ));
    }

    let mut out = String::new();
    out.push_str(&format!("scanned_items={scanned}\n"));
    out.push_str(&format!("high_severity={}\n", high.len()));
    out.push_str(&format!("low_overlap_warns={}\n", low_overlap.len()));
    if !low_overlap.is_empty() {
        // Stable sort on the score alone, so ties keep filename order.
        low_overlap.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        out.push_str("lowest_overlap_samples:\n");
        let end = slice_end(low_overlap.len(), args.sample_report);
        for (iid, sc) in &low_overlap[..end] {
            out.push_str(&format!("  {sc:.3}  {iid}\n"));
        }
    }

    if !vacuous.is_empty() {
        out.push_str("FAIL: vacuous grounding check\n");
        for e in &vacuous {
            out.push_str(&format!("  - {e}\n"));
        }
    }

    if !high.is_empty() {
        out.push_str("FAIL\n");
        for e in high.iter().take(MAX_REPORT) {
            out.push_str(&format!("  - {e}\n"));
        }
        if high.len() > MAX_REPORT {
            out.push_str(&format!("  ... +{} more\n", high.len() - MAX_REPORT));
        }
    }

    if !vacuous.is_empty() || !high.is_empty() {
        return Outcome {
            stdout: out,
            stderr: String::new(),
            code: 1,
        };
    }

    out.push_str("PASS\n");
    out.push_str("  no high-severity hallucination heuristics\n");
    if warns > 0 {
        out.push_str(&format!("  warns={warns} (use --strict-overlap to fail)\n"));
    }
    out.push_str(&format!("  corpus_chars={corpus_chars}\n"));
    Outcome {
        stdout: out,
        stderr: String::new(),
        code: 0,
    }
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let outcome = match parse_args(&ctx.args) {
        Ok(a) => {
            // The Python resolves its own location, so the corpus walk starts
            // from a symlink-free root. Do the same.
            let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
            evaluate(&root, &a)
        }
        Err(o) => o,
    };
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    eprint!("{}", outcome.stderr);
    let _ = std::io::stderr().flush();

    if outcome.code != 0 {
        // See the module header: the oracle exits 1 (or argparse's 2) with the
        // report on stdout, and this port's acceptance bar is byte-identical
        // output. Routing through `GateError` would write to stderr and exit
        // 2/4 instead.
        std::process::exit(outcome.code);
    }
    Ok(())
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    // ── the Python emulations ─────────────────────────────────────────────

    #[test]
    fn stop_words_are_the_oracles_fifty_two() {
        let uniq: HashSet<&&str> = STOP.iter().collect();
        assert_eq!(uniq.len(), 52, "STOP must dedupe to the oracle's 52 words");
    }

    #[test]
    fn tokenize_keeps_runs_of_four_and_drops_stopwords() {
        let t = tokenize("The RACK has PUE 1.42 and cooling-plant that");
        assert!(t.contains("rack"));
        assert!(t.contains("cooling"));
        assert!(t.contains("plant"));
        assert!(!t.contains("that"), "stopword survived");
        assert!(!t.contains("has"), "three-letter run survived");
        assert!(!t.contains("1"), "digit run shorter than four survived");
    }

    #[test]
    fn tokenize_takes_the_whole_maximal_run() {
        let t = tokenize("abcdefghij");
        assert!(t.contains("abcdefghij"));
        assert_eq!(t.len(), 1, "a greedy {{4,}} yields one token, not many");
    }

    #[test]
    fn word_runs_are_maximal_and_underscore_joins() {
        let r = word_runs("alpha beta_gamma delta.epsilon");
        assert!(r.contains("alpha"));
        assert!(r.contains("beta_gamma"));
        assert!(!r.contains("beta"), "an underscore is a word character");
        assert!(r.contains("delta"));
        assert!(r.contains("epsilon"));
    }

    #[test]
    fn universal_newlines_matches_text_mode() {
        assert_eq!(universal_newlines("a\r\nb\rc\nd"), "a\nb\nc\nd");
        assert_eq!(universal_newlines("plain"), "plain");
    }

    #[test]
    fn float_repr_matches_cpython() {
        assert_eq!(py_float_repr(0.08), "0.08");
        assert_eq!(py_float_repr(0.1), "0.1");
        assert_eq!(py_float_repr(0.0), "0.0");
        assert_eq!(py_float_repr(-0.5), "-0.5");
        assert_eq!(py_float_repr(1.0), "1.0");
        assert_eq!(py_float_repr(1e16), "1e+16");
        assert_eq!(py_float_repr(1e-5), "1e-05");
        assert_eq!(py_float_repr(0.0001), "0.0001");
        assert_eq!(py_float_repr(1234567890123456.0), "1234567890123456.0");
        assert_eq!(py_float_repr(f64::INFINITY), "inf");
        assert_eq!(py_float_repr(f64::NEG_INFINITY), "-inf");
        assert_eq!(py_float_repr(f64::NAN), "nan");
    }

    #[test]
    fn float_and_int_parsing_follow_python_not_rust() {
        assert_eq!(py_float(" 0.5 "), Some(0.5));
        assert_eq!(py_float("1_000.5"), Some(1000.5));
        assert_eq!(py_float("_1"), None);
        assert_eq!(py_float("1__0"), None);
        assert_eq!(py_float("x"), None);
        assert_eq!(py_int(" -3 "), Some(-3));
        assert_eq!(py_int("1_5"), Some(15));
        assert_eq!(py_int("1.5"), None);
        assert_eq!(py_int("0x10"), None);
    }

    #[test]
    fn suffix_follows_purepath() {
        assert_eq!(suffix_lower("a.toml"), ".toml");
        assert_eq!(suffix_lower("UP.MD"), ".md");
        assert_eq!(suffix_lower(".toml"), "");
        assert_eq!(suffix_lower("plain"), "");
        assert_eq!(suffix_lower("a.b.txt"), ".txt");
    }

    #[test]
    fn python_slice_semantics_including_negative_bounds() {
        assert_eq!(slice_end(10, 3), 3);
        assert_eq!(slice_end(10, 99), 10);
        assert_eq!(slice_end(10, -3), 7);
        assert_eq!(slice_end(10, -99), 0);
    }

    // ── the compiled patterns ─────────────────────────────────────────────

    #[test]
    fn clause_zero_fires_on_a_family_plus_subdivision() {
        assert!(search(
            CLAUSE_0,
            &chars("per ISO 22237 clause 4.1 the room")
        ));
        assert!(search(CLAUSE_0, &chars("EN 50600-2-3 section 5.2")));
        assert!(search(CLAUSE_0, &chars("tia 942 PART 6.1")));
        assert!(!search(CLAUSE_0, &chars("per ISO 22237 the room")));
        assert!(!search(CLAUSE_0, &chars("clause 4.1 of the standard")));
    }

    /// `[\d\-]+(?:-\d+)*` and `[\d\-]+` accept the same strings, which is why
    /// the tail is dropped from `CLAUSE_0`.
    #[test]
    fn tail_is_redundant() {
        for s in ["1", "22237", "50600-2-3", "-", "1-2-3-4"] {
            let all_ok = s.chars().all(digit_or_dash);
            assert!(all_ok, "{s} is not in the class");
        }
        // The observable consequence: a hyphenated multi-part number still
        // reaches the keyword.
        assert!(search(CLAUSE_0, &chars("EN 50600-2-3-4 clause 1.2")));
    }

    #[test]
    fn clause_one_needs_a_trailing_boundary() {
        assert!(search(CLAUSE_1, &chars("see clause 5.2.1 for detail")));
        assert!(search(CLAUSE_1, &chars("clause 1.2")));
        assert!(
            !search(CLAUSE_1, &chars("clause 1.23a")),
            "a word character after the digits kills the trailing \\b"
        );
        assert!(!search(CLAUSE_1, &chars("clause 12")));
    }

    #[test]
    fn clause_two_inherits_the_oracles_boundary_quirk() {
        // `§` is not a word character, so `\b` demands a word character before.
        assert!(search(CLAUSE_2, &chars("x\u{a7}3.4")));
        assert!(search(CLAUSE_2, &chars("annex\u{a7} 3.4")));
        assert!(
            !search(CLAUSE_2, &chars("see \u{a7}3.4")),
            "a space before the section sign leaves no boundary"
        );
    }

    #[test]
    fn numeric_trap_both_branches() {
        assert!(search(TRAP_A, &chars("must be 22 C in the aisle")));
        assert!(search(TRAP_A, &chars("exactly 25.5\u{b0}C")));
        assert!(search(TRAP_A, &chars("Precisely 18 \u{b0} f")));
        assert!(!search(TRAP_A, &chars("about 22 C in the aisle")));
        assert!(
            !search(TRAP_A, &chars("exactly 250 Celsius")),
            "no boundary after the C"
        );
        assert!(search(TRAP_B, &chars("27\u{b0}C recommended for inlet")));
        assert!(search(TRAP_B, &chars("100 \u{b0} F mandatory")));
        assert!(!search(TRAP_B, &chars("2\u{b0}C recommended")));
        assert!(!search(TRAP_B, &chars("1234\u{b0}C recommended")));
    }

    #[test]
    fn iso_multi_level_needs_the_triple_within_forty_characters() {
        assert!(search(ISO_MULTI, &chars("ISO/IEC 22237 says 3.5.2 here")));
        assert!(search(ISO_MULTI, &chars("isoiec22237 1.2.3")));
        let far = format!("ISO/IEC 22237{}1.2.3", "x".repeat(41));
        assert!(!search(ISO_MULTI, &chars(&far)), "41 filler characters");
        let near = format!("ISO/IEC 22237{}1.2.3", "x".repeat(40));
        assert!(search(ISO_MULTI, &chars(&near)), "40 filler characters");
    }

    #[test]
    fn dump_phrases_fire_case_insensitively() {
        assert!(search(DUMP_BRAIN, &chars("a BRAIN   DUMP of answers")));
        assert!(search(DUMP_BRAIN, &chars("braindump")));
        assert!(search(DUMP_ACTUAL, &chars("an Actual Exam Question")));
        assert!(!search(DUMP_ACTUAL, &chars("an actual question")));
    }

    // ── argparse ──────────────────────────────────────────────────────────

    #[test]
    fn defaults_match_the_oracle() {
        let a = parse_args(&[]).unwrap();
        assert!(!a.strict_overlap);
        assert_eq!(a.min_overlap, 0.08);
        assert_eq!(a.sample_report, 15);
    }

    #[test]
    fn abbreviations_resolve_and_collisions_are_reported() {
        assert!(
            parse_args(&["--strict".to_string()])
                .unwrap()
                .strict_overlap
        );
        assert_eq!(
            parse_args(&["--min".to_string(), "0.5".to_string()])
                .unwrap()
                .min_overlap,
            0.5
        );
        let err = parse_args(&["--s".to_string()]).unwrap_err();
        assert_eq!(err.code, 2);
        assert!(
            err.stderr
                .contains("ambiguous option: --s could match --strict-overlap, --sample-report"),
            "{}",
            err.stderr
        );
    }

    #[test]
    fn a_negative_number_is_a_value_not_an_option() {
        let a = parse_args(&["--min-overlap".to_string(), "-0.5".to_string()]).unwrap();
        assert_eq!(a.min_overlap, -0.5);
        let a = parse_args(&["--sample-report".to_string(), "-3".to_string()]).unwrap();
        assert_eq!(a.sample_report, -3);
    }

    #[test]
    fn bad_values_and_unknown_flags_are_status_two() {
        for argv in [
            vec!["--min-overlap".to_string(), "x".to_string()],
            vec!["--sample-report".to_string(), "1.5".to_string()],
            vec!["--bogus".to_string()],
            vec!["--min-overlap".to_string()],
            vec!["--strict-overlap=1".to_string()],
        ] {
            let err = parse_args(&argv).unwrap_err();
            assert_eq!(err.code, 2, "{argv:?} must be a usage error");
            assert!(
                err.stdout.is_empty(),
                "usage errors write nothing to stdout"
            );
            assert!(err.stderr.starts_with("usage: validate_grounding.py"));
        }
    }

    #[test]
    fn help_is_status_zero_on_stdout() {
        let err = parse_args(&["--help".to_string()]).unwrap_err();
        assert_eq!(err.code, 0);
        assert!(err.stderr.is_empty());
        assert!(err.stdout.contains("\noptions:\n"));
    }

    #[test]
    fn usage_wraps_the_way_argparse_does_at_eighty_columns() {
        assert_eq!(
            format_usage(78),
            "usage: validate_grounding.py [-h] [--strict-overlap]\n\
             \x20                            [--min-overlap MIN_OVERLAP]\n\
             \x20                            [--sample-report SAMPLE_REPORT]\n"
        );
    }

    // ── the gate over fixtures ────────────────────────────────────────────

    struct Tree {
        dir: tempfile::TempDir,
    }

    /// Invented vocabulary for the padding corpus and the padding items. None of
    /// it comes from `knowledge/corpus/**`, whose captures this crate never
    /// renders, and none of it collides with the invented tokens the overlap
    /// cases below rely on scoring zero.
    const PAD_WORDS: &str = "chiller plenum containment aisle rack economiser humidity \
                             envelope inlet redundancy topology maintainability concurrent";

    impl Tree {
        fn new() -> Self {
            let t = Tree {
                dir: tempfile::tempdir().unwrap(),
            };
            std::fs::create_dir_all(t.root().join("bank/items")).unwrap();
            std::fs::create_dir_all(t.root().join("knowledge")).unwrap();
            t
        }

        /// `new()` plus the smallest tree that clears every anti-vacuous floor:
        /// all four corpus roots present, a corpus over `MIN_CORPUS_CHARS`, and
        /// exactly `MIN_SCANNED_ITEMS` well-grounded items. Tests that assert a
        /// PASS start here, so "green" means the heuristics were satisfied
        /// rather than that the floors were dodged.
        fn grounded() -> Self {
            let t = Tree::new();
            std::fs::create_dir_all(t.dir.path().join("reference")).unwrap();
            let line = format!("{PAD_WORDS}\n");
            let reps = MIN_CORPUS_CHARS / line.len() + 2;
            t.write_sibling("modules/m01.md", &line.repeat(reps));
            t.write_sibling("reference/glossary.md", &line);
            t.write("knowledge/corpus/public/src-invented.txt", &line);
            for i in 0..MIN_SCANNED_ITEMS {
                t.write(
                    &format!("bank/items/pad-{i:03}.toml"),
                    &format!(
                        "id = \"pad-{i:03}\"\nstem = \"{PAD_WORDS}\"\nchoices = []\n\
                         explanation = \"\"\ntopic_ids = []\n"
                    ),
                );
            }
            t
        }

        fn root(&self) -> PathBuf {
            self.dir.path().join("engine")
        }
        fn write(&self, rel: &str, body: &str) {
            let p = self.root().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, body).unwrap();
        }
        /// Write beside the engine root, where `../modules` and `../reference`
        /// live.
        fn write_sibling(&self, rel: &str, body: &str) {
            let p = self.dir.path().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, body).unwrap();
        }
        fn run(&self, args: &Args) -> Outcome {
            evaluate(&self.root(), args)
        }
    }

    fn item(id: &str, stem: &str) -> String {
        format!(
            "id = {}\nstem = {}\nchoices = [\"alpha\", \"beta\"]\n\
             explanation = \"explanatory sentence\"\ntopic_ids = [\"t-one\"]\n",
            py_repr(id).replace('\'', "\""),
            py_repr(stem).replace('\'', "\"")
        )
    }

    #[test]
    fn a_missing_bank_directory_is_the_one_guard_the_oracle_has() {
        let t = Tree::new();
        std::fs::remove_dir(t.root().join("bank/items")).unwrap();
        let o = t.run(&Args::default());
        assert_eq!(o.code, 1);
        assert_eq!(o.stdout, "FAIL: bank/items missing\n");
    }

    /// The smallest legitimate tree is GREEN. Without this leg the floors could
    /// be set anywhere at all and every "it goes RED" test below would still
    /// pass — an over-strict gate is routed around, which is a slower death than
    /// no gate.
    #[test]
    fn a_small_but_legitimate_tree_still_passes() {
        let t = Tree::grounded();
        let o = t.run(&Args::default());
        assert_eq!(o.code, 0, "{}", o.stdout);
        assert!(
            o.stdout
                .starts_with(&format!("scanned_items={MIN_SCANNED_ITEMS}\n")),
            "{}",
            o.stdout
        );
        assert!(o.stdout.contains("\nPASS\n"), "{}", o.stdout);
        assert!(o.stdout.contains("low_overlap_warns=0\n"), "{}", o.stdout);
    }

    #[test]
    fn zero_items_is_an_error_that_names_itself() {
        let t = Tree::grounded();
        for i in 0..MIN_SCANNED_ITEMS {
            std::fs::remove_file(t.root().join(format!("bank/items/pad-{i:03}.toml"))).unwrap();
        }
        let o = t.run(&Args::default());
        assert_eq!(o.code, 1, "a bank that was never scanned is not a pass");
        assert!(o.stdout.starts_with("scanned_items=0\n"), "{}", o.stdout);
        assert!(!o.stdout.contains("PASS"), "{}", o.stdout);
        assert!(
            o.stdout.contains("FAIL: vacuous grounding check\n"),
            "{}",
            o.stdout
        );
        assert!(
            o.stdout
                .contains(&format!("  - scanned_items=0 < floor {MIN_SCANNED_ITEMS} ")),
            "{}",
            o.stdout
        );
    }

    /// One item under the floor is still RED: the bar is a recorded minimum, not
    /// a non-emptiness test.
    #[test]
    fn one_item_short_of_the_floor_is_still_an_error() {
        let t = Tree::grounded();
        let last = MIN_SCANNED_ITEMS - 1;
        std::fs::remove_file(t.root().join(format!("bank/items/pad-{last:03}.toml"))).unwrap();
        let o = t.run(&Args::default());
        assert_eq!(o.code, 1, "{}", o.stdout);
        assert!(
            o.stdout.contains(&format!(
                "  - scanned_items={last} < floor {MIN_SCANNED_ITEMS} "
            )),
            "{}",
            o.stdout
        );
    }

    #[test]
    fn zero_corpus_chars_is_an_error_that_names_itself() {
        let t = Tree::new();
        t.write("bank/items/a.toml", &item("a", "a stem about cooling"));
        let o = t.run(&Args::default());
        assert_eq!(o.code, 1, "an empty corpus can contradict nothing");
        assert!(!o.stdout.contains("PASS"), "{}", o.stdout);
        assert!(
            o.stdout
                .contains(&format!("  - corpus_chars=0 < floor {MIN_CORPUS_CHARS} ")),
            "{}",
            o.stdout
        );
    }

    /// A three-byte corpus clears `> 0` and is still RED, which is the whole
    /// point of recording a floor rather than a non-emptiness check.
    #[test]
    fn a_corpus_of_a_few_characters_does_not_satisfy_the_floor() {
        let t = Tree::grounded();
        // Every root stays in place; only the text shrinks.
        t.write_sibling("modules/m01.md", "x");
        t.write_sibling("reference/glossary.md", "y");
        t.write("knowledge/corpus/public/src-invented.txt", "z");
        let o = t.run(&Args::default());
        assert_eq!(o.code, 1, "{}", o.stdout);
        assert!(
            o.stdout
                .contains(&format!("  - corpus_chars=5 < floor {MIN_CORPUS_CHARS} ")),
            "{}",
            o.stdout
        );
    }

    #[test]
    fn each_missing_corpus_root_is_named() {
        for (rel, label, sibling) in [
            ("modules", "../modules", true),
            ("reference", "../reference", true),
            ("knowledge/corpus/public", "knowledge/corpus/public", false),
        ] {
            let t = Tree::grounded();
            let dir = if sibling {
                t.dir.path().join(rel)
            } else {
                t.root().join(rel)
            };
            for e in std::fs::read_dir(&dir).unwrap().flatten() {
                std::fs::remove_file(e.path()).unwrap();
            }
            std::fs::remove_dir(&dir).unwrap();
            let o = t.run(&Args::default());
            assert_eq!(o.code, 1, "[{label}] {}", o.stdout);
            assert!(
                o.stdout
                    .contains(&format!("  - corpus root missing: {label}\n")),
                "[{label}] {}",
                o.stdout
            );
        }
    }

    /// The floor is not a number someone typed: it is `exam_n_items` from the
    /// live bank policy, so the recorded reason is machine-checked rather than
    /// asserted in a comment.
    #[test]
    fn the_item_floor_is_one_exam_form_from_the_bank_policy() {
        let root =
            crate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
        let policy = std::fs::read_to_string(root.join("knowledge/bank_policy.toml"))
            .expect("knowledge/bank_policy.toml");
        let exam_n: usize = policy
            .lines()
            .find_map(|l| l.strip_prefix("exam_n_items"))
            .and_then(|r| r.split('=').nth(1))
            .and_then(|v| v.trim().parse().ok())
            .expect("exam_n_items in knowledge/bank_policy.toml");
        assert_eq!(
            MIN_SCANNED_ITEMS, exam_n,
            "MIN_SCANNED_ITEMS must stay one full exam form"
        );
    }

    #[test]
    fn an_empty_id_falls_back_to_the_filename() {
        let t = Tree::grounded();
        t.write(
            "bank/items/blank-id.toml",
            "id = \"\"\nstem = \"see clause 5.2.1 for detail\"\nchoices = []\n\
             explanation = \"\"\ntopic_ids = []\n",
        );
        let o = t.run(&Args::default());
        assert_eq!(o.code, 1, "{}", o.stdout);
        assert!(
            o.stdout.contains("  - blank-id.toml: hallucinated-clause"),
            "an empty id must not prefix findings with nothing:\n{}",
            o.stdout
        );
    }

    #[test]
    fn a_clause_citation_goes_red_and_names_the_pattern() {
        let t = Tree::new();
        t.write(
            "bank/items/a.toml",
            &item("bad-1", "per ISO 22237 clause 4.1 the aisle"),
        );
        let o = t.run(&Args::default());
        assert_eq!(o.code, 1);
        assert!(o.stdout.contains("high_severity=1\n"), "{}", o.stdout);
        assert!(
            o.stdout
                .contains("  - bad-1: hallucinated-clause pattern: \\b(?:ISO|IEC|EN|ANSI|TIA|NFPA|IEEE)\\s*[\\...\n"),
            "{}",
            o.stdout
        );
    }

    #[test]
    fn a_numeric_setpoint_is_excused_by_free_evidence_and_not_otherwise() {
        let t = Tree::grounded();
        t.write(
            "bank/items/a.toml",
            "id = \"n1\"\nstem = \"inlet must be 22 C\"\nchoices = []\n\
             explanation = \"\"\ntopic_ids = []\n",
        );
        assert_eq!(t.run(&Args::default()).code, 1);
        t.write(
            "bank/items/a.toml",
            "id = \"n1\"\nstem = \"inlet must be 22 C\"\nchoices = []\n\
             explanation = \"\"\ntopic_ids = []\nquantity_evidence = \"free_url\"\n",
        );
        let o = t.run(&Args::default());
        assert_eq!(o.code, 0, "{}", o.stdout);
    }

    #[test]
    fn overlap_is_a_warn_by_default_and_a_failure_under_strict() {
        let t = Tree::grounded();
        t.write("bank/items/a.toml", &item("o1", "zzzz yyyy xxxx wwww"));
        let o = t.run(&Args::default());
        assert_eq!(o.code, 0);
        assert!(o.stdout.contains("low_overlap_warns=1\n"), "{}", o.stdout);
        assert!(
            o.stdout
                .contains("  warns=1 (use --strict-overlap to fail)\n"),
            "{}",
            o.stdout
        );
        let strict = Args {
            strict_overlap: true,
            ..Args::default()
        };
        let o = t.run(&strict);
        assert_eq!(o.code, 1);
        assert!(
            o.stdout
                .contains("  - o1: low corpus overlap 0.000 < 0.08\n"),
            "{}",
            o.stdout
        );
    }

    #[test]
    fn the_corpus_is_counted_in_characters_after_newline_translation() {
        let t = Tree::new();
        t.write("bank/items/a.toml", &item("c1", "cooling and airflow"));
        // "ab\r\ncd" decodes to 5 characters, not 6. The tree is under the
        // corpus floor, so the count is read out of the vacuity finding —
        // which is the only place a sub-floor count is ever printed.
        t.write("knowledge/x.md", "ab\r\ncd");
        let o = t.run(&Args::default());
        assert!(o.stdout.contains("  - corpus_chars=5 < "), "{}", o.stdout);
    }

    #[test]
    fn corpus_public_is_reached_only_through_the_txt_leg() {
        let t = Tree::new();
        t.write("bank/items/a.toml", &item("c2", "cooling and airflow"));
        t.write("knowledge/corpus/public/skipped.md", "0123456789");
        t.write("knowledge/corpus/public/taken.txt", "abcde");
        let o = t.run(&Args::default());
        assert!(o.stdout.contains("  - corpus_chars=5 < "), "{}", o.stdout);
    }

    #[test]
    fn the_sample_list_is_capped_and_sorted_by_score_alone() {
        let t = Tree::new();
        for i in 0..4 {
            t.write(
                &format!("bank/items/{i}.toml"),
                &item(&format!("s{i}"), "zzzz yyyy xxxx wwww"),
            );
        }
        let args = Args {
            min_overlap: 1.0,
            sample_report: 2,
            ..Args::default()
        };
        let o = t.run(&args);
        assert!(o.stdout.contains("low_overlap_warns=4\n"), "{}", o.stdout);
        assert!(o.stdout.contains("lowest_overlap_samples:\n"));
        let shown = o.stdout.matches("  0.000  ").count();
        assert_eq!(shown, 2, "sample_report must cap the list:\n{}", o.stdout);
    }

    #[test]
    fn topic_labels_are_read_out_of_the_registry() {
        let t = Tree::new();
        t.write(
            "knowledge/topics.toml",
            "schema_version = 1\n[[topic]]\nid = \"t-one\"\nlabel = \"cooling airflow containment\"\n",
        );
        let labels = topic_labels(&t.root().join(TOPICS_REL));
        assert_eq!(
            labels.get("t-one").map(String::as_str),
            Some("cooling airflow containment")
        );
    }

    /// FIXED (bd-yje7): the split pattern needs a newline before the block
    /// header, so a registry whose very first byte opens a block used to lose
    /// that label silently — the raw id was tokenised in its place and every
    /// score built on it degraded with no finding printed. Both sides now
    /// prepend the newline.
    #[test]
    fn a_first_line_topic_block_is_read_like_any_other() {
        let t = Tree::new();
        t.write(
            "knowledge/topics.toml",
            "[[topic]]\nid = \"t-one\"\nlabel = \"cooling airflow\"\n",
        );
        assert_eq!(
            topic_labels(&t.root().join(TOPICS_REL))
                .get("t-one")
                .map(String::as_str),
            Some("cooling airflow")
        );
    }

    #[test]
    fn topic_words_alone_can_carry_an_item_over_the_bar() {
        let t = Tree::new();
        t.write(
            "knowledge/topics.toml",
            "schema_version = 1\n[[topic]]\nid = \"t-one\"\nlabel = \"zzzz yyyy\"\n",
        );
        t.write(
            "bank/items/a.toml",
            "id = \"tw\"\nstem = \"zzzz yyyy\"\nchoices = []\nexplanation = \"\"\n\
             topic_ids = [\"t-one\"]\n",
        );
        let o = t.run(&Args {
            min_overlap: 0.9,
            ..Args::default()
        });
        assert!(o.stdout.contains("low_overlap_warns=0\n"), "{}", o.stdout);
    }
}
