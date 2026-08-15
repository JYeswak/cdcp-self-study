//! verify-bank — Rust port of `scripts/verify_bank.py`
//! (bd-substrate-rust-migration-jhd.7).
//!
//! # WHICH POOL THE FLOORS MEASURE (bd-8exw)
//!
//! Every floor here — `pool_min_items`, each `[[domain_min]]`, and both
//! correct-letter diversity rules — is measured against the **approved** pool,
//! `status == "approved"`, and never against the file set. C1 restricts
//! assembly to approved items (`cdcp_assemble::sample_item_ids` filters on
//! `is_approved()`), so a floor measured over every loaded item is a promise
//! about a population the sampler will never draw from, and it can only ever
//! err in the generous direction — the file set is a superset of the drawable
//! pool. That is fail-open.
//!
//! Until 2026-08-13 the bank held exactly ONE non-approved item, so the file
//! count and the drawable count were the same number in every module and the
//! distinction was invisible. bd-tetz then retired 24 near-duplicates *in
//! place* — every FILE stayed — and this gate went on reporting `items=804`,
//! `multiplier≈20.1x` and `modules={… 14: 44 …}` about a pool that was really
//! 779 items and 42 in m14.
//!
//! Measured 2026-08-14 at the moment of the fix: 804 scanned, 779 approved, 25
//! retired. NO MODULE BREACHED its floor on the approved pool — tightest is m02
//! at 42 approved against 28, largest single drop is m06's 136 files to 130
//! drawable. A defect, not an incident.
//!
//! One number DID cross, and nothing gates it: `pool_target_items = 800` in
//! `bank_policy.toml` is aspirational. The file set (804) clears it, the
//! drawable pool (779) does not, and the printed multiplier fell 20.1x -> 19.5x.
//! Prose quoting "~20x" is quoting the file set.
//!
//! Two counts stay deliberately on the FILE SET and say so in the output:
//! `unique_ids` (a collision is a collision whatever the status) and the
//! `MANIFEST item_count` cross-check (drift is a property of the files on disk;
//! counting it on the approved pool would hide a retirement that never reached
//! the manifest).
//!
//! A status outside `approved`/`draft`/`retired` is an ERROR naming the item,
//! never a silent drop into "not approved". An absent `status` is `draft`,
//! matching `cdcp_bank::ItemStatus`'s serde default: silence is not approval.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **the item bank is large enough, structurally
//! well formed, and letter-diverse enough to sample a mock exam from.** It goes
//! RED when any of these hold —
//!
//!   0. *unmodelled status* — an item whose `status` is not one of
//!      `approved`/`draft`/`retired`.
//!   1. *pool too small* — fewer APPROVED items than `pool_min_items` (default
//!      400, i.e. ten exam forms), or zero approved items at all in a non-empty
//!      bank. A library that cannot outlast a learner is not a bank.
//!   2. *schema defect* on an item — missing/blank `id`, empty `stem`, a
//!      `choices` array that is not exactly four non-blank entries, `correct`
//!      outside A–D, an `explanation` under 12 characters, missing `topic_ids`,
//!      a `topic_id` absent from `knowledge/topics.toml`, `source_class` other
//!      than `original`, a `quantity_evidence` outside the `fact_policy.toml`
//!      allowlist, a `bloom` verb outside the taxonomy, or a `module` that is
//!      not coercible to an integer.
//!   3. *collision* — two items sharing an `id`, or two items sharing a stem.
//!   4. *starved domain* — a module whose APPROVED count is below its
//!      `[[domain_min]]` floor, so stratified sampling could never fill that
//!      slice.
//!   5. *letter monoculture* — in an APPROVED pool of 40 or more, one `correct`
//!      letter above 70%, or fewer than three distinct letters used at all.
//!   6. *manifest drift* — `bank/MANIFEST.toml`'s `item_count` disagreeing with
//!      the number of items actually loaded.
//!   7. *unusable policy* — a `pool_min_items` or `exam_n_items` present in
//!      `knowledge/bank_policy.toml` that is not an integer above zero.
//!
//! # TWO OUTPUT-SHAPE CONTRACTS (bd-hw3, fixed 2026-08-14 in the oracle first)
//!
//! - **A policy floor is a positive integer or the key is absent.** The oracle
//!   read these as `int(bp.get(key) or default)` until 2026-08-14. `or` treats
//!   `0` as absent, so `exam_n_items = 0` silently became 40, while the truthy
//!   `exam_n_items = "0"` survived to `n / exam_n` and raised ZeroDivisionError
//!   *after three lines of a PASS report had already been written*. A negative
//!   floor was worse again: it disabled the pool check and reported PASS. All of
//!   those spellings are now the same recorded finding on both sides, and
//!   [`PolicyInt::Bad`] is what carries it.
//! - **The verdict is composed before it is written.** Both sides build every
//!   line of the report and only then emit it, so a raise can cost you the
//!   verdict but can never leave a wrong one on stdout with a non-zero exit
//!   behind it. `evaluate` still returns whatever `Outcome.stdout` holds at the
//!   moment of a raise; the point of the restructure is that the verdict is
//!   never among it.
//!
//! Anti-vacuous (L4): a missing `bank/items/`, a missing `knowledge/topics.toml`,
//! a topics registry with zero ids, and an **empty** `bank/items/` are each a
//! non-zero exit, never a pass. An empty bank exits non-zero on two counts at
//! once — `zero items loaded` and `pool too small: 0 < pool_min_items N` — so a
//! bank that was never populated can never report like one that was checked and
//! came back clean. The rule holds at FILE granularity too (bd-0czh): a single
//! bank file whose `items[]` yields zero items is named and is RED, because
//! `zero items loaded` would otherwise stay satisfied by its neighbours.
//!
//! # WHAT THIS GATE CANNOT DECIDE
//!
//! It cannot tell whether an item is *correct*. Every check here is structural:
//! a stem of gibberish with four plausible-looking distractors, a `correct`
//! letter pointing at the wrong choice, and an explanation that contradicts the
//! answer all clear this gate exactly as a rigorous item does. It reads
//! `source_class = "original"` as a self-declaration and does nothing to detect
//! copied material — the field is an assertion by the author, not evidence. It
//! resolves `topic_ids` against the registry but says nothing about whether the
//! topic named is the *right* topic, nor whether the item's difficulty matches
//! its `bloom` verb. Letter diversity above 70% is a crude screen against an
//! all-B library; it detects nothing about positional bias inside a mock, and a
//! pool at 69% is as unbalanced in substance as one at 71%. The `domain_min`
//! floors count items, not coverage: twenty near-identical items satisfy a floor
//! of twenty. Grounding and citation quality belong to other gates entirely.
//!
//! The floor moves from *silence* to *every item parses, resolves, and fits the
//! schema, and the pool is big and varied enough to sample*. That is the whole
//! claim, and this header will not stretch it.
//!
//! # BYTE-EXACTNESS WITH THE PYTHON ORACLE
//!
//! `scripts/verify_bank.py` stays in the tree as the differential oracle for
//! this port. `tests/diff_verify_bank.rs` runs BOTH on every case — the live
//! repo, an empty bank, a missing bank, a missing registry, malformed fields,
//! duplicate ids, duplicate stems, domain shortfalls, the two diversity rules,
//! manifest drift, the 80-error truncation — and asserts stdout, stderr, and
//! exit code match byte for byte.
//!
//! Two consequences, both deliberate and both recorded here rather than made
//! quietly:
//!
//! - **The report goes to stdout and the process exits 1**, not through
//!   `GateError`. `GateError::report` writes to stderr and maps to exit 2 or 4,
//!   which the oracle never produces; routing through it would make the two
//!   sides differ on every RED case. `crate::exit`'s codes are therefore not
//!   used by this gate's verdict path. (An invocation error — this gate takes no
//!   arguments — still returns `GateError::Usage`, since the oracle has no
//!   opinion there; see `run`.)
//! - This module carries hand-written emulations of CPython behaviour —
//!   `str.strip`, `repr()` of a `str` and of a `float`, `int()` coercion,
//!   truthiness, `\s` under `re`, `dict` repr, floor division, `f"{x:.0%}"` —
//!   rather than the idiomatic Rust nearest-neighbour, because the acceptance
//!   bar is identical bytes and not merely an identical verdict.
//!
//! ## Modelling the oracle's uncaught exceptions
//!
//! Most of the oracle's exotic inputs (a non-string `stem`, an unhashable
//! `topic_id`, a non-numeric `pool_min_items`) raise rather than report. CPython
//! then flushes whatever was already printed, writes a traceback to stderr, and
//! exits 1. `Outcome` models that exactly: the partial stdout is kept, the exit
//! code is 1, and stderr carries a one-line description. **stdout and the exit
//! code stay byte-identical on those paths; the traceback text is the one
//! surface this port does not reproduce**, and `tests/diff_verify_bank.rs`
//! asserts precisely that (equal stdout, equal code, both stderrs non-empty).
//!
//! ## Relationship to `cdcp_bank::Bank::load_dir`
//!
//! Commit 5c98662 unified `BankItem::validate`'s rules with this script's, so
//! the two agree on *what is legal*. They deliberately do not agree on
//! *reporting*: `load_dir` is a typed serde loader that stops at the first bad
//! item and renders messages with Rust's `{:?}`, and it cannot express "no id or
//! items[]", the per-letter distribution, or the 80-line truncation. Delegating
//! to it would lose byte-exactness with the oracle, so this gate re-reads the
//! bank as untyped TOML. The rule sets are duplicated on purpose and the
//! duplication is load-bearing; `docs/TESTING.md` holds the parity table.
//!
//! ## Known residual deviations (each documented, none reachable from the live
//! tree or from any differential case)
//!
//! - Python `int` is unbounded; module keys here are `i128`. A `module` given as
//!   a float above ~1.7e38 saturates instead of widening.
//! - `repr()` of a `datetime` renders as the TOML datetime, not
//!   `datetime.datetime(...)`.
//! - `repr()` of a `dict` inside an error message iterates in key order (the
//!   `toml` crate's `Table` is sorted), where CPython iterates insertion order.
//! - `int("١٢")` (non-ASCII decimal digits) succeeds in CPython; here it is a
//!   `ValueError`, i.e. `bad module`.
//! - A `bank/items/` that exists but cannot be read yields zero files, matching
//!   `pathlib.Path.glob`'s swallowing of `PermissionError`; both sides then exit
//!   non-zero via `zero items loaded`.
//! - TOML parse errors and IO errors render with this crate's message text, not
//!   `tomllib`'s. Both sides exit 1 with the same (empty) stdout.

