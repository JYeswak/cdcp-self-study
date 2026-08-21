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
//! ## What the advertised number counts (decided 2026-08-14, bd-wf2;
//! permanent 2026-08-15, bd-n7uk)
//!
//! Exactly one population: the receipts emitted by the registered SHELL selftest
//! suites (`scripts/selftest_*.sh`) during a real `check.sh` run. The Rust
//! known-bad legs in this crate's `#[cfg(test)]` modules are deliberately NOT in
//! the total, and the reason is stated because an exclusion without a reason is a
//! schema error: they emit no receipt, so there is nothing for `check.sh` to tee
//! into the log and nothing here to sum, and their number would have to be
//! hand-typed somewhere — which is the defect this gate exists to remove.
//!
//! The honest consequence: the advertised number is a FLOOR on the repo's
//! known-bad population, not its total, so README names the population it counts
//! ("shell selftest suites"). An advertisement line that says "known-bad" (or
//! the shields.io spelling "known--bad") without "shell" or "selftest" on that
//! line is RED — the badge is two of the five sites and is included. Folding
//! the Rust legs in later is a mechanism change, not a number change — they
//! would first have to emit receipts that `check.sh` aggregates and be
//! registered in [`REGISTERED_SUITES`].
//!
//! The number is REGENERATED, never hand-maintained: `--write-readme` rewrites
//! every advertisement site **and each per-suite `n` cell** from the receipts
//! that were actually collected, and refuses to write when those receipts are
//! unsound, so a bogus log cannot launder a wrong number into README.
//!
//! The reachable caller is `check.sh` with `CDCP_INJECTION_COUNT_WRITE_README=1`
//! [bd-injection-count-regen-unreachable-lu45]. Without that flag the same
//! invocation is still a drift check (RED on disagreement). The flag cannot
//! launder an unsound total: `--write-readme` refuses to write when the
//! receipts are not sound.
//!
//! ## Partial coverage is an error too (bd-wf2)
//!
//! "No site parses" was already caught. The subtler shape is ONE site quietly
//! falling out of the scanner while the others still parse — coverage drops and
//! the report is indistinguishable from full coverage. Two defences: counts
//! spelled in English words parse (zero..ninety-nine), and
//! [`MIN_ADVERTISEMENT_SITES`] is a floor on how many sites must parse at all, so
//! a site that becomes unreadable in any other way still trips the gate.
//!
//! ## What it cannot decide
//!
//! * Whether a receipt is **honest**. The counter is incremented by the suite's
//!   own assert helper. A suite that increments without observing a real RED is
//!   invisible here — that is what the suites' own known-bad cases are for.
//! * Whether the log came from **this** run. It reads whatever file it is handed;
//!   freshness is `check.sh`'s job (it mktemps a new log per invocation).
//! * Whether README's **prose** is accurate about anything other than the
//!   numbers it scans (injection count, selftest-suite count, per-suite `n`)
//!   and the shell/selftest qualifier on a known-bad advertisement line.
//! * Whether a count spelled in a form outside the word vocabulary ("three
//!   dozen", anything above ninety-nine in words) means what it says. Such a site
//!   is not read as a number at all; the site floor is what keeps that fail-closed
//!   rather than silent.
//! * Whether the registry itself is right. `--require` names the suites that must
//!   report; a suite nobody registered and nobody runs is outside its reach.
//!
//! # Retirement boundary
//!
//! The former Python oracle and differential harness have been retired. This
//! Rust gate retains the established stdout and exit semantics, and
//! `scripts/selftest_injection_count.sh` owns the known-bad drift cases. The
//! three original port corrections remain covered by the Rust tests: duplicate
//! `--require` entries are usage errors, findings name the file actually
//! scanned, and word-spelled counts parse.
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
use std::collections::{BTreeMap, BTreeSet};
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
///   the Rust `#[cfg(test)]` known-bad legs — see the module header. They emit no
///     receipt, so they cannot be summed; registering them without a receipt
///     mechanism would put a hand-typed number back in the badge.
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
    "wasm-freshness",
];

/// How many advertisement sites must parse before the comparison is worth
/// anything. The shipped README advertises the count at five sites (the badge
/// markup contributes two), and the selftest's specimen README also writes five.
///
/// A FLOOR, not an equality: adding an advertisement is free, removing or
/// obscuring one is a deliberate decision that has to edit this constant. Without
/// it, a README where one site stopped parsing reports exactly like a README where
/// all of them still do. Mirrors `MIN_ADVERTISEMENT_SITES` in the Python original.
pub const MIN_ADVERTISEMENT_SITES: usize = 5;

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

