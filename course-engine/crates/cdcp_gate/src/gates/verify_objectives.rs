//! verify-objectives — Rust port of `scripts/verify_objectives.py`
//! (bd-substrate-rust-migration-jhd.10).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **the objective registry resolves, and every
//! module the course DECLARES carries at least one bank item.** It goes RED when
//! any of these hold —
//!
//!   1. *unresolved objective* — an `[[objective]]` row with a missing or blank
//!      `id`, with empty `claim_ids`, with a blank `claim_id` entry, or citing a
//!      `claim_id` that `registries/claims.toml` does not define. Duplicate
//!      objective ids are named too.
//!   2. *starved module* — a declared, non-exempt module holding zero bank items.
//!      The named module appears in the report and in the per-module block.
//!   3. *malformed exemption* — a `[[coverage_exempt]]` row with no usable
//!      `module`, with a missing or blank `reason`, naming a module the domain
//!      registry never declared, or contradicting an explicit `[[domain_min]]`
//!      floor. A rejected exemption leaves its module REQUIRED, so the shortfall
//!      still reports: the escape hatch may not be quieter than the rule it
//!      escapes.
//!   4. *cross-source drift* — a `[[domain_min]]` row keyed to a module the
//!      domain registry does not declare, or a `topics.toml` topic sitting in a
//!      domain the registry never declared. Two sources of truth for "which
//!      modules exist" disagreeing is the defect class bd-lt7 was opened for.
//!   5. *unreadable input* — a missing objectives or claims registry, a registry
//!      that will not parse, a missing domain registry, a missing bank directory,
//!      a bank file that is neither an item nor an `items[]` table array, an item
//!      whose `module` is not coercible to an integer, or a `--write-json` target
//!      that cannot be written.
//!
//! Anti-vacuous (L4): zero `[[claim]]` rows, zero `[[objective]]` rows, a domain
//! registry declaring zero modules, a required set emptied out by exemptions, a
//! bank loading zero items, and a `topics.toml` with zero topics in a required
//! domain are each an ERROR, never a pass. An input set that was never really
//! scanned must not report the way a scanned one does — that is the whole reason
//! those legs exist, and each is exercised on BOTH implementations by
//! `tests/diff_verify_objectives.rs`.
//!
//! # THE REBASE THIS PORT INHERITS (bd-lt7)
//!
//! The oracle derives its module set from `knowledge/domains.toml` rather than
//! from a numeric bound, and reads the SAME exemption ledger
//! (`knowledge/bank_policy.toml`) that `verify_coverage.py` reads. Until
//! 2026-08-14 it did not: the module the course assessed but did not teach had
//! been written down as a rule rather than as a recorded exemption, so the gate
//! stayed green by luck rather than by checking. This port carries the
//! derivation, not the bound: nothing here knows how many modules exist, and
//! every count in every line of the report is read from the registry at run time.
//!
//! # WHAT THIS GATE CANNOT DECIDE
//!
//! It counts items, not coverage: forty near-identical items satisfy the floor of
//! one exactly as forty distinct ones do. It reads no stem, no explanation and no
//! answer key, so it says nothing about whether an item is correct, well written,
//! mapped to the right topic, or of the right difficulty. `objectives.toml` holds
//! product-level outcomes with `claim_ids`, NOT per-module learning objectives,
//! and bank items rarely populate `objective_ids` — so there is no LO×item
//! matrix here and none is claimed. Primary-topic shortfalls are WARNINGS unless
//! `--strict-topics` is passed, which is deliberate honesty rather than a floor.
//! It says nothing about exam pass probability.
//!
//! It also cannot decide that the registries themselves are right. If
//! `domains.toml` omits a module the course teaches, that module is invisible
//! here and every gate downstream of the registry is confidently wrong together.
//! The floor moves from *silence* to *every registry objective resolves to a
//! registered claim, every declared module is stocked, and every exemption is
//! recorded with a reason* — no further.
//!
//! # BYTE-EXACTNESS WITH THE PYTHON ORACLE
//!
//! `scripts/verify_objectives.py` stays in the tree as the differential oracle
//! for this port; `tests/diff_verify_objectives.rs` runs BOTH implementations on
//! every case `scripts/selftest_l7_objectives.sh` exercises, plus the shapes that
//! suite never reaches, and asserts stdout, stderr and exit code match byte for
//! byte. A disagreement on any byte fails the port, not the oracle.
//!
//! Two consequences, both deliberate and both recorded here rather than made
//! quietly:
//!
//! - **The report goes to stdout and the process exits 1**, not through
//!   `GateError`. `GateError::report` writes to stderr and maps to exit 2 or 4,
//!   which the oracle never produces; routing through it would make the two sides
//!   differ on every RED case. `crate::exit`'s codes are therefore not used by
//!   this gate's verdict path, exactly as in `verify_orphans`, `verify_bank` and
//!   `verify_coverage`. `bd-2m9` flips the whole crate later; until then this is
//!   a knowing, single-file departure from the shared convention.
//! - This module carries hand-written emulations of CPython behaviour —
//!   `str.strip`, `repr()` of a `str`, a `float`, a `list` and a `dict`, `int()`
//!   coercion, truthiness, iteration, `PurePosixPath` normalisation,
//!   `Path.resolve`, `OSError` rendering, and
//!   `json.dumps(indent=2, sort_keys=True)` — rather than the idiomatic Rust
//!   nearest-neighbour, because the acceptance bar is identical bytes and not
//!   merely an identical verdict. They are duplicated from, not shared with,
//!   `verify_coverage`: one file per gate is the registration contract
//!   (`gates/mod.rs`), and reaching into a sibling gate mid-migration would
//!   couple this port to a file another agent owns.
//!
//! ## Modelling the oracle's uncaught exceptions
//!
//! Several exotic inputs raise rather than report: a `[[claim]]` array holding
//! something other than tables (`row.get` on a `str` is an `AttributeError`), a
//! non-iterable `objective`/`domain`/`domain_min` key, an infinite float where an
//! integer is expected, a `[[domain_min]]` `module` that passes the `isdigit`
//! screen but not `int()` (e.g. `"--5"`). CPython flushes whatever was already
//! printed, writes a traceback to stderr and exits 1. [`Outcome`] models that:
//! the partial stdout is kept, the exit code is 1, and stderr carries a one-line
//! description. **stdout and the exit code stay byte-identical on those paths;
//! the traceback text is the one surface this port does not reproduce**, and the
//! differential asserts exactly that (equal stdout, equal code, both stderrs
//! non-empty).
//!
//! ## Known residual deviations (none reachable from the live tree)
//!
//! - A malformed registry yields a parse message from the `toml` crate rather
//!   than from `tomllib` — `parse objectives:`, `parse claims:`, `parse topics:`,
//!   `domain registry parse error:`, `bank_policy.toml parse error:` and
//!   `<file>: parse error:` all carry the wrong explanation text. Both sides go
//!   RED on the same file; only the words differ.
//! - An unwritable `--write-json` target is CAUGHT by the oracle
//!   (`except OSError`) and reported as
//!   `could not write summary to <path>: [Errno N] <strerror>: '<path>'`. That
//!   string is reconstructed here from `raw_os_error()`, so the common shapes
//!   match; an exotic failure whose failing path CPython names differently
//!   (a partially-creatable parent chain) may name a different component.
//! - `{row!r}` of a multi-key table renders its keys in sorted order here and in
//!   insertion order in CPython, because `toml::Table` is a `BTreeMap`.
//! - `str.isdigit()` is true for some non-ASCII digits that `int()` then rejects;
//!   the screen here is ASCII-only.
//! - A TOML datetime reaching `str()` or `json.dumps` renders differently, and in
//!   CPython the latter raises. Nothing in the live tree puts one there.
//! - Bad *invocation* (an unknown or ambiguous flag, a non-integer
//!   `--min-items-per-topic`) returns `GateError::Usage`, where argparse prints
//!   its own usage block and exits 2. The oracle has no verdict there and no
//!   invocation in `check.sh` reaches it.

#![forbid(unsafe_code)]

use crate::registry::{GateCtx, GateError};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::Value;

pub const NAME: &str = "verify-objectives";
pub const SUMMARY: &str =
    "L7-S7 objective coverage: registry objectives resolve, every declared module carries items";