use crate::registry::{GateCtx, GateError};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::Value;

pub const NAME: &str = "verify-bank";
pub const SUMMARY: &str =
    "bank pool floors: schema, topic ids, source_class, domain minimums, correct-letter diversity";

/// Engine-root-relative paths, mirroring the oracle's module constants. The
/// oracle derives them from `Path(__file__).resolve().parents[1]` and never
/// prints them, so this gate's root resolution cannot affect its output bytes.
pub const ITEMS_DIR: &str = "bank/items";
pub const TOPICS_PATH: &str = "knowledge/topics.toml";
pub const MANIFEST_PATH: &str = "bank/MANIFEST.toml";
pub const FACT_POLICY_PATH: &str = "knowledge/fact_policy.toml";
pub const BANK_POLICY_PATH: &str = "knowledge/bank_policy.toml";

pub const ALLOWED_CORRECT: [&str; 4] = ["A", "B", "C", "D"];
pub const ALLOWED_BLOOM: [&str; 6] = [
    "remember",
    "understand",
    "apply",
    "analyze",
    "evaluate",
    "create",
];
/// The fallback used when `knowledge/fact_policy.toml` is absent or declares no
/// `allowed_quantity_evidence`.
pub const DEFAULT_QUANTITY_EVIDENCE: [&str; 4] = [
    "free_url",
    "licensed_note",
    "qualitative_only",
    "exam_form_public",
];

/// How many failure lines the report prints before truncating (`errors[:80]`).
pub const MAX_REPORT: usize = 80;
/// `pool_min` / `exam_n` before `knowledge/bank_policy.toml` overrides them.
pub const DEFAULT_POOL_MIN: i128 = 400;
pub const DEFAULT_EXAM_N: i128 = 40;
/// `len(explanation.strip()) < 12` is "too short".
pub const MIN_EXPLANATION_LEN: usize = 12;
/// The APPROVED pool size at or above which the letter-diversity rules apply.
/// Gated on the drawable count, not the file count: 40 files of which 39 are
/// retired is not a pool worth screening (bd-8exw).
pub const DIVERSITY_MIN_POOL: usize = 40;

/// C1 lifecycle. `APPROVED` is the ONLY status `cdcp_assemble` may draw, so it
/// is the only population a floor may be measured against.
pub const APPROVED: &str = "approved";
/// The statuses `cdcp_bank::ItemStatus` models. Anything else is a finding, not
/// a silent "not approved".
pub const KNOWN_STATUSES: [&str; 3] = ["approved", "draft", "retired"];

// ── the oracle's uncaught-exception channel ────────────────────────────────

/// A point where the oracle raises instead of reporting. CPython flushes the
/// stdout written so far, prints a traceback, and exits 1; so does this port,
/// minus the traceback text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raise(pub String);

type R<T> = Result<T, Raise>;

/// `int()` failed. `Caught` is the `except (TypeError, ValueError)` the module
/// loop wraps around `int(mod)`; `Uncaught` is everything that escapes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntErr {
    Caught(String),
    Uncaught(String),
}

// ── Python-behaviour emulations ────────────────────────────────────────────

/// The `\s` class of Python's `re` on `str` patterns, which is also the set
/// `str.strip()` removes: Unicode `White_Space` plus the four ASCII information
/// separators (0x1C–0x1F) that `str.isspace()` counts and Rust's
/// `char::is_whitespace` does not.
pub fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `str.strip()` with no argument.
pub fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

/// `repr()` of a Python `str`: single quotes unless the value contains `'` and
/// no `"`; backslash, the active quote, and the three named escapes escaped; C0
/// controls and DEL as `\xNN`. Non-ASCII passes through, matching CPython for
/// printable code points.
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

/// `repr()` of a Python `float`: shortest round-tripping digits, `.0` appended
/// to integral values, and exponent notation when the decimal point sits at or
/// left of -4 or right of 16 (CPython's `format_float_short`, format code 'r').
pub fn py_float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".to_string();
    }
    if x.is_infinite() {
        return if x < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    let sign = if x.is_sign_negative() { "-" } else { "" };
    let a = x.abs();
    if a == 0.0 {
        return format!("{sign}0.0");
    }
    // `{:e}` yields the shortest round-tripping mantissa, e.g. "1.234e2".
    let sci = format!("{a:e}");
    let (mant, exp) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let raw: String = mant.chars().filter(|c| *c != '.').collect();
    let trimmed = raw.trim_end_matches('0');
    let digits = if trimmed.is_empty() { "0" } else { trimmed };
    // value == 0.<digits> * 10^decpt
    let decpt = exp + 1;
    let body = if decpt <= -4 || decpt > 16 {
        let e2 = decpt - 1;
        let mut m = String::new();
        m.push_str(&digits[..1]);
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        format!(
            "{m}e{}{:02}",
            if e2 < 0 { '-' } else { '+' },
            e2.unsigned_abs()
        )
    } else if decpt <= 0 {
        format!("0.{}{digits}", "0".repeat((-decpt) as usize))
    } else if decpt as usize >= digits.len() {
        format!("{digits}{}.0", "0".repeat(decpt as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..decpt as usize],
            &digits[decpt as usize..]
        )
    };
    format!("{sign}{body}")
}

/// CPython's type name as it appears in a `TypeError`/`AttributeError` message.
pub fn py_type_name(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "'str'",
        Value::Integer(_) => "'int'",
        Value::Float(_) => "'float'",
        Value::Boolean(_) => "'bool'",
        Value::Datetime(_) => "'datetime.datetime'",
        Value::Array(_) => "'list'",
        Value::Table(_) => "'dict'",
    }
}

