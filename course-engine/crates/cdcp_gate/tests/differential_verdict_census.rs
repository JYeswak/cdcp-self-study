//! META-GATE over the differential harnesses and the gate roster.
//!
//! ─── WHY THIS FILE EXISTS ────────────────────────────────────────────────────
//!
//! A differential harness proves that two implementations AGREE. Agreement is a
//! NECESSARY condition for a correct port. It is NOT a SUFFICIENT condition for
//! a correct gate, because a defect present in BOTH sides agrees with itself.
//! Every case that asserts only agreement is silent about whether the shared
//! behaviour is right.
//!
//! Measured 2026-08-14 in `verify_objectives`:
//!
//!     PASS ... primary_topics=106 covered=106 shortfalls=0 min_per_topic=0
//!              mode=strict ... EXIT 0
//!
//! 106 of 106 topics reported covered, in STRICT mode, with not one comparison
//! performed. `diff_verify_objectives.rs` had been running that exact case
//! (`m min topic 0`) and PASSING it for weeks. The harness header already said
//! "a defect faithfully ported is still a defect"; the defect landed in the
//! harness's own blind spot.
//!
//! ─── THE CENSUS, MEASURED 2026-08-14 ────────────────────────────────────────
//!
//! Derived by reading the code, not estimated. A CASE is one invocation, from
//! inside a `#[test]` fn, of the harness's own byte-comparison helper. A case
//! carries an INDEPENDENT VERDICT if it also asserts what the shared output IS
//! — a claim that would still be checkable if only ONE implementation existed.
//!
//!   harness                          cases  indep  agreement-only   blind
//!   ---------------------------------------------------------------------
//!   diff_build_glossary_json.rs          9      8         1         11.1%
//!   diff_build_units.rs                 14     13         1          7.1%
//!   diff_validate_grounding.rs          35     33         2          5.7%
//!   diff_verify_bank.rs                 31     31         0          0.0%
//!   diff_verify_content_lock.rs         31     31         0          0.0%
//!   diff_verify_coverage.rs             42     33         9         21.4%
//!   diff_verify_doc_consistency.rs      28     27         1          3.6%
//!   diff_verify_injection_count.rs      31     26         5         16.1%
//!   diff_verify_knowledge_paths.rs      25     25         0          0.0%
//!   diff_verify_objectives.rs           54     44        10         18.5%
//!   diff_verify_orphans.rs              26     19         7         26.9%
//!   ---------------------------------------------------------------------
//!   OVERALL                            326    290        36         11.0%
//!
//! AFTER THIS WAVE, re-measured the same evening: 28 of 320. Two harnesses
//! moved. `diff_verify_orphans.rs` went 26/7 to 23/0 by the conversion recipe
//! below. `diff_build_units.rs` went to zero differential cases at all — its
//! owner converted it to direct tests of the Rust, which is the retirement
//! policy working.
//!
//! Re-measured 2026-08-15 after bd-diff-coverage-agreement-only-25ux:
//! coverage went 9 agreement-only to 0. The ratchet is set to 18, the
//! measured total, not 28 with headroom: a ratchet with slack is not a
//! ratchet. The CHARTER mutate/delete pair that proves the budget assert is
//! load-bearing lives in `census_charter_pair.rs` (bd-census-mutation-pair-unproven-pi0v).
//!
//! THE SHAPE OF THE 36 IS THE FINDING, NOT THE COUNT. They are not spread
//! evenly. Twenty-three of them — nearly two thirds — sit in one test per
//! harness, `path_and_option_shapes_are_byte_identical`, which was built to
//! check ARGUMENT PARSING and then quietly accumulated cases that change the
//! gate's SEMANTICS. `m min topic 0` was filed there. So were `m strict flag`,
//! `m skip flag`, `m both flags`. A semantic case in an option-shape costume
//! gets an option-shape assertion, which is agreement and nothing else.
//!
//! Five more are the `the_harness_compared_something` self-checks, which are
//! agreement-only on purpose — their job is to increment a counter.
//!
//! A SECOND AXIS, worth more than the first for the retirement decision:
//! of the 290 cases that do carry a verdict, 52 assert ONLY the exit code and
//! not one byte of what the gate said. So 88 of 326 cases (27.0%) hold no
//! independent claim about output at all. (Four of the 52 — the emission-order
//! cases in coverage and objectives — assert content through derived offsets
//! rather than a substring and are strong in substance; the machine count is
//! left at 52 rather than quietly adjusted.)
//!
//! Twenty-six of the 52 are in `diff_verify_injection_count.rs`, and they
//! cannot be otherwise: that harness's comparator returns `i32`, so stdout is
//! unreachable from every call site in the file. Zero of its 31 cases can
//! assert what the gate said. Tracked as bd-diff-injection-i32-comparator-fo1l.
//!
//! ─── WHAT THIS MEANS FOR RETIRING THE ORACLES ───────────────────────────────
//!
//! An oracle's job is to prove the port FAITHFUL, and that job completes at
//! port time; the standing policy is to retire it then — change the Rust,
//! delete the .py, convert the differential cases into direct tests that assert
//! the verdict.
//!
//! RETIREMENT HAS TWO INDEPENDENT BLOCKERS. (A) Is the oracle still invoked? If
//! check.sh still runs it, it cannot be retired at any price. (B) What does
//! retiring cost? That is the `agreement-only` column above.
//!
//! BLOCKER A, MEASURED TRANSITIVELY 2026-08-14 — and note that
//! `grep python3 check.sh` DOES NOT ANSWER IT: check.sh calls other shell
//! scripts and THOSE invoke the oracles, and two more hide behind
//! `python3 "$CHECKER"`, which no grep for a .py filename can see. Six oracles
//! are still invoked: verify_orphans (check.sh:508 -> selftest_orphan.sh),
//! verify_objectives (:714), verify_coverage (:683), verify_bank
//! (:719 -> smoke_slo.sh), and — behind $CHECKER — verify_doc_consistency
//! (:767) and verify_injection_count (:802). validate_grounding has only a `-f`
//! presence check at :493. verify_content_lock, verify_knowledge_paths and
//! build_glossary_json are referenced by no script. build_units.py is gone.
//!
//! READING THE TWO TOGETHER: retirable TODAY at zero cost — verify_content_lock
//! and verify_knowledge_paths (0 agreement-only, never invoked). Blocked on the
//! INVOCATION only, their harnesses losing nothing — verify_bank and, since
//! this wave, verify_orphans. Everything else is blocked on both.
//!
//! `agreement_only` is exactly the number of cases that EVAPORATE under
//! conversion. A case that says only "the two sides agree" says nothing at all
//! when there is only one side. So the column is a conversion cost sheet:
//!
//!   * 0 agreement-only  -> the oracle can be retired today, losing nothing.
//!     (verify_bank, verify_content_lock, verify_knowledge_paths.)
//!   * n agreement-only  -> retiring costs n cases unless they are converted
//!     first, and CONVERSION IS NOT REWRITING THE ASSERTION. Every one of the
//!     23 option-shape cases runs against the GREEN live tree. Asserting "this
//!     spelling still passes" on a tree that passes is worth nothing: no
//!     suppression defect can show up in it. The conversion must change the
//!     INPUT to one whose verdict is non-trivial — run the spelling against a
//!     tree with a KNOWN planted finding and assert the spelling still reaches
//!     it. That is done for `diff_verify_orphans.rs` in this wave and is the
//!     recipe for the rest.
//!   * harnesses that are ALL agreement-only: NONE. Measured, and reported as a
//!     measured zero rather than passed over in silence.
//!
//! ─── WHAT THIS FILE ENFORCES, AND WHAT IT DELIBERATELY DOES NOT ─────────────
//!
//! LEG A  the census exists at all: diff_*.rs discovered from the TREE, every
//!        discovered file registered, the registry itself above its floor. An
//!        empty discovery is an ERROR unless retirement is DECLARED.
//! LEG B  THE RATCHET: total agreement-only cases may fall and may never rise.
//!        Enforced on the TOTAL, not per harness, and that is a deliberate
//!        choice — four of these files were owned by other agents mid-wave, and
//!        a gate that goes red on a file its owner may not touch gets disabled.
//!        A disabled detector is worth LESS than none, because it also carries
//!        the belief that the class is covered. Per-harness numbers are printed
//!        every run; the total is what fails the build.
//! LEG C  every gate the dispatcher lists carries a registry row, so a new gate
//!        cannot ship unregistered.
//! LEG D  bd-j8b2's verdict-shape leg, moved off the harnesses and onto the
//!        GATES: for every gate, on an empty root and (where safe) on the live
//!        root, `exit == 0` is EQUIVALENT to "stdout carries this gate's
//!        success token". Proven by execution, not by grep — bd-0czh already
//!        measured a message-keyed scan being fooled by the same string that
//!        fooled the auditor.
//! LEG E  anti-vacuous on the empty root: no gate may pass over an empty tree.
//! LEG F  anti-vacuous on the PROBE: at least one gate must be observed GREEN
//!        carrying its token, or leg D's token side never ran.
//! LEG G  a gate registered token-free really is: neither probe may print any
//!        word from the known verdict vocabulary.
//! LEG H  the classifier is proven to trip on a planted known-bad, and proven
//!        NOT to trip on the three known-GOOD shapes around it.
//!
//! NOT ENFORCED, on purpose: that each harness carries its own per-case leg.
//! bd-j8b2 proposed that and measured 3 of 11 carrying it. The leg is per-CASE
//! and unbounded; the defect is per-GATE and there are nineteen gates. Checking
//! the gates covers every case at once and needs no edit to any file another
//! agent owns.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn registry_path() -> PathBuf {
    engine_root().join("registries/differential_harnesses.toml")
}

