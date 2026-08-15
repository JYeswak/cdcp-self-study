//! verify-step-count — L4 drift guard for the advertised `check.sh` step count.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **the number README.md advertises about the length
//! of the `check.sh` gate chain can no longer drift silently away from the number
//! that chain actually emits.** It is the third number in README's gate sentence;
//! the other two (selftest suites, known-bad injections) were already enforced by
//! `verify-injection-count`, and this one was folklore.
//!
//! ## Why the count cannot be parsed out of the script
//!
//! Measured 2026-08-14 (bd-1sd.13). Three attempts were tried and rejected:
//!
//! * **Counting `ok` call sites statically** under-reports and over-reports at the
//!   same time. Several sites are conditional — the WASM dual-path leg, the SLO
//!   budgets, the `serve` verb — so a run on a machine without the wasm32 target
//!   legitimately emits fewer receipts than the script contains sites for.
//! * **`grep -c 'check.sh: ok:'` over a transcript OVER-COUNTS.** `check.sh` runs
//!   nested copies of itself: `substrate-guard --prove-wired` executes a full
//!   `check.sh` inside `target/cdcp-substrate-probe/`, and the evidence block that
//!   probe prints on success carries the tail of that child's transcript — five
//!   `check.sh: ok:` lines — straight into the parent's stdout. The advertised
//!   `76` that this bead was filed against is exactly that: 71 real receipts plus
//!   the child's 5.
//! * **Hand-maintaining it** is the defect itself. It was stale by thirteen for
//!   weeks, and the hand correction that followed was wrong again ten minutes
//!   later, through a fully green gate chain.
//!
//! So `check.sh` COUNTS AS IT RUNS. Its `ok()` helper increments a shell counter,
//! its honest skips increment a second one, and on the success path it emits one
//! machine-readable receipt — the same contract the selftest suites already use
//! for `INJECTIONS=<n> SUITE=<name>`:
//!
//! ```text
//! CHECK_STEPS=<total> OK=<n> SKIPPED=<k> NESTED_OK=<m> DEPTH=<d> RUN=<id>
//! ```
//!
//! `total` is `ok + skipped`, so the advertised number describes the CHAIN rather
//! than one machine's environment: a leg that honestly skipped is still a step the
//! chain defines. `DEPTH` is 0 for the outer run and higher for every nested one.
//!
//! ## How a nested child is kept out of the count
//!
//! Three independent reasons, because one of them is only a convention:
//!
//! 1. The counter is a shell variable in the running process. A nested `check.sh`
//!    is a separate process; shell variables do not propagate upward. Nothing the
//!    child counts can reach the parent's counter.
//! 2. Nothing aggregates the count from a transcript. The receipt is written by
//!    the process that did the counting, into a file that process created.
//! 3. The receipt carries `DEPTH`, so even a log that somehow collected both is
//!    decidable here: exactly one `DEPTH=0` receipt is required, a log holding
//!    only nested receipts is an ERROR rather than a fallback, and two `DEPTH=0`
//!    receipts are an ERROR rather than a sum.
//!
//! And the run measures the contamination it is immune to: `NESTED_OK` is the
//! number of `check.sh: ok:` receipts the probe child actually wrote to its own
//! transcript during this run. `NESTED_OK=0` is an ERROR — a run where the nested
//! path never produced a single competing receipt did not exercise the hazard, and
//! reporting it as a pass would be the vacuous-scan failure one level up.
//!
//! ## The ordering leg
//!
//! A step added AFTER the receipt is emitted could never be counted, and the gate
//! would stay green while the chain grew. Deriving the COUNT from the script text
//! is undecidable-in-practice (above); deriving the ORDER is not. `check.sh`
//! carries one `STEP-COUNT-RECEIPT-BOUNDARY` marker, and this gate requires that
//! no `ok` call site appears after it. It also requires at least one `ok` site to
//! exist at all — a script that scanned to zero sites is an ERROR, not a pass —
//! and that the runtime count never exceeds the number of sites, which is the one
//! statically checkable falsification of "the parent counted the child's lines".
//!
//! ## What it cannot decide
//!
//! * Whether a step is **useful**. It counts receipts, not value.
//! * Whether the receipt is **honest**. `ok()` increments when `check.sh` says a
//!   step passed; a step that reports ok without checking anything is invisible
//!   here. That is what the per-gate known-bad injections are for.
//! * Whether the log came from **this** run. `check.sh` mktemps a fresh receipt
//!   file per invocation; freshness is its job, not this gate's.
//! * Whether a count spelled outside the word vocabulary ("four dozen steps")
//!   means what it says. Such a site is not read as a number at all, and the site
//!   floor is what keeps that fail-closed rather than silent.
//!
//! ## Exit codes
//!
//! The crate's shared codes: 2 for drift (a number disagrees), 4 for anything that
//! makes the comparison untrustworthy (missing/empty/ill-formed receipt, zero
//! steps counted, zero nested receipts observed, README that advertises nothing,
//! coverage below the site floor). 4 is never confused with 0: a count that could
//! not be checked must not report like one that was.

use crate::registry::{GateCtx, GateError};
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const NAME: &str = "verify-step-count";
pub const SUMMARY: &str =
    "compare the step count check.sh measured at runtime against the count README advertises";