/// `str(v)` for a value `tomllib` would have produced.
pub fn py_str_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => py_float_repr(*f),
        Value::Boolean(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Datetime(d) => d.to_string(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter()
                .map(py_repr_value_inner)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Table(t) => format!(
            "{{{}}}",
            t.iter()
                .map(|(k, v)| format!("{}: {}", py_repr(k), py_repr_value_inner(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn py_repr_value_inner(v: &Value) -> String {
    match v {
        Value::String(s) => py_repr(s),
        other => py_str_value(other),
    }
}

/// `repr(v)`, where an absent key is Python's `None`.
pub fn py_repr_value(v: Option<&Value>) -> String {
    match v {
        None => "None".to_string(),
        Some(other) => py_repr_value_inner(other),
    }
}

/// Python truthiness.
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

/// A key that collides exactly when Python's `==`/`hash` would, so `x in set`
/// can be answered with a `BTreeSet`. `1`, `1.0`, and `True` share a key
/// because CPython treats them as the same dict/set member. Unhashable values
/// raise, as they do in CPython.
pub fn hash_key(v: Option<&Value>) -> R<String> {
    Ok(match v {
        None => "n:None".to_string(),
        Some(Value::String(s)) => format!("s:{s}"),
        Some(Value::Integer(i)) => format!("i:{i}"),
        Some(Value::Boolean(b)) => format!("i:{}", i32::from(*b)),
        Some(Value::Float(f)) => {
            if f.is_finite() && *f == f.trunc() && f.abs() < 9.0e18 {
                format!("i:{}", *f as i64)
            } else {
                format!("f:{}", py_float_repr(*f))
            }
        }
        Some(Value::Datetime(d)) => format!("d:{d}"),
        Some(other @ Value::Array(_)) | Some(other @ Value::Table(_)) => {
            return Err(Raise(format!(
                "TypeError: unhashable type: {}",
                py_type_name(other)
            )))
        }
    })
}

/// `v in set`, where `set` holds [`hash_key`]s.
pub fn set_contains(set: &BTreeSet<String>, v: Option<&Value>) -> R<bool> {
    Ok(set.contains(&hash_key(v)?))
}

fn key_set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| format!("s:{s}")).collect()
}

/// `for x in v` — the container protocols `tomllib` values can satisfy.
pub fn py_iter(v: &Value) -> R<Vec<Value>> {
    Ok(match v {
        Value::Array(a) => a.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Table(t) => t.keys().map(|k| Value::String(k.clone())).collect(),
        other => {
            return Err(Raise(format!(
                "TypeError: {} object is not iterable",
                py_type_name(other)
            )))
        }
    })
}

/// `int(s)` for a `str`: surrounding whitespace, an optional sign, then ASCII
/// digits with single underscores between them.
pub fn py_int_from_str(s: &str) -> Option<i128> {
    let t = py_strip(s);
    let (neg, rest) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    if rest.is_empty() {
        return None;
    }
    let mut prev_underscore = true; // a leading underscore is invalid
    let mut digits = String::with_capacity(rest.len());
    for c in rest.chars() {
        if c == '_' {
            if prev_underscore {
                return None;
            }
            prev_underscore = true;
            continue;
        }
        if !c.is_ascii_digit() {
            return None;
        }
        prev_underscore = false;
        digits.push(c);
    }
    if prev_underscore {
        return None;
    }
    let mag = digits.parse::<i128>().ok()?;
    Some(if neg { -mag } else { mag })
}

/// `int(v)`, distinguishing the errors the module loop catches from the ones
/// that escape it (`OverflowError` on an infinite float).
pub fn py_int(v: Option<&Value>) -> Result<i128, IntErr> {
    match v {
        None => Err(IntErr::Caught(
            "TypeError: int() argument must be a string, a bytes-like object or a real number, not 'NoneType'".to_string(),
        )),
        Some(Value::Integer(i)) => Ok(*i as i128),
        Some(Value::Boolean(b)) => Ok(i128::from(*b)),
        Some(Value::Float(f)) => {
            if f.is_nan() {
                Err(IntErr::Caught(
                    "ValueError: cannot convert float NaN to integer".to_string(),
                ))
            } else if f.is_infinite() {
                Err(IntErr::Uncaught(
                    "OverflowError: cannot convert float infinity to integer".to_string(),
                ))
            } else {
                Ok(f.trunc() as i128)
            }
        }
        Some(Value::String(s)) => py_int_from_str(s).ok_or_else(|| {
            IntErr::Caught(format!(
                "ValueError: invalid literal for int() with base 10: {}",
                py_repr(s)
            ))
        }),
        Some(other) => Err(IntErr::Caught(format!(
            "TypeError: int() argument must be a string, a bytes-like object or a real number, not {}",
            py_type_name(other)
        ))),
    }
}

/// `int(v)` at a call site with no `except` around it.
fn py_int_uncaught(v: Option<&Value>) -> R<i128> {
    py_int(v).map_err(|e| match e {
        IntErr::Caught(m) | IntErr::Uncaught(m) => Raise(m),
    })
}

/// Python's `//` on ints: floors toward negative infinity, and raises on zero.
pub fn py_floordiv(a: i128, b: i128) -> R<i128> {
    if b == 0 {
        return Err(Raise(
            "ZeroDivisionError: integer division or modulo by zero".to_string(),
        ));
    }
    let q = a / b;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        Ok(q - 1)
    } else {
        Ok(q)
    }
}

/// `f"{x:.0%}"` — CPython multiplies by 100 as a double, formats fixed with the
/// given precision (round-half-to-even on the exact binary value, which is what
/// Rust's `{:.N}` also does), then appends `%`.
pub fn py_percent0(frac: f64) -> String {
    format!("{:.0}%", frac * 100.0)
}

/// `f"{x:.1f}"`.
pub fn py_fixed1(x: f64) -> String {
    format!("{x:.1}")
}

/// `repr(dict)` for the `{'A': 198, ...}` shape.
pub fn py_dict_str_keys(m: &BTreeMap<String, u64>) -> String {
    let body: Vec<String> = m
        .iter()
        .map(|(k, v)| format!("{}: {v}", py_repr(k)))
        .collect();
    format!("{{{}}}", body.join(", "))
}

/// `repr(dict)` for the `{1: 36, ...}` shape.
pub fn py_dict_int_keys(m: &BTreeMap<i128, u64>) -> String {
    let body: Vec<String> = m.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    format!("{{{}}}", body.join(", "))
}

/// `str(list_of_str)`.
fn py_list_of_str(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|s| py_repr(s))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

// ── file access ────────────────────────────────────────────────────────────

/// `Path.read_text(encoding="utf-8")`, including universal-newline translation.
pub fn read_text_universal(path: &Path) -> R<String> {
    let bytes =
        std::fs::read(path).map_err(|e| Raise(format!("OSError: {}: {e}", path.display())))?;
    let s = String::from_utf8(bytes).map_err(|_| {
        Raise(format!(
            "UnicodeDecodeError: {} is not utf-8",
            path.display()
        ))
    })?;
    Ok(s.replace("\r\n", "\n").replace('\r', "\n"))
}

/// `tomllib.load(open(path, "rb"))`.
pub fn load_toml(path: &Path) -> R<toml::Table> {
    let bytes =
        std::fs::read(path).map_err(|e| Raise(format!("OSError: {}: {e}", path.display())))?;
    let text = String::from_utf8(bytes).map_err(|_| {
        Raise(format!(
            "UnicodeDecodeError: {} is not utf-8",
            path.display()
        ))
    })?;
    text.parse::<toml::Table>()
        .map_err(|e| Raise(format!("TOMLDecodeError: {}: {e}", path.display())))
}

/// The `(?m)^\s*id\s*=\s*"([^"]+)"` scan `topic_ids_from_registry` runs, in
/// match order. `\s*` may span newlines, so this is deliberately not a
/// line-by-line loop; `^` still anchors every attempt to a line start.
pub fn find_topic_ids(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos <= n {
        let at_line_start = pos == 0 || chars[pos - 1] == '\n';
        if at_line_start {
            if let Some((val, end)) = match_id_at(&chars, pos) {
                out.push(val);
                pos = if end > pos { end } else { pos + 1 };
                continue;
            }
        }
        pos += 1;
    }
    out
}

/// One attempt at `\s*id\s*=\s*"([^"]+)"` anchored at `start`. No backtracking
/// is needed: every `\s*` here is followed by a non-space literal, so the greedy
/// run has exactly one viable end, and `[^"]+` ends exactly at the next quote.
fn match_id_at(c: &[char], start: usize) -> Option<(String, usize)> {
    let n = c.len();
    let mut i = start;
    while i < n && py_space(c[i]) {
        i += 1;
    }
    if i + 1 >= n || c[i] != 'i' || c[i + 1] != 'd' {
        return None;
    }
    i += 2;
    while i < n && py_space(c[i]) {
        i += 1;
    }
    if i >= n || c[i] != '=' {
        return None;
    }
    i += 1;
    while i < n && py_space(c[i]) {
        i += 1;
    }
    if i >= n || c[i] != '"' {
        return None;
    }
    i += 1;
    let s = i;
    while i < n && c[i] != '"' {
        i += 1;
    }
    if i >= n || i == s {
        return None;
    }
    Some((c[s..i].iter().collect(), i + 1))
}

/// `sorted(ITEMS_DIR.glob("*.toml"))`. `pathlib` does not hide dotfiles and `*`
/// matches the empty string, so a file literally named `.toml` is in scope.
fn item_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().ends_with(".toml"))
                .unwrap_or(false)
        })
        .collect();
    out.sort();
    out
}

// ── the gate ───────────────────────────────────────────────────────────────

/// Exactly what the oracle writes, and the status it exits with. `stderr` is
/// non-empty only on the uncaught-exception paths, where CPython would print a
/// traceback instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// The outcome of reading one `bank_policy.toml` floor, mirroring the oracle's
/// `policy_positive_int`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyInt {
    /// The key was absent, so the built-in default stands.
    Default(i128),
    /// The key held an integer above zero.
    Given(i128),
    /// The key held something else. A finding was recorded; the check that
    /// consumes this value must be skipped, never defaulted.
    Bad,
}

impl PolicyInt {
    /// The usable value, or `None` when the policy was rejected.
    pub fn value(self) -> Option<i128> {
        match self {
            PolicyInt::Default(v) | PolicyInt::Given(v) => Some(v),
            PolicyInt::Bad => None,
        }
    }
}

/// `policy_positive_int(bp, key, default, errors)`.
///
/// Absent → `default`. Present → coerced with Python's `int()` and required to
/// be above zero; `0`, `"0"`, a negative and non-numeric junk are one and the
/// same finding. An `OverflowError` (a float infinity) escapes the oracle's
/// `except (TypeError, ValueError)` and so raises here too.
pub fn policy_positive_int(
    table: &toml::Table,
    key: &str,
    default: i128,
    errors: &mut Vec<String>,
) -> R<PolicyInt> {
    let Some(raw) = table.get(key) else {
        return Ok(PolicyInt::Default(default));
    };
    let val = match py_int(Some(raw)) {
        Ok(v) => v,
        Err(IntErr::Caught(_)) => {
            errors.push(format!(
                "bank_policy.toml: {key} must be an integer, got {}",
                py_repr_value(Some(raw))
            ));
            return Ok(PolicyInt::Bad);
        }
        Err(IntErr::Uncaught(m)) => return Err(Raise(m)),
    };
    if val <= 0 {
        errors.push(format!("bank_policy.toml: {key} must be > 0, got {val}"));
        return Ok(PolicyInt::Bad);
    }
    Ok(PolicyInt::Given(val))
}

/// Run the whole check against `root` and render the oracle's report.
pub fn evaluate(root: &Path) -> Outcome {
    let mut out = String::new();
    match main_impl(root, &mut out) {
        Ok(code) => Outcome {
            stdout: out,
            stderr: String::new(),
            code,
        },
        Err(Raise(msg)) => Outcome {
            // CPython flushes what was already printed before the traceback.
            stdout: out,
            stderr: format!("verify-bank: {msg}\n"),
            code: 1,
        },
    }
}