// ── the registry ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Registry {
    schema_version: u32,
    census: Census,
    probe: Probe,
    harness: Vec<HarnessRow>,
    gate: Vec<GateRow>,
}

#[derive(Deserialize)]
struct Census {
    registered_harness_floor: usize,
    active_harness_floor: usize,
    agreement_only_total_budget: usize,
    all_harnesses_retired: bool,
    all_harnesses_retired_reason: String,
}

#[derive(Deserialize)]
struct Probe {
    green_with_token_floor: usize,
    token_vocabulary: Vec<String>,
}

#[derive(Deserialize)]
struct HarnessRow {
    file: String,
    #[allow(dead_code)]
    gate: String,
    comparators: Vec<String>,
    #[allow(dead_code)]
    cases: usize,
    agreement_only: usize,
    #[serde(default)]
    owner_bead: Option<String>,
}

#[derive(Deserialize)]
struct GateRow {
    name: String,
    #[serde(default)]
    success_token: Option<String>,
    #[serde(default)]
    token_free: bool,
    #[serde(default)]
    token_free_reason: Option<String>,
    #[serde(default = "yes")]
    live_probe: bool,
    #[serde(default)]
    live_probe_skip_reason: Option<String>,
    #[serde(default = "yes")]
    empty_probe: bool,
    #[serde(default)]
    empty_probe_skip_reason: Option<String>,
}

