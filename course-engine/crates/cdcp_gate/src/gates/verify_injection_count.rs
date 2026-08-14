//! verify-injection-count — L4 drift guard for the advertised known-bad count.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor and one floor only: **the number README.md
//! advertises about known-bad injections can no longer drift silently away from
//! the number the selftest suites actually assert at runtime.** Each suite
//! self-reports one `INJECTIONS=<n> SUITE=<name>` receipt on its success path;
//! `check.sh` tees those receipts into a log; this gate sums the registered ones
//! and compares the total against every count README advertises. A registered
//! suite that emits no receipt is an ERROR, never a silent zero, and an empty log
//! is an ERROR, never a pass.
//!
//! ## What it cannot decide
//!
//! * Whether a receipt is **honest**. The counter is incremented by the suite's
//!   own assert helper. A suite that increments without observing a real RED is
//!   invisible here — that is what the suites' own known-bad cases are for.
//! * Whether the log came from **this** run. It reads whatever file it is handed;
//!   freshness is `check.sh`'s job (it mktemps a new log per invocation).
//! * Whether README's **prose** is accurate about anything other than the two
//!   numbers it scans (injection count, selftest-suite count). A count spelled in
//!   words instead of digits is not seen at all.
//! * Whether the registry itself is right. `--require` names the suites that must
//!   report; a suite nobody registered and nobody runs is outside its reach.
//!
//! # This is a PORT, not a rewrite
//!
//! It replaces `scripts/verify_injection_count.py` byte-for-byte on stdout and on
//! the exit code, verified case-by-case against the Python original by
//! `tests/diff_verify_injection_count.rs`. Behaviour that looked like a bug in the
//! Python was reproduced, not corrected — a port that fixes a bug is an unreviewed
//! behaviour change. Two reproduced quirks worth knowing:
//!
//! * A `--require` list containing the same suite twice **double-counts** that
//!   suite's receipt into `measured_total`. Reproduced deliberately.
//! * Failure findings name `README.md:<lineno>` literally, even when `--readme`
//!   pointed somewhere else. Reproduced deliberately.
//!
//! ## Exit codes: 0 / 1, not the crate's 0 / 2 / 3 / 4
//!
//! The failure path exits **1**, matching the Python it replaces, because
//! byte-exact substitution is this port's acceptance bar and `check.sh` consumes
//! only "zero vs non-zero". The crate's structured codes are still used for the
//! one case the Python cannot be matched on: a bad invocation. Python's argparse
//! writes a usage block whose line wrapping depends on `COLUMNS`, i.e. on the
//! terminal, not on the input — so there is no byte-exact target to hit. Bad argv
//! therefore returns `GateError::Usage` (exit 3, message on stderr) instead.

use crate::registry::{GateCtx, GateError};
use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const NAME: &str = "verify-injection-count";
pub const SUMMARY: &str =
    "sum the suites' self-reported known-bad injections and compare with README's advertised count";

/// Every suite that asserts known-bad injections. Mirrors `REGISTERED_SUITES` in
/// the Python original; it is the default value of `--require`.
///
/// Deliberately NOT registered, with the reason (an exclusion without a reason is
/// a schema error):
///   tests/publishability-bar.sh — asserts facts about the repo. It plants no
///     known-bad and asserts no RED, so counting it would inflate the advertised
///     number with checks that never showed they can trip.
pub const REGISTERED_SUITES: &[&str] = &[
    "selftest_known_bad",
    "selftest_l5",
    "selftest_l5_honesty",
    "selftest_l6_coverage",
    "selftest_l7_objectives",
    "selftest_reconstructed",
    "selftest_orphan",
    "selftest_doc_consistency",
    "selftest_injection_count",
];

// ───────────────────────── Python runtime emulation ────────────────────────
//
// The port has to agree with CPython on four primitives, because all four are
// load-bearing in the original's output: `str.isspace`, `str.splitlines`,
// `repr(str)`, and `str(PurePosixPath)`. Each is reimplemented here rather than
// approximated with the nearest Rust equivalent, which differs in every case.