fn main_impl(root: &Path, out: &mut String) -> R<i32> {
    let items_dir = root.join(ITEMS_DIR);
    let topics_path = root.join(TOPICS_PATH);
    let manifest_path = root.join(MANIFEST_PATH);
    let fact_policy_path = root.join(FACT_POLICY_PATH);
    let bank_policy_path = root.join(BANK_POLICY_PATH);

    if !items_dir.is_dir() {
        out.push_str("FAIL: bank/items/ missing\n");
        return Ok(1);
    }
    if !topics_path.is_file() {
        out.push_str("FAIL: knowledge/topics.toml missing\n");
        return Ok(1);
    }

    let mut errors: Vec<String> = Vec::new();

    let topics_text = read_text_universal(&topics_path)?;
    let known: BTreeSet<String> = find_topic_ids(&topics_text)
        .iter()
        .map(|id| format!("s:{id}"))
        .collect();
    if known.is_empty() {
        errors.push("topics.toml has zero topic ids".to_string());
    }

    // fact_policy.toml may replace the quantity_evidence allowlist.
    let mut allowed_qe = key_set(&DEFAULT_QUANTITY_EVIDENCE);
    if fact_policy_path.is_file() {
        let pol = load_toml(&fact_policy_path)?;
        if let Some(v) = pol.get("allowed_quantity_evidence") {
            if py_truthy(v) {
                let mut s = BTreeSet::new();
                for item in py_iter(v)? {
                    s.insert(hash_key(Some(&item))?);
                }
                allowed_qe = s;
            }
        }
    }

    // bank_policy.toml may replace the pool floors and add domain minimums.
    let mut pool_min = PolicyInt::Default(DEFAULT_POOL_MIN);
    let mut exam_n = PolicyInt::Default(DEFAULT_EXAM_N);
    let mut domain_mins: BTreeMap<i128, i128> = BTreeMap::new();
    if bank_policy_path.is_file() {
        let bp = load_toml(&bank_policy_path)?;
        // Source order matters: these two findings land between the topics
        // check and the per-file load errors, exactly as the oracle's do.
        pool_min = policy_positive_int(&bp, "pool_min_items", DEFAULT_POOL_MIN, &mut errors)?;
        exam_n = policy_positive_int(&bp, "exam_n_items", DEFAULT_EXAM_N, &mut errors)?;
        if let Some(v) = bp.get("domain_min") {
            if py_truthy(v) {
                for row in py_iter(v)? {
                    let t = row.as_table().ok_or_else(|| {
                        Raise(format!(
                            "TypeError: {} indices must be integers",
                            py_type_name(&row)
                        ))
                    })?;
                    let module = t
                        .get("module")
                        .ok_or_else(|| Raise("KeyError: 'module'".to_string()))?;
                    let min_items = t
                        .get("min_items")
                        .ok_or_else(|| Raise("KeyError: 'min_items'".to_string()))?;
                    let m = py_int_uncaught(Some(module))?;
                    let k = py_int_uncaught(Some(min_items))?;
                    domain_mins.insert(m, k);
                }
            }
        }
    }

    // ── load the bank ──────────────────────────────────────────────────────
    let mut loaded: Vec<(String, Value)> = Vec::new();
    for path in item_files(&items_dir) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let data = load_toml(&path)?;
        match data.get("items") {
            Some(Value::Array(arr)) => {
                let before = loaded.len();
                for it in arr {
                    loaded.push((name.clone(), it.clone()));
                }
                if loaded.len() == before {
                    // Anti-vacuous at FILE granularity (bd-0czh). An `items[]`
                    // that yields nothing took this branch and can never reach
                    // the `no id or items[]` leg — Python's `elif` cannot run
                    // once the `if` has — so without this the file is scanned,
                    // contributes zero items, and is never named while the
                    // aggregate `zero items loaded` check stays satisfied by
                    // its neighbours.
                    errors.push(format!(
                        "{name}: items[] yielded zero items (vacuous file scan is ERROR)"
                    ));
                }
            }
            _ => {
                if data.contains_key("id") {
                    loaded.push((name, Value::Table(data)));
                } else {
                    errors.push(format!("{name}: no id or items[]"));
                }
            }
        }
    }

    let n = loaded.len();
    let (approved_n, status_errors) = count_approved(&loaded)?;
    errors.extend(status_errors);

    if n == 0 {
        errors.push("zero items loaded".to_string());
    } else if approved_n == 0 {
        // A bank FULL of files and empty of drawable items is the exact state a
        // file-counting floor reported green on. Named separately from the
        // empty-bank leg because it is a different failure with the same
        // verdict.
        errors.push(format!(
            "zero approved items ({n} scanned): the floors measure a pool no \
             learner can be assessed from (vacuous scan is ERROR)"
        ));
    }
    // Skipped only when the floor itself is unusable — that config error is
    // already recorded above, so this can never turn a bad policy into a pass.
    //
    // Measured against `approved_n`, never `n` (bd-8exw). Both numbers are in
    // the message: `804 scanned` under a floor of 400 is exactly the reading
    // that let this fail open once the bank grew a real retired set.
    if let (Some(floor), Some(size)) = (pool_min.value(), exam_n.value()) {
        if (approved_n as i128) < floor {
            let multiple = py_floordiv(floor, size)?;
            errors.push(format!(
                "pool too small: {approved_n} approved < pool_min_items {floor} \
                 ({n} scanned, {} not approved; need ≥{multiple}× exam size {size})",
                n - approved_n
            ));
        }
    }

    // ── per-item schema ────────────────────────────────────────────────────
    let allowed_correct = key_set(&ALLOWED_CORRECT);
    let allowed_bloom = key_set(&ALLOWED_BLOOM);

    let mut ids: Vec<String> = Vec::new();
    let mut letter_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut module_counts: BTreeMap<i128, u64> = BTreeMap::new();
    let mut scanned_module_counts: BTreeMap<i128, u64> = BTreeMap::new();

    for (fname, raw) in &loaded {
        let it = as_item(raw)?;
        let Some(iid) = item_id(it) else {
            errors.push(format!("{fname}: missing id"));
            continue;
        };
        ids.push(iid.clone());
        let drawable = is_approved(it);

        let stem = item_text(it, "stem")?;
        if stem.is_empty() {
            errors.push(format!("{iid}: empty stem"));
        }

        match it.get("choices") {
            Some(Value::Array(a)) if a.len() == 4 => {
                if a.iter().any(|c| py_strip(&py_str_value(c)).is_empty()) {
                    errors.push(format!("{iid}: empty choice text"));
                }
            }
            _ => errors.push(format!("{iid}: choices must be length 4")),
        }

        let correct = it.get("correct");
        if set_contains(&allowed_correct, correct)? {
            // Letter diversity is a claim about the pool a mock is sampled
            // from. Measured over the file set it is a claim about a population
            // `sample_item_ids` never sees (bd-8exw).
            if drawable {
                let letter = py_str_value(correct.expect("membership implies presence"));
                *letter_counts.entry(letter).or_insert(0) += 1;
            }
        } else {
            errors.push(format!(
                "{iid}: correct must be A-D, got {}",
                py_repr_value(correct)
            ));
        }

        let expl = item_text(it, "explanation")?;
        if expl.chars().count() < MIN_EXPLANATION_LEN {
            errors.push(format!("{iid}: explanation too short"));
        }

        let tids = match it.get("topic_ids") {
            Some(v) if py_truthy(v) => py_iter(v)?,
            _ => Vec::new(),
        };
        if tids.is_empty() {
            errors.push(format!("{iid}: topic_ids required"));
        }
        for t in &tids {
            if !set_contains(&known, Some(t))? {
                errors.push(format!(
                    "{iid}: unknown topic_id {}",
                    py_repr_value(Some(t))
                ));
            }
        }

        let sc = it.get("source_class");
        if !matches!(sc, Some(Value::String(s)) if s == "original") {
            errors.push(format!(
                "{iid}: source_class must be original, got {}",
                py_repr_value(sc)
            ));
        }

        let qe = it.get("quantity_evidence");
        if !set_contains(&allowed_qe, qe)? {
            errors.push(format!(
                "{iid}: bad quantity_evidence {}",
                py_repr_value(qe)
            ));
        }

        let bloom = it.get("bloom");
        if !set_contains(&allowed_bloom, bloom)? {
            errors.push(format!("{iid}: bad bloom {}", py_repr_value(bloom)));
        }

        let module = it.get("module");
        match py_int(module) {
            Ok(mi) => {
                *scanned_module_counts.entry(mi).or_insert(0) += 1;
                if drawable {
                    *module_counts.entry(mi).or_insert(0) += 1;
                }
            }
            Err(IntErr::Caught(_)) => {
                errors.push(format!("{iid}: bad module {}", py_repr_value(module)))
            }
            Err(IntErr::Uncaught(m)) => return Err(Raise(m)),
        }
    }

    // ── duplicate ids (Counter order == first appearance) ──────────────────
    let unique_ids: BTreeSet<&String> = ids.iter().collect();
    if ids.len() != unique_ids.len() {
        let mut order: Vec<String> = Vec::new();
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for id in &ids {
            let c = counts.entry(id.as_str()).or_insert(0);
            *c += 1;
            if *c == 1 {
                order.push(id.clone());
            }
        }
        let dup: Vec<String> = order
            .into_iter()
            .filter(|id| counts.get(id.as_str()).copied().unwrap_or(0) > 1)
            .take(10)
            .collect();
        errors.push(format!("duplicate ids: {}", py_list_of_str(&dup)));
    }

    // ── duplicate stems, heaviest group first then by first id ─────────────
    let mut stem_groups: Vec<(String, Vec<String>)> = Vec::new();
    let mut stem_index: BTreeMap<String, usize> = BTreeMap::new();
    for (_fname, raw) in &loaded {
        let it = as_item(raw)?;
        let Some(iid) = item_id(it) else { continue };
        let stem = item_text(it, "stem")?;
        if stem.is_empty() {
            continue;
        }
        match stem_index.get(&stem) {
            Some(&i) => stem_groups[i].1.push(iid),
            None => {
                stem_index.insert(stem.clone(), stem_groups.len());
                stem_groups.push((stem, vec![iid]));
            }
        }
    }
    let mut order: Vec<usize> = (0..stem_groups.len()).collect();
    // Stable sort on (-len(group), group[0]), matching the oracle's sort key.
    order.sort_by(|&a, &b| {
        stem_groups[b]
            .1
            .len()
            .cmp(&stem_groups[a].1.len())
            .then_with(|| stem_groups[a].1[0].cmp(&stem_groups[b].1[0]))
    });
    for i in order {
        let (stem, group) = &stem_groups[i];
        if group.len() > 1 {
            let head: String = stem.chars().take(100).collect();
            errors.push(format!(
                "duplicate stem ({} items {}): {}",
                group.len(),
                py_list_of_str(group),
                py_repr(&head)
            ));
        }
    }

    // ── per-domain floors, on the approved pool ────────────────────────────
    // Both numbers in the message: `44 scanned` under a floor of 24 is exactly
    // the reading that hid the shortfall (bd-8exw).
    for (module, need) in &domain_mins {
        let have = module_counts.get(module).copied().unwrap_or(0) as i128;
        let seen = scanned_module_counts.get(module).copied().unwrap_or(0) as i128;
        if have < *need {
            errors.push(format!(
                "module {module}: {have} approved items < domain_min {need} \
                 ({seen} scanned, {} not approved)",
                seen - have
            ));
        }
    }

    // ── correct-letter diversity, on the approved pool ─────────────────────
    if approved_n >= DIVERSITY_MIN_POOL {
        // The oracle iterated a frozenset here until 2026-08-14, so its
        // emission order was PYTHONHASHSEED-dependent; it now iterates its own
        // `CORRECT_LETTERS` tuple, matching this A–D order by construction
        // rather than by the arithmetic accident that no two letters can both
        // exceed 70%. `tests/diff_verify_bank.rs` pins both the tuple and the
        // cross-seed byte-equality of the oracle's stdout.
        for letter in ALLOWED_CORRECT {
            let count = letter_counts.get(letter).copied().unwrap_or(0);
            let frac = count as f64 / approved_n as f64;
            if frac > 0.70 {
                errors.push(format!(
                    "correct={letter} is {} of approved pool (max 70% for diversity)",
                    py_percent0(frac)
                ));
            }
        }
        let used = ALLOWED_CORRECT
            .iter()
            .filter(|l| letter_counts.get(**l).copied().unwrap_or(0) > 0)
            .count();
        if used < 3 {
            errors
                .push("need at least 3 distinct correct letters in the approved pool".to_string());
        }
    }

    // ── MANIFEST cross-check ───────────────────────────────────────────────
    // Deliberately the FILE SET (`n`), not the approved pool: manifest drift is
    // a property of the files on disk, and a retirement that never reached the
    // manifest is exactly the drift this catches. Counting it on the approved
    // pool would hide that (bd-8exw).
    if manifest_path.is_file() {
        let man = load_toml(&manifest_path)?;
        // A TOML value is never `None`, so any present `item_count` is checked.
        if let Some(mc) = man.get("item_count") {
            if py_int_uncaught(Some(mc))? != n as i128 {
                errors.push(format!(
                    "MANIFEST item_count {} != loaded {n}",
                    py_str_value(mc)
                ));
            }
        }
    }

    // ── report ─────────────────────────────────────────────────────────────
    // Composed in full before a single byte of it is written. A gate that
    // prints PASS and then dies leaves stdout and CI disagreeing, and which one
    // wins depends on whether anyone looked.
    if !errors.is_empty() {
        let mut report = String::from("FAIL\n");
        for e in errors.iter().take(MAX_REPORT) {
            report.push_str(&format!("  - {e}\n"));
        }
        if errors.len() > MAX_REPORT {
            report.push_str(&format!("  ... +{} more\n", errors.len() - MAX_REPORT));
        }
        out.push_str(&report);
        return Ok(1);
    }

    // Unreachable with a rejected policy: `PolicyInt::Bad` recorded a finding,
    // and a non-empty `errors` returned above.
    let (Some(floor), Some(size)) = (pool_min.value(), exam_n.value()) else {
        return Err(Raise(
            "AssertionError: pool_min/exam_n rejected but no finding recorded".to_string(),
        ));
    };
    // Every count names its population. The two that are FILE-SET properties —
    // `unique_ids` (a collision is a collision whatever the status) and the
    // MANIFEST cross-check above — say so; everything a floor consumes is the
    // approved pool.
    let report = format!(
        "PASS\n  \
         items={n} scanned, {approved_n} approved (floors count the approved pool only)\n  \
         unique_ids={} (file set)\n  \
         pool_min={floor} exam_n={size} multiplier≈{}x (approved pool)\n  \
         topics_registry={}\n  \
         domain_floors={} checked (approved pool)\n  \
         correct_dist(approved)={}\n  \
         modules(approved)={}\n  \
         modules(scanned)={}\n  \
         source_class=original\n",
        unique_ids.len(),
        py_fixed1(approved_n as f64 / size as f64),
        known.len(),
        // How many per-module floors were actually enforced. A policy that lost
        // its `[[domain_min]]` rows reports identically to one that checked
        // fifteen of them; printing the count makes a zero READ as zero instead
        // of as silence. Whether zero should be RED is bd-bank-zero-domain-floors-vacuous-o80a.
        domain_mins.len(),
        py_dict_str_keys(&letter_counts),
        py_dict_int_keys(&module_counts),
        py_dict_int_keys(&scanned_module_counts),
    );
    out.push_str(&report);
    Ok(0)
}

