//! verify-coverage — Rust port of `scripts/verify_coverage.py`
//! (bd-substrate-rust-migration-jhd.6).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **every module the course DECLARES carries at
//! least its floor of APPROVED bank items.** It goes RED when any of these hold —
//!
//!   1. *starved module* — a declared, non-exempt module holding fewer approved
//!      items than its floor (`[[domain_min]] min_items` when the policy FILE is
//!      present, else the OQ-05 default of one). The named module and both
//!      numbers appear in the report.
//!   1b. *missing policy* — `bank_policy.toml` is not a file (bd-j98g). The
//!      sized floors live there; treating absence as N=1 would lower them.
//!      A present file with empty `[[domain_min]]` is the honest OQ-05 default
//!      and is a different path. This is the opposite of `verify_objectives`,
//!      where absence removes exemptions and makes the gate stricter.
//!   2. *malformed exemption* — a `[[coverage_exempt]]` row with no usable
//!      `module`, with a missing or blank `reason`, naming a module the domain
//!      registry never declared, or contradicting an explicit floor. The escape
//!      hatch may not be quieter than the rule it escapes, and a rejected
//!      exemption leaves its module REQUIRED, so the shortfall still reports.
//!   3. *cross-source drift* — a `[[domain_min]]` row keyed to a module the
//!      domain registry does not declare. Two sources of truth for "which
//!      modules exist" disagreeing is the defect class bd-lt7 was opened for.
//!   4. *unreadable registry* — a missing domain registry, or one whose
//!      `[[domain]]` rows carry no usable `order`, or two rows claiming one
//!      order.
//!   5. *unreadable bank* — a missing bank directory, a file that is neither an
//!      item nor an `items[]` table array, an item whose `module` is not
//!      coercible to an integer, or an item whose `status` is outside the C1
//!      lifecycle (`approved`/`draft`/`retired`).
//!   6. *unwritable summary* — a `--write-json` target whose directory cannot be
//!      created or whose file cannot be written. See VERDICT SHAPE below.
//!
//! Anti-vacuous (L4): a domain registry declaring zero modules, a bank loading
//! zero items, a bank whose APPROVED pool is empty, and a required set emptied
//! out by exemptions are each an ERROR, never a pass. An input set that was
//! never really scanned must not report the way a scanned one does — that is the
//! whole reason those legs exist, and each is exercised on both implementations
//! by `tests/diff_verify_coverage.rs`.
//!
//! # WHICH POOL THE FLOOR MEASURES (bd-coverage-counts-retired-items-49jh)
//!
//! The floor is measured against `status == "approved"` and never against the
//! file set. C1 restricts assembly to approved items
//! (`cdcp_assemble::sample_item_ids`), so a floor counted over every file is a
//! floor over a population no learner is ever assessed from — and it fails OPEN,
//! because the file count can only ever be >= the number that matters.
//!
//! Until 2026-08-14 both implementations counted files. It was invisible because
//! the bank held exactly ONE non-approved item; bd-tetz then retired 24
//! duplicates and the gap became 25 across ten modules, `m14: 44 (min 24) [ok]`
//! against 42 drawable. Nothing breached, so this was a defect and not an
//! incident — but the claim had been unearned since the first retirement.
//! Every line of the report that carries a count now names BOTH numbers.
//!
//! An item whose `status` is not in the C1 lifecycle is an ERROR naming the
//! item, never a silent drop into "not approved": `cdcp_bank` rejects an unknown
//! status at load for exactly the same reason. An ABSENT status is `draft` by
//! C1's default — silence never publishes — and is not an error here.
//!
//! # VERDICT SHAPE (bd-verify-coverage-verdict-before-write-rk9n)
//!
//! **No success token reaches stdout on a path that can still return non-zero.**
//! This port used to push `status` into the `out` buffer and only then run the
//! `--write-json` side effect, whose two `?` sites map to [`Halt`]. `Halt` does
//! NOT discard the buffer: [`evaluate`] returns `Outcome { stdout: out, code: 1 }`,
//! so a failed write printed the already-buffered PASS and exited 1. Stdout said
//! PASS, the process said 1, and which one won depended on who looked.
//!
//! [`report`] now composes into a local buffer and copies it into `out` only
//! after the write has succeeded, so a failed write emits an EMPTY stdout and a
//! non-zero exit — matching the oracle, which raises before its single `print`.
//! The write is atomic (temp file beside the target, then rename), so a refused
//! or torn write leaves NO partial artifact.
//!
//! # THE REBASE THIS PORT INHERITS (bd-lt7)
//!
//! The oracle derives its module set from `knowledge/domains.toml` rather than
//! from a numeric bound. Until 2026-08-14 it did not, and the module the course
//! assessed but did not teach had been written down as a rule rather than as a
//! recorded exemption — so the gate stayed green by luck rather than by
//! checking. This port carries the derivation, not the bound: nothing here
//! knows how many modules exist, and the count in every line of the report is
//! read from the registry at run time.
//!
//! # WHAT THIS GATE CANNOT DECIDE
//!
//! It counts items, not coverage: twenty near-identical items satisfy a floor of
//! twenty exactly as twenty distinct ones do. It reads no stem, no explanation,
//! and no topic mapping, so it says nothing about whether an item is correct,
//! well written, mapped to the right topic, or of the right difficulty. It says
//! nothing about exam pass probability. A module above its floor is a module
//! that is not STARVED, which is all that is claimed.
//!
//! It also cannot decide that the registry itself is right. If `domains.toml`
//! omits a module the course teaches, that module is invisible here and every
//! gate downstream of the registry is confidently wrong together. The floor
//! moves from *silence* to *every declared module is stocked to its recorded
//! floor, and every exemption is recorded with a reason* — no further.
//!
//! # BYTE-EXACTNESS WITH THE PYTHON ORACLE
//!
//! `scripts/verify_coverage.py` stays in the tree as the differential oracle for
//! this port; `tests/diff_verify_coverage.rs` runs BOTH implementations on every
//! case `scripts/selftest_l6_coverage.sh` exercises, plus the shapes that suite
//! never reaches, and asserts stdout, stderr, and exit code match byte for byte.
//! A disagreement on any byte fails the port, not the oracle.
//!
//! Two consequences, both deliberate and both recorded here rather than made
//! quietly:
//!
//! - **The report goes to stdout and the process exits 1**, not through
//!   `GateError`. `GateError::report` writes to stderr and maps to exit 2 or 4,
//!   which the oracle never produces; routing through it would make the two
//!   sides differ on every RED case. `crate::exit`'s codes are therefore not
//!   used by this gate's verdict path, exactly as in `verify_orphans` and
//!   `verify_bank`. `bd-2m9` flips the whole crate later; until then this is a
//!   knowing, single-file departure from the shared convention.
//! - This module carries hand-written emulations of CPython behaviour —
//!   `str.strip`, `repr()` of a `str`, a `float` and a `dict`, `int()`
//!   coercion, truthiness, iteration, `PurePosixPath` normalisation,
//!   `Path.resolve`, and `json.dumps(indent=2, sort_keys=True)` — rather than
//!   the idiomatic Rust nearest-neighbour, because the acceptance bar is
//!   identical bytes and not merely an identical verdict.
//!
//! ## Modelling the oracle's uncaught exceptions
//!
//! Several of the oracle's exotic inputs raise rather than report: a malformed
//! `bank_policy.toml` (both policy loads are unguarded), a `module` value that
//! passes the `isdigit` screen but not `int()`, a non-iterable `[[domain]]` key,
//! an infinite float where an integer is expected, an unwritable `--write-json`
//! target. CPython flushes whatever was already printed, writes a traceback to
//! stderr, and exits 1. [`Outcome`] models that: the partial stdout is kept, the
//! exit code is 1, and stderr carries a one-line description. **stdout and the
//! exit code stay byte-identical on those paths; the traceback text is the one
//! surface this port does not reproduce**, and the differential asserts exactly
//! that (equal stdout, equal code, both stderrs non-empty).
//!
//! ## Known residual deviations (none reachable from the live tree)
//!
//! - A malformed `domains.toml` yields `domain registry parse error: <msg>`,
//!   where `<msg>` comes from the `toml` crate rather than from `tomllib`; a
//!   malformed bank file yields `<file>: parse error: <msg>` the same way. Both
//!   sides go RED on the same file and the same line; only the explanation text
//!   differs.
//! - `{row!r}` of a multi-key table renders its keys in sorted order here and in
//!   insertion order in CPython, because `toml::Table` is a `BTreeMap`. Reaching
//!   it needs a `[[domain]]`, `[[domain_min]]`, or `[[coverage_exempt]]` row
//!   that is malformed AND carries more than one key.
//! - `str.isdigit()` is true for some non-ASCII digits (superscripts, other
//!   scripts) that `int()` then rejects; the screen here is ASCII-only.
//! - An unreadable or non-UTF-8 registry raises in CPython and reads as a parse
//!   error here. Both are non-zero; the bytes differ.
//! - Bad *invocation* (an unknown or ambiguous flag) returns `GateError::Usage`,
//!   where argparse prints its own usage block and exits 2. The oracle has no
//!   verdict there and no invocation in `check.sh` reaches it.