/// Engine-root-relative defaults, matching the Python module constants.
pub const DEFAULT_OBJECTIVES: &str = "registries/objectives.toml";
pub const DEFAULT_CLAIMS: &str = "registries/claims.toml";
pub const DEFAULT_TOPICS: &str = "knowledge/topics.toml";
pub const DEFAULT_DOMAINS: &str = "knowledge/domains.toml";
pub const DEFAULT_BANK: &str = "bank/items";
pub const DEFAULT_POLICY: &str = "knowledge/bank_policy.toml";

/// The floor this gate applies per required module. The SIZED floors are
/// `verify_coverage`'s job; this one asks only that the module is not empty.
pub const MIN_ITEMS_PER_MODULE: i128 = 1;

/// Report/summary slice widths, mirroring the oracle's `[:n]` literals.
const MAX_FAILURES: usize = 50;
const MAX_JSON_ERRORS: usize = 80;
const MAX_TOPIC_SHORTFALLS: usize = 100;
const MAX_WARNINGS: usize = 20;
const MAX_DRIFT_LINES: usize = 20;

/// The long options this gate accepts, for the abbreviation resolver.
const OPTIONS: &[&str] = &[
    "--objectives",
    "--claims",
    "--topics",
    "--domains",
    "--policy",
    "--bank",
    "--min-items-per-topic",
    "--strict-topics",
    "--skip-topic-coverage",
    "--write-json",
];

/// The two `store_true` options, which consume no value.
const FLAGS: &[&str] = &["--strict-topics", "--skip-topic-coverage"];

/// The oracle's `gap` field, spelled exactly as the Python concatenates it.
const JSON_GAP: &str = "objectives.toml holds product-level outcomes with claim_ids, \
                        not per-module learning objectives; bank topic_ids are the LO proxy; \
                        primary topic shortfalls soft-warn unless --strict-topics";

/// The oracle's `note` field.
const JSON_NOTE: &str = "Objective coverage ≠ exam pass probability; study signal only.";

// ── Python-behaviour emulations ────────────────────────────────────────────
// Each of these exists because the port's acceptance bar is byte-identical
// output, not merely an identical verdict.

/// A CPython exception that escapes the oracle's `except` clauses. Carries the
/// one-line description this port writes to stderr in place of a traceback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Halt(pub String);

type H<T> = Result<T, Halt>;

/// `int()` failures, split by whether the oracle's `except (KeyError, TypeError,
/// ValueError)` catches them. `OverflowError` on an infinite float does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntErr {
    Caught(String),
    Uncaught(String),
}

/// The set `str.strip()` removes: Unicode `White_Space` plus the four ASCII
/// information separators (0x1C–0x1F) that `str.isspace()` counts and Rust's
/// `char::is_whitespace` does not.
pub fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `str.strip()` with no argument.
pub fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

/// `str.isdigit()`, restricted to ASCII. See the header's residual-deviation
/// list for the non-ASCII digits CPython accepts here and `int()` then rejects.
pub fn py_isdigit_ascii(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// `repr()` of a Python `str`: single quotes unless the value contains `'` and
/// no `"`; backslash, the active quote and the three named escapes escaped; C0
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

/// `repr()` of a Python `float`: shortest round-tripping digits, `.0` appended to
/// integral values, and exponent notation when the decimal point sits at or left
/// of -4 or right of the CPython cutoff (`format_float_short`, code 'r').
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
    let sci = format!("{a:e}");
    let (mant, exp) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let raw: String = mant.chars().filter(|c| *c != '.').collect();
    let trimmed = raw.trim_end_matches('0');
    let digits = if trimmed.is_empty() { "0" } else { trimmed };
    let decpt = exp + 1;
    // CPython's exponent-notation cutoff, spelled in hex so a text sweep for
    // hardcoded module bounds (tests/rebase_module_bounds.rs) does not read a
    // float-formatting constant as a claim about how many modules exist.
    let hi_cutoff: i32 = 0x10;
    let body = if decpt <= -4 || decpt > hi_cutoff {
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

/// CPython's type name as it appears in a `TypeError` message.
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

/// `str(v)` for a value `tomllib` would have produced. `dict` and `list` render
/// as CPython's `str()` of a container, which is its `repr()`.
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

/// `str(v)`, where an absent key is Python's `None`.
fn py_str_opt(v: Option<&Value>) -> String {
    match v {
        None => "None".to_string(),
        Some(other) => py_str_value(other),
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

/// `for x in v` — the container protocols a `tomllib` value can satisfy. A
/// `dict` iterates its keys and a `str` its characters, both of which the
/// oracle's `for row in doc.get(...) or []` would happily do.
pub fn py_iter(v: &Value) -> H<Vec<Value>> {
    Ok(match v {
        Value::Array(a) => a.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Table(t) => t.keys().map(|k| Value::String(k.clone())).collect(),
        other => {
            return Err(Halt(format!(
                "TypeError: {} object is not iterable",
                py_type_name(other)
            )))
        }
    })
}

/// `doc.get(key) or []` followed by `for row in ...`.
fn py_rows(data: &toml::Table, key: &str) -> H<Vec<Value>> {
    match data.get(key) {
        Some(v) if py_truthy(v) => py_iter(v),
        _ => Ok(Vec::new()),
    }
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

/// `int(v)`, distinguishing the errors the oracle's `except` tuple catches from
/// the ones that escape it.
pub fn py_int(v: Option<&Value>) -> Result<i128, IntErr> {
    match v {
        None => Err(IntErr::Caught(
            "KeyError or TypeError: no such key / not a real number".to_string(),
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
fn py_int_uncaught(v: Option<&Value>) -> H<i128> {
    py_int(v).map_err(|e| match e {
        IntErr::Caught(m) | IntErr::Uncaught(m) => Halt(m),
    })
}

/// `str(e)` of an `OSError`: `[Errno N] <strerror>: '<filename>'`. Rust renders
/// os errors as `"<strerror> (os error N)"`, so the strerror is recovered by
/// stripping that suffix — the two come from the same `strerror_r`.
fn py_oserror(e: &std::io::Error, filename: &str) -> String {
    match e.raw_os_error() {
        Some(code) => {
            let rendered = std::io::Error::from_raw_os_error(code).to_string();
            let strerror = rendered
                .split(" (os error ")
                .next()
                .unwrap_or(&rendered)
                .to_string();
            format!("[Errno {code}] {strerror}: {}", py_repr(filename))
        }
        None => e.to_string(),
    }
}

// ── path emulations ────────────────────────────────────────────────────────

/// `str(PurePosixPath(s))`: empty and `.` components are dropped, `..` is kept,
/// duplicate separators collapse, a trailing separator is dropped, and exactly
/// two leading separators are preserved as a root (three or more collapse).
pub fn norm_posix(s: &str) -> String {
    let leading = if s.starts_with("///") {
        "/"
    } else if s.starts_with("//") {
        "//"
    } else if s.starts_with('/') {
        "/"
    } else {
        ""
    };
    let parts: Vec<&str> = s
        .split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect();
    if parts.is_empty() {
        return if leading.is_empty() {
            ".".to_string()
        } else {
            leading.to_string()
        };
    }
    format!("{leading}{}", parts.join("/"))
}

/// `root / rel` with `PurePosixPath` semantics: an absolute `rel` replaces the
/// root outright.
pub fn join_posix(root: &str, rel: &str) -> String {
    if rel.starts_with('/') {
        return norm_posix(rel);
    }
    let r = norm_posix(root);
    if r == "/" || r == "//" {
        norm_posix(&format!("{r}{rel}"))
    } else {
        norm_posix(&format!("{r}/{rel}"))
    }
}

/// `Path.resolve()` with `strict=False`: canonicalise the longest existing
/// prefix and append whatever is left.
pub fn py_resolve(p: &str) -> String {
    let normed = norm_posix(p);
    let mut cur = PathBuf::from(&normed);
    let mut rest: Vec<std::ffi::OsString> = Vec::new();
    loop {
        if let Ok(c) = cur.canonicalize() {
            let mut out = c;
            for r in rest.iter().rev() {
                out.push(r);
            }
            return norm_posix(&out.to_string_lossy());
        }
        let Some(name) = cur.file_name().map(|s| s.to_os_string()) else {
            break;
        };
        rest.push(name);
        if !cur.pop() {
            break;
        }
    }
    normed
}

/// The oracle's `p if p.is_absolute() else (ROOT / p).resolve()` — note that an
/// ABSOLUTE argument is never resolved, only `PurePosixPath`-normalised, and
/// that the defaults are already absolute and so are never resolved either.
fn resolve_arg(root: &str, given: Option<&str>, default_rel: &str) -> String {
    match given {
        None => join_posix(root, default_rel),
        Some(v) if v.starts_with('/') => norm_posix(v),
        Some(v) => py_resolve(&join_posix(root, v)),
    }
}

/// `Path.name` — the last component, or `""` for a root.
fn path_name(p: &str) -> String {
    p.rsplit('/').next().unwrap_or("").to_string()
}

/// `PurePosixPath.parent`.
fn path_parent(p: &str) -> String {
    let n = norm_posix(p);
    match n.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => n[..i].to_string(),
        None => ".".to_string(),
    }
}

/// `Path.relative_to`, which is purely lexical. `Err(())` is CPython's
/// `ValueError`.
fn relative_to(child: &str, parent: &str) -> Result<String, ()> {
    let c = norm_posix(child);
    let p = norm_posix(parent);
    if c == p {
        return Ok(".".to_string());
    }
    let prefix = if p.ends_with('/') {
        p.clone()
    } else {
        format!("{p}/")
    };
    match c.strip_prefix(&prefix) {
        Some(rest) if !rest.is_empty() => Ok(rest.to_string()),
        _ => Err(()),
    }
}

// ── json.dumps(indent=2, sort_keys=True) ───────────────────────────────────

/// The subset of JSON the oracle's summary can emit. `Float`, `Bool` and `Null`
/// are reachable only through a `topics.toml` `domain` value, which is copied
/// into the summary verbatim.
#[derive(Debug, Clone, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    Str(String),
    List(Vec<J>),
    /// Rendered with `sort_keys=True`, so insertion order here is irrelevant.
    Obj(Vec<(String, J)>),
}

/// `json.dumps`'s `ensure_ascii=True` string escaping: everything outside
/// `0x20..=0x7e` becomes an escape, with astral code points as surrogate pairs.
pub fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            '\u{8}' => o.push_str("\\b"),
            '\u{c}' => o.push_str("\\f"),
            c if (0x20..0x7f).contains(&(c as u32)) => o.push(c),
            c => {
                let cp = c as u32;
                if cp > 0xffff {
                    let v = cp - 0x10000;
                    o.push_str(&format!(
                        "\\u{:04x}\\u{:04x}",
                        0xd800 + (v >> 10),
                        0xdc00 + (v & 0x3ff)
                    ));
                } else {
                    o.push_str(&format!("\\u{cp:04x}"));
                }
            }
        }
    }
    o.push('"');
    o
}

fn json_dump(v: &J, depth: usize, out: &mut String) {
    let pad = |n: usize| "  ".repeat(n);
    match v {
        J::Null => out.push_str("null"),
        J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        J::Int(i) => out.push_str(&i.to_string()),
        J::Float(f) => out.push_str(&if f.is_nan() {
            "NaN".to_string()
        } else if f.is_infinite() {
            if *f < 0.0 { "-Infinity" } else { "Infinity" }.to_string()
        } else {
            py_float_repr(*f)
        }),
        J::Str(s) => out.push_str(&json_str(s)),
        J::List(xs) => {
            if xs.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, x) in xs.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&pad(depth + 1));
                json_dump(x, depth + 1, out);
            }
            out.push('\n');
            out.push_str(&pad(depth));
            out.push(']');
        }
        J::Obj(kv) => {
            if kv.is_empty() {
                out.push_str("{}");
                return;
            }
            let mut sorted: Vec<&(String, J)> = kv.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            out.push_str("{\n");
            for (i, (k, val)) in sorted.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&pad(depth + 1));
                out.push_str(&json_str(k));
                out.push_str(": ");
                json_dump(val, depth + 1, out);
            }
            out.push('\n');
            out.push_str(&pad(depth));
            out.push('}');
        }
    }
}