fn yes() -> bool {
    true
}

fn registry() -> Registry {
    let p = registry_path();
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| {
        panic!(
            "{}: the census registry is REQUIRED ({e}). A meta-gate that cannot \
             read its registry must fail, never skip.",
            p.display()
        )
    });
    let r: Registry = toml::from_str(&body)
        .unwrap_or_else(|e| panic!("{}: registry does not parse: {e}", p.display()));
    assert_eq!(r.schema_version, 1, "unknown registry schema_version");
    r
}

// ── the census parser ──────────────────────────────────────────────────────
//
// Rust source is scanned with comments and string literals blanked to spaces of
// the same length, so a `;` inside a comment or a `"` inside a message cannot
// move a statement boundary. That is not a nicety: the first cut of this
// analysis misread four call sites in diff_verify_orphans.rs because a bead id
// written `(bd-9nyt);` inside a doc comment looked like the end of the previous
// statement.

fn blank(src: &str) -> String {
    let b = src.as_bytes();
    let mut out: Vec<u8> = b.to_vec();
    let mut i = 0usize;
    let blankout = |out: &mut Vec<u8>, k: usize| {
        if out[k] != b'\n' {
            out[k] = b' ';
        }
    };
    while i < b.len() {
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                blankout(&mut out, i);
                i += 1;
            }
        } else if b[i..].starts_with(b"r#\"") {
            let end = src[i + 3..]
                .find("\"#")
                .map(|k| i + 3 + k + 2)
                .unwrap_or(b.len());
            for k in i..end {
                blankout(&mut out, k);
            }
            i = end;
        } else if b[i] == b'"' {
            blankout(&mut out, i);
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    blankout(&mut out, i);
                    i += 1;
                }
                if i < b.len() {
                    blankout(&mut out, i);
                    i += 1;
                }
            }
            if i < b.len() {
                blankout(&mut out, i);
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("blanking preserves utf8 boundaries")
}