/// How many advertisement sites must parse before the comparison is worth
/// anything. The shipped README advertises the step count at four sites (the badge
/// markup contributes two: its label and its shields.io path).
///
/// A FLOOR, not an equality: adding an advertisement is free, removing or
/// obscuring one is a deliberate decision that has to edit this constant. Without
/// it, a README where one site stopped parsing reports exactly like a README where
/// all of them still do.
pub const MIN_STEP_SITES: usize = 4;

/// The single marker `check.sh` carries immediately before it emits its receipt.
/// Used only to decide ORDER — see the module header on why order is decidable
/// from the script text and the count is not.
///
/// WHAT IT MATCHES, stated because a scanner whose scope is left implicit will be
/// tripped by prose about itself. A boundary line is a COMMENT LINE WHOSE FIRST
/// TOKEN IS THIS MARKER — `#` (optionally repeated or spaced) then the marker.
/// A mention anywhere else on a line is text ABOUT the boundary, not the boundary:
/// this file's own module header names it, `check.sh`'s explanatory comment named
/// it and made the count 2, and README quotes it in prose. Measured 2026-08-14 —
/// the first draft matched the bare substring and its own documentation broke it.
/// The same substring-scope defect has bitten this repo three times this session.
pub const BOUNDARY_MARKER: &str = "STEP-COUNT-RECEIPT-BOUNDARY";

/// The six keys of the receipt, in the order they must appear.
const KEYS: [&str; 6] = ["CHECK_STEPS", "OK", "SKIPPED", "NESTED_OK", "DEPTH", "RUN"];

/// One `CHECK_STEPS=` receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub total: u64,
    pub ok: u64,
    pub skipped: u64,
    pub nested_ok: u64,
    pub depth: u64,
    pub run: String,
}

/// Parse one already-trimmed receipt line.
///
/// Deliberately strict: six whitespace-separated `KEY=VALUE` tokens, keys in the
/// fixed order above, five of them unsigned decimal. A receipt that does not parse
/// is an ERROR rather than a skipped line, because a receipt shape that drifted is
/// indistinguishable from one that was never emitted.
pub fn parse_receipt(line: &str) -> Result<Receipt, String> {
    let toks: Vec<&str> = line.split_ascii_whitespace().collect();
    if toks.len() != KEYS.len() {
        return Err(format!(
            "expected {} KEY=VALUE tokens ({}), found {}",
            KEYS.len(),
            KEYS.join(" "),
            toks.len()
        ));
    }
    let mut nums = [0u64; 5];
    let mut run = String::new();
    for (i, tok) in toks.iter().enumerate() {
        let Some((k, v)) = tok.split_once('=') else {
            return Err(format!("token {} is not KEY=VALUE: {tok:?}", i + 1));
        };
        if k != KEYS[i] {
            return Err(format!(
                "token {} is {k:?}; this position must be {:?}",
                i + 1,
                KEYS[i]
            ));
        }
        if i == 5 {
            if v.is_empty() {
                return Err("RUN= carries no run id".to_string());
            }
            run = v.to_string();
            continue;
        }
        if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("{k}={v:?} is not an unsigned decimal"));
        }
        nums[i] = v.parse::<u64>().map_err(|e| format!("{k}={v:?}: {e}"))?;
    }
    Ok(Receipt {
        total: nums[0],
        ok: nums[1],
        skipped: nums[2],
        nested_ok: nums[3],
        depth: nums[4],
        run,
    })
}

// ───────────────────────────── README scanning ─────────────────────────────
//
// `(\d+|\b<cardinal>)[\s_]+(?:ordered[\s_]+)?(?:gate[\s_]+)?steps?\b`, case
// insensitive, hand-compiled rather than pulled in as a regex dependency — the
// same choice `verify_injection_count` made, and the cardinal table is shared
// with it so the two gates read prose the same way.
//
// Word-spelled counts parse on purpose. The subtle failure is not "no site
// parses" — the site floor already catches that. It is ONE site quietly leaving
// the scanner while the others still parse: coverage drops and the report is
// indistinguishable from full coverage. Rewriting a site as "seventy-two ordered
// steps" therefore keeps it under the gate instead of removing it from the gate.

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn at_word_boundary(s: &str, p: usize) -> bool {
    let before = s[..p].chars().next_back().is_some_and(is_word);
    let after = s[p..].chars().next().is_some_and(is_word);
    before != after
}

fn digits_end(s: &str, p: usize) -> usize {
    p + s[p..].bytes().take_while(u8::is_ascii_digit).count()
}

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

fn lit_ci(s: &str, p: usize, lit: &str) -> Option<usize> {
    let end = p + lit.len();
    if end <= s.len() && s.is_char_boundary(end) && s[p..end].eq_ignore_ascii_case(lit) {
        Some(end)
    } else {
        None
    }
}

fn sep_run(s: &str, p: usize) -> usize {
    run_end(s, p, |c| c.is_whitespace() || c == '_')
}