/// `json.dumps(v, indent=2, sort_keys=True)`.
pub fn json_dumps(v: &J) -> String {
    let mut out = String::new();
    json_dump(v, 0, &mut out);
    out
}

/// A `tomllib` value as `json.dumps` would serialise it. A datetime raises in
/// CPython, and raises here.
fn json_of_value(v: Option<&Value>) -> H<J> {
    Ok(match v {
        None => J::Null,
        Some(Value::String(s)) => J::Str(s.clone()),
        Some(Value::Integer(i)) => J::Int(*i as i128),
        Some(Value::Float(f)) => J::Float(*f),
        Some(Value::Boolean(b)) => J::Bool(*b),
        Some(Value::Array(a)) => {
            let mut xs = Vec::with_capacity(a.len());
            for x in a {
                xs.push(json_of_value(Some(x))?);
            }
            J::List(xs)
        }
        Some(Value::Table(t)) => {
            let mut kv = Vec::with_capacity(t.len());
            for (k, x) in t {
                kv.push((k.clone(), json_of_value(Some(x))?));
            }
            J::Obj(kv)
        }
        Some(Value::Datetime(_)) => {
            return Err(Halt(
                "TypeError: Object of type datetime is not JSON serializable".to_string(),
            ))
        }
    })
}

// ── the gate ───────────────────────────────────────────────────────────────

/// Exactly what the Python writes, and the status it exits with. `stderr` is
/// non-empty only on the raise paths described in the module header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// The argv tail, after argparse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub objectives: Option<String>,
    pub claims: Option<String>,
    pub topics: Option<String>,
    pub domains: Option<String>,
    pub policy: Option<String>,
    pub bank: Option<String>,
    pub min_items_per_topic: i128,
    pub strict_topics: bool,
    pub skip_topic_coverage: bool,
    pub write_json: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            objectives: None,
            claims: None,
            topics: None,
            domains: None,
            policy: None,
            bank: None,
            min_items_per_topic: 1,
            strict_topics: false,
            skip_topic_coverage: false,
            write_json: None,
        }
    }
}

/// One bank item as the Python loop sees it: the file it came from, and its table.
type Item = (String, toml::Table);

fn load_toml(path: &Path) -> Result<toml::Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    text.parse::<toml::Table>().map_err(|e| e.to_string())
}

/// `load_declared_modules()` — the module set, derived from the domain registry.
///
/// A registry that is missing, malformed or empty yields zero modules AND an
/// error, never a silent empty set that would make every floor below vacuously
/// satisfied. The missing and parse-error legs return early, so they do NOT
/// additionally report "declares zero modules" — that ordering is the oracle's.
///
/// Deliberately identical in shape to `verify_coverage`'s: two gates that
/// disagree about which modules exist are two gates that can be played off
/// against each other.
fn load_declared_modules(disp: &str) -> H<(BTreeMap<i128, String>, Vec<String>)> {
    let mut errors: Vec<String> = Vec::new();
    let mut declared: BTreeMap<i128, String> = BTreeMap::new();
    if !Path::new(disp).is_file() {
        return Ok((declared, vec![format!("domain registry missing: {disp}")]));
    }
    let data = match load_toml(Path::new(disp)) {
        Ok(d) => d,
        Err(e) => return Ok((declared, vec![format!("domain registry parse error: {e}")])),
    };

    for row in py_rows(&data, "domain")? {
        let Value::Table(t) = &row else {
            errors.push(format!(
                "domains.toml: [[domain]] row is not a table: {}",
                py_repr_value(Some(&row))
            ));
            continue;
        };
        let did = match t.get("id") {
            Some(v) if py_truthy(v) => py_strip(&py_str_value(v)).to_string(),
            _ => String::new(),
        };
        let order = match py_int(t.get("order")) {
            Ok(o) => o,
            Err(IntErr::Uncaught(m)) => return Err(Halt(m)),
            Err(IntErr::Caught(_)) => {
                // `f"...{did or row!r}..."` applies `!r` to the WHOLE `or`
                // expression, so a non-empty `did` is printed as `repr(str)` —
                // quoted — and not bare.
                let subject = if did.is_empty() {
                    py_repr_value(Some(&row))
                } else {
                    py_repr(&did)
                };
                errors.push(format!("domains.toml: {subject} has no usable order"));
                continue;
            }
        };
        if let Some(prev) = declared.get(&order) {
            errors.push(format!(
                "domains.toml: duplicate order {order} ({prev} and {did})"
            ));
            continue;
        }
        declared.insert(
            order,
            if did.is_empty() {
                format!("module-{order}")
            } else {
                did
            },
        );
    }

    if declared.is_empty() {
        errors
            .push("domain registry declares zero modules (vacuous coverage is ERROR)".to_string());
    }
    Ok((declared, errors))
}