/// Byte span of every top-level `fn`, and whether it is a `#[test]`.
fn fn_spans(b: &str) -> Vec<(bool, usize, usize)> {
    let bytes = b.as_bytes();
    let mut out = Vec::new();
    let mut idx = 0usize;
    for line_start in line_starts(b) {
        if line_start < idx {
            continue;
        }
        let rest = &b[line_start..];
        let is_fn = rest.starts_with("fn ") || rest.starts_with("pub fn ");
        if !is_fn {
            continue;
        }
        let head = &b[..line_start];
        let is_test = head
            .trim_end()
            .lines()
            .rev()
            .take(3)
            .any(|l| l.contains("#[test]"));
        let Some(open) = b[line_start..].find('{').map(|k| line_start + k) else {
            continue;
        };
        let mut depth = 0i32;
        let mut j = open;
        while j < bytes.len() {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        out.push((is_test, line_start, j));
        idx = j;
    }
    out
}

fn line_starts(s: &str) -> Vec<usize> {
    let mut v = vec![0usize];
    for (i, c) in s.char_indices() {
        if c == '\n' {
            v.push(i + 1);
        }
    }
    v
}

fn is_ident_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Index just past the `)` matching the `(` that ended at `from`.
fn match_paren(b: &str, from: usize) -> usize {
    let bytes = b.as_bytes();
    let mut depth = 1i32;
    let mut j = from;
    while j < bytes.len() && depth > 0 {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        j += 1;
    }
    j
}

/// Every `name(` occurrence in the blanked source that is a *call*, not the
/// definition, and sits inside a `#[test]` fn.
fn call_sites(b: &str, names: &[String]) -> Vec<(usize, usize)> {
    let spans = fn_spans(b);
    let mut hits: Vec<(usize, usize)> = Vec::new();
    for name in names {
        let mut from = 0usize;
        while let Some(k) = b[from..].find(name.as_str()) {
            let s = from + k;
            from = s + 1;
            let after = s + name.len();
            if s > 0 && is_ident_byte(b.as_bytes()[s - 1]) {
                continue;
            }
            let rest = b[after..].trim_start();
            if !rest.starts_with('(') {
                continue;
            }
            let open = after + b[after..].find('(').expect("checked") + 1;
            // definition, not a call?
            let line_start = b[..s].rfind('\n').map(|k| k + 1).unwrap_or(0);
            let line = b[line_start..s].trim_start();
            if line.starts_with("fn ") || line.starts_with("pub fn ") {
                continue;
            }
            // inside a #[test] fn?
            if !spans.iter().any(|(t, a, z)| *t && *a <= s && s <= *z) {
                continue;
            }
            hits.push((s, open));
        }
    }
    hits.sort_unstable();
    // an inner name (`assert_identical`) can match inside an outer one
    // (`assert_identical_env`); keep the outermost of any overlap.
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (s, open) in hits {
        if out.last().is_some_and(|(_, o)| s < *o) {
            continue;
        }
        out.push((s, open));
    }
    out
}

#[derive(Debug)]
struct CaseCensus {
    cases: usize,
    agreement_only: usize,
    agreement_only_lines: Vec<usize>,
}

fn census_of(src: &str, comparators: &[String]) -> CaseCensus {
    let b = blank(src);
    let spans = fn_spans(&b);
    let calls = call_sites(&b, comparators);
    let mut agreement_only_lines = Vec::new();

    for (i, (s, open)) in calls.iter().enumerate() {
        let end = match_paren(&b, *open);
        let stmt_start = [b[..*s].rfind(';'), b[..*s].rfind('{'), b[..*s].rfind('}')]
            .into_iter()
            .flatten()
            .max()
            .map(|k| k + 1)
            .unwrap_or(0);
        let prefix = b[stmt_start..*s].trim();

        // the call is itself the asserted expression: assert_eq!(check(..), 1)
        if prefix.contains("assert") {
            continue;
        }

        // bound to a name, and something later asserts on that name (or on a
        // value derived from it)?
        let Some(binder) = binder_of(prefix) else {
            agreement_only_lines.push(line_of(src, *s));
            continue;
        };
        let win_end = calls
            .get(i + 1)
            .map(|(n, _)| *n)
            .or_else(|| {
                spans
                    .iter()
                    .find(|(_, a, z)| *a <= *s && *s <= *z)
                    .map(|(_, _, z)| *z)
            })
            .unwrap_or(b.len());
        if !asserts_on(&b[end..win_end], binder) {
            agreement_only_lines.push(line_of(src, *s));
        }
    }

    CaseCensus {
        cases: calls.len(),
        agreement_only: agreement_only_lines.len(),
        agreement_only_lines,
    }
}

fn line_of(src: &str, byte: usize) -> usize {
    src[..byte].matches('\n').count() + 1
}

fn binder_of(prefix: &str) -> Option<&str> {
    let rest = prefix.strip_prefix("let ")?;
    let rest = rest.strip_prefix("mut ").unwrap_or(rest);
    let name: &str = rest.split(|c: char| !is_ident_byte(c as u8)).next()?;
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn mentions(hay: &str, ident: &str) -> bool {
    let hb = hay.as_bytes();
    let mut from = 0usize;
    while let Some(k) = hay[from..].find(ident) {
        let s = from + k;
        from = s + 1;
        let before_ok = s == 0 || !is_ident_byte(hb[s - 1]);
        let e = s + ident.len();
        let after_ok = e >= hb.len() || !is_ident_byte(hb[e]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Does anything in `window` assert on `binder`, or on a value one or two
/// `let`s downstream of it? `let body = rs.body(); assert!(body.contains(..))`
/// is an independent verdict and must not be miscounted as blind.
fn asserts_on(window: &str, binder: &str) -> bool {
    let mut alias: BTreeSet<String> = BTreeSet::new();
    alias.insert(binder.to_string());
    for _ in 0..3 {
        let mut from = 0usize;
        while let Some(k) = window[from..].find("let ") {
            let s = from + k + 4;
            from = s;
            let Some(semi) = window[s..].find(';').map(|x| s + x) else {
                break;
            };
            let Some(eq) = window[s..semi].find('=').map(|x| s + x) else {
                continue;
            };
            let name: &str = window[s..eq]
                .trim()
                .trim_start_matches("mut ")
                .split(|c: char| !is_ident_byte(c as u8))
                .next()
                .unwrap_or("");
            let rhs = &window[eq + 1..semi];
            if !name.is_empty() && alias.iter().any(|a| mentions(rhs, a)) {
                alias.insert(name.to_string());
            }
        }
    }
    let mut from = 0usize;
    while let Some(k) = window[from..].find("assert") {
        let s = from + k;
        from = s + 1;
        let Some(open) = window[s..].find('(').map(|x| s + x + 1) else {
            continue;
        };
        if window[s..open].contains(';') {
            continue;
        }
        let close = match_paren(window, open);
        let body = &window[open..close.min(window.len())];
        if alias.iter().any(|a| mentions(body, a)) {
            return true;
        }
    }
    false
}

// ── LEG A + LEG B: discovery, registration, and the ratchet ────────────────

fn discovered_harnesses() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(tests_dir())
        .expect("tests dir")
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            (n.starts_with("diff_") && n.ends_with(".rs")).then_some(n)
        })
        .collect();
    v.sort();
    v
}

#[test]
fn the_census_discovers_harnesses_and_every_one_is_registered() {
    let reg = registry();
    let found = discovered_harnesses();
    let registered: BTreeSet<&str> = reg.harness.iter().map(|h| h.file.as_str()).collect();

    assert!(
        reg.harness.len() >= reg.census.registered_harness_floor,
        "the registry holds {} harness rows, below its floor of {}. Rows are \
         MONOTONE — a retired harness keeps its row — so a count below the floor \
         means rows were deleted, and a census that forgot what it used to \
         measure cannot tell retirement from erasure.",
        reg.harness.len(),
        reg.census.registered_harness_floor
    );

    if found.is_empty() {
        assert!(
            reg.census.all_harnesses_retired && !reg.census.all_harnesses_retired_reason.is_empty(),
            "ZERO differential harnesses found in {}. An empty scan set is an \
             ERROR, never a pass. If every harness really has been retired, that \
             end state must be DECLARED in the registry \
             (all_harnesses_retired = true with a reason), not inferred from an \
             empty directory listing.",
            tests_dir().display()
        );
        return;
    }

    for f in &found {
        assert!(
            registered.contains(f.as_str()),
            "{f} is a differential harness in the tree with no row in {}. A new \
             harness must be registered before it can be censused, or the blind \
             spot grows unmeasured.",
            registry_path().display()
        );
    }

    // rows whose file is gone are RETIRED, which is now policy, not a defect.
    let retired: Vec<&str> = reg
        .harness
        .iter()
        .map(|h| h.file.as_str())
        .filter(|f| !found.iter().any(|g| g == f))
        .collect();
    println!("[census] harnesses in tree: {}", found.len());
    println!("[census] rows retired (file gone): {retired:?}");
}

#[test]
fn the_agreement_only_ratchet_holds() {
    let reg = registry();
    let found = discovered_harnesses();
    if found.is_empty() {
        return; // covered, with its reason, by the discovery test above
    }

    let mut total = 0usize;
    let mut active = 0usize;
    let mut per: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();

    for row in &reg.harness {
        let path = tests_dir().join(&row.file);
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue; // retired; reported by the discovery test
        };
        let c = census_of(&src, &row.comparators);
        if c.cases == 0 {
            // The harness no longer performs a differential comparison — it has
            // been converted to direct tests. That is the retirement policy
            // working. It must not, however, have become an empty file: a
            // converted harness that asserts nothing reports exactly like one
            // that asserted everything.
            let tests = src.matches("#[test]").count();
            let asserts = src.matches("assert").count();
            assert!(
                tests > 0 && asserts > 0,
                "{} holds zero differential cases AND zero tests/assertions \
                 ({tests} tests, {asserts} assertions). A harness that compares \
                 nothing and asserts nothing is a vacuous pass, not a conversion.",
                row.file
            );
            println!(
                "[census] {:38} CONVERTED (0 differential cases, {tests} tests)",
                row.file
            );
            continue;
        }
        active += 1;
        total += c.agreement_only;
        let owner = row.owner_bead.clone().unwrap_or_default();
        println!(
            "[census] {:38} cases={:3} agreement_only={:2} (budget {:2}) {} {}",
            row.file,
            c.cases,
            c.agreement_only,
            row.agreement_only,
            if c.agreement_only > row.agreement_only {
                "OVER"
            } else {
                ""
            },
            if owner.is_empty() {
                String::new()
            } else {
                format!("owner={owner}")
            }
        );
        if !c.agreement_only_lines.is_empty() {
            println!("           blind at lines {:?}", c.agreement_only_lines);
        }
        per.insert(
            row.file.clone(),
            (c.cases, c.agreement_only, row.agreement_only),
        );
    }

    assert!(
        active >= reg.census.active_harness_floor,
        "only {active} active differential harnesses were censused, below the \
         floor of {}. A census that measured nothing is an ERROR.",
        reg.census.active_harness_floor
    );

    println!(
        "[census] TOTAL agreement-only across {active} active harnesses: {total} \
         (budget {})",
        reg.census.agreement_only_total_budget
    );

    assert!(
        total <= reg.census.agreement_only_total_budget,
        "THE RATCHET SLIPPED. {total} agreement-only cases across the tree, above \
         the budget of {}. Agreement is necessary for a correct port and is NOT \
         sufficient for a correct gate: every case in this count is one that no \
         shared defect can make fail, and one that evaporates entirely when its \
         oracle is retired. The budget may fall; it may not rise. Per harness: \
         {per:?}",
        reg.census.agreement_only_total_budget
    );

    if total < reg.census.agreement_only_total_budget {
        println!(
            "[census] RATCHET SLACK: measured {total} < budget {}. Lower \
             agreement_only_total_budget in {} to {total} so the number stays honest.",
            reg.census.agreement_only_total_budget,
            registry_path().display()
        );
    }
}

// ── LEG H: the classifier is proven to trip, and proven not to over-trip ───

#[test]
fn the_census_parser_catches_a_planted_agreement_only_case() {
    let comparators = vec!["compare".to_string()];

    // known-BAD: a bare call. Nothing about the shared output is asserted.
    let bad = r#"
#[test]
fn t() {
    compare("a", &f);
}
"#;
    assert_eq!(
        census_of(bad, &comparators).agreement_only,
        1,
        "planted bare call not caught"
    );

    // known-BAD, subtler: the result is bound and then never asserted on.
    let bound_unused = r#"
#[test]
fn t() {
    let rs = compare("a", &f);
    assert!(other.is_empty());
}
"#;
    assert_eq!(
        census_of(bound_unused, &comparators).agreement_only,
        1,
        "a bound-but-never-asserted result is agreement-only too"
    );

    // known-GOOD 1: a direct verdict on the result.
    let good_direct = r#"
#[test]
fn t() {
    let rs = compare("a", &f);
    assert_ne!(rs.code, 0, "{}", rs.out());
}
"#;
    assert_eq!(census_of(good_direct, &comparators).agreement_only, 0);

    // known-GOOD 2: the verdict is asserted one hop downstream. This is the leg
    // that an over-strict census gets wrong, and an over-strict meta-gate over
    // eleven harnesses is one that gets routed around.
    let good_alias = r#"
#[test]
fn t() {
    let rs = compare("a", &f);
    let body = rs.body();
    assert!(body.contains("x"), "{body}");
}
"#;
    assert_eq!(census_of(good_alias, &comparators).agreement_only, 0);

    // known-GOOD 3: the call is the asserted expression.
    let good_inline = r#"
#[test]
fn t() {
    assert_eq!(compare("a", &f), 1, "must be RED");
}
"#;
    assert_eq!(census_of(good_inline, &comparators).agreement_only, 0);

    // known-GOOD 4: a `;` inside a comment must not move the statement
    // boundary. Measured: this exact shape misread four call sites.
    let good_comment = r#"
#[test]
fn t() {
    // the floor case (bd-9nyt); the verdict set lives elsewhere
    let rs = compare("a", &f);
    assert_ne!(rs.code, 0);
}
"#;
    assert_eq!(
        census_of(good_comment, &comparators).agreement_only,
        0,
        "a semicolon inside a comment moved the statement boundary"
    );

    // anti-vacuous on the parser itself: it must find the cases it classifies.
    assert_eq!(census_of(good_direct, &comparators).cases, 1);
    assert_eq!(census_of("fn compare() {}\n", &comparators).cases, 0);
}

// ── LEG C..G: the verdict-shape invariant, checked against the GATES ───────

struct ProbeRun {
    code: i32,
    stdout: String,
}

fn run_gate(root: &Path, gate: &str) -> ProbeRun {
    let out = Command::new(BIN)
        .current_dir(root)
        .arg("--root")
        .arg(root)
        .arg(gate)
        .output()
        .unwrap_or_else(|e| panic!("run {gate}: {e}"));
    ProbeRun {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
    }
}

/// The whole of bd-j8b2's leg, as one function, so it can be proven to trip.
///
/// A success token counts only as a LINE PREFIX. A gate that mentions "PASS"
/// inside a failure message has not claimed to pass; a gate that opens a line
/// with it has.
fn carries_success_token(stdout: &str, token: &str) -> bool {
    stdout.lines().any(|l| l.starts_with(token))
}

fn verdict_shape_violation(code: i32, stdout: &str, token: &str) -> bool {
    carries_success_token(stdout, token) != (code == 0)
}

#[test]
fn the_verdict_shape_detector_trips_on_a_planted_known_bad() {
    // known-BAD: the exact class bd-lt7 / qm65 / rk9n each patched by hand — a
    // success token printed on a path that returns non-zero.
    assert!(
        verdict_shape_violation(1, "PASS\n  items=0\n", "PASS"),
        "a PASS printed under exit 1 is the defect this leg exists to catch"
    );
    // known-BAD, the other direction: a green run that never says so, which is
    // how a reader (and a downstream grep) reads a pass as a failure.
    assert!(verdict_shape_violation(0, "FAIL\n", "PASS"));
    // known-GOOD: both correct shapes.
    assert!(!verdict_shape_violation(0, "PASS\n  items=804\n", "PASS"));
    assert!(!verdict_shape_violation(
        1,
        "FAIL\n  - orphan topic\n",
        "PASS"
    ));
    // known-GOOD: the token named inside a failure line is not a claim to pass.
    assert!(
        !verdict_shape_violation(1, "FAIL\n  - expected PASS, got FAIL\n", "PASS"),
        "a message-keyed check would fire here; a line-prefix check must not"
    );
}

#[test]
fn every_dispatched_gate_is_registered_and_holds_the_verdict_shape() {
    let reg = registry();
    let root = engine_root();

    // LEG C: the roster comes from the BINARY, so a new gate cannot slip in.
    let listed = Command::new(BIN)
        .arg("list")
        .output()
        .expect("cdcp_gate list");
    assert_eq!(listed.status.code(), Some(0), "cdcp_gate list failed");
    let roster: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter_map(|l| l.split('\t').next())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    assert!(
        !roster.is_empty(),
        "cdcp_gate list named zero gates. A meta-gate over an empty roster is a \
         vacuous pass; an empty input set is an ERROR."
    );
    let rows: BTreeMap<&str, &GateRow> = reg.gate.iter().map(|g| (g.name.as_str(), g)).collect();
    for g in &roster {
        assert!(
            rows.contains_key(g.as_str()),
            "gate `{g}` is dispatched but has no row in {}. Its success token is \
             what the verdict-shape leg is keyed on, and a token cannot be \
             guessed from a literal — that is the message-keyed scan bd-j8b2 \
             rejected.",
            registry_path().display()
        );
    }

    let empty = tempfile::tempdir().expect("tempdir");
    let mut green_with_token = 0usize;
    let mut probes = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for name in &roster {
        let row = rows[name.as_str()];

        // every skip is a registered decision with a reason, never a default
        if row.token_free {
            assert!(
                row.token_free_reason
                    .as_deref()
                    .is_some_and(|r| !r.is_empty()),
                "{name}: token_free without a reason is a schema error"
            );
        } else {
            assert!(
                row.success_token.as_deref().is_some_and(|t| !t.is_empty()),
                "{name}: a gate must declare a success token or declare itself \
                 token-free with a reason"
            );
        }
        for (on, reason, what) in [
            (row.live_probe, &row.live_probe_skip_reason, "live_probe"),
            (row.empty_probe, &row.empty_probe_skip_reason, "empty_probe"),
        ] {
            if !on {
                assert!(
                    reason.as_deref().is_some_and(|r| !r.is_empty()),
                    "{name}: {what} = false without a reason is a schema error"
                );
            }
        }

        let mut runs: Vec<(&str, ProbeRun)> = Vec::new();
        if row.empty_probe {
            let r = run_gate(empty.path(), name);
            // LEG E: an empty tree must never be a pass.
            assert_ne!(
                r.code, 0,
                "{name} PASSED over an empty root. An empty input set is an \
                 ERROR, never a pass:\n{}",
                r.stdout
            );
            runs.push(("empty root", r));
        }
        if row.live_probe {
            runs.push(("live root", run_gate(&root, name)));
        }

        for (where_, r) in &runs {
            probes += 1;
            if row.token_free {
                // LEG G: prove the absence rather than assert it.
                for word in &reg.probe.token_vocabulary {
                    assert!(
                        !carries_success_token(&r.stdout, word),
                        "{name} is registered token-free but opened a line with \
                         `{word}` on the {where_} probe:\n{}",
                        r.stdout
                    );
                }
                continue;
            }
            let token = row.success_token.as_deref().expect("checked above");
            if verdict_shape_violation(r.code, &r.stdout, token) {
                violations.push(format!(
                    "{name} [{where_}] exit={} token `{token}` present={} \n{}",
                    r.code,
                    carries_success_token(&r.stdout, token),
                    r.stdout
                ));
            }
            if r.code == 0 && carries_success_token(&r.stdout, token) {
                green_with_token += 1;
            }
        }
    }

    // LEG D
    assert!(
        violations.is_empty(),
        "VERDICT-SHAPE VIOLATIONS ({}). `exit == 0` must be EQUIVALENT to \
         \"stdout carries this gate's success token\". A success token printed \
         on a path that returns non-zero is the class bd-lt7, \
         bd-builder-verdict-shape-qm65 and bd-verify-coverage-verdict-before-write-rk9n \
         each patched by hand; at three instances the fix is a detector, not a \
         fourth patch.\n\n{}",
        violations.len(),
        violations.join("\n---\n")
    );

    // LEG F: anti-vacuous on the probe itself.
    assert!(
        probes > 0,
        "zero gate probes ran; a meta-gate that probed nothing is an ERROR"
    );
    assert!(
        green_with_token >= reg.probe.green_with_token_floor,
        "no gate was observed GREEN carrying its success token across {probes} \
         probes. The token side of the equivalence never ran, so \"no token on a \
         failing path\" passed for free — which is exactly the vacuous green this \
         leg exists to forbid."
    );
    println!(
        "[probe] {probes} gate probes over {} gates; {green_with_token} observed \
         GREEN carrying their token",
        roster.len()
    );
}