/// CPython `str.splitlines(keepends=True)`: the same boundaries as
/// [`py_splitlines`], with each terminator kept on the line it ends. Used only by
/// `--write-readme`, where the file has to come back out byte-for-byte apart from
/// the count tokens themselves — reassembling from `lines()` would rewrite every
/// `\r\n` in the file as `\n`.
fn py_splitlines_keepends(s: &str) -> Vec<&str> {
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
        let mut end = i + c.len_utf8();
        if c == '\r' && it.peek().map(|&(_, n)| n) == Some('\n') {
            it.next();
            end += 1;
        }
        out.push(&s[start..end]);
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
// CPython's `\d` and `int()` accept Unicode Nd digits. Keep the same conversion
// for receipts, advertised counts, and suite rows; otherwise a Python GREEN can
// become a Rust RED before the gate compares anything.

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `\b` at byte offset `p` (which must be a char boundary).
fn at_word_boundary(s: &str, p: usize) -> bool {
    let before = s[..p].chars().next_back().is_some_and(is_word);
    let after = s[p..].chars().next().is_some_and(is_word);
    before != after
}

/// Return the decimal value of one Unicode Nd character.
fn unicode_decimal_digit(c: char) -> Option<u8> {
    const BLOCKS: &[u32] = &[
        0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66,
        0x0CE6, 0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946,
        0x19D0, 0x1A80, 0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0,
        0xA9F0, 0xAA50, 0xABF0, 0xFF10, 0x104A0, 0x10D30, 0x11066, 0x110F0, 0x11136, 0x111D0,
        0x112F0, 0x11450, 0x114D0, 0x11650, 0x116C0, 0x11730, 0x118E0, 0x11950, 0x11C50, 0x11D50,
        0x11DA0, 0x16A60, 0x16AC0, 0x16B50, 0x1E140, 0x1E2F0, 0x1E950, 0x1FBF0, 0x1D7CE, 0x1D7D8,
        0x1D7E2, 0x1D7EC, 0x1D7F6,
    ];
    let cp = c as u32;
    BLOCKS
        .iter()
        .find_map(|start| (cp >= *start && cp < *start + 10).then(|| (cp - *start) as u8))
}

/// Maximal run of Unicode decimal digits from `p` (Python's `\d`).
fn digits_end(s: &str, p: usize) -> usize {
    s[p..]
        .char_indices()
        .take_while(|(_, c)| unicode_decimal_digit(*c).is_some())
        .last()
        .map_or(p, |(i, c)| p + i + c.len_utf8())
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

/// Normalize decimal digits the way `int()` then `str()` would: `"٠٠٧"` -> `"7"`.
/// Working in normalized decimal strings rather than a fixed-width integer keeps
/// an absurdly large advertised count printing exactly as CPython would.
fn norm_digits(d: &str) -> String {
    let normalized: String = d
        .chars()
        .map(|c| char::from(b'0' + unicode_decimal_digit(c).expect("digit run")))
        .collect();
    let t = normalized.trim_start_matches('0');
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

/// English cardinals zero..ninety-nine, hyphen and space compounds both, in the
/// alternation order the Python builds: longest first, ties broken
/// lexicographically. Order is part of the pattern — `re` alternation is
/// leftmost-first, not longest-match, so "eighteen" must be offered before
/// "eight" and "twenty-one" before "twenty".
///
/// Bounded on purpose. A count above ninety-nine spelled in words is not
/// recognised; it drops the site out of `advertised`, which trips the
/// [`MIN_ADVERTISEMENT_SITES`] floor rather than passing silently.
pub fn cardinals() -> &'static [(String, u128)] {
    static TABLE: std::sync::OnceLock<Vec<(String, u128)>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        const ONES: [&str; 20] = [
            "zero",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];
        // The Python's TENS is indexed 0..9 with two unused leading holes; this
        // drops them and carries the +2 offset, which builds the same 172 words.
        const TENS: [&str; 8] = [
            "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];
        let mut v: Vec<(String, u128)> = Vec::with_capacity(172);
        for (i, w) in ONES.iter().enumerate() {
            v.push(((*w).to_string(), i as u128));
        }
        for (ti, tw) in TENS.iter().enumerate() {
            let t = ti + 2;
            v.push(((*tw).to_string(), (t * 10) as u128));
            for (u, one) in ONES.iter().enumerate().take(10).skip(1) {
                for sep in ['-', ' '] {
                    v.push((format!("{tw}{sep}{one}"), (t * 10 + u) as u128));
                }
            }
        }
        // CPython: sorted(WORD_NUM, key=lambda w: (-len(w), w)). Rust's `str` Ord
        // is byte-wise, which agrees with CPython's code-point order on ASCII.
        v.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
        v
    })
}

/// `finditer` of
/// `(\d+|\b(?:<cardinal>))[\s_]+(?:known-bad[\s_]+)?(?:injections?|faults)`,
/// IGNORECASE, over one line. Returns `(start, end, value)` for group 1 — the
/// span so `--write-readme` can rewrite exactly the count token, the value
/// normalized to decimal.
///
/// Greedy runs are again forced: `[\s_]+` is followed by `k`, `i` or `f`, none of
/// which is whitespace or underscore. The optional `known-bad` group is tried
/// first (greedy `?`) and falls back to being skipped. Alternatives are tried in
/// pattern order and the first that completes the whole match wins, which is what
/// leftmost-first alternation with backtracking does.
///
/// The `\b` guards the WORD branch only, exactly as in the Python: a digit run
/// after a dot ("v1.7 injections") is still a number, but "eight" inside
/// "freighter" is not.
pub fn scan_advertised_spans(line: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut p = 0usize;
    while p < line.len() {
        if !line.is_char_boundary(p) {
            p += 1;
            continue;
        }
        let hit = (|| {
            let mut alts: Vec<(usize, String)> = Vec::new();
            let d_end = digits_end(line, p);
            if d_end > p {
                alts.push((d_end, norm_digits(&line[p..d_end])));
            }
            if at_word_boundary(line, p) {
                for (w, v) in cardinals() {
                    if let Some(e) = lit_ci(line, p, w) {
                        alts.push((e, v.to_string()));
                    }
                }
            }
            for (c_end, value) in alts {
                let w_end = run_end(line, c_end, |c| py_is_space(c) || c == '_');
                if w_end == c_end {
                    continue;
                }
                if let Some(k_end) = lit_ci(line, w_end, "known-bad") {
                    let k2 = run_end(line, k_end, |c| py_is_space(c) || c == '_');
                    if k2 > k_end {
                        if let Some(e) = advertised_tail(line, k2) {
                            return Some((c_end, e, value));
                        }
                    }
                }
                if let Some(e) = advertised_tail(line, w_end) {
                    return Some((c_end, e, value));
                }
            }
            None
        })();
        match hit {
            Some((c_end, e, value)) => {
                out.push((p, c_end, value));
                p = e;
            }
            None => {
                p += line[p..].chars().next().map_or(1, char::len_utf8);
            }
        }
    }
    out
}

/// The advertised counts on one line, without their spans.
pub fn scan_advertised(line: &str) -> Vec<String> {
    scan_advertised_spans(line)
        .into_iter()
        .map(|(_, _, v)| v)
        .collect()
}

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

/// `finditer` of `\b(\d+|<cardinal>)\s+(?:selftest\s+)?suites?\b`, IGNORECASE.
/// Here the `\b` precedes the whole alternation, digits included.
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
            for (w, v) in cardinals() {
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

/// Parse scope (so scanners do not eat their own docs): a suite row is a
/// markdown table row whose first cell is a backticked name matching
/// `selftest_[a-z0-9_]+` and whose second cell is an integer. Mentions of the
/// table in this crate's comments, in `selftest_injection_count.sh` headers, or
/// in CHARTER are NOT rows — only `--readme` is scanned. Zero parsed rows is
/// an ERROR when `--require` names any `selftest_*` suite.
///
/// Returns `(suite, digits, digit_start, digit_end)` so `--write-readme` can
/// rewrite just the `n` cell.
pub fn parse_suite_row(line: &str) -> Option<(&str, &str, usize, usize)> {
    let body = line
        .strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .or_else(|| line.strip_suffix('\r'))
        .unwrap_or(line);
    let mut i = run_end(body, 0, py_is_space);
    if body.as_bytes().get(i) != Some(&b'|') {
        return None;
    }
    i += 1;
    i = run_end(body, i, py_is_space);
    if body.as_bytes().get(i) != Some(&b'`') {
        return None;
    }
    i += 1;
    let ns = i;
    let name_end = if body[ns..].starts_with("wasm-freshness") {
        ns + "wasm-freshness".len()
    } else if body[ns..].starts_with("selftest_") {
        let after = ns + "selftest_".len();
        let end = run_end(body, after, |c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
        });
        if end == after {
            return None;
        }
        end
    } else {
        return None;
    };
    let name = &body[ns..name_end];
    i = name_end;
    if body.as_bytes().get(i) != Some(&b'`') {
        return None;
    }
    i += 1;
    i = run_end(body, i, py_is_space);
    if body.as_bytes().get(i) != Some(&b'|') {
        return None;
    }
    i += 1;
    i = run_end(body, i, py_is_space);
    let ds = i;
    let de = digits_end(body, ds);
    if de == ds {
        return None;
    }
    i = run_end(body, de, py_is_space);
    if body.as_bytes().get(i) != Some(&b'|') {
        return None;
    }
    Some((name, &body[ds..de], ds, de))
}

fn is_selftest_suite(name: &str) -> bool {
    let rest = match name.strip_prefix("selftest_") {
        Some(r) if !r.is_empty() => r,
        _ => return false,
    };
    rest.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn rewrite_suite_cells(text: &str, counts: &BTreeMap<&str, u128>) -> String {
    if counts.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for line in py_splitlines_keepends(text) {
        let body = line
            .strip_suffix("\r\n")
            .or_else(|| line.strip_suffix('\n'))
            .or_else(|| line.strip_suffix('\r'))
            .unwrap_or(line);
        let eol = &line[body.len()..];
        if let Some((suite, digits, ds, de)) = parse_suite_row(body) {
            if let Some(&n) = counts.get(suite) {
                let nn = n.to_string();
                if nn != digits {
                    out.push_str(&body[..ds]);
                    out.push_str(&nn);
                    out.push_str(&body[de..]);
                    out.push_str(eol);
                    continue;
                }
            }
        }
        out.push_str(line);
    }
    out
}

// ────────────────────────────── the gate ───────────────────────────────────

/// Rewrite every advertised injection count in `text` to `total`.
///
/// Returns the new text and the number of sites rewritten. Line terminators are
/// preserved exactly, and only the count token itself is replaced, so surrounding
/// markup and prose are untouched. A word-spelled site is normalised to digits —
/// regeneration produces the checkable form.
///
/// Suite counts are NOT rewritten: the roster changes only when a suite is added
/// or removed, which is already a deliberate edit to [`REGISTERED_SUITES`], and
/// rewriting them would rewrite prose ("Nine selftest suites") that no caller
/// asked this gate to author.
pub fn regenerate(text: &str, total: &str) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut rewritten = 0usize;
    for line in py_splitlines_keepends(text) {
        let spans = scan_advertised_spans(line);
        if spans.is_empty() {
            out.push_str(line);
            continue;
        }
        let mut last = 0usize;
        for (s, e, _) in spans {
            out.push_str(&line[last..s]);
            out.push_str(total);
            last = e;
            rewritten += 1;
        }
        out.push_str(&line[last..]);
    }
    (out, rewritten)
}

/// The whole gate as a pure function: bodies in, exact stdout and exit code out,
/// plus the new README body when `--write-readme` asked for one (the caller does
/// the writing, so this stays testable without a filesystem).
///
/// `None` for a body means "not a regular file", which is what `Path.is_file()`
/// reports for a missing path, a directory, and `/dev/null` alike.
pub fn render(
    log_display: &str,
    log_body: Option<&str>,
    readme_display: &str,
    readme_body: Option<&str>,
    require_raw: &str,
    write_readme: bool,
) -> (String, i32, Option<String>) {
    let required: Vec<&str> = require_raw
        .split(',')
        .map(py_strip)
        .filter(|s| !s.is_empty())
        .collect();
    if required.is_empty() {
        return (
            "FAIL\n  - no suites required (a gate over an empty registry is vacuous)\n".to_string(),
            1,
            None,
        );
    }

    // A suite named twice would be summed twice, inflating measured_total — the
    // one direction that turns real drift GREEN. Silently de-duplicating would
    // accept a caller that does not know its own roster, so this is an ERROR.
    // BTreeSet ordering == CPython's `sorted({...})` on ASCII suite names.
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut duplicated: BTreeSet<&str> = BTreeSet::new();
    for s in required.iter().copied() {
        if !seen.insert(s) {
            duplicated.insert(s);
        }
    }
    if !duplicated.is_empty() {
        let listed: Vec<String> = duplicated.iter().map(|s| py_repr(s)).collect();
        return (
            format!(
                "FAIL\n  - --require names {} more than once; a repeated suite is summed twice, which inflates measured_total and is the direction that turns real drift GREEN\n",
                listed.join(", ")
            ),
            1,
            None,
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

    // `required` is duplicate-free by the check above, so this is a plain sum.
    let total: u128 = required
        .iter()
        .map(|s| counts.get(s).copied().unwrap_or(0))
        .sum();
    let total_norm = total.to_string();

    // Regeneration runs BEFORE the comparison, and only when the receipts
    // themselves are sound. A missing suite, a zero suite, an unparseable line or
    // an unregistered suite means the total is not trustworthy, and writing an
    // untrustworthy number into README would launder it into a certificate.
    let receipts_sound = errors.is_empty();
    let mut regen_note: Option<String> = None;
    let mut new_readme: Option<String> = None;
    let mut readme_text: Option<String> = readme_body.map(str::to_string);

    if write_readme {
        match readme_body {
            _ if !receipts_sound => {
                regen_note = Some(
                    "regeneration SKIPPED: the receipts are not sound, so the total is not a number worth writing"
                        .to_string(),
                );
            }
            None => {
                regen_note = Some("regeneration SKIPPED: README is not readable".to_string());
            }
            Some(before) => {
                let (after, sites) = regenerate(before, &total_norm);
                let after = rewrite_suite_cells(&after, &counts);
                if after == before {
                    regen_note = Some(if sites == 0 {
                        format!(
                            "regeneration wrote nothing: {readme_display} advertises no parseable count to rewrite"
                        )
                    } else {
                        format!(
                            "regenerated {readme_display}: {sites} site(s) already advertise {total}"
                        )
                    });
                } else {
                    regen_note = Some(if sites == 0 {
                        format!("regenerated {readme_display}: per-suite cells now match receipts")
                    } else {
                        format!(
                            "regenerated {readme_display}: {sites} site(s) now advertise {total}"
                        )
                    });
                    new_readme = Some(after.clone());
                    readme_text = Some(after);
                }
            }
        }
    }

    let mut advertised: Vec<(usize, String)> = Vec::new();
    match readme_text.as_deref() {
        None => errors.push(format!("README missing: {readme_display}")),
        Some(text) => {
            let mut suite_claims: Vec<(usize, u128)> = Vec::new();
            let mut col_rows: Vec<(usize, &str, String)> = Vec::new();
            let mut col_seen: BTreeMap<&str, usize> = BTreeMap::new();
            for (i, line) in py_splitlines(text).iter().enumerate() {
                let lineno = i + 1;
                let adv_hits = scan_advertised(line);
                if !adv_hits.is_empty() {
                    let low = line.to_ascii_lowercase();
                    if (low.contains("known-bad") || low.contains("known--bad"))
                        && !(low.contains("shell") || low.contains("selftest"))
                    {
                        errors.push(format!(
                            "{readme_display}:{lineno} advertises known-bad injections without a shell/selftest qualifier — the counted population is shell selftest suites, not every known-bad in the repo"
                        ));
                    }
                }
                for n in adv_hits {
                    advertised.push((lineno, n));
                }
                for n in scan_suite_counts(line) {
                    suite_claims.push((lineno, n));
                }
                if let Some((suite, digits, _, _)) = parse_suite_row(line) {
                    if col_seen.contains_key(suite) {
                        errors.push(format!(
                            "{readme_display}:{lineno} suite {suite} appears more than once in the per-suite table"
                        ));
                    } else {
                        col_seen.insert(suite, lineno);
                        col_rows.push((lineno, suite, norm_digits(digits)));
                    }
                }
            }
            if advertised.is_empty() {
                errors.push(
                    "README advertises no known-bad injection count at all (nothing to check is an ERROR, not a pass)"
                        .to_string(),
                );
            } else if advertised.len() < MIN_ADVERTISEMENT_SITES {
                errors.push(format!(
                    "only {} advertisement site(s) parsed in {readme_display}; at least {MIN_ADVERTISEMENT_SITES} are expected — a site that stopped parsing loses coverage while reporting exactly like full coverage",
                    advertised.len()
                ));
            }
            // Findings name the file that was actually scanned. Hardcoding
            // "README.md" sent the next reader to an innocent file whenever
            // --readme pointed elsewhere.
            for (lineno, n) in &advertised {
                if *n != total_norm {
                    errors.push(format!(
                        "{readme_display}:{lineno} advertises {n} known-bad injections; the suites self-reported {total}"
                    ));
                }
            }
            for (lineno, n) in &suite_claims {
                if *n != required.len() as u128 {
                    errors.push(format!(
                        "{readme_display}:{lineno} advertises {n} selftest suites; {} are registered",
                        required.len()
                    ));
                }
            }
            // Per-suite n column. Applied when --require names any selftest_*
            // suite (the live roster) or when the file already has such a row.
            // Specimens that use synthetic names (spec_alpha) and carry no
            // table are unchanged — their contract is still the total.
            let expect_col =
                required.iter().copied().any(is_selftest_suite) || !col_rows.is_empty();
            if expect_col {
                if col_rows.is_empty() {
                    errors.push(
                        "README per-suite injection table parsed to zero suite rows (empty scan set is an ERROR, not a pass)"
                            .to_string(),
                    );
                } else {
                    for (lineno, suite, n) in &col_rows {
                        if !required.contains(suite) {
                            errors.push(format!(
                                "{readme_display}:{lineno} table row {} is not in REGISTERED_SUITES",
                                py_repr(suite)
                            ));
                            continue;
                        }
                        if let Some(&obs) = counts.get(suite) {
                            if obs.to_string() != *n {
                                errors.push(format!(
                                    "{readme_display}:{lineno} suite {suite} advertises {n} injections; the suite self-reported {obs}"
                                ));
                            }
                        }
                    }
                    for suite in &required {
                        if !col_seen.contains_key(suite) {
                            errors.push(format!(
                                "registered suite {} has no per-suite table row — that is an ERROR, never a silent skip",
                                py_repr(suite)
                            ));
                        }
                    }
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
    if let Some(note) = &regen_note {
        out.push_str(&format!("  {note}\n"));
    }

    if !errors.is_empty() {
        out.push_str("  failures:\n");
        for e in errors.iter().take(40) {
            out.push_str(&format!("    - {e}\n"));
        }
        if errors.len() > 40 {
            out.push_str(&format!("    ... +{} more\n", errors.len() - 40));
        }
        return (out, 1, new_readme);
    }

    out.push_str(&format!(
        "  injection count GREEN (README and the suites both say {total})\n"
    ));
    (out, 0, new_readme)
}

struct Args {
    log: String,
    readme: Option<String>,
    require: String,
    write_readme: bool,
}

/// `--flag value` and `--flag=value`, both forms. Anything else is USAGE — a
/// typo'd flag must never read as "the gate passed". Unlike argparse this does
/// NOT accept unique prefixes (`--lo`); see the module header on why the argparse
/// surface is not a byte-exact target.
fn parse_args(argv: &[String]) -> Result<Args, GateError> {
    let mut log: Option<String> = None;
    let mut readme: Option<String> = None;
    let mut require: Option<String> = None;
    let mut write_readme = false;
    let mut i = 0usize;
    while i < argv.len() {
        let a = &argv[i];
        let (key, inline) = match a.split_once('=') {
            Some((k, v)) => (k, Some(v.to_string())),
            None => (a.as_str(), None),
        };
        // The one store_true flag: it takes no value, and argparse rejects
        // `--write-readme=x` too ("ignored explicit argument").
        if key == "--write-readme" {
            if inline.is_some() {
                return Err(GateError::usage(format!(
                    "argument --write-readme: ignored explicit argument in {a:?}"
                )));
            }
            write_readme = true;
            i += 1;
            continue;
        }
        let slot = match key {
            "--log" => &mut log,
            "--readme" => &mut readme,
            "--require" => &mut require,
            _ => {
                return Err(GateError::usage(format!(
                    "unknown argument {a:?}; known: --log <path> --readme <path> --require <a,b,c> --write-readme"
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
        write_readme,
    })
}

/// `Path.is_file()` then `read_text(encoding="utf-8")`, with the decode failure
/// surfaced as an ERROR. CPython raises `UnicodeDecodeError` here and dies with a
/// traceback carrying absolute paths and line numbers, which is not a byte-exact
/// target; refusing to evaluate is the honest substitute. It is never a pass.
fn read_if_file(p: &Path) -> Result<Option<String>, GateError> {
    // ABSENT-OK: this helper returns None; the caller records `missing` on
    // that None. Absence here is not the verdict.
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

    let (text, code, new_readme) = render(
        &log_display,
        log_body.as_deref(),
        &readme_display,
        readme_body.as_deref(),
        &args.require,
        args.write_readme,
    );

    // Write before reporting, so a failed write can never be reported as a PASS.
    // CPython would raise here and die with a traceback carrying absolute paths,
    // which is not a byte-exact target; refusing to report is the substitute.
    if let Some(body) = new_readme {
        std::fs::write(Path::new(&readme_display), body).map_err(|e| {
            GateError::error(format!(
                "--write-readme: could not rewrite {readme_display}: {e}"
            ))
        })?;
    }

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

    const R7: &str = "# Specimen readme\n\n[![known-bad (shell selftest suites): 7 injections](https://img.shields.io/badge/known--bad-7_injections_all_RED-success.svg)](#x)\n\n| **Gate** | 2 selftest suites; 7 known-bad injections that must all go RED |\n\nTwo selftest suites inject **7 known-bad faults** and assert the build fails.\n\n| **L4 — gates proven to trip** | ok | 2 suites, 7 injections, anti-vacuous |\n";
    const GOOD_LOG: &str = "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n";
    const REQ: &str = "spec_alpha,spec_beta";

    /// `render` without the write leg, which is what almost every case wants.
    fn check(
        log_display: &str,
        log_body: Option<&str>,
        readme_display: &str,
        readme_body: Option<&str>,
        require_raw: &str,
    ) -> (String, i32) {
        let (out, code, written) = render(
            log_display,
            log_body,
            readme_display,
            readme_body,
            require_raw,
            false,
        );
        assert!(written.is_none(), "no --write-readme, no rewrite");
        (out, code)
    }

    #[test]
    fn baseline_is_green() {
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(R7), REQ);
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
        let (out, code) = check("L", None, "R", Some(R7), REQ);
        assert_eq!(code, 1);
        assert!(out.contains("    - injection log missing: L\n"), "{out}");
    }

    #[test]
    fn an_empty_log_is_red_not_a_pass() {
        for body in ["", "\n\n   \n"] {
            let (out, code) = check("L", Some(body), "R", Some(R7), REQ);
            assert_eq!(code, 1, "{out}");
            assert!(out.contains("injection log is empty"), "{out}");
        }
    }

    #[test]
    fn a_suite_reporting_zero_is_red() {
        let log = "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=0 SUITE=spec_beta\n";
        let (out, code) = check("L", Some(log), "R", Some(R7), REQ);
        assert_eq!(code, 1);
        assert!(out.contains("is not a gate"), "{out}");
    }

    #[test]
    fn drift_is_caught_in_both_directions() {
        let under = "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=1 SUITE=spec_beta\n";
        let over = "INJECTIONS=9 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n";
        let (o1, c1) = check("L", Some(under), "R", Some(R7), REQ);
        assert_eq!(c1, 1);
        assert!(
            o1.contains("advertises 7 known-bad injections; the suites self-reported 4"),
            "{o1}"
        );
        let (o2, c2) = check("L", Some(over), "R", Some(R7), REQ);
        assert_eq!(c2, 1);
        assert!(
            o2.contains("advertises 7 known-bad injections; the suites self-reported 13"),
            "{o2}"
        );
    }

    #[test]
    fn an_empty_require_list_is_red() {
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(R7), ",, ,");
        assert_eq!(code, 1);
        assert_eq!(
            out,
            "FAIL\n  - no suites required (a gate over an empty registry is vacuous)\n"
        );
    }

    // ─────────────────────── bd-wf2: the three holes ───────────────────────

    #[test]
    fn duplicate_require_is_an_error_not_an_inflated_total() {
        // Formerly double-counted: one suite reporting 3 totalled 6, and an
        // inflated measured_total is the direction that turns real drift GREEN.
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(R7), "spec_alpha,spec_alpha");
        assert_eq!(code, 1, "{out}");
        // No report block at all: there is no total worth reporting.
        assert!(!out.contains("\n  measured_total="), "{out}");
        assert!(
            out.contains("--require names 'spec_alpha' more than once"),
            "{out}"
        );

        // The sharp shape: one suite reporting 3 must not certify a README
        // advertising 6 across "two suites".
        let r6 = R7.replace('7', "6");
        let (out, code) = check(
            "L",
            Some("INJECTIONS=3 SUITE=spec_alpha\n"),
            "R",
            Some(&r6),
            "spec_alpha,spec_alpha",
        );
        assert_eq!(code, 1, "{out}");
    }

    #[test]
    fn findings_name_the_file_that_was_actually_scanned() {
        let r8 = R7.replace('7', "8");
        let (out, code) = check("L", Some(GOOD_LOG), "README_off.md", Some(&r8), REQ);
        assert_eq!(code, 1);
        assert!(out.contains("README_off.md:3 advertises 8"), "{out}");
        assert!(
            !out.contains("\n    - README.md:"),
            "a finding must not send the reader to an innocent file: {out}"
        );
    }

    #[test]
    fn a_word_spelled_count_is_read_not_skipped() {
        // The middle case: three sites in digits, one in prose. Before bd-wf2 the
        // prose site was invisible and the drift on it reported as GREEN.
        let drifted = R7.replace("**7 known-bad faults**", "**thirty-six known-bad faults**");
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(&drifted), REQ);
        assert_eq!(code, 1, "{out}");
        assert!(
            out.contains("advertises 36 known-bad injections; the suites self-reported 7"),
            "{out}"
        );

        // ...and a word-spelled site that AGREES stays green.
        let agreeing = R7.replace("**7 known-bad faults**", "**seven known-bad faults**");
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(&agreeing), REQ);
        assert_eq!(code, 0, "{out}");
    }

    #[test]
    fn a_site_that_stops_parsing_trips_the_floor() {
        // A count outside the word vocabulary ("three dozen") removes the site
        // from the scanner entirely. Partial coverage must not report like full.
        let obscured = R7.replace("**7 known-bad faults**", "**three dozen known-bad faults**");
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(&obscured), REQ);
        assert_eq!(code, 1, "{out}");
        assert!(
            out.contains("only 4 advertisement site(s) parsed in R; at least 5 are expected"),
            "{out}"
        );
    }

    #[test]
    fn known_bad_without_a_population_qualifier_is_red() {
        let unqual = R7.replace("known-bad (shell selftest suites):", "known-bad:");
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(&unqual), REQ);
        assert_eq!(code, 1, "{out}");
        assert!(
            out.contains("R:3 advertises known-bad injections without a shell/selftest qualifier"),
            "{out}"
        );

        let shell_only = R7.replace("shell selftest suites", "shell");
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(&shell_only), REQ);
        assert_eq!(code, 0, "{out}");
    }

    #[test]
    fn the_site_floor_is_a_floor_not_an_equality() {
        // An EXTRA advertisement is not drift; only losing one is.
        let extra = format!("{R7}\nand 7 injections mentioned elsewhere\n");
        let (out, code) = check("L", Some(GOOD_LOG), "R", Some(&extra), REQ);
        assert_eq!(code, 0, "{out}");
    }

    // ───────────────────── bd-wf2: regenerated, not typed ──────────────────

    #[test]
    fn write_readme_regenerates_every_site_from_the_receipts() {
        let r8 = R7.replace('7', "8");
        let (out, code, written) = render("L", Some(GOOD_LOG), "R", Some(&r8), REQ, true);
        assert_eq!(code, 0, "{out}");
        assert!(
            out.contains("  regenerated R: 5 site(s) now advertise 7\n"),
            "{out}"
        );
        assert_eq!(written.as_deref(), Some(R7), "the rewrite must land on 7");
    }

    #[test]
    fn write_readme_normalises_a_word_spelled_site_to_digits() {
        let prose = R7.replace("**7 known-bad faults**", "**eight known-bad faults**");
        let (out, code, written) = render("L", Some(GOOD_LOG), "R", Some(&prose), REQ, true);
        assert_eq!(code, 0, "{out}");
        assert_eq!(written.as_deref(), Some(R7));
    }

    #[test]
    fn per_suite_column_trips_in_both_directions_and_anti_vacuous() {
        let req = "selftest_alpha,selftest_beta";
        let log = "INJECTIONS=3 SUITE=selftest_alpha\nINJECTIONS=4 SUITE=selftest_beta\n";
        let table = |alpha: &str, extra: &str| {
            format!(
                "{R7}| Suite | n | Injections |\n|---|---|---|\n| `selftest_alpha` | {alpha} | x |\n| `selftest_beta` | 4 | x |{extra}"
            )
        };
        let (out, code) = check("L", Some(log), "R", Some(&table("3", "")), req);
        assert_eq!(code, 0, "{out}");

        let (out, code) = check("L", Some(log), "R", Some(&table("2", "")), req);
        assert_eq!(code, 1);
        assert!(
            out.contains("suite selftest_alpha advertises 2 injections; the suite self-reported 3"),
            "{out}"
        );
        assert!(out.contains("R:"), "{out}");

        let (out, code) = check("L", Some(log), "R", Some(&table("4", "")), req);
        assert_eq!(code, 1);
        assert!(
            out.contains("suite selftest_alpha advertises 4 injections; the suite self-reported 3"),
            "{out}"
        );

        let missing = format!("{R7}| Suite | n |\n|---|---|\n| `selftest_beta` | 4 | x |\n");
        let (out, code) = check("L", Some(log), "R", Some(&missing), req);
        assert_eq!(code, 1);
        assert!(out.contains("has no per-suite table row"), "{out}");

        let (out, code) = check(
            "L",
            Some(log),
            "R",
            Some(&table("3", "\n| `selftest_not_a_real_suite` | 1 | x |\n")),
            req,
        );
        assert_eq!(code, 1);
        assert!(out.contains("is not in REGISTERED_SUITES"), "{out}");

        let (out, code) = check("L", Some(log), "R", Some(R7), req);
        assert_eq!(code, 1);
        assert!(out.contains("parsed to zero suite rows"), "{out}");

        let (out, code, written) = render("L", Some(log), "R", Some(&table("2", "")), req, true);
        assert_eq!(code, 0, "{out}");
        let w = written.expect("cells must be rewritten");
        assert!(w.contains("| `selftest_alpha` | 3 |"), "{w}");
        assert!(!w.contains("| `selftest_alpha` | 2 |"), "{w}");

        let (out, code, written) = render(
            "L",
            Some("INJECTIONS=3 SUITE=selftest_alpha\n"),
            "R",
            Some(&table("2", "")),
            req,
            true,
        );
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("regeneration SKIPPED"), "{out}");
        assert!(written.is_none());
    }

    #[test]
    fn write_readme_refuses_to_launder_an_unsound_total() {
        // spec_beta never reported: the total is not a number worth writing.
        let r8 = R7.replace('7', "8");
        let (out, code, written) = render(
            "L",
            Some("INJECTIONS=3 SUITE=spec_alpha\n"),
            "R",
            Some(&r8),
            REQ,
            true,
        );
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("regeneration SKIPPED"), "{out}");
        assert!(written.is_none(), "a bogus log must not rewrite README");
    }

    #[test]
    fn write_readme_preserves_line_terminators_and_untouched_bytes() {
        let src = "8 injections\r\n8 known-bad faults\rtrailing 8 faults";
        let (after, sites) = regenerate(src, "7");
        assert_eq!(sites, 3);
        assert_eq!(
            after,
            "7 injections\r\n7 known-bad faults\rtrailing 7 faults"
        );
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

        // bd-wf2: word-spelled counts are read, not skipped.
        assert_eq!(
            scan_advertised("thirty-six injections"),
            vec!["36".to_string()]
        );
        assert_eq!(
            scan_advertised("thirty six known-bad faults"),
            vec!["36".to_string()]
        );
        assert_eq!(scan_advertised("SEVEN INJECTIONS"), vec!["7".to_string()]);
        // Longest-first alternation: "eighteen" must win over "eight".
        assert_eq!(scan_advertised("eighteen faults"), vec!["18".to_string()]);
        assert_eq!(scan_advertised("twenty-one faults"), vec!["21".to_string()]);
        // The word branch carries a `\b`; the digit branch does not.
        assert_eq!(
            scan_advertised("freighter injections"),
            Vec::<String>::new()
        );
        assert_eq!(scan_advertised("v1.7 injections"), vec!["7".to_string()]);
        // Outside the vocabulary the site is not read at all — the site floor,
        // not the scanner, is what keeps that fail-closed.
        assert_eq!(
            scan_advertised("three dozen injections"),
            Vec::<String>::new()
        );
        assert_eq!(scan_advertised("zero injections"), vec!["0".to_string()]);

        // The spans are what --write-readme rewrites: group 1 only.
        assert_eq!(
            scan_advertised_spans("say thirty-six injections"),
            vec![(4usize, 14usize, "36".to_string())]
        );

        assert_eq!(scan_suite_counts("9 selftest suites;"), vec![9]);
        assert_eq!(scan_suite_counts("Nine selftest suites inject"), vec![9]);
        assert_eq!(scan_suite_counts("2 suites, 7 injections"), vec![2]);
        assert_eq!(scan_suite_counts("forty-two suites"), vec![42]);
        assert_eq!(
            scan_suite_counts("suitesx and 3 suitesy"),
            Vec::<u128>::new()
        );

        // Per-suite table row. Mentions in comments / CHARTER are not scanned.
        let row = "| `selftest_orphan` | 6 | empty bank |";
        let p = parse_suite_row(row).expect("row");
        assert_eq!(p.0, "selftest_orphan");
        assert_eq!(p.1, "6");
        assert_eq!(&row[p.2..p.3], "6");
        assert!(parse_suite_row("| Suite | n | Injections |").is_none());
        assert!(parse_suite_row("| `spec_alpha` | 3 | x |").is_none());
        assert!(parse_suite_row("mentions `selftest_orphan` | 6 | in prose").is_none());
        let wasm_row = "| `wasm-freshness` | 2 | flipped blob · native-only constant |";
        let w = parse_suite_row(wasm_row).expect("wasm-freshness row");
        assert_eq!(w.0, "wasm-freshness");
        assert_eq!(w.1, "2");
    }

    #[test]
    fn the_cardinal_vocabulary_is_complete_and_ordered() {
        let t = cardinals();
        // zero..nineteen + eight tens + 8*9*2 compounds
        assert_eq!(t.len(), 20 + 8 + 144);
        let mut seen = std::collections::BTreeSet::new();
        for (w, _) in t {
            assert!(seen.insert(w.clone()), "duplicate cardinal {w}");
        }
        // Longest first, ties lexicographic — the alternation order the Python
        // builds with sorted(key=lambda w: (-len(w), w)).
        for pair in t.windows(2) {
            let (a, b) = (&pair[0].0, &pair[1].0);
            assert!(
                a.len() > b.len() || (a.len() == b.len() && a < b),
                "ordering broken at {a:?} / {b:?}"
            );
        }
        let by_word = |w: &str| t.iter().find(|(k, _)| k == w).map(|(_, v)| *v);
        assert_eq!(by_word("zero"), Some(0));
        assert_eq!(by_word("nineteen"), Some(19));
        assert_eq!(by_word("ninety-nine"), Some(99));
        assert_eq!(by_word("ninety nine"), Some(99));
        assert_eq!(by_word("one hundred"), None, "bounded at ninety-nine");
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
        assert!(!a.write_readme);

        let a = parse_args(&["--log".into(), "x".into(), "--write-readme".into()]).unwrap();
        assert!(a.write_readme);
        assert_eq!(a.log, "x");
        assert!(parse_args(&["--log=x".into(), "--write-readme=1".into()]).is_err());
    }

    #[test]
    fn check_sh_wires_write_readme_behind_the_env_flag() {
        // BUILT ≠ WIRED [bd-injection-count-regen-unreachable-lu45]:
        // --write-readme is reachable from check.sh only through
        // CDCP_INJECTION_COUNT_WRITE_README=1. Without the flag, drift stays
        // RED. Always-passing --write-readme would rewrite on every run and
        // hide drift.
        let root = crate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root");
        let text = std::fs::read_to_string(root.join("scripts/check.sh"))
            .expect("scripts/check.sh must exist");
        let block = injection_count_invoke_block(&text)
            .expect("check.sh must invoke verify-injection-count");
        assert!(
            block.contains("CDCP_INJECTION_COUNT_WRITE_README:-0"),
            "check.sh must honour CDCP_INJECTION_COUNT_WRITE_README; \
             --write-readme is otherwise unreachable\n{block}"
        );
        let write_lines: Vec<&str> = block
            .lines()
            .filter(|l| l.contains("verify-injection-count") && l.contains("--write-readme"))
            .collect();
        assert_eq!(
            write_lines.len(),
            1,
            "exactly one verify-injection-count invocation may pass --write-readme \
             (the flag=1 path)\n{block}"
        );
        let default_lines: Vec<&str> = block
            .lines()
            .filter(|l| l.contains("verify-injection-count") && !l.contains("--write-readme"))
            .collect();
        assert!(
            !default_lines.is_empty(),
            "without the flag, check.sh must still invoke verify-injection-count \
             (drift stays RED)\n{block}"
        );
    }

    fn injection_count_invoke_block(text: &str) -> Option<String> {
        let start = text.find("cdcp_gate verify-injection-count (advertised known-bad count)")?;
        let rest = &text[start..];
        let end = rest.find("advertised known-bad injection count ==")?;
        Some(rest[..end].to_string())
    }
}