/// `steps?\b` at `p`, greedy `s?` tried first.
fn steps_tail(s: &str, p: usize) -> Option<usize> {
    let e = lit_ci(s, p, "step")?;
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

/// The optional `ordered` / `gate` qualifiers, then `steps?`.
fn step_phrase_end(line: &str, from: usize) -> Option<usize> {
    let mut starts = vec![from];
    if let Some(e) = lit_ci(line, from, "ordered") {
        let e2 = sep_run(line, e);
        if e2 > e {
            starts.push(e2);
            if let Some(g) = lit_ci(line, e2, "gate") {
                let g2 = sep_run(line, g);
                if g2 > g {
                    starts.push(g2);
                }
            }
        }
    }
    if let Some(g) = lit_ci(line, from, "gate") {
        let g2 = sep_run(line, g);
        if g2 > g {
            starts.push(g2);
        }
    }
    // Longest qualifier chain first, so "ordered gate steps" is not read as a
    // failed match on "ordered steps".
    starts.sort_unstable();
    starts.reverse();
    starts.into_iter().find_map(|s| steps_tail(line, s))
}

/// Every advertised step count on one line as `(start, end, value)` spans over the
/// count token itself, so `--write-readme` can rewrite exactly that token and
/// leave surrounding markup and prose byte-identical.
pub fn scan_step_spans(line: &str) -> Vec<(usize, usize, u64)> {
    let mut out = Vec::new();
    let mut p = 0usize;
    while p < line.len() {
        if !line.is_char_boundary(p) {
            p += 1;
            continue;
        }
        let hit = (|| {
            let mut alts: Vec<(usize, u64)> = Vec::new();
            let d_end = digits_end(line, p);
            if d_end > p {
                alts.push((d_end, line[p..d_end].parse::<u64>().unwrap_or(u64::MAX)));
            }
            if at_word_boundary(line, p) {
                for (w, v) in crate::gates::verify_injection_count::cardinals() {
                    if let Some(e) = lit_ci(line, p, w) {
                        alts.push((e, *v as u64));
                    }
                }
            }
            for (c_end, value) in alts {
                let w_end = sep_run(line, c_end);
                if w_end == c_end {
                    continue;
                }
                if let Some(e) = step_phrase_end(line, w_end) {
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

/// Rewrite every advertised step count in `text` to `total`. Returns the new text
/// and how many sites were rewritten. A word-spelled site is normalised to digits:
/// regeneration produces the checkable form.
pub fn regenerate(text: &str, total: u64) -> (String, usize) {
    let repl = total.to_string();
    let mut out = String::with_capacity(text.len());
    let mut rewritten = 0usize;
    let mut rest = text;
    while !rest.is_empty() {
        let (line, tail) = match rest.find('\n') {
            Some(i) => (&rest[..=i], &rest[i + 1..]),
            None => (rest, ""),
        };
        let spans = scan_step_spans(line);
        if spans.is_empty() {
            out.push_str(line);
        } else {
            let mut last = 0usize;
            for (s, e, _) in spans {
                out.push_str(&line[last..s]);
                out.push_str(&repl);
                last = e;
                rewritten += 1;
            }
            out.push_str(&line[last..]);
        }
        rest = tail;
    }
    (out, rewritten)
}

// ────────────────────────── check.sh structure scan ────────────────────────

/// 1-based line numbers of the `ok "…"` call sites in a shell script.
///
/// Comment lines are excluded. The call must be `ok` at a command position — start
/// of line or after whitespace, `;`, `&`, `|`, `(` or `{` — immediately followed by
/// a space and a double quote, which is the form every site in `check.sh` uses.
/// This is a scan for ORDER, never for a count; see the module header.
pub fn ok_call_lines(script: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, line) in script.lines().enumerate() {
        if line.trim_start().starts_with('#') {
            continue;
        }
        let b = line.as_bytes();
        let mut j = 0usize;
        while let Some(rel) = line[j..].find("ok \"") {
            let at = j + rel;
            let prev_ok =
                at == 0 || matches!(b[at - 1], b' ' | b'\t' | b';' | b'&' | b'|' | b'(' | b'{');
            if prev_ok {
                out.push(i + 1);
                break;
            }
            j = at + 2;
        }
    }
    out
}

/// 1-based line numbers that ARE the boundary — a comment line whose first token
/// is [`BOUNDARY_MARKER`]. See that constant for why a bare substring match is
/// wrong: it matches the documentation of the marker as readily as the marker.
pub fn boundary_lines(script: &str) -> Vec<usize> {
    script
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim_start();
            let Some(rest) = t.strip_prefix('#') else {
                return false;
            };
            rest.trim_start_matches(['#', ' ', '\t'])
                .starts_with(BOUNDARY_MARKER)
        })
        .map(|(i, _)| i + 1)
        .collect()
}

/// 1-based line numbers that emit a `CHECK_STEPS=` receipt (comments excluded).
pub fn emission_lines(script: &str) -> Vec<usize> {
    script
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim_start().starts_with('#') && l.contains("CHECK_STEPS="))
        .map(|(i, _)| i + 1)
        .collect()
}

// ─────────────────────────────── the gate ──────────────────────────────────

/// Everything the gate decides, as a pure function: bodies in, exact stdout and
/// exit code out, plus the new README body when `--write-readme` earned one. The
/// caller does the writing, so this stays testable without a filesystem.
///
/// `None` for a body means "not a regular file" — missing, a directory, or
/// unreadable alike.
#[allow(clippy::too_many_arguments)]
pub fn render(
    log_display: &str,
    log_body: Option<&str>,
    readme_display: &str,
    readme_body: Option<&str>,
    script_display: &str,
    script_body: Option<&str>,
    write_readme: bool,
) -> (String, u8, Option<String>) {
    // Two buckets, two exit codes. `errors` means the comparison itself cannot be
    // trusted (exit 4); `violations` means it ran and a number disagreed (exit 2).
    let mut errors: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    // ── the receipt ────────────────────────────────────────────────────────
    let mut outer: Option<Receipt> = None;
    let mut nested: Vec<Receipt> = Vec::new();
    match log_body {
        None => errors.push(format!("step receipt log missing: {log_display}")),
        Some(text) => {
            let lines: Vec<&str> = text
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            if lines.is_empty() {
                errors.push(
                    "step receipt log is empty — check.sh emitted no CHECK_STEPS= receipt (an empty scan set is an ERROR, not a pass)"
                        .to_string(),
                );
            }
            let mut outers: Vec<Receipt> = Vec::new();
            for raw in lines {
                match parse_receipt(raw) {
                    Err(why) => errors.push(format!("unparseable receipt line {raw:?}: {why}")),
                    Ok(r) if r.depth == 0 => outers.push(r),
                    Ok(r) => nested.push(r),
                }
            }
            match outers.len() {
                0 => {
                    if !nested.is_empty() {
                        errors.push(format!(
                            "the log holds {} nested receipt(s) (DEPTH>0) and no DEPTH=0 receipt — a nested check.sh counts its own chain, never the outer one; falling back to it would be the contamination this gate exists to refuse",
                            nested.len()
                        ));
                    }
                }
                1 => outer = outers.pop(),
                n => errors.push(format!(
                    "{n} DEPTH=0 receipts in one log — exactly one outer run may report, and summing or picking between them would invent a number no run measured"
                )),
            }
        }
    }

    // ── the receipt's internal arithmetic ──────────────────────────────────
    let mut total: Option<u64> = None;
    if let Some(r) = &outer {
        if r.ok == 0 {
            errors.push(
                "the run counted ZERO ok steps — a counter that returns 0 and compares 0 against 0 is a vacuous pass, so this is an ERROR"
                    .to_string(),
            );
        }
        if r.nested_ok == 0 {
            errors.push(
                "NESTED_OK=0 — the nested check.sh child emitted no `check.sh: ok:` receipt during this run, so the run never exercised the path a transcript counter would over-count on. ERROR, not a pass"
                    .to_string(),
            );
        }
        if r.total != r.ok + r.skipped {
            violations.push(format!(
                "receipt does not add up: CHECK_STEPS={} but OK={} + SKIPPED={} = {}",
                r.total,
                r.ok,
                r.skipped,
                r.ok + r.skipped
            ));
        }
        total = Some(r.total);
    }

    // ── the script's structure ─────────────────────────────────────────────
    let mut ok_sites = 0usize;
    match script_body {
        None => errors.push(format!(
            "check.sh missing or unreadable: {script_display} — the ordering leg cannot be evaluated, which is an ERROR, not a pass"
        )),
        Some(script) => {
            let oks = ok_call_lines(script);
            ok_sites = oks.len();
            if oks.is_empty() {
                errors.push(format!(
                    "{script_display} scanned to zero `ok \"…\"` call sites — a vacuous scan is an ERROR, not a pass"
                ));
            }
            let bounds = boundary_lines(script);
            match bounds.len() {
                1 => {
                    let b = bounds[0];
                    let after: Vec<usize> = oks.iter().copied().filter(|&l| l > b).collect();
                    if !after.is_empty() {
                        violations.push(format!(
                            "{script_display} calls ok at line(s) {} — after the {BOUNDARY_MARKER} at line {b}. A step emitted after the receipt can never be counted, so the advertised number would stay green while the chain grew",
                            after
                                .iter()
                                .map(usize::to_string)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    let emits: Vec<usize> =
                        emission_lines(script).into_iter().filter(|&l| l > b).collect();
                    if emits.is_empty() {
                        errors.push(format!(
                            "{script_display} carries the {BOUNDARY_MARKER} at line {b} but emits no CHECK_STEPS= receipt after it — the receipt this gate reads is not produced by the script it is judging"
                        ));
                    }
                }
                0 => errors.push(format!(
                    "{script_display} carries no {BOUNDARY_MARKER} marker — without it the ordering leg cannot say where the count is sealed. ERROR, not a pass"
                )),
                n => errors.push(format!(
                    "{script_display} carries {n} {BOUNDARY_MARKER} markers at line(s) {}; exactly one seals the count",
                    bounds
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            }
        }
    }

    // Each `ok` site in this linear script runs at most once, so the runtime count
    // can never exceed the number of sites. It is the one statically checkable
    // falsification of "the parent counted a nested child's receipts". If a future
    // step legitimately puts an `ok` inside a loop, this is the line to revisit —
    // deliberately, and in writing.
    if let (Some(r), true) = (&outer, ok_sites > 0) {
        if r.ok as usize > ok_sites {
            violations.push(format!(
                "the run counted OK={} receipts but {script_display} holds only {ok_sites} `ok` call site(s); a count above the site total means receipts from somewhere other than this chain were counted",
                r.ok
            ));
        }
    }

    // ── regeneration, only from a sound receipt ────────────────────────────
    let receipt_sound = errors.is_empty() && violations.is_empty() && total.is_some();
    let mut regen_note: Option<String> = None;
    let mut new_readme: Option<String> = None;
    let mut readme_text: Option<String> = readme_body.map(str::to_string);

    if write_readme {
        match (receipt_sound, readme_body) {
            (false, _) => {
                regen_note = Some(
                    "regeneration SKIPPED: the receipt is not sound, so the total is not a number worth writing"
                        .to_string(),
                );
            }
            (true, None) => {
                regen_note = Some("regeneration SKIPPED: README is not readable".to_string());
            }
            (true, Some(before)) => {
                let t = total.unwrap_or(0);
                let (after, sites) = regenerate(before, t);
                if sites == 0 {
                    regen_note = Some(format!(
                        "regeneration wrote nothing: {readme_display} advertises no parseable step count to rewrite"
                    ));
                } else if after == before {
                    regen_note = Some(format!(
                        "regenerated {readme_display}: {sites} site(s) already advertise {t}"
                    ));
                } else {
                    regen_note = Some(format!(
                        "regenerated {readme_display}: {sites} site(s) now advertise {t}"
                    ));
                    new_readme = Some(after.clone());
                    readme_text = Some(after);
                }
            }
        }
    }

    // ── README ─────────────────────────────────────────────────────────────
    let mut advertised: Vec<(usize, u64)> = Vec::new();
    match readme_text.as_deref() {
        None => errors.push(format!("README missing: {readme_display}")),
        Some(text) => {
            for (i, line) in text.lines().enumerate() {
                for (_, _, n) in scan_step_spans(line) {
                    advertised.push((i + 1, n));
                }
            }
            if advertised.is_empty() {
                errors.push(format!(
                    "{readme_display} advertises no check.sh step count at all (nothing to check is an ERROR, not a pass)"
                ));
            } else if advertised.len() < MIN_STEP_SITES {
                errors.push(format!(
                    "only {} step advertisement site(s) parsed in {readme_display}; at least {MIN_STEP_SITES} are expected — a site that stopped parsing loses coverage while reporting exactly like full coverage",
                    advertised.len()
                ));
            }
            if let Some(t) = total {
                for (lineno, n) in &advertised {
                    if *n != t {
                        violations.push(format!(
                            "{readme_display}:{lineno} advertises {n} check.sh steps; this run measured {t}"
                        ));
                    }
                }
            }
        }
    }

    // ── report ─────────────────────────────────────────────────────────────
    let mut claims: Vec<u64> = advertised.iter().map(|(_, n)| *n).collect();
    claims.sort_unstable();
    claims.dedup();

    let failed = !errors.is_empty() || !violations.is_empty();
    let mut out = String::new();
    out.push_str(if failed { "FAIL\n" } else { "PASS\n" });
    out.push_str(&format!("  log={log_display}\n"));
    out.push_str(&format!("  script={script_display}\n"));
    out.push_str(&format!("  readme={readme_display}\n"));
    match &outer {
        None => out.push_str("  measured_steps=MISSING (no DEPTH=0 receipt)\n"),
        Some(r) => {
            out.push_str(&format!(
                "  measured_steps={} (ok={} skipped={} run={})\n",
                r.total, r.ok, r.skipped, r.run
            ));
            out.push_str(&format!(
                "  nested_ok_receipts={} written by the nested check.sh child; a transcript counter would have had to exclude every one of them\n",
                r.nested_ok
            ));
        }
    }
    out.push_str(&format!(
        "  nested_receipts_ignored={} (DEPTH>0)\n",
        nested.len()
    ));
    out.push_str(&format!("  ok_call_sites={ok_sites}\n"));
    out.push_str(&format!(
        "  readme_claims=[{}]\n",
        claims
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    ));
    if let Some(note) = &regen_note {
        out.push_str(&format!("  {note}\n"));
    }

    if failed {
        if !errors.is_empty() {
            out.push_str("  errors (the comparison could not be trusted):\n");
            for e in errors.iter().take(40) {
                out.push_str(&format!("    - {e}\n"));
            }
        }
        if !violations.is_empty() {
            out.push_str("  violations (the numbers disagree):\n");
            for v in violations.iter().take(40) {
                out.push_str(&format!("    - {v}\n"));
            }
            out.push_str(
                "  the advertised number is REGENERATED, never typed: re-run with\n    CDCP_STEP_COUNT_WRITE_README=1 sh scripts/check.sh\n",
            );
        }
        let code = if errors.is_empty() {
            crate::exit::VIOLATION
        } else {
            crate::exit::ERROR
        };
        return (out, code, new_readme);
    }

    out.push_str(&format!(
        "  step count GREEN (README and this run both say {})\n",
        total.unwrap_or(0)
    ));
    (out, crate::exit::OK, new_readme)
}

struct Args {
    log: String,
    readme: Option<String>,
    script: Option<String>,
    write_readme: bool,
}

/// `--flag value` and `--flag=value`, both forms. Anything else is USAGE — a
/// typo'd flag must never read as "the gate passed".
fn parse_args(argv: &[String]) -> Result<Args, GateError> {
    let mut log: Option<String> = None;
    let mut readme: Option<String> = None;
    let mut script: Option<String> = None;
    let mut write_readme = false;
    let mut i = 0usize;
    while i < argv.len() {
        let a = &argv[i];
        let (key, inline) = match a.split_once('=') {
            Some((k, v)) => (k, Some(v.to_string())),
            None => (a.as_str(), None),
        };
        if key == "--write-readme" {
            if inline.is_some() {
                return Err(GateError::usage(format!(
                    "argument --write-readme takes no value: {a:?}"
                )));
            }
            write_readme = true;
            i += 1;
            continue;
        }
        let slot = match key {
            "--log" => &mut log,
            "--readme" => &mut readme,
            "--script" => &mut script,
            _ => {
                return Err(GateError::usage(format!(
                    "unknown argument {a:?}; known: --log <path> --readme <path> --script <path> --write-readme"
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
        script,
        write_readme,
    })
}

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

    let readme_display = match &args.readme {
        Some(r) => r.clone(),
        // README.md lives one level above the engine root.
        None => {
            let base: PathBuf = ctx
                .root
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| ctx.root.clone());
            base.join("README.md").to_string_lossy().into_owned()
        }
    };
    let script_display = match &args.script {
        Some(s) => s.clone(),
        None => ctx
            .root
            .join("scripts")
            .join("check.sh")
            .to_string_lossy()
            .into_owned(),
    };

    let log_body = read_if_file(Path::new(&args.log))?;
    let readme_body = read_if_file(Path::new(&readme_display))?;
    let script_body = read_if_file(Path::new(&script_display))?;

    let (text, code, new_readme) = render(
        &args.log,
        log_body.as_deref(),
        &readme_display,
        readme_body.as_deref(),
        &script_display,
        script_body.as_deref(),
        args.write_readme,
    );

    // Write before reporting, so a failed write can never be reported as a PASS.
    if let Some(body) = new_readme {
        std::fs::write(Path::new(&readme_display), body).map_err(|e| {
            GateError::error(format!(
                "--write-readme: could not rewrite {readme_display}: {e}"
            ))
        })?;
    }

    print!("{text}");
    std::io::stdout().flush().ok();

    match code {
        crate::exit::OK => Ok(()),
        crate::exit::VIOLATION => Err(GateError::violation([
            "advertised check.sh step count disagrees with the run that measured it (detail above)"
                .to_string(),
        ])),
        _ => Err(GateError::error(
            "the check.sh step count could not be honestly compared (detail above)",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A specimen README with four step advertisement sites, mirroring the shape
    /// of the shipped one (badge label, badge path, TL;DR row, gate section).
    fn readme(n: &str) -> String {
        format!(
            "# Specimen\n\n\
             [![gate: {n} steps](https://img.shields.io/badge/gate-{n}_ordered_steps-success.svg)](#the-gate)\n\n\
             | **Gate** | {n} ordered steps; 9 selftest suites |\n\n\
             {n} steps, fail-closed, each naming the script that failed.\n"
        )
    }

    const SCRIPT: &str = concat!(
        "#!/usr/bin/env sh\n",
        "ok() { echo x; }\n",
        "ok \"one\"\n",
        "ok \"two\"\n",
        "# a comment mentioning ok \"three\" must not count\n",
        "# STEP-COUNT-RECEIPT-BOUNDARY\n",
        "printf 'CHECK_STEPS=%s' \"$N\"\n"
    );

    fn good_log(total: u64, ok: u64, skipped: u64, nested_ok: u64) -> String {
        format!("CHECK_STEPS={total} OK={ok} SKIPPED={skipped} NESTED_OK={nested_ok} DEPTH=0 RUN=pid1\n")
    }

    fn check(log: Option<&str>, rm: Option<&str>, sc: Option<&str>) -> (String, u8) {
        let (out, code, written) = render("L", log, "R", rm, "S", sc, false);
        assert!(written.is_none(), "no --write-readme, no rewrite");
        (out, code)
    }

    #[test]
    fn baseline_is_green() {
        let r = readme("2");
        let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::OK, "{out}");
        assert!(out.starts_with("PASS\n"), "{out}");
        assert!(out.contains("readme_claims=[2]"), "{out}");
        assert!(out.contains("step count GREEN"), "{out}");
    }

    #[test]
    fn a_missing_log_is_an_error_never_a_silent_zero() {
        let r = readme("2");
        let (out, code) = check(None, Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("step receipt log missing"), "{out}");
    }

    #[test]
    fn an_empty_log_is_an_error_not_a_pass() {
        let r = readme("2");
        for body in ["", "\n\n   \n"] {
            let (out, code) = check(Some(body), Some(&r), Some(SCRIPT));
            assert_eq!(code, crate::exit::ERROR, "{out}");
            assert!(out.contains("step receipt log is empty"), "{out}");
        }
    }

    #[test]
    fn zero_counted_steps_is_an_error_not_a_zero_to_zero_pass() {
        // The purest vacuous pass: a counter that returns 0 compared against a
        // README that says 0 would otherwise be GREEN.
        let r = readme("0");
        let (out, code) = check(Some(&good_log(0, 0, 0, 5)), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("counted ZERO ok steps"), "{out}");
    }

    #[test]
    fn zero_nested_receipts_is_an_error() {
        let r = readme("2");
        let (out, code) = check(Some(&good_log(2, 2, 0, 0)), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("NESTED_OK=0"), "{out}");
    }

    #[test]
    fn a_receipt_that_does_not_add_up_is_red() {
        let r = readme("9");
        let (out, code) = check(Some(&good_log(9, 2, 0, 5)), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::VIOLATION, "{out}");
        assert!(out.contains("receipt does not add up"), "{out}");
    }

    #[test]
    fn skipped_legs_still_count_toward_the_chain_length() {
        // One honest skip: the chain is still 2 steps long, so a machine without
        // the optional toolchain advertises the same number.
        let r = readme("2");
        let (out, code) = check(Some(&good_log(2, 1, 1, 5)), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::OK, "{out}");
    }

    #[test]
    fn a_nested_receipt_alone_is_an_error_never_a_fallback() {
        let log = "CHECK_STEPS=5 OK=5 SKIPPED=0 NESTED_OK=5 DEPTH=1 RUN=pid2\n";
        let r = readme("2");
        let (out, code) = check(Some(log), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("no DEPTH=0 receipt"), "{out}");
    }

    #[test]
    fn a_nested_receipt_beside_the_outer_one_is_ignored_not_summed() {
        // The contamination case, decided rather than hoped: the child reports 5,
        // the parent 2, and the answer is 2.
        let log = format!(
            "{}CHECK_STEPS=5 OK=5 SKIPPED=0 NESTED_OK=0 DEPTH=1 RUN=pid2\n",
            good_log(2, 2, 0, 5)
        );
        let r = readme("2");
        let (out, code) = check(Some(&log), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::OK, "{out}");
        assert!(out.contains("nested_receipts_ignored=1"), "{out}");
        assert!(out.contains("both say 2"), "{out}");
    }

    #[test]
    fn two_outer_receipts_are_an_error() {
        let log = format!("{}{}", good_log(2, 2, 0, 5), good_log(3, 3, 0, 5));
        let r = readme("2");
        let (out, code) = check(Some(&log), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("2 DEPTH=0 receipts"), "{out}");
    }

    #[test]
    fn an_unparseable_receipt_is_an_error() {
        for bad in [
            "CHECK_STEPS=2 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0",
            "STEPS=2 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=p",
            "CHECK_STEPS=x OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=p",
            "CHECK_STEPS=2 OK=2 NESTED_OK=5 SKIPPED=0 DEPTH=0 RUN=p",
        ] {
            let r = readme("2");
            let (out, code) = check(Some(bad), Some(&r), Some(SCRIPT));
            assert_eq!(code, crate::exit::ERROR, "{bad}: {out}");
            assert!(out.contains("unparseable receipt line"), "{bad}: {out}");
        }
    }

    #[test]
    fn readme_drift_is_red_in_both_directions() {
        for advertised in ["1", "3"] {
            let r = readme(advertised);
            let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&r), Some(SCRIPT));
            assert_eq!(code, crate::exit::VIOLATION, "{out}");
            assert!(
                out.contains(&format!("advertises {advertised} check.sh steps")),
                "{out}"
            );
            assert!(out.contains("this run measured 2"), "{out}");
        }
    }

    #[test]
    fn a_readme_advertising_nothing_is_an_error() {
        let (out, code) = check(
            Some(&good_log(2, 2, 0, 5)),
            Some("# nothing advertised here\n"),
            Some(SCRIPT),
        );
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("advertises no check.sh step count"), "{out}");
    }

    #[test]
    fn losing_one_advertisement_site_trips_the_floor_not_drift() {
        let thin: String = readme("2")
            .lines()
            .filter(|l| !l.starts_with("2 steps, fail-closed"))
            .map(|l| format!("{l}\n"))
            .collect();
        let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&thin), Some(SCRIPT));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("only 3 step advertisement site(s)"), "{out}");
    }

    #[test]
    fn a_word_spelled_site_stays_under_the_gate() {
        let drifted = readme("2").replace("2 steps, fail-closed", "thirty-six steps, fail-closed");
        let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&drifted), Some(SCRIPT));
        assert_eq!(code, crate::exit::VIOLATION, "{out}");
        assert!(out.contains("advertises 36 check.sh steps"), "{out}");

        let agreeing = readme("2").replace("2 steps, fail-closed", "two steps, fail-closed");
        let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&agreeing), Some(SCRIPT));
        assert_eq!(code, crate::exit::OK, "{out}");
    }

    #[test]
    fn an_ok_call_after_the_boundary_is_red() {
        let bad = format!("{SCRIPT}ok \"added after the receipt\"\n");
        let r = readme("2");
        let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&r), Some(&bad));
        assert_eq!(code, crate::exit::VIOLATION, "{out}");
        assert!(
            out.contains("after the STEP-COUNT-RECEIPT-BOUNDARY"),
            "{out}"
        );
    }

    #[test]
    fn prose_about_the_marker_is_not_the_marker() {
        // Measured 2026-08-14: the first draft matched the bare substring, so
        // check.sh's own comment explaining the boundary counted as a second
        // boundary and the gate reported "2 markers" against a correct script. A
        // scanner that its own documentation can break will be broken by the next
        // person who documents it.
        let prose = concat!(
            "#!/usr/bin/env sh\n",
            "# the receipt is sealed at STEP-COUNT-RECEIPT-BOUNDARY near the end\n",
            "ok \"one\"\n",
            "echo 'see STEP-COUNT-RECEIPT-BOUNDARY for where this is sealed'\n",
            "#   STEP-COUNT-RECEIPT-BOUNDARY [bd-1sd.13]\n",
            "printf 'CHECK_STEPS=%s' \"$N\"\n"
        );
        assert_eq!(
            boundary_lines(prose),
            vec![5],
            "only the comment line whose FIRST token is the marker is the boundary"
        );
        let r = readme("1");
        let (out, code) = check(Some(&good_log(1, 1, 0, 5)), Some(&r), Some(prose));
        assert_eq!(code, crate::exit::OK, "{out}");
    }

    #[test]
    fn a_script_without_the_boundary_is_an_error() {
        let bad = "#!/usr/bin/env sh\nok \"one\"\n";
        let r = readme("1");
        let (out, code) = check(Some(&good_log(1, 1, 0, 5)), Some(&r), Some(bad));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(
            out.contains("carries no STEP-COUNT-RECEIPT-BOUNDARY"),
            "{out}"
        );
    }

    #[test]
    fn a_boundary_with_no_emission_after_it_is_an_error() {
        let bad = "#!/usr/bin/env sh\nok \"one\"\n# STEP-COUNT-RECEIPT-BOUNDARY\necho done\n";
        let r = readme("1");
        let (out, code) = check(Some(&good_log(1, 1, 0, 5)), Some(&r), Some(bad));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(
            out.contains("emits no CHECK_STEPS= receipt after it"),
            "{out}"
        );
    }

    #[test]
    fn a_script_with_no_ok_sites_is_a_vacuous_scan_error() {
        let bad = "#!/usr/bin/env sh\n# STEP-COUNT-RECEIPT-BOUNDARY\nprintf 'CHECK_STEPS=0'\n";
        let r = readme("2");
        let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&r), Some(bad));
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("zero `ok \"…\"` call sites"), "{out}");
    }

    #[test]
    fn counting_more_receipts_than_the_script_has_sites_is_red() {
        // The static falsification of "the parent counted the child's lines".
        let r = readme("7");
        let (out, code) = check(Some(&good_log(7, 7, 0, 5)), Some(&r), Some(SCRIPT));
        assert_eq!(code, crate::exit::VIOLATION, "{out}");
        assert!(out.contains("holds only 2 `ok` call site(s)"), "{out}");
    }

    #[test]
    fn a_missing_script_is_an_error() {
        let r = readme("2");
        let (out, code) = check(Some(&good_log(2, 2, 0, 5)), Some(&r), None);
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(out.contains("check.sh missing or unreadable"), "{out}");
    }

    #[test]
    fn write_readme_regenerates_every_site_from_the_receipt() {
        let before = readme("9");
        let (out, code, written) = render(
            "L",
            Some(&good_log(2, 2, 0, 5)),
            "R",
            Some(&before),
            "S",
            Some(SCRIPT),
            true,
        );
        assert_eq!(code, crate::exit::OK, "{out}");
        let after = written.expect("a drifted README is rewritten");
        assert_eq!(after, readme("2"));
        assert!(out.contains("4 site(s) now advertise 2"), "{out}");
    }

    #[test]
    fn write_readme_refuses_an_unsound_receipt() {
        let before = readme("9");
        let (out, code, written) = render(
            "L",
            Some("CHECK_STEPS=2 OK=0 SKIPPED=2 NESTED_OK=5 DEPTH=0 RUN=p"),
            "R",
            Some(&before),
            "S",
            Some(SCRIPT),
            true,
        );
        assert_eq!(code, crate::exit::ERROR, "{out}");
        assert!(written.is_none(), "an unsound total must not be written");
        assert!(out.contains("regeneration SKIPPED"), "{out}");
    }

    #[test]
    fn the_shipped_readme_and_script_are_the_ones_this_gate_reads() {
        // Anti-vacuous at the wiring level: the gate's defaults must resolve to
        // real files in THIS repo, not to plausible paths.
        let root = crate::root::resolve(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("engine root");
        let script = root.join("scripts/check.sh");
        let text = std::fs::read_to_string(&script).expect("scripts/check.sh must exist");
        assert_eq!(
            boundary_lines(&text).len(),
            1,
            "scripts/check.sh must carry exactly one {BOUNDARY_MARKER}"
        );
        assert!(
            ok_call_lines(&text).len() > 20,
            "scripts/check.sh scanned to {} ok sites — a vacuous scan",
            ok_call_lines(&text).len()
        );
        let readme = root.parent().expect("repo root").join("README.md");
        let rtext = std::fs::read_to_string(&readme).expect("README.md must exist");
        let sites: usize = rtext.lines().map(|l| scan_step_spans(l).len()).sum();
        assert!(
            sites >= MIN_STEP_SITES,
            "README.md advertises the step count at {sites} site(s); the floor is {MIN_STEP_SITES}"
        );
    }
}