#![forbid(unsafe_code)]

use crate::registry::{GateCtx, GateError};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::Value;

pub const NAME: &str = "verify-coverage";
pub const SUMMARY: &str =
    "L6 domain coverage: every module the domain registry declares meets its item floor";

/// Engine-root-relative defaults, matching the Python module constants.
///
/// `DEFAULT_POLICY` is the live-tree location. It is NOT applied when
/// `--policy` is omitted (bd-conu): omitted `--policy` resolves to
/// `bank_policy.toml` beside the domains file this run actually loaded.
pub const DEFAULT_BANK: &str = "bank/items";
pub const DEFAULT_POLICY: &str = "knowledge/bank_policy.toml";
pub const DEFAULT_DOMAINS: &str = "knowledge/domains.toml";

/// `DEFAULT_N` — the OQ-05 ASSUMED floor applied when the policy FILE is
/// present and no `[[domain_min]]` row names a module. File-absent is ERROR
/// (bd-j98g), not this default.
pub const DEFAULT_N: i128 = 1;

/// The C1 status a floor may be measured against — the only one
/// `cdcp_assemble` may draw.
pub const APPROVED: &str = "approved";

/// The C1 lifecycle. A `status` outside this set is an ERROR, not a bucket
/// chosen by guess. An ABSENT status is the `draft` default and is not an error.
pub const KNOWN_STATUSES: &[&str] = &["approved", "draft", "retired"];