/// `it.get("status", "draft") == "approved"` — is this item in the pool
/// `cdcp_assemble` may draw from?
///
/// One definition, used by the aggregate pass and by the per-module and
/// per-letter tallies, so the two can never drift into measuring different
/// populations — which is exactly the defect bd-8exw records.
pub fn is_approved(it: &toml::Table) -> bool {
    matches!(it.get("status"), Some(Value::String(s)) if s == APPROVED)
}

/// `it.get("status", "draft") not in KNOWN_STATUSES` is a finding. An absent
/// key resolves to the `"draft"` default and is therefore always known; a
/// non-`str` value can never equal a member of the tuple, so it is not.
fn status_is_known(it: &toml::Table) -> bool {
    match it.get("status") {
        None => true,
        Some(Value::String(s)) => KNOWN_STATUSES.contains(&s.as_str()),
        Some(_) => false,
    }
}

/// `it.get("id") or fname`, rendered by an f-string, i.e. `str()` of whichever
/// won. A falsy or absent id falls back to the file name.
fn item_label(it: &toml::Table, fname: &str) -> String {
    match it.get("id") {
        Some(v) if py_truthy(v) => py_str_value(v),
        _ => fname.to_string(),
    }
}

/// `count_approved(loaded)` — `(approved_n, errors)` over everything that
/// loaded.
///
/// `approved_n` is the drawable pool: what every floor in this gate is measured
/// against. The caller keeps `loaded.len()` separately so the report can print
/// BOTH numbers; a report that showed only one of them is how a floor came to
/// be checked against a set no learner draws from.
pub fn count_approved(loaded: &[(String, Value)]) -> R<(usize, Vec<String>)> {
    let mut approved = 0usize;
    let mut errors: Vec<String> = Vec::new();
    for (fname, raw) in loaded {
        // `it.get` on a non-mapping raises here, one loop earlier than it used
        // to; stdout is empty at this point either way, so the oracle and the
        // port still agree byte for byte on that path.
        let it = as_item(raw)?;
        if is_approved(it) {
            approved += 1;
        } else if !status_is_known(it) {
            // Fail-closed AND loud. Dropping an unmodelled status silently into
            // "not approved" would be a bucket decided by guess rather than by
            // the recorded lifecycle.
            errors.push(format!(
                "{}: unknown status {}",
                item_label(it, fname),
                py_repr_value(it.get("status"))
            ));
        }
    }
    Ok((approved, errors))
}

/// `it.get(...)` requires a mapping; anything else raises `AttributeError`.
fn as_item(raw: &Value) -> R<&toml::Table> {
    raw.as_table().ok_or_else(|| {
        Raise(format!(
            "AttributeError: {} object has no attribute 'get'",
            py_type_name(raw)
        ))
    })
}