/// CPython `str.isspace()` for one char. Rust's `char::is_whitespace` is the
/// Unicode White_Space property and therefore MISSES U+001C..U+001F, which
/// CPython treats as whitespace for both `strip()` and regex `\s`.
fn py_is_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// CPython `str.strip()` with no argument.
fn py_strip(s: &str) -> &str {
    s.trim_matches(py_is_space)
}

/// CPython `str.splitlines()`. Rust's `str::lines` splits on `\n` / `\r\n` only;
/// CPython additionally splits on `\r`, `\x0b`, `\x0c`, `\x1c`, `\x1d`, `\x1e`,
/// `\u{85}`, `\u{2028}`, `\u{2029}`. README line NUMBERS in the findings depend on
/// this, so the difference is observable.
fn py_splitlines(s: &str) -> Vec<&str> {
    fn is_boundary(c: char) -> bool {
        matches!(
            c,
            '\n' | '\r'
                | '\u{0b}'
                | '\u{0c}'
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
        if !is_boundary(c) {
            continue;
        }
        out.push(&s[start..i]);
        let mut end = i + c.len_utf8();
        if c == '\r' && it.peek().map(|&(_, n)| n) == Some('\n') {
            it.next();
            end += 1;
        }
        start = end;
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

/// CPython `repr()` of a `str`. Quote selection and escaping both matter: the
/// original embeds `{raw.strip()!r}` in a finding, so an unparseable receipt
/// containing a quote changes the bytes on stdout.
///
/// Printability follows `str.isprintable()`: everything in Unicode categories Cc,
/// Cf, Cs, Co, Cn, Zl, Zp and Zs is escaped, except U+0020 itself. Rust's std
/// exposes no category table, so `py_is_printable` reconstructs the predicate
/// from `is_control` + `is_whitespace` + an explicit Cf/Co range list. Cn
/// (unassigned) code points are the residue it does not model; they pass through
/// where CPython would escape them. Reachable only from adversarial log content,
/// and only inside a finding that is already RED.
fn py_is_printable(c: char) -> bool {
    if c == ' ' {
        return true;
    }
    if c.is_ascii() {
        return matches!(c as u32, 0x21..=0x7e);
    }
    if c.is_control() || c.is_whitespace() {
        return false; // Cc, Zs, Zl, Zp, U+0085
    }
    !matches!(c as u32,
        0x00ad
        | 0x0600..=0x0605 | 0x061c | 0x06dd | 0x070f | 0x08e2
        | 0x180e | 0x200b..=0x200f | 0x202a..=0x202e
        | 0x2060..=0x2064 | 0x2066..=0x206f
        | 0xd800..=0xf8ff // Cs + Co private use
        | 0xfeff | 0xfff9..=0xfffb
        | 0xf_0000..=0x10_fffd // Co supplementary private use
    )
}

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
            c if !py_is_printable(c) => {
                let n = c as u32;
                if n < 0x100 {
                    out.push_str(&format!("\\x{n:02x}"));
                } else if n < 0x1_0000 {
                    out.push_str(&format!("\\u{n:04x}"));
                } else {
                    out.push_str(&format!("\\U{n:08x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `str(pathlib.PurePosixPath(s))`. argparse builds a `Path` from the raw argv
/// string and the original interpolates it into `log=…` and into two findings, so
/// `.//x` must print as `x` and `""` must print as `.`.
pub fn py_path_str(s: &str) -> String {
    if s.is_empty() {
        return ".".to_string();
    }
    let leading = s.bytes().take_while(|&b| b == b'/').count();
    let root = match leading {
        0 => "",
        2 => "//",
        _ => "/",
    };
    let parts: Vec<&str> = s
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    if parts.is_empty() {
        return if root.is_empty() {
            ".".to_string()
        } else {
            root.to_string()
        };
    }
    format!("{root}{}", parts.join("/"))
}

// ─────────────────────────── regex emulation ───────────────────────────────
//
// The original's three `re` patterns are hand-compiled below rather than pulled
// in as a dependency. Each is deliberately annotated with the pattern it stands
// for, because the port is only correct if the emulation is.
//
// One deliberate narrowing across all three: CPython's `\d` matches every Unicode
// Nd digit and `int()` accepts them; these accept ASCII 0-9 only. The effect is
// one-directional — an exotic digit makes a receipt UNPARSEABLE (still RED) or a
// README count INVISIBLE (which trips the "advertises no count at all" ERROR). It
// can turn a Python green into a Rust red, never a Python red into a Rust green.

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `\b` at byte offset `p` (which must be a char boundary).
fn at_word_boundary(s: &str, p: usize) -> bool {
    let before = s[..p].chars().next_back().is_some_and(is_word);
    let after = s[p..].chars().next().is_some_and(is_word);
    before != after
}

/// Maximal run of ASCII digits from `p`. Returns the end offset (== `p` if none).
fn digits_end(s: &str, p: usize) -> usize {
    p + s[p..].bytes().take_while(u8::is_ascii_digit).count()
}

/// Maximal run of chars satisfying `f` from `p`.
fn run_end(s: &str, p: usize, f: impl Fn(char) -> bool) -> usize {
    let mut e = p;
    for c in s[p..].chars() {
        if !f(c) {
            break;
        }
        e += c.len_utf8();
    }
    e
}

/// ASCII case-insensitive literal match at `p`; returns the end offset.
fn lit_ci(s: &str, p: usize, lit: &str) -> Option<usize> {
    let end = p + lit.len();
    if end <= s.len() && s.is_char_boundary(end) && s[p..end].eq_ignore_ascii_case(lit) {
        Some(end)
    } else {
        None
    }
}

/// Drop leading zeros the way `int()` then `str()` would: `"007"` -> `"7"`.
/// Working in normalized decimal strings rather than a fixed-width integer keeps
/// an absurdly large advertised count printing exactly as CPython would.
fn norm_digits(d: &str) -> String {
    let t = d.trim_start_matches('0');
    if t.is_empty() {
        "0".to_string()
    } else {
        t.to_string()
    }
}

/// Numeric ordering over normalized decimal strings.
fn cmp_norm(a: &String, b: &String) -> std::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// `^INJECTIONS=(\d+)\s+SUITE=(\S+)\s*$` applied to an already-stripped line.
///
/// No backtracking is needed: `\d+` is followed by `\s+` and `\s+` by a literal
/// `S`, and the classes are disjoint, so the greedy runs are forced. After
/// `strip()` there is no trailing whitespace, so `(\S+)\s*$` reduces to "the rest
/// is non-empty and holds no whitespace".
fn parse_receipt(s: &str) -> Option<(&str, &str)> {
    // `^INJECTIONS=` is case-SENSITIVE in the original pattern.
    if !s.starts_with("INJECTIONS=") {
        return None;
    }
    let p = "INJECTIONS=".len();
    let d_end = digits_end(s, p);
    if d_end == p {
        return None;
    }
    let w_end = run_end(s, d_end, py_is_space);
    if w_end == d_end {
        return None;
    }
    if !s[w_end..].starts_with("SUITE=") {
        return None;
    }
    let suite = &s[w_end + "SUITE=".len()..];
    if suite.is_empty() || suite.chars().any(py_is_space) {
        return None;
    }
    Some((&s[p..d_end], suite))
}

/// `(?:injections?|faults)` at `p`, IGNORECASE.
fn advertised_tail(s: &str, p: usize) -> Option<usize> {
    if let Some(e) = lit_ci(s, p, "injection") {
        return Some(lit_ci(s, e, "s").unwrap_or(e));
    }
    lit_ci(s, p, "faults")
}

/// `finditer` of `(\d+)[\s_]+(?:known-bad[\s_]+)?(?:injections?|faults)`,
/// IGNORECASE, over one line. Returns each group-1 value, normalized.
///
/// Greedy runs are again forced: `[\s_]+` is followed by `k`, `i` or `f`, none of
/// which is whitespace or underscore. The optional `known-bad` group is tried
/// first (greedy `?`) and falls back to being skipped.
pub fn scan_advertised(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut p = 0usize;
    while p < line.len() {
        if !line.is_char_boundary(p) {
            p += 1;
            continue;
        }
        let end = (|| {
            let d_end = digits_end(line, p);
            if d_end == p {
                return None;
            }
            let w_end = run_end(line, d_end, |c| py_is_space(c) || c == '_');
            if w_end == d_end {
                return None;
            }
            if let Some(k_end) = lit_ci(line, w_end, "known-bad") {
                let k2 = run_end(line, k_end, |c| py_is_space(c) || c == '_');
                if k2 > k_end {
                    if let Some(e) = advertised_tail(line, k2) {
                        return Some((d_end, e));
                    }
                }
            }
            advertised_tail(line, w_end).map(|e| (d_end, e))
        })();
        match end {
            Some((d_end, e)) => {
                out.push(norm_digits(&line[p..d_end]));
                p = e;
            }
            None => {
                p += line[p..].chars().next().map_or(1, char::len_utf8);
            }
        }
    }
    out
}

/// The word-number alternatives, in the original dict's insertion order — regex
/// alternation is leftmost-first, so the order is part of the pattern.
const WORD_NUM: &[(&str, u128)] = &[
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
];

/// `suites?\b` at `p`, with the greedy `s?` tried first and backtracked.
fn suites_tail(s: &str, p: usize) -> Option<usize> {
    let e = lit_ci(s, p, "suite")?;
    if let Some(e2) = lit_ci(s, e, "s") {
        if at_word_boundary(s, e2) {
            return Some(e2);
        }
    }
    if at_word_boundary(s, e) {
        return Some(e);
    }
    None
}

/// `finditer` of `\b(\d+|one|…|twelve)\s+(?:selftest\s+)?suites?\b`, IGNORECASE.
pub fn scan_suite_counts(line: &str) -> Vec<u128> {
    let mut out = Vec::new();
    let mut p = 0usize;
    while p < line.len() {
        if !line.is_char_boundary(p) {
            p += 1;
            continue;
        }
        let hit = (|| {
            if !at_word_boundary(line, p) {
                return None;
            }
            // alternation: \d+ first, then the words in order
            let mut alts: Vec<(usize, u128)> = Vec::new();
            let d_end = digits_end(line, p);
            if d_end > p {
                alts.push((
                    d_end,
                    norm_digits(&line[p..d_end]).parse().unwrap_or(u128::MAX),
                ));
            }
            for (w, v) in WORD_NUM {
                if let Some(e) = lit_ci(line, p, w) {
                    alts.push((e, *v));
                }
            }
            for (e1, value) in alts {
                let e2 = run_end(line, e1, py_is_space);
                if e2 == e1 {
                    continue;
                }
                if let Some(st) = lit_ci(line, e2, "selftest") {
                    let e4 = run_end(line, st, py_is_space);
                    if e4 > st {
                        if let Some(e) = suites_tail(line, e4) {
                            return Some((e, value));
                        }
                    }
                }
                if let Some(e) = suites_tail(line, e2) {
                    return Some((e, value));
                }
            }
            None
        })();
        match hit {
            Some((e, v)) => {
                out.push(v);
                p = e;
            }
            None => {
                p += line[p..].chars().next().map_or(1, char::len_utf8);
            }
        }
    }
    out
}

// ────────────────────────────── the gate ───────────────────────────────────

/// The whole gate as a pure function: bodies in, exact stdout and exit code out.
/// `None` for a body means "not a regular file", which is what `Path.is_file()`
/// reports for a missing path, a directory, and `/dev/null` alike.
pub fn render(
    log_display: &str,
    log_body: Option<&str>,
    readme_display: &str,
    readme_body: Option<&str>,
    require_raw: &str,
) -> (String, i32) {
    let required: Vec<&str> = require_raw
        .split(',')
        .map(py_strip)
        .filter(|s| !s.is_empty())
        .collect();
    if required.is_empty() {
        return (
            "FAIL\n  - no suites required (a gate over an empty registry is vacuous)\n".to_string(),
            1,
        );
    }

    let mut errors: Vec<String> = Vec::new();
    let mut counts: BTreeMap<&str, u128> = BTreeMap::new();

    match log_body {
        None => errors.push(format!("injection log missing: {log_display}")),
        Some(text) => {
            let lines: Vec<&str> = py_splitlines(text)
                .into_iter()
                .filter(|ln| !py_strip(ln).is_empty())
                .collect();
            if lines.is_empty() {
                errors.push(
                    "injection log is empty — zero suites self-reported (empty scan set is an ERROR, not a pass)"
                        .to_string(),
                );
            } else {
                for raw in lines {
                    let s = py_strip(raw);
                    match parse_receipt(s) {
                        None => {
                            errors.push(format!("unparseable receipt line: {}", py_repr(s)));
                        }
                        Some((digits, suite)) => {
                            let n: u128 = norm_digits(digits).parse().unwrap_or(u128::MAX);
                            if let Some(&prev) = counts.get(suite) {
                                if prev != n {
                                    errors.push(format!(
                                        "suite {suite} reported two different counts ({prev} then {n}) in one run"
                                    ));
                                }
                            }
                            counts.insert(suite, n);
                        }
                    }
                }
            }
        }
    }

    for suite in &required {
        match counts.get(suite) {
            None => errors.push(format!(
                "registered suite {} emitted no INJECTIONS= line — that is an ERROR, never a silent zero",
                py_repr(suite)
            )),
            Some(&c) if c == 0 => errors.push(format!(
                "suite {} self-reported {c} injections — a known-bad suite that asserts no RED is not a gate",
                py_repr(suite)
            )),
            Some(_) => {}
        }
    }
    // BTreeMap iteration order == CPython's `sorted(counts)`: both order by
    // Unicode scalar value (Rust compares UTF-8 bytes, which agrees).
    for suite in counts.keys() {
        if !required.contains(suite) {
            errors.push(format!(
                "suite {} self-reported but is not registered in REGISTERED_SUITES — register it (and update the advertised count) rather than letting the total drift",
                py_repr(suite)
            ));
        }
    }

    // Iterating `required`, not `counts`: a suite listed twice in --require is
    // summed twice. Reproduced from the original on purpose.
    let total: u128 = required
        .iter()
        .map(|s| counts.get(s).copied().unwrap_or(0))
        .sum();
    let total_norm = total.to_string();

    let mut advertised: Vec<(usize, String)> = Vec::new();
    match readme_body {
        None => errors.push(format!("README missing: {readme_display}")),
        Some(text) => {
            let mut suite_claims: Vec<(usize, u128)> = Vec::new();
            for (i, line) in py_splitlines(text).iter().enumerate() {
                let lineno = i + 1;
                for n in scan_advertised(line) {
                    advertised.push((lineno, n));
                }
                for n in scan_suite_counts(line) {
                    suite_claims.push((lineno, n));
                }
            }
            if advertised.is_empty() {
                errors.push(
                    "README advertises no known-bad injection count at all (nothing to check is an ERROR, not a pass)"
                        .to_string(),
                );
            }
            for (lineno, n) in &advertised {
                if *n != total_norm {
                    errors.push(format!(
                        "README.md:{lineno} advertises {n} known-bad injections; the suites self-reported {total}"
                    ));
                }
            }
            for (lineno, n) in &suite_claims {
                if *n != required.len() as u128 {
                    errors.push(format!(
                        "README.md:{lineno} advertises {n} selftest suites; {} are registered",
                        required.len()
                    ));
                }
            }
        }
    }

    let mut claims: Vec<String> = advertised.iter().map(|(_, n)| n.clone()).collect();
    claims.sort_by(cmp_norm);
    claims.dedup();

    let mut out = String::new();
    out.push_str(if errors.is_empty() {
        "PASS\n"
    } else {
        "FAIL\n"
    });
    out.push_str(&format!("  log={log_display}\n"));
    out.push_str(&format!("  registered_suites={}\n", required.len()));
    out.push_str(&format!("  measured_total={total}\n"));
    out.push_str(&format!("  readme_claims=[{}]\n", claims.join(", ")));
    for suite in &required {
        match counts.get(suite) {
            None => out.push_str(&format!("    {suite}: MISSING\n")),
            Some(c) => out.push_str(&format!("    {suite}: {c}\n")),
        }
    }

    if !errors.is_empty() {
        out.push_str("  failures:\n");
        for e in errors.iter().take(40) {
            out.push_str(&format!("    - {e}\n"));
        }
        if errors.len() > 40 {
            out.push_str(&format!("    ... +{} more\n", errors.len() - 40));
        }
        return (out, 1);
    }

    out.push_str(&format!(
        "  injection count GREEN (README and the suites both say {total})\n"
    ));
    (out, 0)
}

struct Args {
    log: String,
    readme: Option<String>,
    require: String,
}

/// `--flag value` and `--flag=value`, both forms. Anything else is USAGE — a
/// typo'd flag must never read as "the gate passed". Unlike argparse this does
/// NOT accept unique prefixes (`--lo`); see the module header on why the argparse
/// surface is not a byte-exact target.
fn parse_args(argv: &[String]) -> Result<Args, GateError> {
    let mut log: Option<String> = None;
    let mut readme: Option<String> = None;
    let mut require: Option<String> = None;
    let mut i = 0usize;
    while i < argv.len() {
        let a = &argv[i];
        let (key, inline) = match a.split_once('=') {
            Some((k, v)) => (k, Some(v.to_string())),
            None => (a.as_str(), None),
        };
        let slot = match key {
            "--log" => &mut log,
            "--readme" => &mut readme,
            "--require" => &mut require,
            _ => {
                return Err(GateError::usage(format!(
                    "unknown argument {a:?}; known: --log <path> --readme <path> --require <a,b,c>"
                )))
            }
        };
        let value = match inline {
            Some(v) => {
                i += 1;
                v
            }
            None => {
                let Some(v) = argv.get(i + 1) else {
                    return Err(GateError::usage(format!(
                        "argument {key}: expected one argument"
                    )));
                };
                i += 2;
                v.clone()
            }
        };
        *slot = Some(value);
    }
    let Some(log) = log else {
        return Err(GateError::usage(
            "the following arguments are required: --log",
        ));
    };
    Ok(Args {
        log,
        readme,
        require: require.unwrap_or_else(|| REGISTERED_SUITES.join(",")),
    })
}

/// `Path.is_file()` then `read_text(encoding="utf-8")`, with the decode failure
/// surfaced as an ERROR. CPython raises `UnicodeDecodeError` here and dies with a
/// traceback carrying absolute paths and line numbers, which is not a byte-exact
/// target; refusing to evaluate is the honest substitute. It is never a pass.
fn read_if_file(p: &Path) -> Result<Option<String>, GateError> {
    if !p.is_file() {
        return Ok(None);
    }
    match std::fs::read_to_string(p) {
        Ok(s) => Ok(Some(s)),
        Err(e) => Err(GateError::error(format!(
            "{} exists but could not be read as UTF-8: {e}",
            p.display()
        ))),
    }
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    let args = parse_args(&ctx.args)?;

    let log_display = py_path_str(&args.log);
    let readme_display = match &args.readme {
        Some(r) => py_path_str(r),
        // Python: ENGINE.parent / "README.md", where ENGINE is the engine root.
        None => {
            let base: PathBuf = ctx
                .root
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| ctx.root.clone());
            py_path_str(&base.join("README.md").to_string_lossy())
        }
    };

    let log_body = read_if_file(Path::new(&log_display))?;
    let readme_body = read_if_file(Path::new(&readme_display))?;

    let (text, code) = render(
        &log_display,
        log_body.as_deref(),
        &readme_display,
        readme_body.as_deref(),
        &args.require,
    );

    print!("{text}");
    std::io::stdout().flush().ok();

    if code == 0 {
        Ok(())
    } else {
        // Exit 1, not GateError's 2 — see the module header. Everything the
        // caller reads has already been written and flushed above.
        std::process::exit(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R7: &str = "# Specimen readme\n\n[![known-bad: 7 injections](https://img.shields.io/badge/known--bad-7_injections_all_RED-success.svg)](#x)\n\n| **Gate** | 2 selftest suites; 7 known-bad injections that must all go RED |\n\nTwo selftest suites inject **7 known-bad faults** and assert the build fails.\n\n| **L4 — gates proven to trip** | ok | 2 suites, 7 injections, anti-vacuous |\n";
    const GOOD_LOG: &str = "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n";
    const REQ: &str = "spec_alpha,spec_beta";

    #[test]
    fn baseline_is_green() {
        let (out, code) = render("L", Some(GOOD_LOG), "R", Some(R7), REQ);
        assert_eq!(code, 0, "{out}");
        assert!(out.starts_with("PASS\n"), "{out}");
        assert!(out.contains("  readme_claims=[7]\n"), "{out}");
        assert!(
            out.ends_with("  injection count GREEN (README and the suites both say 7)\n"),
            "{out}"
        );
    }

    #[test]
    fn a_missing_log_is_red_never_a_silent_zero() {
        let (out, code) = render("L", None, "R", Some(R7), REQ);
        assert_eq!(code, 1);
        assert!(out.contains("    - injection log missing: L\n"), "{out}");
    }

    #[test]
    fn an_empty_log_is_red_not_a_pass() {
        for body in ["", "\n\n   \n"] {
            let (out, code) = render("L", Some(body), "R", Some(R7), REQ);
            assert_eq!(code, 1, "{out}");
            assert!(out.contains("injection log is empty"), "{out}");
        }
    }

    #[test]
    fn a_suite_reporting_zero_is_red() {
        let log = "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=0 SUITE=spec_beta\n";
        let (out, code) = render("L", Some(log), "R", Some(R7), REQ);
        assert_eq!(code, 1);
        assert!(out.contains("is not a gate"), "{out}");
    }

    #[test]
    fn drift_is_caught_in_both_directions() {
        let under = "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=1 SUITE=spec_beta\n";
        let over = "INJECTIONS=9 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n";
        let (o1, c1) = render("L", Some(under), "R", Some(R7), REQ);
        assert_eq!(c1, 1);
        assert!(
            o1.contains("advertises 7 known-bad injections; the suites self-reported 4"),
            "{o1}"
        );
        let (o2, c2) = render("L", Some(over), "R", Some(R7), REQ);
        assert_eq!(c2, 1);
        assert!(
            o2.contains("advertises 7 known-bad injections; the suites self-reported 13"),
            "{o2}"
        );
    }

    #[test]
    fn an_empty_require_list_is_red() {
        let (out, code) = render("L", Some(GOOD_LOG), "R", Some(R7), ",, ,");
        assert_eq!(code, 1);
        assert_eq!(
            out,
            "FAIL\n  - no suites required (a gate over an empty registry is vacuous)\n"
        );
    }

    #[test]
    fn duplicate_require_double_counts_as_the_original_does() {
        // Reproduced quirk, not endorsed: one suite reporting 3 totals 6.
        let (out, _) = render("L", Some(GOOD_LOG), "R", Some(R7), "spec_alpha,spec_alpha");
        assert!(out.contains("  measured_total=6\n"), "{out}");
    }

    #[test]
    fn python_primitives_match_cpython() {
        assert_eq!(py_path_str(""), ".");
        assert_eq!(py_path_str(".//good.log"), "good.log");
        assert_eq!(py_path_str("good.log/"), "good.log");
        assert_eq!(py_path_str("//a"), "//a");
        assert_eq!(py_path_str("///a"), "/a");
        assert_eq!(py_repr("it's a 'quoted' line"), "\"it's a 'quoted' line\"");
        assert_eq!(
            py_repr("line with \"double\" and 'single'"),
            "'line with \"double\" and \\'single\\''"
        );
        assert_eq!(py_repr("TABBED\\back"), "'TABBED\\\\back'");
        assert_eq!(py_splitlines("a\rb\nc"), vec!["a", "b", "c"]);
        assert_eq!(py_splitlines("a\r\nb"), vec!["a", "b"]);
        assert_eq!(py_strip("\u{1c}x\u{1f}"), "x");
        assert_eq!(norm_digits("007"), "7");
        assert_eq!(norm_digits("000"), "0");
    }

    #[test]
    fn regex_emulation_matches_the_patterns() {
        assert_eq!(parse_receipt("INJECTIONS=3 SUITE=a"), Some(("3", "a")));
        assert_eq!(parse_receipt("INJECTIONS=3\tSUITE=a"), Some(("3", "a")));
        assert_eq!(parse_receipt("INJECTIONS=x SUITE=a"), None);
        assert_eq!(parse_receipt("INJECTIONS=3 SUITE=a b"), None);
        assert_eq!(parse_receipt("INJECTIONS=3 SUITE="), None);
        assert_eq!(parse_receipt("garbage line"), None);

        // The badge line yields TWO matches, as the original does.
        assert_eq!(
            scan_advertised("[![known-bad: 7 injections](https://x/badge/known--bad-7_injections_all_RED-success.svg)]"),
            vec!["7".to_string(), "7".to_string()]
        );
        assert_eq!(scan_advertised("7 known-bad faults"), vec!["7".to_string()]);
        assert_eq!(scan_advertised("07 injections"), vec!["7".to_string()]);
        assert_eq!(
            scan_advertised("thirty-six injections"),
            Vec::<String>::new()
        );

        assert_eq!(scan_suite_counts("9 selftest suites;"), vec![9]);
        assert_eq!(scan_suite_counts("Nine selftest suites inject"), vec![9]);
        assert_eq!(scan_suite_counts("2 suites, 7 injections"), vec![2]);
        assert_eq!(
            scan_suite_counts("suitesx and 3 suitesy"),
            Vec::<u128>::new()
        );
    }

    #[test]
    fn the_shipped_registry_is_not_empty() {
        assert!(
            !REGISTERED_SUITES.is_empty(),
            "a drift guard over zero suites is vacuous"
        );
        let mut seen = std::collections::BTreeSet::new();
        for s in REGISTERED_SUITES {
            assert!(
                seen.insert(*s),
                "duplicate registered suite {s} would double-count"
            );
        }
    }

    #[test]
    fn bad_argv_is_usage_not_a_pass() {
        assert!(parse_args(&[]).is_err());
        assert!(parse_args(&["--bogus".into(), "x".into()]).is_err());
        assert!(parse_args(&["--log".into()]).is_err());
        let a = parse_args(&["--log=x".into()]).unwrap();
        assert_eq!(a.log, "x");
        assert_eq!(a.require, REGISTERED_SUITES.join(","));
        assert!(a.readme.is_none());
    }
}