/// How many failures the report prints before it truncates. Mirrors the
/// oracle's `errors[:40]` slice.
pub const MAX_REPORT: usize = 40;

/// The long options this gate accepts, for the abbreviation resolver.
const OPTIONS: &[&str] = &["--bank", "--policy", "--domains", "--write-json"];

/// The oracle's `note` field, spelled exactly as the Python concatenates it.
const JSON_NOTE: &str = "Coverage ≠ exam pass probability; study signal only. \
                         Optional --write-json operator summary; not a shipped \
                         product input.";

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

/// `str.isdigit()`, restricted to ASCII. See the header's residual-deviation
/// list for the non-ASCII digits CPython accepts here and `int()` then rejects.
pub fn py_isdigit_ascii(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
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
/// left of -4 or right of the CPython cutoff (`format_float_short`, code 'r').
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
/// oracle's `for row in bp.get(...) or []` would happily do.
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

/// `bp.get(key) or []` followed by `for row in ...`.
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
/// prefix and append whatever is left. Symlinks in the existing prefix are
/// followed, as CPython follows them; `..` inside the non-existent tail is
/// normalised lexically by [`norm_posix`] beforehand, which is where this
/// differs from CPython on a path that mixes a symlink with a `..` it cannot
/// stat.
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

/// The oracle's `if not p.is_absolute(): p = (ROOT / p).resolve()` — note that
/// an ABSOLUTE argument is never resolved, only `PurePosixPath`-normalised, and
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

/// `Path.parent` after the same normalisation `PurePosixPath` applies.
/// `/a/b/c` → `/a/b`; `/a` → `/`; `c` → `.`; `/` and `//` stay themselves.
fn posix_parent(p: &str) -> String {
    let n = norm_posix(p);
    if n == "/" || n == "//" {
        return n;
    }
    match n.rsplit_once('/') {
        Some(("", _)) => "/".to_string(),
        Some((parent, _)) => parent.to_string(),
        None => ".".to_string(),
    }
}

/// Omitted `--policy` (bd-conu): the policy file beside the domains registry,
/// never `DEFAULT_POLICY` joined to the engine root.
fn policy_beside_domains(domains_disp: &str) -> String {
    join_posix(&posix_parent(domains_disp), &path_name(DEFAULT_POLICY))
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

/// The subset of JSON the oracle's summary uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum J {
    Int(i128),
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
        J::Int(i) => out.push_str(&i.to_string()),
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args {
    pub bank: Option<String>,
    pub policy: Option<String>,
    pub domains: Option<String>,
    pub write_json: Option<String>,
}

/// One bank item as the Python loop sees it: the file it came from, and its table.
type Item = (String, toml::Table);

/// What `count_modules` returns: the APPROVED counts per module (what the floors
/// are measured against), the SCANNED counts per module (every item that loaded,
/// whatever its status), and the per-item errors. Two populations, deliberately
/// both — see [`count_modules`].
type CountedModules = (BTreeMap<i128, u64>, BTreeMap<i128, u64>, Vec<String>);

fn load_toml(path: &Path) -> Result<toml::Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    text.parse::<toml::Table>().map_err(|e| e.to_string())
}

/// `load_declared_modules()` — the module set, derived from the domain registry.
///
/// A registry that is missing, malformed, or empty yields zero modules AND an
/// error, never a silent empty set that would make every floor below vacuously
/// satisfied. The missing and parse-error legs return early, so they do NOT
/// additionally report "declares zero modules" — that ordering is the oracle's.
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
                // quoted — and not bare. Measured against the oracle 2026-08-14.
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

/// `load_exemptions()` — recorded `[[coverage_exempt]]` rows.
///
/// Every rejection path `continue`s WITHOUT recording the exemption, so a
/// malformed row leaves its module REQUIRED and the shortfall still reports.
/// That is the property `tests/diff_verify_coverage.rs` pins: the escape hatch
/// cannot be quieter than the rule it escapes.
fn load_exemptions(
    policy_disp: &str,
    declared: &BTreeMap<i128, String>,
) -> H<(BTreeMap<i128, String>, Vec<String>)> {
    let mut errors: Vec<String> = Vec::new();
    let mut exempt: BTreeMap<i128, String> = BTreeMap::new();
    // Missing file: empty exemptions (stricter). Do NOT copy verify_objectives'
    // ABSENT-OK sentence here: the floors path ERRORs on the same absence
    // (bd-j98g), because absence would lower sized [[domain_min]] rows.
    if !Path::new(policy_disp).is_file() {
        return Ok((exempt, errors));
    }
    // The oracle does NOT guard this load; a malformed policy raises.
    let bp = load_toml(Path::new(policy_disp))
        .map_err(|e| Halt(format!("tomllib.TOMLDecodeError: {policy_disp}: {e}")))?;

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

/// `load_domain_mins()` — per-module floors, defaulting to [`DEFAULT_N`] when
/// the policy FILE is present but a module has no row.
///
/// Absence of the file is an ERROR, not a fallback (bd-j98g). The sized floors
/// live here; defaulting to N=1 would lower them (fail-open). A present file
/// with empty `[[domain_min]]` is the honest N=1 default — distinguishable
/// from a missing file.
///
/// A `[[domain_min]]` row keyed to a module the registry does not declare is
/// the cross-source drift leg: the two sources of truth for "which modules
/// exist" have diverged.
fn load_domain_mins(
    policy_disp: &str,
    required: &[i128],
) -> H<(BTreeMap<i128, i128>, Vec<String>)> {
    let mut errors: Vec<String> = Vec::new();
    let mut mins: BTreeMap<i128, i128> = required.iter().map(|m| (*m, DEFAULT_N)).collect();
    if !Path::new(policy_disp).is_file() {
        errors.push(format!(
            "bank_policy.toml missing: {policy_disp} (absence would lower sized [[domain_min]] floors to N=1)"
        ));
        return Ok((mins, errors));
    }
    let bp = load_toml(Path::new(policy_disp))
        .map_err(|e| Halt(format!("tomllib.TOMLDecodeError: {policy_disp}: {e}")))?;

    for row in py_rows(&bp, "domain_min")? {
        // `row["module"]` on a non-dict raises TypeError, which the oracle's
        // except tuple catches — the row lands in the same "unusable" bucket.
        let pair = match &row {
            Value::Table(t) => {
                let m = py_int(t.get("module"));
                let n = py_int(t.get("min_items"));
                match (m, n) {
                    (Err(IntErr::Uncaught(e)), _) | (Ok(_), Err(IntErr::Uncaught(e))) => {
                        return Err(Halt(e))
                    }
                    (Ok(m), Ok(n)) => Some((m, n)),
                    _ => None,
                }
            }
            _ => None,
        };
        let Some((module, need)) = pair else {
            errors.push(format!(
                "bank_policy.toml: unusable [[domain_min]] row {}",
                py_repr_value(Some(&row))
            ));
            continue;
        };
        // `if mod in mins` — a floor may only RAISE a module that is already
        // required, never introduce one. Never below the OQ-05 floor either.
        match mins.entry(module) {
            std::collections::btree_map::Entry::Occupied(mut e) => {
                e.insert(std::cmp::max(DEFAULT_N, need));
            }
            std::collections::btree_map::Entry::Vacant(_) => errors.push(format!(
                "bank_policy.toml: [[domain_min]] module {module} is not a required \
                 module in the domain registry"
            )),
        }
    }
    Ok((mins, errors))
}

/// `load_items()` — every `*.toml` under the bank dir, in `sorted()` order.
///
/// Carries the file-granular anti-vacuous rule the oracle grew in bd-0czh (the
/// class sweep of bd-2kr): an `items[]` that yields zero items is named and is
/// RED. Ported AFTER the Python, so the differential stayed the judge of the
/// change rather than being reshaped by it.
fn read_items(disp: &str) -> (Vec<Item>, Vec<String>) {
    let dir = Path::new(disp);
    let mut errors: Vec<String> = Vec::new();
    let mut loaded: Vec<Item> = Vec::new();
    if !dir.is_dir() {
        return (loaded, vec![format!("bank dir missing: {disp}")]);
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return (loaded, errors);
    };
    // `pathlib.glob("*.toml")` matches dotfiles and is case-sensitive, and
    // `sorted()` orders by name.
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
                    // Anti-vacuous at FILE granularity (bd-0czh). An `items[]`
                    // that yields nothing took this branch and can never reach
                    // the `no id or items[]` leg — Python's `elif` cannot run
                    // once the `if` has — so without this the file is scanned,
                    // contributes zero items, and is never named while the
                    // aggregate `empty bank` check stays satisfied by its
                    // neighbours.
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

/// `count_modules()` — TWO `Counter`s keyed by the integer `module` of each item.
///
/// The first is the APPROVED pool, which is what the floors are measured
/// against; the second is everything that scanned, whatever its status, so the
/// report can name both numbers side by side. A report that showed only one of
/// them is how a floor came to be checked against a set no learner draws from
/// (bd-coverage-counts-retired-items-49jh).
fn count_modules(loaded: &[Item]) -> H<CountedModules> {
    let mut approved: BTreeMap<i128, u64> = BTreeMap::new();
    let mut scanned: BTreeMap<i128, u64> = BTreeMap::new();
    let mut errors: Vec<String> = Vec::new();
    for (fname, it) in loaded {
        let module = it.get("module");
        let mi = match py_int(module) {
            Ok(mi) => mi,
            Err(IntErr::Uncaught(m)) => return Err(Halt(m)),
            Err(IntErr::Caught(_)) => {
                let iid = match it.get("id") {
                    Some(v) if py_truthy(v) => py_str_value(v),
                    _ => fname.clone(),
                };
                errors.push(format!("{iid}: bad module {}", py_repr_value(module)));
                continue;
            }
        };
        *scanned.entry(mi).or_insert(0) += 1;
        // `it.get("status", "draft")`: an absent status is the C1 default and is
        // NOT an error; a present one must be a str equal to a known status.
        let status = it.get("status");
        if matches!(status, Some(Value::String(s)) if s == APPROVED) {
            *approved.entry(mi).or_insert(0) += 1;
        } else {
            let known = match status {
                None => true,
                Some(Value::String(s)) => KNOWN_STATUSES.contains(&s.as_str()),
                Some(_) => false,
            };
            if !known {
                // Fail-closed AND loud. Dropping an unmodelled status silently
                // into "not approved" would be the same defect one level down.
                let iid = match it.get("id") {
                    Some(v) if py_truthy(v) => py_str_value(v),
                    _ => fname.clone(),
                };
                errors.push(format!("{iid}: unknown status {}", py_repr_value(status)));
            }
        }
    }
    Ok((approved, scanned, errors))
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

/// Write the `--write-json` summary so a FAILED write leaves NOTHING behind.
///
/// Temp file beside the target, then `rename`, which is atomic on one
/// filesystem. Mirrors the oracle's `write_summary`, including the temp name
/// (`<target>.tmp`) — a name nothing ever observes, because it either becomes
/// the target or is removed, and the caller never prints a verdict over a
/// failure here.
fn write_summary(out_path: &str, body: &str) -> H<()> {
    let p = Path::new(out_path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Halt(format!("OSError: {}: {e}", parent.display())))?;
    }
    let tmp = PathBuf::from(format!("{out_path}.tmp"));
    let res = std::fs::write(&tmp, body).and_then(|()| std::fs::rename(&tmp, p));
    if let Err(e) = res {
        let _ = std::fs::remove_file(&tmp);
        return Err(Halt(format!("OSError: {out_path}: {e}")));
    }
    Ok(())
}

fn report(out: &mut String, root_str: &str, args: &Args) -> H<i32> {
    let bank_disp = resolve_arg(root_str, args.bank.as_deref(), DEFAULT_BANK);
    let domains_disp = resolve_arg(root_str, args.domains.as_deref(), DEFAULT_DOMAINS);
    let policy_disp = match args.policy.as_deref() {
        Some(v) => resolve_arg(root_str, Some(v), DEFAULT_POLICY),
        None => policy_beside_domains(&domains_disp),
    };

    let mut errors: Vec<String> = Vec::new();

    let (declared, declared_errors) = load_declared_modules(&domains_disp)?;
    errors.extend(declared_errors);
    let (exempt, exempt_errors) = load_exemptions(&policy_disp, &declared)?;
    errors.extend(exempt_errors);

    let required: Vec<i128> = declared
        .keys()
        .copied()
        .filter(|m| !exempt.contains_key(m))
        .collect();
    let (domain_mins, min_errors) = load_domain_mins(&policy_disp, &required)?;
    errors.extend(min_errors);

    let (loaded, load_errors) = read_items(&bank_disp);
    let (module_counts, scanned_counts, mod_errors) = count_modules(&loaded)?;
    errors.extend(load_errors);
    errors.extend(mod_errors);

    let n = loaded.len();
    let approved_n: u64 = module_counts.values().sum();
    if n == 0 {
        errors.push("empty bank: zero items loaded (vacuous coverage is ERROR)".to_string());
    } else if approved_n == 0 {
        // A bank FULL of files and empty of drawable items is the exact state a
        // file-counting floor reported green on. Named separately from the
        // empty-bank leg because it is a different failure with the same verdict.
        errors.push(format!(
            "zero approved items ({n} scanned): the floors measure a pool no \
             learner can be assessed from (vacuous coverage is ERROR)"
        ));
    }
    if required.is_empty() {
        errors
            .push("zero required modules after exemptions (vacuous coverage is ERROR)".to_string());
    }

    let mut shortfalls: Vec<(i128, u64, i128, u64)> = Vec::new();
    for module in &required {
        let need = domain_mins.get(module).copied().unwrap_or(DEFAULT_N);
        let have = module_counts.get(module).copied().unwrap_or(0);
        let seen = scanned_counts.get(module).copied().unwrap_or(0);
        if i128::from(have) < need {
            // Both numbers, deliberately: `44 scanned` under a floor of 24 is
            // exactly the reading that made this fail open for a week.
            errors.push(format!(
                "module {module}: {have} approved < min {need} \
                 ({seen} scanned, {} not approved)",
                seen - have
            ));
            shortfalls.push((*module, have, need, seen));
        }
    }

    // Report: every required module, then recorded exemptions, then anything the
    // bank carries that the registry never declared.
    //
    // COMPOSED INTO A LOCAL BUFFER, not into `out`. See the module header: `out`
    // survives a `Halt` into `Outcome.stdout`, so anything pushed there before
    // the `--write-json` side effect would be printed alongside exit 1. The copy
    // into `out` is the LAST thing this function does before returning a code.
    let status = if errors.is_empty() { "PASS" } else { "FAIL" };
    let mut body = String::new();
    body.push_str(status);
    body.push('\n');
    body.push_str(&format!("  bank={bank_disp}\n"));
    body.push_str(&format!(
        "  items={n} scanned, {approved_n} approved \
         (floors count the approved pool only)\n"
    ));
    body.push_str(&format!(
        "  policy={}\n",
        if Path::new(&policy_disp).is_file() {
            "present"
        } else {
            "absent"
        }
    ));
    body.push_str(&format!(
        "  registry={} declares={}\n",
        path_name(&domains_disp),
        declared.len()
    ));
    body.push_str(&format!(
        "  modules ({} required, derived from the domain registry):\n",
        required.len()
    ));
    for module in &required {
        let have = module_counts.get(module).copied().unwrap_or(0);
        let seen = scanned_counts.get(module).copied().unwrap_or(0);
        let need = domain_mins.get(module).copied().unwrap_or(DEFAULT_N);
        let flag = if i128::from(have) >= need && n > 0 {
            "ok"
        } else {
            "SHORT"
        };
        body.push_str(&format!(
            "    m{module:02}: {have} approved of {seen} scanned (min {need}) [{flag}]\n"
        ));
    }
    if !exempt.is_empty() {
        body.push_str("  recorded exemptions (bank_policy.toml [[coverage_exempt]]):\n");
        for (module, reason) in &exempt {
            let have = module_counts.get(module).copied().unwrap_or(0);
            let seen = scanned_counts.get(module).copied().unwrap_or(0);
            body.push_str(&format!(
                "    m{module:02}: {have} approved of {seen} scanned — exempt: {reason}\n"
            ));
        }
    }
    // Drift is a property of the FILE SET, not of the drawable pool: a retired
    // item filed under a module the registry never declared is still drift, and
    // counting extras on the approved pool would hide it.
    let extras: Vec<i128> = scanned_counts
        .keys()
        .copied()
        .filter(|m| !declared.contains_key(m))
        .collect();
    if !extras.is_empty() {
        body.push_str("  undeclared modules present in the bank (not required for green):\n");
        for module in &extras {
            let seen = scanned_counts.get(module).copied().unwrap_or(0);
            body.push_str(&format!(
                "    m{module:02}: {seen} scanned (not in the domain registry)\n"
            ));
        }
    }

    if let Some(target) = &args.write_json {
        // Prefer repo-relative bank path in JSON for portable commits.
        let bank_rel = relative_to(&py_resolve(&bank_disp), &py_resolve(root_str))
            .unwrap_or_else(|()| bank_disp.clone());
        let summary = summary_json(
            status,
            &bank_rel,
            n,
            approved_n,
            &path_name(&domains_disp),
            &declared,
            &required,
            &exempt,
            &domain_mins,
            &module_counts,
            &scanned_counts,
            &extras,
            &shortfalls,
        );
        let out_path = if target.starts_with('/') {
            norm_posix(target)
        } else {
            py_resolve(&join_posix(root_str, target))
        };
        // THE SIDE EFFECT RUNS BEFORE THE VERDICT REACHES `out`. Nothing has
        // been copied out of `body` yet, so a failed write returns Halt with an
        // EMPTY stdout — never with a PASS a reader would have believed. The
        // `status` baked into the summary is the pre-write verdict, which is
        // sound precisely because the file only exists when the write succeeded,
        // and when it succeeded the pre-write verdict is the final one.
        write_summary(&out_path, &format!("{}\n", json_dumps(&summary)))?;
        body.push_str(&format!("  wrote {out_path}\n"));
    }

    if !errors.is_empty() {
        body.push_str("  failures:\n");
        for e in errors.iter().take(MAX_REPORT) {
            body.push_str(&format!("    - {e}\n"));
        }
        if errors.len() > MAX_REPORT {
            body.push_str(&format!("    ... +{} more\n", errors.len() - MAX_REPORT));
        }
        out.push_str(&body);
        return Ok(1);
    }

    // Enumerated, not spanned: an exemption can leave a gap, and a range would
    // read as covering a module that was held out.
    let span = required
        .iter()
        .map(|m| format!("m{m:02}"))
        .collect::<Vec<_>>()
        .join(" ");
    body.push_str(&format!(
        "  coverage GREEN ({} required modules ≥ domain_min: {span})\n",
        required.len()
    ));
    out.push_str(&body);
    Ok(0)
}

#[allow(clippy::too_many_arguments)]
fn summary_json(
    status: &str,
    bank_rel: &str,
    n: usize,
    approved_n: u64,
    module_source: &str,
    declared: &BTreeMap<i128, String>,
    required: &[i128],
    exempt: &BTreeMap<i128, String>,
    domain_mins: &BTreeMap<i128, i128>,
    module_counts: &BTreeMap<i128, u64>,
    scanned_counts: &BTreeMap<i128, u64>,
    extras: &[i128],
    shortfalls: &[(i128, u64, i128, u64)],
) -> J {
    let ints = |xs: &[i128]| J::List(xs.iter().map(|x| J::Int(*x)).collect());
    J::Obj(vec![
        // v3: `counts` changed population — it was the file set and is now the
        // APPROVED pool, a semantic change no consumer could detect from the
        // numbers alone, so the version moves with it. `item_count` keeps its
        // old meaning and `approved_count`/`scanned_counts` are added, so both
        // populations are in the ledger.
        ("schema_version".into(), J::Int(3)),
        ("gate".into(), J::Str("l6-domain-coverage".into())),
        ("status".into(), J::Str(status.to_lowercase())),
        ("bank".into(), J::Str(bank_rel.to_string())),
        ("item_count".into(), J::Int(n as i128)),
        ("approved_count".into(), J::Int(i128::from(approved_n))),
        ("module_source".into(), J::Str(module_source.to_string())),
        (
            "declared_modules".into(),
            ints(&declared.keys().copied().collect::<Vec<_>>()),
        ),
        ("primary_modules".into(), ints(required)),
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
            "domain_min".into(),
            J::Obj(
                domain_mins
                    .iter()
                    .map(|(k, v)| (k.to_string(), J::Int(*v)))
                    .collect(),
            ),
        ),
        (
            "counts".into(),
            J::Obj(
                required
                    .iter()
                    .map(|k| {
                        (
                            k.to_string(),
                            J::Int(i128::from(module_counts.get(k).copied().unwrap_or(0))),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "scanned_counts".into(),
            J::Obj(
                required
                    .iter()
                    .map(|k| {
                        (
                            k.to_string(),
                            J::Int(i128::from(scanned_counts.get(k).copied().unwrap_or(0))),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "extra_counts".into(),
            J::Obj(
                extras
                    .iter()
                    .map(|k| {
                        (
                            k.to_string(),
                            J::Int(i128::from(scanned_counts.get(k).copied().unwrap_or(0))),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "shortfalls".into(),
            J::List(
                shortfalls
                    .iter()
                    .map(|(m, have, need, seen)| {
                        J::Obj(vec![
                            ("module".into(), J::Int(*m)),
                            ("have".into(), J::Int(i128::from(*have))),
                            ("min".into(), J::Int(*need)),
                            ("scanned".into(), J::Int(i128::from(*seen))),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("oq05_default_n".into(), J::Int(DEFAULT_N)),
        ("note".into(), J::Str(JSON_NOTE.to_string())),
    ])
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

/// The argv tail, accepting `--opt v`, `--opt=v`, and the unambiguous prefixes
/// argparse allows. A repeated option keeps the last value, as argparse does.
pub fn parse_args(args: &[String]) -> Result<Args, GateError> {
    let mut parsed = Args::default();
    let mut i = 0;
    while i < args.len() {
        let (name, inline) = match args[i].split_once('=') {
            Some((n, v)) => (n, Some(v.to_string())),
            None => (args[i].as_str(), None),
        };
        let opt = resolve_option(name)?;
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
            "--bank" => parsed.bank = Some(value),
            "--policy" => parsed.policy = Some(value),
            "--domains" => parsed.domains = Some(value),
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
    // and only to the root: an absolute `--bank`/`--policy`/`--domains` is
    // printed exactly as `PurePosixPath` normalises it, never canonicalised.
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
    fn repr_matches_cpython_on_the_shapes_this_gate_prints() {
        assert_eq!(py_repr("abc"), "'abc'");
        assert_eq!(py_repr("it's"), "\"it's\"");
        assert_eq!(py_repr("a\nb"), "'a\\nb'");
        assert_eq!(py_repr_value(None), "None");
        assert_eq!(py_repr_value(Some(&Value::Integer(7))), "7");
        assert_eq!(py_repr_value(Some(&Value::Boolean(true))), "True");
        assert_eq!(py_repr_value(Some(&Value::Float(2.5))), "2.5");
    }

    #[test]
    fn int_coercion_splits_caught_from_uncaught() {
        assert_eq!(py_int(Some(&Value::Integer(9))), Ok(9));
        assert_eq!(py_int(Some(&Value::Boolean(true))), Ok(1));
        assert_eq!(py_int(Some(&Value::Float(3.9))), Ok(3));
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
    fn isdigit_screen_is_ascii_only_and_never_vacuous() {
        assert!(py_isdigit_ascii("20"));
        assert!(!py_isdigit_ascii(""));
        assert!(!py_isdigit_ascii("2.0"));
        assert!(!py_isdigit_ascii("-"));
    }

    #[test]
    fn posix_normalisation_matches_purepath() {
        assert_eq!(norm_posix("./bank//items/"), "bank/items");
        assert_eq!(norm_posix("/a//b/./c/"), "/a/b/c");
        assert_eq!(norm_posix("//a/b"), "//a/b");
        assert_eq!(norm_posix("///a/b"), "/a/b");
        assert_eq!(norm_posix(""), ".");
        assert_eq!(join_posix("/root", "bank/items"), "/root/bank/items");
        assert_eq!(join_posix("/root", "/abs"), "/abs");
        assert_eq!(path_name("/a/b/domains.toml"), "domains.toml");
        assert_eq!(posix_parent("/a/b/domains.toml"), "/a/b");
        assert_eq!(posix_parent("/a"), "/");
        assert_eq!(posix_parent("domains.toml"), ".");
        assert_eq!(posix_parent("/"), "/");
        assert_eq!(
            policy_beside_domains("/tmp/fix/d.toml"),
            "/tmp/fix/bank_policy.toml"
        );
        assert_eq!(
            policy_beside_domains("/eng/knowledge/domains.toml"),
            "/eng/knowledge/bank_policy.toml"
        );
    }

    #[test]
    fn relative_to_is_lexical_and_raises_when_outside() {
        assert_eq!(relative_to("/r/bank/items", "/r"), Ok("bank/items".into()));
        assert_eq!(relative_to("/r", "/r"), Ok(".".into()));
        assert_eq!(relative_to("/tmp/x", "/r"), Err(()));
    }

    #[test]
    fn json_dumps_matches_python_indent_and_sort_keys() {
        let v = J::Obj(vec![
            ("b".into(), J::Int(2)),
            ("a".into(), J::List(vec![J::Int(1)])),
            ("c".into(), J::Obj(vec![])),
            ("d".into(), J::List(vec![])),
        ]);
        assert_eq!(
            json_dumps(&v),
            "{\n  \"a\": [\n    1\n  ],\n  \"b\": 2,\n  \"c\": {},\n  \"d\": []\n}"
        );
        assert_eq!(json_str("≠"), "\"\\u2260\"");
        assert_eq!(json_str("plain"), "\"plain\"");
    }

    #[test]
    fn option_prefixes_resolve_the_way_argparse_does() {
        assert_eq!(resolve_option("--bank").unwrap(), "--bank");
        assert_eq!(resolve_option("--b").unwrap(), "--bank");
        assert_eq!(resolve_option("--d").unwrap(), "--domains");
        assert_eq!(resolve_option("--w").unwrap(), "--write-json");
        assert!(resolve_option("--nope").is_err());
        let a = parse_args(&["--bank=x".to_string(), "--policy".into(), "y".into()]).unwrap();
        assert_eq!(a.bank.as_deref(), Some("x"));
        assert_eq!(a.policy.as_deref(), Some("y"));
        assert_eq!(a.domains, None);
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
        assert!(errors[0].contains("has no reason"), "{:?}", errors);
    }

    #[test]
    fn an_empty_registry_is_an_error_not_an_empty_pass() {
        let td = tempfile::tempdir().unwrap();
        let reg = td.path().join("domains.toml");
        std::fs::write(&reg, "schema_version = 1\n").unwrap();
        let (declared, errors) = load_declared_modules(reg.to_str().unwrap()).expect("no raise");
        assert!(declared.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("declares zero modules"), "{:?}", errors);

        let missing = td.path().join("nope.toml");
        let (declared, errors) =
            load_declared_modules(missing.to_str().unwrap()).expect("no raise");
        assert!(declared.is_empty());
        assert_eq!(errors.len(), 1, "the missing leg returns early: {errors:?}");
        assert!(errors[0].starts_with("domain registry missing:"));
    }

    #[test]
    fn the_gate_is_registered_under_a_kebab_case_name() {
        assert_eq!(NAME, "verify-coverage");
        assert!(crate::registry::find(NAME).is_some());
    }

    /// Known-bad for bd-conu: a policy planted at the engine root must not be
    /// read when `--bank`/`--domains` are an isolated fixture and `--policy`
    /// was never passed. Restoring `resolve_arg(..., DEFAULT_POLICY)` makes
    /// this name module 9. bd-j98g: the same absence is now ERROR (missing
    /// local file), not GREEN-at-N=1 — still without reading the planted
    /// engine-root policy.
    #[test]
    fn omitted_policy_does_not_read_a_policy_at_the_engine_root() {
        let td = tempfile::tempdir().unwrap();
        let live = td.path().join("engine");
        std::fs::create_dir_all(live.join("knowledge")).unwrap();
        std::fs::write(
            live.join("knowledge/bank_policy.toml"),
            "[[domain_min]]\nmodule = 9\nmin_items = 99\n",
        )
        .unwrap();
        let fx = td.path().join("fx");
        std::fs::create_dir_all(fx.join("bank")).unwrap();
        std::fs::write(
            fx.join("d.toml"),
            "schema_version = 1\n\n[[domain]]\nid = \"d\"\norder = 1\n",
        )
        .unwrap();
        std::fs::write(
            fx.join("bank/a.toml"),
            "id = \"a\"\nmodule = 1\nstatus = \"approved\"\n",
        )
        .unwrap();
        let args = Args {
            bank: Some(fx.join("bank").to_string_lossy().into_owned()),
            domains: Some(fx.join("d.toml").to_string_lossy().into_owned()),
            policy: None,
            write_json: None,
        };
        let out = evaluate(&live.to_string_lossy(), &args);
        assert_ne!(
            out.code, 0,
            "missing local policy must be RED:\n{}",
            out.stdout
        );
        assert!(out.stdout.contains("policy=absent"), "{}", out.stdout);
        assert!(
            !out.stdout.contains("policy=absent (N=1 OQ-05)"),
            "N=1 must not be claimed as a fallback:\n{}",
            out.stdout
        );
        assert!(
            out.stdout.contains("bank_policy.toml missing:"),
            "the missing local file must be named:\n{}",
            out.stdout
        );
        assert!(
            !out.stdout.contains("module 9"),
            "engine-root policy leaked:\n{}",
            out.stdout
        );
        assert!(
            !out.stdout.contains("[[domain_min]] module"),
            "engine-root domain_min rows leaked:\n{}",
            out.stdout
        );
    }
}