/// `iid` if it is a non-empty `str`, else `None` (the oracle's "missing id").
fn item_id(it: &toml::Table) -> Option<String> {
    match it.get("id") {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// `(it.get(key) or "").strip()` — falsy is `""`, a truthy non-`str` raises.
fn item_text(it: &toml::Table, key: &str) -> R<String> {
    match it.get(key) {
        None => Ok(String::new()),
        Some(v) if !py_truthy(v) => Ok(String::new()),
        Some(Value::String(s)) => Ok(py_strip(s).to_string()),
        Some(other) => Err(Raise(format!(
            "AttributeError: {} object has no attribute 'strip'",
            py_type_name(other)
        ))),
    }
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    // The oracle takes no arguments and silently ignores any it is given. This
    // port rejects them instead: a typo'd flag must not read as "the gate
    // passed". That is the single deliberate divergence outside the verdict
    // path, and it cannot change the bytes of any argument-free invocation.
    if let Some(a) = ctx.args.first() {
        return Err(GateError::usage(format!(
            "verify-bank takes no arguments; got {a:?}"
        )));
    }

    let outcome = evaluate(&ctx.root);
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    if !outcome.stderr.is_empty() {
        eprint!("{}", outcome.stderr);
        let _ = std::io::stderr().flush();
    }
    if outcome.code != 0 {
        // See the module header: the oracle exits 1 with this report on stdout,
        // and byte-identical output is this port's acceptance bar. Routing
        // through `GateError` would write to stderr and exit 2 instead.
        std::process::exit(outcome.code);
    }
    Ok(())
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn v_str(s: &str) -> Value {
        Value::String(s.to_string())
    }

    // ── the Python emulations ─────────────────────────────────────────────

    #[test]
    fn strip_covers_the_information_separators() {
        assert_eq!(py_strip("\u{1c}\u{1f} x \t\n"), "x");
        assert!(py_space('\u{1d}'));
        assert!(py_space('\u{00a0}'));
    }

    #[test]
    fn repr_of_a_str_matches_cpython() {
        assert_eq!(py_repr("abc"), "'abc'");
        assert_eq!(py_repr("it's"), "\"it's\"");
        assert_eq!(py_repr("it's a \"q\""), "'it\\'s a \"q\"'");
        assert_eq!(py_repr("a\nb"), "'a\\nb'");
        assert_eq!(py_repr("a\\b"), "'a\\\\b'");
        assert_eq!(py_repr("\u{7f}"), "'\\x7f'");
    }

    #[test]
    fn repr_of_a_float_matches_cpython() {
        assert_eq!(py_float_repr(1.0), "1.0");
        assert_eq!(py_float_repr(-0.0), "-0.0");
        assert_eq!(py_float_repr(3.9), "3.9");
        assert_eq!(py_float_repr(0.0001), "0.0001");
        assert_eq!(py_float_repr(0.00001), "1e-05");
        assert_eq!(py_float_repr(1e15), "1000000000000000.0");
        assert_eq!(py_float_repr(1e16), "1e+16");
        assert_eq!(py_float_repr(f64::NAN), "nan");
        assert_eq!(py_float_repr(f64::NEG_INFINITY), "-inf");
    }

    #[test]
    fn percent_and_fixed_formats_round_half_to_even_like_cpython() {
        // Measured against CPython 3.14: '70%', '71%', '20.1'.
        assert_eq!(py_percent0(0.705), "70%");
        assert_eq!(py_percent0(29.0 / 41.0), "71%");
        assert_eq!(py_percent0(1.0), "100%");
        assert_eq!(py_fixed1(804.0 / 40.0), "20.1");
    }

    #[test]
    fn int_coercion_matches_cpython() {
        assert_eq!(py_int(Some(&v_str("  07_2 "))), Ok(72));
        assert_eq!(py_int(Some(&v_str("-4"))), Ok(-4));
        assert_eq!(py_int(Some(&Value::Float(3.9))), Ok(3));
        assert_eq!(py_int(Some(&Value::Float(-3.9))), Ok(-3));
        assert_eq!(py_int(Some(&Value::Boolean(true))), Ok(1));
        assert!(matches!(py_int(None), Err(IntErr::Caught(_))));
        assert!(matches!(
            py_int(Some(&v_str("1__2"))),
            Err(IntErr::Caught(_))
        ));
        assert!(matches!(py_int(Some(&v_str("_1"))), Err(IntErr::Caught(_))));
        assert!(matches!(py_int(Some(&v_str("1_"))), Err(IntErr::Caught(_))));
        assert!(matches!(py_int(Some(&v_str("x"))), Err(IntErr::Caught(_))));
        assert!(matches!(
            py_int(Some(&Value::Float(f64::NAN))),
            Err(IntErr::Caught(_))
        ));
        // OverflowError escapes the module loop's `except`.
        assert!(matches!(
            py_int(Some(&Value::Float(f64::INFINITY))),
            Err(IntErr::Uncaught(_))
        ));
    }

    #[test]
    fn floor_division_floors_toward_negative_infinity() {
        assert_eq!(py_floordiv(400, 40), Ok(10));
        assert_eq!(py_floordiv(401, 40), Ok(10));
        assert_eq!(py_floordiv(-401, 40), Ok(-11));
        assert!(py_floordiv(1, 0).is_err());
    }

    #[test]
    fn truthiness_matches_python() {
        assert!(!py_truthy(&v_str("")));
        assert!(!py_truthy(&Value::Integer(0)));
        assert!(!py_truthy(&Value::Boolean(false)));
        assert!(!py_truthy(&Value::Array(vec![])));
        assert!(py_truthy(&Value::Integer(1)));
        assert!(py_truthy(&Value::Array(vec![v_str("a")])));
    }

    #[test]
    fn set_membership_follows_python_equality_and_hashing() {
        let s = key_set(&["A", "B"]);
        assert!(set_contains(&s, Some(&v_str("A"))).unwrap());
        assert!(!set_contains(&s, Some(&v_str("a"))).unwrap());
        assert!(!set_contains(&s, None).unwrap());
        assert!(!set_contains(&s, Some(&Value::Integer(1))).unwrap());
        // 1 == 1.0 == True share a set slot in CPython.
        let nums: BTreeSet<String> = [hash_key(Some(&Value::Integer(1))).unwrap()]
            .into_iter()
            .collect();
        assert!(set_contains(&nums, Some(&Value::Float(1.0))).unwrap());
        assert!(set_contains(&nums, Some(&Value::Boolean(true))).unwrap());
        // Unhashable values raise, as `x in frozenset` does.
        assert!(set_contains(&s, Some(&Value::Array(vec![]))).is_err());
    }

    #[test]
    fn dict_reprs_match_cpython() {
        let mut letters = BTreeMap::new();
        letters.insert("A".to_string(), 198u64);
        letters.insert("B".to_string(), 228u64);
        assert_eq!(py_dict_str_keys(&letters), "{'A': 198, 'B': 228}");
        assert_eq!(py_dict_str_keys(&BTreeMap::new()), "{}");
        let mut mods = BTreeMap::new();
        mods.insert(1i128, 36u64);
        mods.insert(15i128, 39u64);
        assert_eq!(py_dict_int_keys(&mods), "{1: 36, 15: 39}");
    }

    #[test]
    fn list_and_value_reprs_match_cpython() {
        assert_eq!(py_list_of_str(&["a".into(), "b".into()]), "['a', 'b']");
        assert_eq!(py_repr_value(None), "None");
        assert_eq!(py_repr_value(Some(&v_str("E"))), "'E'");
        assert_eq!(py_repr_value(Some(&Value::Integer(7))), "7");
        assert_eq!(py_repr_value(Some(&Value::Boolean(false))), "False");
        assert_eq!(
            py_repr_value(Some(&Value::Array(vec![Value::Integer(1), v_str("a")]))),
            "[1, 'a']"
        );
    }

    // ── the topics regex ──────────────────────────────────────────────────

    #[test]
    fn finds_ids_in_match_order_with_duplicates_kept() {
        let text = "[[topic]]\nid = \"a\"\n\n[[topic]]\n  id=\"b\"\n[[topic]]\nid = \"a\"\n";
        assert_eq!(find_topic_ids(text), vec!["a", "b", "a"]);
    }

    #[test]
    fn id_pattern_is_line_anchored() {
        let text = "topic_id = \"x\"\nlabel = \"y\" id = \"z\"\nid = \"real\"\n";
        assert_eq!(find_topic_ids(text), vec!["real"]);
    }

    #[test]
    fn id_pattern_spans_newlines_because_backslash_s_does() {
        assert_eq!(find_topic_ids("id\n=\n\"spanning\"\n"), vec!["spanning"]);
    }

    #[test]
    fn empty_and_unterminated_values_do_not_match() {
        assert!(find_topic_ids("id = \"\"\n").is_empty());
        assert!(find_topic_ids("id = \"unterminated\n").is_empty());
    }

    // ── the gate, against fixtures ────────────────────────────────────────

    struct Fx {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    impl Fx {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().to_path_buf();
            let f = Fx { _dir: dir, root };
            std::fs::create_dir_all(f.root.join("bank/items")).unwrap();
            std::fs::create_dir_all(f.root.join("knowledge")).unwrap();
            f.write("knowledge/topics.toml", "[[topic]]\nid = \"t-one\"\n");
            f.write(
                "knowledge/bank_policy.toml",
                "exam_n_items = 1\npool_min_items = 2\n",
            );
            f
        }
        fn write(&self, rel: &str, body: &str) {
            let p = self.root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, body).unwrap();
        }
        fn eval(&self) -> Outcome {
            evaluate(&self.root)
        }
    }

    /// A schema-clean, APPROVED item. The explicit `status` is load-bearing:
    /// an absent status is `draft`, so a fixture that omitted it would be
    /// counted out of every floor (bd-8exw).
    fn good(id: &str) -> String {
        good_with_status(id, "approved")
    }

    fn good_with_status(id: &str, status: &str) -> String {
        format!(
            "[[items]]\nid = {id:?}\nmodule = 1\nstem = \"stem {id}\"\n\
             choices = [\"a\", \"b\", \"c\", \"d\"]\ncorrect = \"A\"\n\
             explanation = \"explanation long enough\"\ntopic_ids = [\"t-one\"]\n\
             bloom = \"apply\"\nsource_class = \"original\"\n\
             quantity_evidence = \"free_url\"\nstatus = {status:?}\n\n"
        )
    }

    #[test]
    fn a_missing_bank_directory_is_reported_before_anything_else() {
        let dir = tempfile::tempdir().unwrap();
        let out = evaluate(dir.path());
        assert_eq!(out.stdout, "FAIL: bank/items/ missing\n");
        assert_eq!(out.code, 1);
        assert!(out.stderr.is_empty());
    }

    #[test]
    fn a_missing_topics_registry_is_reported_next() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("bank/items")).unwrap();
        let out = evaluate(dir.path());
        assert_eq!(out.stdout, "FAIL: knowledge/topics.toml missing\n");
        assert_eq!(out.code, 1);
    }

    #[test]
    fn an_empty_bank_directory_is_never_a_pass() {
        let f = Fx::new();
        let out = f.eval();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert!(
            out.stdout.contains("  - zero items loaded\n"),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout.contains(
                "  - pool too small: 0 approved < pool_min_items 2 (0 scanned, 0 not approved; need ≥2× exam size 1)\n"
            ),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn a_clean_pool_passes_and_prints_the_distribution() {
        let f = Fx::new();
        f.write(
            "bank/items/pool.toml",
            &format!("{}{}", good("i-one"), good("i-two")),
        );
        let out = f.eval();
        assert_eq!(out.code, 0, "{}", out.stdout);
        assert_eq!(
            out.stdout,
            concat!(
                "PASS\n",
                "  items=2 scanned, 2 approved (floors count the approved pool only)\n",
                "  unique_ids=2 (file set)\n",
                "  pool_min=2 exam_n=1 multiplier≈2.0x (approved pool)\n",
                "  topics_registry=1\n",
                "  domain_floors=0 checked (approved pool)\n",
                "  correct_dist(approved)={'A': 2}\n",
                "  modules(approved)={1: 2}\n",
                "  modules(scanned)={1: 2}\n",
                "  source_class=original\n",
            )
        );
    }

    #[test]
    fn a_file_with_neither_id_nor_items_is_a_finding() {
        let f = Fx::new();
        f.write("bank/items/junk.toml", "note = \"nothing here\"\n");
        f.write(
            "bank/items/pool.toml",
            &format!("{}{}", good("i-one"), good("i-two")),
        );
        let out = f.eval();
        assert!(
            out.stdout.contains("  - junk.toml: no id or items[]\n"),
            "{}",
            out.stdout
        );
        assert_eq!(out.code, 1);
    }

    #[test]
    fn duplicate_ids_and_stems_are_both_reported() {
        let f = Fx::new();
        f.write(
            "bank/items/pool.toml",
            &format!("{}{}", good("dup"), good("dup")),
        );
        let out = f.eval();
        assert!(
            out.stdout.contains("  - duplicate ids: ['dup']\n"),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout
                .contains("  - duplicate stem (2 items ['dup', 'dup']): 'stem dup'\n"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn the_report_truncates_at_eighty_findings() {
        let f = Fx::new();
        let mut body = String::new();
        for i in 0..90 {
            body.push_str(&format!(
                "[[items]]\nid = \"bad-{i:03}\"\nmodule = 1\nstem = \"s{i}\"\n\
                 choices = [\"a\", \"b\", \"c\", \"d\"]\ncorrect = \"E\"\n\
                 explanation = \"explanation long enough\"\ntopic_ids = [\"t-one\"]\n\
                 bloom = \"apply\"\nsource_class = \"original\"\n\
                 quantity_evidence = \"free_url\"\nstatus = \"approved\"\n\n"
            ));
        }
        f.write("bank/items/pool.toml", &body);
        let out = f.eval();
        let shown = out.stdout.lines().filter(|l| l.starts_with("  - ")).count();
        assert_eq!(shown, MAX_REPORT);
        // 90 bad letters + "correct=? is .. of pool" is not tripped (no letter
        // counted at all), + "need at least 3 distinct correct letters".
        assert!(out.stdout.contains("  ... +11 more\n"), "{}", out.stdout);
    }

    #[test]
    fn a_non_string_stem_raises_the_way_the_oracle_does() {
        let f = Fx::new();
        f.write(
            "bank/items/pool.toml",
            "[[items]]\nid = \"x\"\nstem = 5\nmodule = 1\n",
        );
        let out = f.eval();
        assert_eq!(out.code, 1);
        assert!(out.stdout.is_empty(), "{}", out.stdout);
        assert!(
            out.stderr.contains("has no attribute 'strip'"),
            "{}",
            out.stderr
        );
    }

    /// bd-hw3. Both spellings of a zero exam size are the SAME finding, and
    /// neither reaches stdout behind a verdict. Before 2026-08-14 `= 0` was
    /// falsy and silently became 40, while `= "0"` printed three lines of a
    /// PASS report and then raised ZeroDivisionError.
    #[test]
    fn both_spellings_of_a_zero_exam_size_are_one_finding_and_no_verdict_precedes_them() {
        for spelling in ["0", "\"0\""] {
            let f = Fx::new();
            f.write(
                "knowledge/bank_policy.toml",
                &format!("exam_n_items = {spelling}\npool_min_items = 1\n"),
            );
            f.write("bank/items/pool.toml", &good("i-one"));
            let out = f.eval();
            assert_eq!(out.code, 1, "[{spelling}] {}", out.stdout);
            assert_eq!(
                out.stdout, "FAIL\n  - bank_policy.toml: exam_n_items must be > 0, got 0\n",
                "[{spelling}] stdout"
            );
            assert!(out.stderr.is_empty(), "[{spelling}] {}", out.stderr);
            assert!(
                !out.stdout.starts_with("PASS"),
                "[{spelling}] a verdict was written before the failure"
            );
        }
    }

    /// A negative floor used to DISABLE the pool check and report PASS: `-1` is
    /// truthy, so `int(... or default)` kept it, and `n < -1` is never true.
    #[test]
    fn a_negative_pool_floor_is_a_finding_not_a_disabled_check() {
        let f = Fx::new();
        f.write(
            "knowledge/bank_policy.toml",
            "exam_n_items = 1\npool_min_items = -1\n",
        );
        f.write("bank/items/pool.toml", &good("i-one"));
        let out = f.eval();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert_eq!(
            out.stdout,
            "FAIL\n  - bank_policy.toml: pool_min_items must be > 0, got -1\n"
        );
    }

    /// Non-numeric junk is the same finding, not an uncaught `ValueError`.
    #[test]
    fn a_non_numeric_policy_floor_is_a_finding_not_a_traceback() {
        let f = Fx::new();
        f.write(
            "knowledge/bank_policy.toml",
            "exam_n_items = 1\npool_min_items = \"lots\"\n",
        );
        f.write("bank/items/pool.toml", &good("i-one"));
        let out = f.eval();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert_eq!(
            out.stdout,
            "FAIL\n  - bank_policy.toml: pool_min_items must be an integer, got 'lots'\n"
        );
        assert!(out.stderr.is_empty(), "{}", out.stderr);
    }

    /// An absent key still takes the built-in default — this is the known-GOOD
    /// leg of the policy rebase, proving it does not refuse legitimate config.
    #[test]
    fn an_absent_policy_key_still_defaults() {
        let mut errors = Vec::new();
        let table: toml::Table = "other = 1\n".parse().unwrap();
        assert_eq!(
            policy_positive_int(&table, "exam_n_items", 40, &mut errors),
            Ok(PolicyInt::Default(40))
        );
        let given: toml::Table = "exam_n_items = 7\n".parse().unwrap();
        assert_eq!(
            policy_positive_int(&given, "exam_n_items", 40, &mut errors),
            Ok(PolicyInt::Given(7))
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn the_live_repo_tree_is_green() {
        let root = crate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let out = evaluate(&root);
        assert_eq!(out.code, 0, "{}{}", out.stdout, out.stderr);
        assert!(out.stdout.starts_with("PASS\n"), "{}", out.stdout);
        assert!(out.stdout.contains("  source_class=original\n"));
        // The two populations are both named and they DIFFER: 804 files, 779
        // drawable. Asserting only the file count is what let bd-8exw hide.
        assert!(
            out.stdout
                .contains("  items=804 scanned, 779 approved (floors count the approved pool only)\n"),
            "{}",
            out.stdout
        );
        // Module 14 carries 44 files but only 42 approved — the exact pair the
        // old single-map report collapsed into one number.
        assert!(out.stdout.contains("14: 42, 15: 39}"), "{}", out.stdout);
        assert!(out.stdout.contains("14: 44, 15: 39}"), "{}", out.stdout);
    }

    // ── bd-8exw: the floors measure the APPROVED pool ─────────────────────

    /// THE KNOWN-BAD, at unit scale. Retiring items IN PLACE — every file
    /// stays, only `status` moves — pushes the drawable pool under the floor.
    /// The file count is untouched, so the pre-fix gate stayed green on this
    /// fixture by construction; that is the proof the population changed.
    #[test]
    fn retiring_in_place_trips_the_pool_floor_without_deleting_a_file() {
        let f = Fx::new();
        f.write(
            "knowledge/bank_policy.toml",
            "exam_n_items = 1\npool_min_items = 3\n",
        );
        let mut body = String::new();
        for i in 0..4 {
            body.push_str(&good(&format!("i-{i}")));
        }
        f.write("bank/items/pool.toml", &body);
        let before = f.eval();
        assert_eq!(before.code, 0, "{}", before.stdout);
        assert!(
            before
                .stdout
                .contains("  items=4 scanned, 4 approved (floors count the approved pool only)\n"),
            "{}",
            before.stdout
        );

        // Same four FILES, two of them retired in place.
        let mut retired = String::new();
        for i in 0..2 {
            retired.push_str(&good(&format!("i-{i}")));
        }
        for i in 2..4 {
            retired.push_str(&good_with_status(&format!("i-{i}"), "retired"));
        }
        f.write("bank/items/pool.toml", &retired);
        let after = f.eval();
        assert_eq!(
            after.code, 1,
            "a drawable pool of 2 under a floor of 3 must be RED:\n{}",
            after.stdout
        );
        assert!(
            after.stdout.contains(
                "  - pool too small: 2 approved < pool_min_items 3 \
                 (4 scanned, 2 not approved; need ≥3× exam size 1)\n"
            ),
            "the finding must name BOTH populations:\n{}",
            after.stdout
        );
    }

    /// The same injection against a `[[domain_min]]` floor.
    #[test]
    fn retiring_in_place_trips_a_domain_floor_and_names_both_numbers() {
        let f = Fx::new();
        f.write(
            "knowledge/bank_policy.toml",
            "exam_n_items = 1\npool_min_items = 1\n\
             [[domain_min]]\nmodule = 1\nmin_items = 3\n",
        );
        let mut body = String::new();
        for i in 0..2 {
            body.push_str(&good(&format!("i-{i}")));
        }
        for i in 2..4 {
            body.push_str(&good_with_status(&format!("i-{i}"), "retired"));
        }
        f.write("bank/items/pool.toml", &body);
        let out = f.eval();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert!(
            out.stdout
                .contains("  - module 1: 2 approved items < domain_min 3 (4 scanned, 2 not approved)\n"),
            "{}",
            out.stdout
        );
    }

    /// ANTI-VACUOUS, the leg the empty-bank case cannot reach: a bank FULL of
    /// files and empty of drawable items. Zero approved is an ERROR that names
    /// the condition, distinct from `zero items loaded`.
    #[test]
    fn a_bank_of_only_retired_items_is_an_error_naming_the_empty_approved_pool() {
        let f = Fx::new();
        f.write(
            "knowledge/bank_policy.toml",
            "exam_n_items = 1\npool_min_items = 1\n",
        );
        let mut body = String::new();
        for i in 0..3 {
            body.push_str(&good_with_status(&format!("i-{i}"), "retired"));
        }
        f.write("bank/items/pool.toml", &body);
        let out = f.eval();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert!(
            out.stdout.contains(
                "  - zero approved items (3 scanned): the floors measure a pool no \
                 learner can be assessed from (vacuous scan is ERROR)\n"
            ),
            "{}",
            out.stdout
        );
        // And it is NOT confused with the empty-bank leg, which counts files.
        assert!(
            !out.stdout.contains("zero items loaded"),
            "three files loaded; the empty-bank leg must stay silent:\n{}",
            out.stdout
        );
    }

    /// An unmodelled status is a finding naming the item, never a silent drop
    /// into "not approved" — a bucket decided by guess is the same defect one
    /// level down.
    #[test]
    fn an_unmodelled_status_is_a_named_finding_not_a_silent_non_approval() {
        let f = Fx::new();
        f.write("bank/items/pool.toml", &good_with_status("i-odd", "published"));
        let out = f.eval();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert!(
            out.stdout.contains("  - i-odd: unknown status 'published'\n"),
            "{}",
            out.stdout
        );
    }

    /// Silence is not approval: an item with no `status` line is `draft`, which
    /// matches `cdcp_bank::ItemStatus`'s serde default and is what keeps this
    /// gate fail-closed against an item that lost its status line.
    #[test]
    fn an_absent_status_is_draft_and_counts_out_of_every_floor() {
        let f = Fx::new();
        f.write(
            "knowledge/bank_policy.toml",
            "exam_n_items = 1\npool_min_items = 2\n",
        );
        // Two files; only one carries a status line.
        let statusless = good("i-bare").replace("status = \"approved\"\n", "");
        assert!(!statusless.contains("status"), "the fixture must be bare");
        f.write(
            "bank/items/pool.toml",
            &format!("{}{}", good("i-ok"), statusless),
        );
        let out = f.eval();
        assert_eq!(out.code, 1, "{}", out.stdout);
        assert!(
            out.stdout.contains(
                "  - pool too small: 1 approved < pool_min_items 2 \
                 (2 scanned, 1 not approved; need ≥2× exam size 1)\n"
            ),
            "{}",
            out.stdout
        );
        // draft is a KNOWN status, so it is not also an unknown-status finding.
        assert!(
            !out.stdout.contains("unknown status"),
            "an absent status is draft, not junk:\n{}",
            out.stdout
        );
    }

    /// Letter diversity is a claim about the drawable pool. 40 files of which
    /// 39 are retired is not a pool worth screening, and a 100%-B APPROVED pool
    /// of 40 must still trip both rules.
    #[test]
    fn letter_diversity_is_gated_and_measured_on_the_approved_pool() {
        // 45 files, 40 approved-B and 5 retired-A: the retired items must not
        // dilute the fraction into passing.
        let f = Fx::new();
        let mut body = String::new();
        for i in 0..40 {
            body.push_str(&good(&format!("b-{i:03}")).replace("correct = \"A\"", "correct = \"B\""));
        }
        for i in 0..5 {
            body.push_str(&good_with_status(&format!("a-{i:03}"), "retired"));
        }
        f.write("bank/items/pool.toml", &body);
        let out = f.eval();
        assert!(
            out.stdout
                .contains("  - correct=B is 100% of approved pool (max 70% for diversity)\n"),
            "45 scanned but 40 approved and all B — 40/45 is 89%, 40/40 is 100%:\n{}",
            out.stdout
        );
        assert!(
            out.stdout
                .contains("  - need at least 3 distinct correct letters in the approved pool\n"),
            "{}",
            out.stdout
        );

        // And the gate: 39 approved among 45 files skips the rules entirely,
        // where a file-set gate of 45 would have applied them.
        let g = Fx::new();
        let mut body = String::new();
        for i in 0..39 {
            body.push_str(&good(&format!("b-{i:03}")).replace("correct = \"A\"", "correct = \"B\""));
        }
        for i in 0..6 {
            body.push_str(&good_with_status(&format!("a-{i:03}"), "retired"));
        }
        g.write("bank/items/pool.toml", &body);
        let below = g.eval();
        assert!(
            !below.stdout.contains("diversity"),
            "39 approved is under the threshold however many files there are:\n{}",
            below.stdout
        );
    }

    /// MANIFEST drift stays on the FILE SET, deliberately: a retirement that
    /// never reached the manifest is exactly what this catches, and counting it
    /// on the approved pool would hide it.
    #[test]
    fn manifest_drift_is_measured_against_the_file_set_on_purpose() {
        let f = Fx::new();
        let mut body = good("i-live");
        body.push_str(&good_with_status("i-dead", "retired"));
        f.write("bank/items/pool.toml", &body);
        f.write("bank/MANIFEST.toml", "item_count = 2\n");
        let ok = f.eval();
        // Assert the MANIFEST question, not the whole verdict. This fixture is
        // deliberately two items, so the pool floor (pool_min_items 2, one of
        // them retired) fires — legitimately, and for an unrelated reason. An
        // exit-0 assertion here would be asserting that BOTH the manifest is in
        // sync AND the pool is big enough, and would go red the moment any
        // other floor was added. What this test forbids is a manifest reading
        // that tracks the APPROVED pool.
        assert!(
            !ok.stdout.contains("MANIFEST item_count"),
            "2 files and a manifest of 2 is in sync even though 1 is retired, \
             so no MANIFEST finding may appear:\n{}",
            ok.stdout
        );
        // The approved count is 1; a manifest that tracked the approved pool
        // would now be RED, which is the reading this test forbids.
        f.write("bank/MANIFEST.toml", "item_count = 1\n");
        let drift = f.eval();
        assert_eq!(drift.code, 1, "{}", drift.stdout);
        assert!(
            drift.stdout.contains("  - MANIFEST item_count 1 != loaded 2\n"),
            "{}",
            drift.stdout
        );
    }

    #[test]
    fn the_gate_takes_no_arguments() {
        let ctx = GateCtx::new(PathBuf::from("/tmp"), vec!["--bank".into()]);
        let err = run(&ctx).unwrap_err();
        assert_eq!(err.code(), crate::exit::USAGE);
    }
}