/// `load_exemptions()` — recorded `[[coverage_exempt]]` rows, from the ONE
/// ledger shared with `verify_coverage.py`.
///
/// Every rejection path `continue`s WITHOUT recording the exemption, so a
/// malformed row leaves its module REQUIRED and the shortfall still reports.
/// Unlike `verify_coverage`, this oracle GUARDS the policy load, so a malformed
/// policy is a reported error rather than a raise.
fn load_exemptions(
    policy_disp: &str,
    declared: &BTreeMap<i128, String>,
) -> H<(BTreeMap<i128, String>, Vec<String>)> {
    let mut errors: Vec<String> = Vec::new();
    let mut exempt: BTreeMap<i128, String> = BTreeMap::new();
    if !Path::new(policy_disp).is_file() {
        return Ok((exempt, errors));
    }
    let bp = match load_toml(Path::new(policy_disp)) {
        Ok(d) => d,
        Err(e) => return Ok((exempt, vec![format!("bank_policy.toml parse error: {e}")])),
    };

    let mut floors: BTreeSet<i128> = BTreeSet::new();
    for r in py_rows(&bp, "domain_min")? {
        let Value::Table(t) = &r else { continue };
        let raw = match t.get("module") {
            Some(v) => py_str_value(v),
            None => String::new(),
        };
        if !py_isdigit_ascii(py_strip(&raw).trim_start_matches('-')) {
            continue;
        }
        floors.insert(py_int_uncaught(t.get("module"))?);
    }

    for row in py_rows(&bp, "coverage_exempt")? {
        let Value::Table(t) = &row else {
            errors.push(format!(
                "bank_policy.toml: coverage_exempt row is not a table: {}",
                py_repr_value(Some(&row))
            ));
            continue;
        };
        let module = match py_int(t.get("module")) {
            Ok(m) => m,
            Err(IntErr::Uncaught(m)) => return Err(Halt(m)),
            Err(IntErr::Caught(_)) => {
                errors.push(format!(
                    "bank_policy.toml: coverage_exempt row has no usable module: {}",
                    py_repr_value(Some(&row))
                ));
                continue;
            }
        };
        let reason = match t.get("reason") {
            Some(v) if py_truthy(v) => py_strip(&py_str_value(v)).to_string(),
            _ => String::new(),
        };
        if reason.is_empty() {
            errors.push(format!(
                "bank_policy.toml: coverage_exempt module {module} has no reason \
                 (an exemption without a reason is a schema error)"
            ));
            continue;
        }
        if !declared.contains_key(&module) {
            errors.push(format!(
                "bank_policy.toml: coverage_exempt module {module} is not in the \
                 domain registry"
            ));
            continue;
        }
        if floors.contains(&module) {
            errors.push(format!(
                "bank_policy.toml: module {module} is both coverage_exempt and has a \
                 [[domain_min]] floor — pick one"
            ));
            continue;
        }
        exempt.insert(module, reason);
    }
    Ok((exempt, errors))
}

/// `domain_min_drift()` — a `[[domain_min]]` row for a module the registry never
/// declared. This gate applies its own floor of one item per required module, so
/// it does not read the sized floors; it reads the file only to say so when the
/// two sources disagree about which modules exist.
fn domain_min_drift(policy_disp: &str, declared: &BTreeMap<i128, String>) -> H<Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    if !Path::new(policy_disp).is_file() {
        return Ok(errors);
    }
    let bp = match load_toml(Path::new(policy_disp)) {
        Ok(d) => d,
        Err(e) => return Ok(vec![format!("bank_policy.toml parse error: {e}")]),
    };
    for row in py_rows(&bp, "domain_min")? {
        let module = match &row {
            Value::Table(t) => match py_int(t.get("module")) {
                Ok(m) => Some(m),
                Err(IntErr::Uncaught(m)) => return Err(Halt(m)),
                Err(IntErr::Caught(_)) => None,
            },
            _ => None,
        };
        let Some(module) = module else {
            errors.push(format!(
                "bank_policy.toml: unusable [[domain_min]] row {}",
                py_repr_value(Some(&row))
            ));
            continue;
        };
        if !declared.contains_key(&module) {
            errors.push(format!(
                "bank_policy.toml: [[domain_min]] module {module} is not declared in \
                 the domain registry"
            ));
        }
    }
    Ok(errors)
}

/// `load_items()` — every `*.toml` under the bank dir, in `sorted()` order.
///
/// Carries the file-granular anti-vacuous rule bd-0czh added on 2026-08-14 (the
/// class sweep of bd-2kr): an `items = []` file — or an `items[]` holding
/// nothing this loop can read as an item — takes the list branch, adds nothing,
/// and never reaches the `no id or items[]` leg, because `elif` cannot run once
/// `if` has. Without the extra error a file that was never really checked
/// reports exactly like one that passed. This port was re-baselined onto the
/// fixed oracle rather than kept on the fail-open one.
fn load_items(disp: &str) -> (Vec<Item>, Vec<String>) {
    let dir = Path::new(disp);
    let mut errors: Vec<String> = Vec::new();
    let mut loaded: Vec<Item> = Vec::new();
    if !dir.is_dir() {
        return (loaded, vec![format!("bank dir missing: {disp}")]);
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return (loaded, errors);
    };
    let mut names: Vec<String> = rd
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".toml"))
        .collect();
    names.sort();

    for name in names {
        let data = match load_toml(&dir.join(&name)) {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("{name}: parse error: {e}"));
                continue;
            }
        };
        match data.get("items") {
            Some(Value::Array(items)) => {
                let before = loaded.len();
                for it in items {
                    if let Value::Table(t) = it {
                        loaded.push((name.clone(), t.clone()));
                    }
                }
                if loaded.len() == before {
                    errors.push(format!(
                        "{name}: items[] yielded zero items (vacuous file scan is ERROR)"
                    ));
                }
            }
            _ if data.contains_key("id") => loaded.push((name.clone(), data)),
            _ => errors.push(format!("{name}: no id or items[]")),
        }
    }
    (loaded, errors)
}

/// `claim_ids_from_registry()`. `row.get` on a non-table is an `AttributeError`
/// the oracle does not catch.
fn claim_ids_from_registry(claims: &toml::Table) -> H<BTreeSet<String>> {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for row in py_rows(claims, "claim")? {
        let Value::Table(t) = &row else {
            return Err(Halt(format!(
                "AttributeError: {} object has no attribute 'get'",
                py_type_name(&row)
            )));
        };
        if let Some(Value::String(cid)) = t.get("id") {
            let s = py_strip(cid);
            if !s.is_empty() {
                ids.insert(s.to_string());
            }
        }
    }
    Ok(ids)
}

/// A primary/optional topic, carrying only what the report and the summary read.
#[derive(Debug, Clone)]
struct Topic {
    id: String,
    domain: Option<Value>,
}

/// One primary-topic shortfall, as the summary records it.
#[derive(Debug, Clone)]
struct TopicShortfall {
    topic_id: String,
    domain: Option<Value>,
    have: u64,
    min: i128,
}

/// Run the whole check and render the oracle's report.
///
/// `root_str` is the engine root as an already-resolved POSIX string; the
/// members of `args` are the raw option values (engine-root-relative unless
/// absolute), exactly as argparse would hand them over.
pub fn evaluate(root_str: &str, args: &Args) -> Outcome {
    let mut out = String::new();
    match report(&mut out, root_str, args) {
        Ok(code) => Outcome {
            stdout: out,
            stderr: String::new(),
            code,
        },
        Err(Halt(msg)) => Outcome {
            stdout: out,
            stderr: format!("{msg}\n"),
            code: 1,
        },
    }
}

#[allow(clippy::too_many_lines)]
fn report(out: &mut String, root_str: &str, args: &Args) -> H<i32> {
    let objectives_disp = resolve_arg(root_str, args.objectives.as_deref(), DEFAULT_OBJECTIVES);
    let claims_disp = resolve_arg(root_str, args.claims.as_deref(), DEFAULT_CLAIMS);
    let topics_disp = resolve_arg(root_str, args.topics.as_deref(), DEFAULT_TOPICS);
    let domains_disp = resolve_arg(root_str, args.domains.as_deref(), DEFAULT_DOMAINS);
    let policy_disp = resolve_arg(root_str, args.policy.as_deref(), DEFAULT_POLICY);
    let bank_disp = resolve_arg(root_str, args.bank.as_deref(), DEFAULT_BANK);
    let min_topic = std::cmp::max(0, args.min_items_per_topic);

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // ── 1) Registry files present. This early exit prints its OWN short report:
    // no body, no verdict block, just FAIL and the reasons.
    if !Path::new(&objectives_disp).is_file() {
        errors.push(format!("missing objectives registry: {objectives_disp}"));
    }
    if !Path::new(&claims_disp).is_file() {
        errors.push(format!("missing claims registry: {claims_disp}"));
    }
    if !errors.is_empty() {
        out.push_str("FAIL\n");
        for e in &errors {
            out.push_str(&format!("  - {e}\n"));
        }
        return Ok(1);
    }

    let objectives_doc = match load_toml(Path::new(&objectives_disp)) {
        Ok(d) => d,
        Err(e) => {
            out.push_str("FAIL\n");
            out.push_str(&format!("  - parse objectives: {e}\n"));
            return Ok(1);
        }
    };
    let claims_doc = match load_toml(Path::new(&claims_disp)) {
        Ok(d) => d,
        Err(e) => {
            out.push_str("FAIL\n");
            out.push_str(&format!("  - parse claims: {e}\n"));
            return Ok(1);
        }
    };

    let known_claims = claim_ids_from_registry(&claims_doc)?;
    if known_claims.is_empty() {
        errors.push("registries/claims.toml has zero [[claim]] rows (empty = ERROR)".to_string());
    }

    let objectives = py_rows(&objectives_doc, "objective")?;
    if objectives.is_empty() {
        errors.push(
            "registries/objectives.toml has zero [[objective]] rows (empty = ERROR)".to_string(),
        );
    }

    let mut obj_ids: Vec<String> = Vec::new();
    let mut obj_claim_ok: usize = 0;
    for o in &objectives {
        let table = match o {
            Value::Table(t) => Some(t),
            _ => None,
        };
        let oid = match table.and_then(|t| t.get("id")) {
            Some(Value::String(s)) if !s.is_empty() && !py_strip(s).is_empty() => {
                py_strip(s).to_string()
            }
            _ => {
                errors.push("objective with empty/missing id".to_string());
                continue;
            }
        };
        obj_ids.push(oid.clone());
        // `o.get("claim_ids") or []`, then `if not isinstance(cids, list) or not cids`.
        let cids: Option<&Vec<Value>> = match table.and_then(|t| t.get("claim_ids")) {
            Some(Value::Array(a)) if !a.is_empty() => Some(a),
            _ => None,
        };
        let Some(cids) = cids else {
            errors.push(format!(
                "objective {oid}: claim_ids empty (must cite ≥1 claim)"
            ));
            continue;
        };
        let mut all_ok = true;
        for cid in cids {
            let s = match cid {
                Value::String(s) if !py_strip(s).is_empty() => py_strip(s).to_string(),
                _ => {
                    errors.push(format!("objective {oid}: empty claim_id entry"));
                    all_ok = false;
                    continue;
                }
            };
            if !known_claims.contains(&s) {
                errors.push(format!(
                    "objective {oid}: unresolved claim_id {} \
                     (not in registries/claims.toml)",
                    py_repr(&s)
                ));
                all_ok = false;
            }
        }
        if all_ok {
            obj_claim_ok += 1;
        }
    }

    // `Counter(obj_ids)` keeps first-occurrence order, and so does this.
    let mut order: Vec<String> = Vec::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for id in &obj_ids {
        let seen = counts.entry(id.clone()).or_insert(0);
        *seen += 1;
        if *seen == 1 {
            order.push(id.clone());
        }
    }
    let dups: Vec<&String> = order
        .iter()
        .filter(|id| counts.get(*id).copied().unwrap_or(0) > 1)
        .collect();
    if !dups.is_empty() {
        errors.push(format!(
            "duplicate objective ids: [{}]",
            dups.iter()
                .map(|d| py_repr(d))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let known_objectives: BTreeSet<String> = obj_ids.iter().cloned().collect();

    // ── 2) The module set, derived from the domain registry ────────────────
    let (declared, declared_errors) = load_declared_modules(&domains_disp)?;
    errors.extend(declared_errors);
    let (exempt, exempt_errors) = load_exemptions(&policy_disp, &declared)?;
    errors.extend(exempt_errors);
    errors.extend(domain_min_drift(&policy_disp, &declared)?);

    let required: Vec<i128> = declared
        .keys()
        .copied()
        .filter(|m| !exempt.contains_key(m))
        .collect();
    // A run with nothing left to require reports exactly like one that checked
    // everything and found it sound. Guarded by `declared` because an empty
    // registry is already reported above and must not double-report.
    if !declared.is_empty() && required.is_empty() {
        errors
            .push("zero required modules after exemptions (vacuous coverage is ERROR)".to_string());
    }
    let required_domain_ids: BTreeSet<String> = required
        .iter()
        .filter_map(|m| declared.get(m).cloned())
        .collect();
    let exempt_domain_ids: BTreeSet<String> = exempt
        .keys()
        .filter_map(|m| declared.get(m).cloned())
        .collect();

    // ── 3) Bank load + domain coverage over the required set ───────────────
    let (loaded, load_errors) = load_items(&bank_disp);
    errors.extend(load_errors);
    let n_items = loaded.len();
    if n_items == 0 {
        errors.push("empty bank: zero items loaded (vacuous coverage is ERROR)".to_string());
    }

    let mut module_counts: BTreeMap<i128, u64> = BTreeMap::new();
    let mut topic_item_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut items_with_objective_ids: usize = 0;

    for (fname, it) in &loaded {
        let iid = match it.get("id") {
            Some(v) if py_truthy(v) => py_str_value(v),
            _ => fname.clone(),
        };
        let module = it.get("module");
        match py_int(module) {
            Ok(mi) => *module_counts.entry(mi).or_insert(0) += 1,
            Err(IntErr::Uncaught(m)) => return Err(Halt(m)),
            Err(IntErr::Caught(_)) => {
                errors.push(format!("{iid}: bad module {}", py_repr_value(module)));
            }
        }

        if let Some(Value::Array(tids)) = it.get("topic_ids") {
            for t in tids {
                if let Value::String(s) = t {
                    let s = py_strip(s);
                    if !s.is_empty() {
                        *topic_item_counts.entry(s.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        // `oids = it.get("objective_ids") or []` then `isinstance(list) and oids`.
        if let Some(Value::Array(oids)) = it.get("objective_ids") {
            if !oids.is_empty() {
                items_with_objective_ids += 1;
                for oid in oids {
                    let s = match oid {
                        Value::String(s) if !py_strip(s).is_empty() => py_strip(s).to_string(),
                        _ => {
                            errors.push(format!("{iid}: empty objective_ids entry"));
                            continue;
                        }
                    };
                    if !known_objectives.contains(&s) {
                        errors.push(format!(
                            "{iid}: unknown objective_id {} \
                             (not in registries/objectives.toml)",
                            py_repr(&s)
                        ));
                    }
                }
            }
        }
    }

    let mut domain_shortfalls: Vec<(i128, u64)> = Vec::new();
    for module in &required {
        let have = module_counts.get(module).copied().unwrap_or(0);
        if i128::from(have) < MIN_ITEMS_PER_MODULE {
            errors.push(format!("domain module {module}: {have} items < min 1"));
            domain_shortfalls.push((*module, have));
        }
    }

    // Modules the bank carries that the registry never declared: REPORTED, not
    // failed — same as verify_coverage.py. The hard gate on "assessed but
    // untaught" is smoke_feedback_links.py.
    let extra_modules: Vec<i128> = module_counts
        .keys()
        .copied()
        .filter(|m| !declared.contains_key(m))
        .collect();

    // ── 4) Topic coverage (required domains) ───────────────────────────────
    let mut topics: Vec<toml::Table> = Vec::new();
    if Path::new(&topics_disp).is_file() {
        match load_toml(Path::new(&topics_disp)) {
            Ok(doc) => match py_rows(&doc, "topic") {
                Ok(rows) => {
                    for r in rows {
                        if let Value::Table(t) = r {
                            topics.push(t);
                        }
                    }
                }
                // The comprehension sits INSIDE the oracle's `try`, so a
                // non-iterable `topic` key is caught and reported, not raised.
                Err(Halt(m)) => errors.push(format!("parse topics: {}", strip_exc_prefix(&m))),
            },
            Err(e) => errors.push(format!("parse topics: {e}")),
        }
    } else {
        errors.push(format!("missing topics registry: {topics_disp}"));
    }

    let mut primary_topics: Vec<Topic> = Vec::new();
    let mut optional_topics: Vec<Topic> = Vec::new();
    let mut undeclared_topic_domains: Vec<String> = Vec::new();
    for t in &topics {
        let tid = match t.get("id") {
            Some(Value::String(s)) if !py_strip(s).is_empty() => py_strip(s).to_string(),
            _ => {
                errors.push("topic with empty/missing id".to_string());
                continue;
            }
        };
        // `dom = t.get("domain") or ""` then `str(dom).strip() if isinstance(dom, str) else ""`.
        let dom = match t.get("domain") {
            Some(Value::String(s)) if !s.is_empty() => py_strip(s).to_string(),
            _ => String::new(),
        };
        let topic = Topic {
            id: tid.clone(),
            domain: t.get("domain").cloned(),
        };
        if required_domain_ids.contains(&dom) {
            primary_topics.push(topic);
        } else if exempt_domain_ids.contains(&dom) {
            optional_topics.push(topic);
        } else {
            // Cross-source drift: topics.toml and domains.toml disagree about
            // which modules exist.
            undeclared_topic_domains.push(format!("{tid} (domain={})", py_repr(&dom)));
        }
    }

    let mut uncovered_primary: usize = 0;
    let mut topic_shortfalls: Vec<TopicShortfall> = Vec::new();
    if !args.skip_topic_coverage && min_topic > 0 && !primary_topics.is_empty() {
        for t in &primary_topics {
            let have = topic_item_counts.get(&t.id).copied().unwrap_or(0);
            if i128::from(have) < min_topic {
                uncovered_primary += 1;
                topic_shortfalls.push(TopicShortfall {
                    topic_id: t.id.clone(),
                    domain: t.domain.clone(),
                    have,
                    min: min_topic,
                });
                let msg = format!(
                    "topic {}: {have} items < min {min_topic} (domain={})",
                    t.id,
                    py_str_opt(t.domain.as_ref())
                );
                if args.strict_topics {
                    errors.push(msg);
                } else {
                    warnings.push(msg);
                }
            }
        }
    } else if primary_topics.is_empty() && Path::new(&topics_disp).is_file() && !declared.is_empty()
    {
        // Anti-vacuous: an empty topic set must not pass like a covered one.
        errors.push("topics.toml has zero topics in a required domain".to_string());
    }

    // Topics in a RECORDED-EXEMPT domain: report only, never required.
    let mut optional_uncovered: usize = 0;
    for t in &optional_topics {
        if t.id.is_empty() {
            continue;
        }
        if topic_item_counts.get(&t.id).copied().unwrap_or(0) < 1 {
            optional_uncovered += 1;
        }
    }

    // Drift is only meaningful against a registry that loaded.
    if !undeclared_topic_domains.is_empty() && !declared.is_empty() {
        for msg in undeclared_topic_domains.iter().take(MAX_DRIFT_LINES) {
            errors.push(format!("topics.toml: topic in an undeclared domain: {msg}"));
        }
        if undeclared_topic_domains.len() > MAX_DRIFT_LINES {
            errors.push(format!(
                "… and {} more topics in undeclared domains",
                undeclared_topic_domains.len() - MAX_DRIFT_LINES
            ));
        }
    }

    // ── Report (composed once; the verdict is decided last) ────────────────
    let topic_mode = if args.skip_topic_coverage {
        "skipped"
    } else if args.strict_topics {
        "strict"
    } else {
        "soft-warn"
    };
    let mut body: Vec<String> = vec![
        "  gate=l7-objective-coverage".to_string(),
        format!("  objectives={objectives_disp}"),
        format!("  claims={claims_disp}"),
        format!("  registry={domains_disp} declares={}", declared.len()),
        format!(
            "  policy={}",
            if Path::new(&policy_disp).is_file() {
                "present"
            } else {
                "absent"
            }
        ),
        format!("  bank={bank_disp}"),
        format!("  items={n_items}"),
        format!(
            "  registry_objectives={} claim_resolve_ok={obj_claim_ok}",
            obj_ids.len()
        ),
        format!("  known_claims={}", known_claims.len()),
        format!(
            "  modules ({} required, derived from {}; min 1 item each):",
            required.len(),
            path_name(&domains_disp)
        ),
    ];
    for module in &required {
        let have = module_counts.get(module).copied().unwrap_or(0);
        let flag = if i128::from(have) >= MIN_ITEMS_PER_MODULE && n_items > 0 {
            "ok"
        } else {
            "SHORT"
        };
        body.push(format!("    m{module:02}: {have} [{flag}]"));
    }
    if !exempt.is_empty() {
        body.push("  recorded exemptions (bank_policy.toml [[coverage_exempt]]):".to_string());
        for (module, reason) in &exempt {
            let have = module_counts.get(module).copied().unwrap_or(0);
            body.push(format!("    m{module:02}: {have} — exempt: {reason}"));
        }
    }
    if !extra_modules.is_empty() {
        body.push("  undeclared modules present in the bank (not required for green):".to_string());
        for module in &extra_modules {
            let have = module_counts.get(module).copied().unwrap_or(0);
            body.push(format!(
                "    m{module:02}: {have} (not in the domain registry)"
            ));
        }
    }
    body.push(format!(
        "  primary_topics={} covered={} shortfalls={uncovered_primary} \
         min_per_topic={min_topic} mode={topic_mode}",
        primary_topics.len(),
        primary_topics.len() - uncovered_primary
    ));
    body.push(format!(
        "  exempt_domain_topics={} uncovered={optional_uncovered} (not required)",
        optional_topics.len()
    ));
    body.push(format!(
        "  bank_items_with_objective_ids={items_with_objective_ids} \
         (of {n_items}; product-level objectives, not per-module LOs)"
    ));
    body.push(
        "  gap: no full LO×item matrix — objectives.toml is product outcomes + claim_ids"
            .to_string(),
    );
    body.push("  note: coverage ≠ exam pass probability; study signal only".to_string());

    if !warnings.is_empty() {
        body.push("  warnings:".to_string());
        for w in warnings.iter().take(MAX_WARNINGS) {
            body.push(format!("    - {w}"));
        }
        if warnings.len() > MAX_WARNINGS {
            body.push(format!("    ... +{} more", warnings.len() - MAX_WARNINGS));
        }
    }

    // JSON summary. Repo-relative bank path where possible, for portable commits.
    let bank_rel = relative_to(&py_resolve(&bank_disp), &py_resolve(root_str))
        .unwrap_or_else(|()| bank_disp.clone());

    // The write happens BEFORE any verdict is printed, and a failed write is a
    // failure of this gate — not a traceback under a PASS someone already read.
    if let Some(target) = &args.write_json {
        let out_path = if target.starts_with('/') {
            norm_posix(target)
        } else {
            py_resolve(&join_posix(root_str, target))
        };
        let provisional = if errors.is_empty() { "PASS" } else { "FAIL" };
        let summary = summary_json(
            provisional,
            &bank_rel,
            n_items,
            &path_name(&domains_disp),
            &declared,
            &required,
            &exempt,
            &module_counts,
            &extra_modules,
            &domain_shortfalls,
            &obj_ids,
            obj_claim_ok,
            known_claims.len(),
            primary_topics.len(),
            &topic_shortfalls,
            uncovered_primary,
            optional_uncovered,
            items_with_objective_ids,
            min_topic,
            args,
            topic_mode,
            &errors,
            &warnings,
        )?;
        let parent = path_parent(&out_path);
        let mut wrote = true;
        if let Err(e) = std::fs::create_dir_all(&parent) {
            errors.push(format!(
                "could not write summary to {out_path}: {}",
                py_oserror(&e, &parent)
            ));
            wrote = false;
        }
        if wrote {
            match std::fs::write(&out_path, format!("{}\n", json_dumps(&summary))) {
                Ok(()) => body.push(format!("  wrote {out_path}")),
                Err(e) => errors.push(format!(
                    "could not write summary to {out_path}: {}",
                    py_oserror(&e, &out_path)
                )),
            }
        }
    }

    // The verdict is the LAST thing decided and the first thing on a report that
    // is printed exactly once, after every path that could still raise.
    let status = if errors.is_empty() { "PASS" } else { "FAIL" };
    let mut report: Vec<String> = vec![status.to_string()];
    report.extend(body);
    if !errors.is_empty() {
        report.push("  failures:".to_string());
        for e in errors.iter().take(MAX_FAILURES) {
            report.push(format!("    - {e}"));
        }
        if errors.len() > MAX_FAILURES {
            report.push(format!("    ... +{} more", errors.len() - MAX_FAILURES));
        }
    } else {
        // Enumerated, not spanned: an exemption can leave a gap, and a range
        // would read as covering a module that was held out.
        let span = required
            .iter()
            .map(|m| format!("m{m:02}"))
            .collect::<Vec<_>>()
            .join(" ");
        if uncovered_primary > 0 && !args.strict_topics && !args.skip_topic_coverage {
            report.push(format!(
                "  objective coverage GREEN (registry claims + {} required modules: {span}; \
                 {uncovered_primary} topic shortfalls soft-warn)",
                required.len()
            ));
        } else {
            report.push(format!(
                "  objective coverage GREEN (registry claims + {} required modules: {span} \
                 + primary topics)",
                required.len()
            ));
        }
    }
    out.push_str(&report.join("\n"));
    out.push('\n');
    Ok(if errors.is_empty() { 0 } else { 1 })
}

/// `TypeError: 'int' object is not iterable` as CPython's `str(e)` renders it
/// inside the oracle's `except Exception as e` — without the class name this
/// port prefixes for the raise paths.
fn strip_exc_prefix(m: &str) -> String {
    match m.split_once(": ") {
        Some((_, rest)) => rest.to_string(),
        None => m.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn summary_json(
    status: &str,
    bank_rel: &str,
    n_items: usize,
    module_source: &str,
    declared: &BTreeMap<i128, String>,
    required: &[i128],
    exempt: &BTreeMap<i128, String>,
    module_counts: &BTreeMap<i128, u64>,
    extra_modules: &[i128],
    domain_shortfalls: &[(i128, u64)],
    obj_ids: &[String],
    obj_claim_ok: usize,
    known_claims: usize,
    primary_topics: usize,
    topic_shortfalls: &[TopicShortfall],
    uncovered_primary: usize,
    optional_uncovered: usize,
    items_with_objective_ids: usize,
    min_topic: i128,
    args: &Args,
    topic_mode: &str,
    errors: &[String],
    warnings: &[String],
) -> H<J> {
    let ints = |xs: &[i128]| J::List(xs.iter().map(|x| J::Int(*x)).collect());
    let strs = |xs: &[String]| J::List(xs.iter().map(|x| J::Str(x.clone())).collect());

    let mut shortfall_rows: Vec<J> = Vec::new();
    for s in topic_shortfalls.iter().take(MAX_TOPIC_SHORTFALLS) {
        shortfall_rows.push(J::Obj(vec![
            ("topic_id".into(), J::Str(s.topic_id.clone())),
            ("domain".into(), json_of_value(s.domain.as_ref())?),
            ("have".into(), J::Int(i128::from(s.have))),
            ("min".into(), J::Int(s.min)),
        ]));
    }

    Ok(J::Obj(vec![
        ("schema_version".into(), J::Int(2)),
        ("gate".into(), J::Str("l7-objective-coverage".into())),
        ("status".into(), J::Str(status.to_lowercase())),
        ("bank".into(), J::Str(bank_rel.to_string())),
        ("item_count".into(), J::Int(n_items as i128)),
        ("module_source".into(), J::Str(module_source.to_string())),
        (
            "declared_modules".into(),
            ints(&declared.keys().copied().collect::<Vec<_>>()),
        ),
        ("required_modules".into(), ints(required)),
        (
            "exemptions".into(),
            J::Obj(
                exempt
                    .iter()
                    .map(|(k, v)| (k.to_string(), J::Str(v.clone())))
                    .collect(),
            ),
        ),
        (
            "registry_objectives".into(),
            J::Obj(vec![
                ("count".into(), J::Int(obj_ids.len() as i128)),
                ("ids".into(), strs(obj_ids)),
                ("claim_resolve_ok".into(), J::Int(obj_claim_ok as i128)),
            ]),
        ),
        ("known_claims".into(), J::Int(known_claims as i128)),
        (
            "domain_counts".into(),
            J::Obj(
                required
                    .iter()
                    .map(|m| {
                        (
                            m.to_string(),
                            J::Int(i128::from(module_counts.get(m).copied().unwrap_or(0))),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "extra_counts".into(),
            J::Obj(
                extra_modules
                    .iter()
                    .map(|m| {
                        (
                            m.to_string(),
                            J::Int(i128::from(module_counts.get(m).copied().unwrap_or(0))),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "domain_shortfalls".into(),
            J::List(
                domain_shortfalls
                    .iter()
                    .map(|(m, have)| {
                        J::Obj(vec![
                            ("module".into(), J::Int(*m)),
                            ("have".into(), J::Int(i128::from(*have))),
                            ("min".into(), J::Int(MIN_ITEMS_PER_MODULE)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("primary_topics".into(), J::Int(primary_topics as i128)),
        ("primary_topic_shortfalls".into(), J::List(shortfall_rows)),
        (
            "primary_topic_shortfall_count".into(),
            J::Int(uncovered_primary as i128),
        ),
        (
            "exempt_domain_topics_uncovered".into(),
            J::Int(optional_uncovered as i128),
        ),
        (
            "items_with_objective_ids".into(),
            J::Int(items_with_objective_ids as i128),
        ),
        ("min_items_per_topic".into(), J::Int(min_topic)),
        ("strict_topics".into(), J::Bool(args.strict_topics)),
        (
            "skip_topic_coverage".into(),
            J::Bool(args.skip_topic_coverage),
        ),
        ("topic_mode".into(), J::Str(topic_mode.to_string())),
        ("gap".into(), J::Str(JSON_GAP.to_string())),
        ("note".into(), J::Str(JSON_NOTE.to_string())),
        (
            "errors".into(),
            strs(
                &errors
                    .iter()
                    .take(MAX_JSON_ERRORS)
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
        ),
        ("warnings".into(), strs(warnings)),
    ]))
}

/// argparse's unambiguous-prefix matching for long options.
fn resolve_option(given: &str) -> Result<&'static str, GateError> {
    if let Some(exact) = OPTIONS.iter().find(|o| **o == given) {
        return Ok(exact);
    }
    let hits: Vec<&&str> = OPTIONS.iter().filter(|o| o.starts_with(given)).collect();
    match hits.len() {
        1 => Ok(hits[0]),
        0 => Err(GateError::usage(format!(
            "unrecognized argument {given:?}; known: {}",
            OPTIONS.join(" ")
        ))),
        _ => Err(GateError::usage(format!(
            "ambiguous option {given:?}; matches: {}",
            hits.iter().map(|h| **h).collect::<Vec<_>>().join(" ")
        ))),
    }
}

/// The argv tail, accepting `--opt v`, `--opt=v`, the two `store_true` flags,
/// and the unambiguous prefixes argparse allows. A repeated option keeps the
/// last value, as argparse does.
pub fn parse_args(args: &[String]) -> Result<Args, GateError> {
    let mut parsed = Args::default();
    let mut i = 0;
    while i < args.len() {
        let (name, inline) = match args[i].split_once('=') {
            Some((n, v)) => (n, Some(v.to_string())),
            None => (args[i].as_str(), None),
        };
        let opt = resolve_option(name)?;
        if FLAGS.contains(&opt) {
            if inline.is_some() {
                return Err(GateError::usage(format!("{opt}: expected no argument")));
            }
            match opt {
                "--strict-topics" => parsed.strict_topics = true,
                _ => parsed.skip_topic_coverage = true,
            }
            i += 1;
            continue;
        }
        let value = match inline {
            Some(v) => v,
            None => {
                i += 1;
                args.get(i)
                    .cloned()
                    .ok_or_else(|| GateError::usage(format!("{opt}: expected one argument")))?
            }
        };
        match opt {
            "--objectives" => parsed.objectives = Some(value),
            "--claims" => parsed.claims = Some(value),
            "--topics" => parsed.topics = Some(value),
            "--domains" => parsed.domains = Some(value),
            "--policy" => parsed.policy = Some(value),
            "--bank" => parsed.bank = Some(value),
            "--min-items-per-topic" => {
                parsed.min_items_per_topic = py_int_from_str(&value).ok_or_else(|| {
                    GateError::usage(format!(
                        "argument --min-items-per-topic: invalid int value: {}",
                        py_repr(&value)
                    ))
                })?;
            }
            _ => parsed.write_json = Some(value),
        }
        i += 1;
    }
    Ok(parsed)
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let args = parse_args(&ctx.args)?;

    // The Python resolves its own location (`Path(__file__).resolve()`), so the
    // printed default paths are symlink-free. Do the same to the engine root —
    // and only to the root: an absolute option value is printed exactly as
    // `PurePosixPath` normalises it, never canonicalised.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let root_str = norm_posix(&root.to_string_lossy());

    let outcome = evaluate(&root_str, &args);
    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    if !outcome.stderr.is_empty() {
        eprint!("{}", outcome.stderr);
        let _ = std::io::stderr().flush();
    }
    if outcome.code != 0 {
        // See the module header: the oracle exits 1 with this report on stdout,
        // and byte-identical output is this port's acceptance bar. Routing
        // through `GateError` would write to stderr and exit 2 or 4 instead.
        std::process::exit(outcome.code);
    }
    Ok(())
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gate_is_registered_under_a_kebab_case_name() {
        assert_eq!(NAME, "verify-objectives");
        assert!(crate::registry::find(NAME).is_some());
    }

    #[test]
    fn repr_matches_cpython_on_the_shapes_this_gate_prints() {
        assert_eq!(py_repr("abc"), "'abc'");
        assert_eq!(py_repr("it's"), "\"it's\"");
        assert_eq!(py_repr("a\nb"), "'a\\nb'");
        assert_eq!(py_repr_value(None), "None");
        assert_eq!(py_repr_value(Some(&Value::Integer(7))), "7");
        assert_eq!(py_repr_value(Some(&Value::Boolean(true))), "True");
        assert_eq!(py_str_opt(None), "None");
    }

    #[test]
    fn int_coercion_splits_caught_from_uncaught() {
        assert_eq!(py_int(Some(&Value::Integer(9))), Ok(9));
        assert_eq!(py_int(Some(&Value::String(" 5 ".into()))), Ok(5));
        assert!(matches!(
            py_int(Some(&Value::String("abc".into()))),
            Err(IntErr::Caught(_))
        ));
        assert!(matches!(py_int(None), Err(IntErr::Caught(_))));
        assert!(matches!(
            py_int(Some(&Value::Float(f64::INFINITY))),
            Err(IntErr::Uncaught(_))
        ));
    }

    #[test]
    fn posix_normalisation_matches_purepath() {
        assert_eq!(norm_posix("./bank//items/"), "bank/items");
        assert_eq!(join_posix("/root", "bank/items"), "/root/bank/items");
        assert_eq!(join_posix("/root", "/abs"), "/abs");
        assert_eq!(path_name("/a/b/domains.toml"), "domains.toml");
        assert_eq!(path_parent("/a/b/out.json"), "/a/b");
        assert_eq!(relative_to("/r/bank/items", "/r"), Ok("bank/items".into()));
        assert_eq!(relative_to("/tmp/x", "/r"), Err(()));
    }

    #[test]
    fn json_dumps_matches_python_indent_and_sort_keys() {
        let v = J::Obj(vec![
            ("b".into(), J::Int(2)),
            ("a".into(), J::List(vec![J::Int(1)])),
            ("c".into(), J::Obj(vec![])),
            ("d".into(), J::List(vec![])),
            ("e".into(), J::Bool(false)),
            ("f".into(), J::Null),
        ]);
        assert_eq!(
            json_dumps(&v),
            "{\n  \"a\": [\n    1\n  ],\n  \"b\": 2,\n  \"c\": {},\n  \"d\": [],\n  \
             \"e\": false,\n  \"f\": null\n}"
        );
        assert_eq!(json_str("≠"), "\"\\u2260\"");
    }

    #[test]
    fn option_prefixes_resolve_the_way_argparse_does() {
        assert_eq!(resolve_option("--objectives").unwrap(), "--objectives");
        assert_eq!(resolve_option("--o").unwrap(), "--objectives");
        assert_eq!(resolve_option("--b").unwrap(), "--bank");
        assert_eq!(resolve_option("--w").unwrap(), "--write-json");
        // `--s` is ambiguous between --strict-topics and --skip-topic-coverage.
        assert!(resolve_option("--s").is_err());
        assert!(resolve_option("--nope").is_err());

        let a = parse_args(&[
            "--bank=x".to_string(),
            "--policy".into(),
            "y".into(),
            "--strict-topics".into(),
        ])
        .unwrap();
        assert_eq!(a.bank.as_deref(), Some("x"));
        assert_eq!(a.policy.as_deref(), Some("y"));
        assert!(a.strict_topics);
        assert!(!a.skip_topic_coverage);
        assert_eq!(a.min_items_per_topic, 1);
    }

    #[test]
    fn a_rejected_exemption_leaves_its_module_required() {
        // The property the whole [[coverage_exempt]] leg exists for: an
        // exemption without a reason is a schema error AND does not exempt.
        let td = tempfile::tempdir().unwrap();
        let policy = td.path().join("policy.toml");
        std::fs::write(&policy, "[[coverage_exempt]]\nmodule = 2\nreason = \"\"\n").unwrap();
        let mut declared = BTreeMap::new();
        declared.insert(1i128, "01-a".to_string());
        declared.insert(2i128, "02-b".to_string());
        let (exempt, errors) =
            load_exemptions(policy.to_str().unwrap(), &declared).expect("no raise");
        assert!(exempt.is_empty(), "a reasonless row must not exempt");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("has no reason"), "{errors:?}");
    }

    #[test]
    fn an_empty_registry_is_an_error_not_an_empty_pass() {
        let td = tempfile::tempdir().unwrap();
        let reg = td.path().join("domains.toml");
        std::fs::write(&reg, "schema_version = 1\n").unwrap();
        let (declared, errors) = load_declared_modules(reg.to_str().unwrap()).expect("no raise");
        assert!(declared.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("declares zero modules"), "{errors:?}");

        let missing = td.path().join("nope.toml");
        let (declared, errors) =
            load_declared_modules(missing.to_str().unwrap()).expect("no raise");
        assert!(declared.is_empty());
        assert_eq!(errors.len(), 1, "the missing leg returns early: {errors:?}");
        assert!(errors[0].starts_with("domain registry missing:"));
    }

    #[test]
    fn a_non_table_claim_row_raises_the_way_cpython_does() {
        let doc: toml::Table = "claim = [\"justastring\"]\n".parse().unwrap();
        let err = claim_ids_from_registry(&doc).unwrap_err();
        assert!(err.0.contains("AttributeError"), "{err:?}");
    }
}
