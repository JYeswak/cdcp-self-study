//! META-GATE over the differential harnesses and the gate roster.
//!
//! Agreement is necessary for a port and not sufficient for a gate: a defect
//! on both sides agrees with itself. Measured 2026-08-14: verify_objectives
//! printed covered=106 shortfalls=0 min_per_topic=0 in STRICT mode (exit 0);
//! `m min topic 0` had been passing for weeks. Live census numbers live in
//! `registries/differential_harnesses.toml` and are re-derived every run.
//! CHARTER pair: `census_charter_pair.rs` (bd-census-mutation-pair-unproven-pi0v).
//!
//! LEG A  every tree `diff_*.rs` is registered; empty discovery is ERROR
//!        unless retirement is declared.
//! LEG B  agreement-only total may fall, never rise (enforced on the total).
//! LEG C  every dispatched gate has a registry row.
//! LEG D  gate binary: exit==0 iff stdout carries the registered token.
//! LEG E  no gate may pass over an empty root.
//! LEG F  at least one gate observed GREEN carrying its token. Writer/flag
//!        gates (bd-gate-token-green-side-unprobed-cbh3) must be observed
//!        GREEN on a private tree / fixture — never by rewriting the live tree.
//! LEG G  token-free gates print no vocabulary word.
//! LEG H  classifier proven on planted known-bad and known-good.
//! LEG I  harness meta-gate (bd-j8b2): every active `diff_*.rs` carries a
//!        per-side nonzero⇒no-registered-token check, or a skip with reason.

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
    verdict_shape: VerdictShape,
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
struct VerdictShape {
    carrying_floor: usize,
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
    #[serde(default = "yes")]
    verdict_shape_leg: bool,
    #[serde(default)]
    verdict_shape_skip_reason: Option<String>,
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
    /// live_probe=false and not one of the private GREEN fixtures: skip
    /// the per-gate green floor only with a reason (FIX #1 of cbh3).
    #[serde(default)]
    green_unprobed_reason: Option<String>,
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
    run_gate_argv(root, &[gate])
}

fn isolate_git(cmd: &mut Command) {
    // A writer fixture must never inherit a GIT_* redirect at the live clone.
    for k in [
        "GIT_INDEX_FILE",
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        cmd.env_remove(k);
    }
}

fn run_gate_argv(root: &Path, args: &[&str]) -> ProbeRun {
    let mut cmd = Command::new(BIN);
    cmd.current_dir(root).arg("--root").arg(root).args(args);
    isolate_git(&mut cmd);
    let out = cmd.output().unwrap_or_else(|e| panic!("run {args:?}: {e}"));
    ProbeRun {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
    }
}

/// The five named by bd-gate-token-green-side-unprobed-cbh3. live_probe=false
/// meant a wrong token was only ever checked on RED — and a token the gate
/// never prints satisfies that for free (verify-step-count already did).
const WRITER_OR_FLAG: &[&str] = &[
    "build-units",
    "build-glossary-json",
    "install-hooks",
    "verify-injection-count",
    "verify-step-count",
];

fn git_isolated(root: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    cmd.current_dir(root).args(args);
    isolate_git(&mut cmd);
    let out = cmd.output().unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// GREEN probe that cannot rewrite a tracked live artifact.
///
/// Writers run in a private tree. Flag gates get the required
/// `--log`/`--readme`/`--script` against fixtures (the registry's
/// `--measured` skip was a misnomer; the flag is `--log`).
fn private_green_probe(engine: &Path, name: &str) -> Option<ProbeRun> {
    match name {
        "install-hooks" => {
            let td = tempfile::tempdir().expect("tmp");
            let root = td.path();
            let src = engine.join("hooks/pre-commit");
            let body =
                std::fs::read(&src).unwrap_or_else(|e| panic!("read {}: {e}", src.display()));
            std::fs::create_dir_all(root.join("hooks")).unwrap();
            std::fs::write(root.join("hooks/pre-commit"), body).unwrap();
            git_isolated(root, &["init", "-q"]);
            Some(run_gate_argv(root, &["install-hooks"]))
        }
        "verify-injection-count" => {
            let td = tempfile::tempdir().expect("tmp");
            let root = td.path();
            let log = root.join("inj.log");
            let readme = root.join("README.md");
            std::fs::write(
                &log,
                "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n",
            )
            .unwrap();
            std::fs::write(
                &readme,
                "# Specimen readme\n\n\
                 [![known-bad (shell selftest suites): 7 injections](https://img.shields.io/badge/known--bad-7_injections_all_RED-success.svg)](#x)\n\n\
                 | **Gate** | 2 selftest suites; 7 known-bad injections that must all go RED |\n\n\
                 Two selftest suites inject **7 known-bad faults** and assert the build fails.\n\n\
                 | **L4 — gates proven to trip** | ok | 2 suites, 7 injections, anti-vacuous |\n",
            )
            .unwrap();
            Some(run_gate_argv(
                root,
                &[
                    "verify-injection-count",
                    "--log",
                    log.to_str().unwrap(),
                    "--readme",
                    readme.to_str().unwrap(),
                    "--require",
                    "spec_alpha,spec_beta",
                ],
            ))
        }
        "verify-step-count" => {
            let td = tempfile::tempdir().expect("tmp");
            let root = td.path();
            let log = root.join("measured.log");
            let readme = root.join("README.md");
            let script = root.join("check.sh");
            std::fs::write(
                &log,
                "CHECK_STEPS=2 OK=2 SKIPPED=0 NESTED_OK=5 DEPTH=0 RUN=probe1\n",
            )
            .unwrap();
            std::fs::write(
                &readme,
                "# Specimen\n\n\
                 [![gate: 2 steps](https://img.shields.io/badge/gate-2_ordered_steps-success.svg)](#the-gate)\n\n\
                 | **Gate** | 2 ordered steps; 9 selftest suites |\n\n\
                 2 steps, fail-closed, each naming the script that failed.\n",
            )
            .unwrap();
            std::fs::write(
                &script,
                "#!/usr/bin/env sh\n\
                 ok() { echo x; }\n\
                 ok \"one\"\n\
                 ok \"two\"\n\
                 # a comment mentioning ok \"three\" must not count\n\
                 # STEP-COUNT-RECEIPT-BOUNDARY\n\
                 printf 'CHECK_STEPS=%s' \"$N\"\n",
            )
            .unwrap();
            Some(run_gate_argv(
                root,
                &[
                    "verify-step-count",
                    "--log",
                    log.to_str().unwrap(),
                    "--readme",
                    readme.to_str().unwrap(),
                    "--script",
                    script.to_str().unwrap(),
                ],
            ))
        }
        _ => None,
    }
}

fn live_watched_paths(engine: &Path) -> Vec<PathBuf> {
    let parent = engine.parent().unwrap_or(engine);
    vec![
        engine.join("web/data/units_index.json"),
        engine.join("web/data/glossary.json"),
        engine.join("README.md"),
        parent.join("README.md"),
        parent.join(".git/hooks/pre-commit"),
    ]
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
    let mut observed_green: BTreeSet<String> = BTreeSet::new();

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
        if let Some(r) = private_green_probe(&root, name) {
            runs.push(("private green", r));
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
                observed_green.insert(name.clone());
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
         GREEN carrying their token; private-green={:?}",
        roster.len(),
        WRITER_OR_FLAG
            .iter()
            .filter(|n| observed_green.iter().any(|g| g == *n))
            .collect::<Vec<_>>()
    );

    // Per-gate GREEN floor (bd-gate-token-green-side-unprobed-cbh3). LEG F is
    // a global floor of 1; a wrong token on an unprobed gate still passes that.
    // Every dispatched member of WRITER_OR_FLAG must be seen GREEN carrying
    // its token. A retired name (not on the roster) is reported, not required.
    let mut missing_green = Vec::new();
    for name in &roster {
        if WRITER_OR_FLAG.contains(&name.as_str()) && !observed_green.contains(name) {
            missing_green.push(name.clone());
        }
    }
    assert!(
        missing_green.is_empty(),
        "writer/flag gates never observed GREEN carrying their token: \
         {missing_green:?}. A wrong registered token on these gates reads as \
         covered — that is the verify-step-count defect this floor exists to \
         make inexpressible."
    );
    for name in WRITER_OR_FLAG {
        if !roster.iter().any(|g| g == name) {
            println!("[probe] {name}: not in dispatcher (retired); no GREEN probe");
        }
    }
    for name in &roster {
        let row = rows[name.as_str()];
        if row.token_free || row.live_probe || WRITER_OR_FLAG.contains(&name.as_str()) {
            continue;
        }
        if observed_green.contains(name) {
            continue;
        }
        assert!(
            row.green_unprobed_reason
                .as_deref()
                .is_some_and(|r| !r.is_empty()),
            "{name}: live_probe=false and never observed GREEN; declare \
             green_unprobed_reason or add a private GREEN probe"
        );
    }
}

#[test]
fn a_wrong_registered_token_on_a_real_green_run_is_red() {
    // The historically-wrong token for verify-step-count. Red-side "no token
    // on FAIL" is true of a token the gate never prints; only a real GREEN
    // run can catch it.
    let engine = engine_root();
    let r = private_green_probe(&engine, "verify-step-count")
        .expect("verify-step-count has a private GREEN fixture");
    assert_eq!(r.code, 0, "fixture must be a real GREEN run:\n{}", r.stdout);
    assert!(
        carries_success_token(&r.stdout, "PASS"),
        "real GREEN must carry PASS:\n{}",
        r.stdout
    );
    let wrong = "verify-step-count: ok:";
    assert!(
        verdict_shape_violation(r.code, &r.stdout, wrong),
        "wrong token on a real GREEN run must be RED; gate=verify-step-count \
         token={wrong:?}\n{}",
        r.stdout
    );
    assert!(
        !carries_success_token(&r.stdout, wrong),
        "the wrong token must not appear as a line prefix:\n{}",
        r.stdout
    );
}

#[test]
fn private_green_probes_do_not_mutate_the_live_tree() {
    let engine = engine_root();
    let paths = live_watched_paths(&engine);
    let before: Vec<Option<Vec<u8>>> = paths.iter().map(|p| std::fs::read(p).ok()).collect();
    for name in WRITER_OR_FLAG {
        let _ = private_green_probe(&engine, name);
    }
    for (p, b) in paths.iter().zip(&before) {
        let after = std::fs::read(p).ok();
        assert_eq!(&after, b, "private GREEN probe mutated {}", p.display());
    }
}

// LEG I — harness meta-gate (bd-j8b2). Token = registry row, not a PASS-grep.
#[rustfmt::skip]
mod leg_i {
    use super::*;
    struct Leg { per_side: bool, token_ok: bool, detail: String }
    fn fns(b: &str) -> BTreeMap<String, (usize, usize)> {
        fn_spans(b).into_iter().filter(|(t, _, _)| !*t).filter_map(|(_, s, e)| {
            let r = b[s..].lines().next().unwrap_or("").trim_start();
            let r = r.strip_prefix("pub fn ").unwrap_or(r);
            let n = r.strip_prefix("fn ").unwrap_or(r).split(|c: char| !is_ident_byte(c as u8)).next().unwrap_or("");
            (!n.is_empty()).then_some((n.to_string(), (s, e)))
        }).collect()
    }
    fn calls(body: &str, name: &str) -> usize {
        body.match_indices(name).filter(|(i, _)| {
            let a = i + name.len();
            (*i == 0 || !is_ident_byte(body.as_bytes()[i - 1]))
                && (a >= body.len() || !is_ident_byte(body.as_bytes()[a]))
                && body[a..].trim_start().starts_with('(')
        }).count()
    }
    fn guard(body: &str) -> bool {
        mentions(body, "code") && (body.contains("== 0") || body.contains("!= 0"))
            && (mentions(body, "out") || mentions(body, "stdout")) && body.contains("assert")
    }
    fn sides(body: &str) -> usize {
        usize::from(mentions(body, "py") || mentions(body, "python"))
            + usize::from(mentions(body, "rs") || mentions(body, "rust"))
    }
    fn token_ok(src: &str, token: &str) -> bool {
        let n = format!("\"{token}\"");
        (src.contains("differential_harnesses.toml") && src.contains("success_token"))
            || src.lines().any(|l| { let t = l.trim_start(); (t.starts_with("const ") || t.starts_with("pub const ")) && t.contains(&n) })
    }
    fn scan(src: &str, comparators: &[String], token: &str) -> Leg {
        let b = blank(src);
        let fns = fns(&b);
        let helpers: Vec<String> = fns.iter().filter(|(_, (s, e))| guard(&b[*s..=*e])).map(|(n, _)| n.clone()).collect();
        let mut seen = false;
        let mut missing = Vec::new();
        for c in comparators {
            let Some((s, e)) = fns.get(c) else { continue };
            let body = &b[*s..=*e];
            if sides(body) >= 2 {
                seen = true;
                let n: usize = helpers.iter().map(|h| calls(body, h)).sum();
                if n < 2 && !guard(body) { missing.push(c.clone()); }
            } else if !comparators.iter().any(|o| o != c && mentions(body, o)) && !helpers.iter().any(|h| mentions(body, h)) {
                missing.push(c.clone());
            }
        }
        let per_side = seen && missing.is_empty();
        Leg { token_ok: per_side && token_ok(src, token), detail: format!("seen={seen} missing={missing:?} helpers={helpers:?}"), per_side }
    }
    #[test]
    fn the_harness_verdict_shape_meta_gate() {
        let tok = "PASS";
        let cmp = vec!["compare".to_string()];
        let good = "const SUCCESS_TOKENS: &[&str] = &[\"PASS\"];\nfn assert_no_success_token_on_a_failing_path(l: &str, s: &str, r: &Run) {\n    if r.code == 0 { return; }\n    for t in SUCCESS_TOKENS { assert!(!r.out().contains(t)); }\n}\nfn compare(l: &str, root: &Path) -> Run {\n    let py = python(root); let rs = rust(root);\n    assert_no_success_token_on_a_failing_path(l, \"python\", &py);\n    assert_no_success_token_on_a_failing_path(l, \"rust\", &rs);\n    assert_eq!(py.code, rs.code); rs\n}\n";
        let g = scan(good, &cmp, tok);
        assert!(g.per_side && g.token_ok, "good {}", g.detail);
        let deleted = good.replace("assert_no_success_token_on_a_failing_path(l, \"python\", &py);\n    assert_no_success_token_on_a_failing_path(l, \"rust\", &rs);\n", "");
        let d = scan(&deleted, &cmp, tok);
        assert!(!d.per_side, "deleted leg must name diff_planted.rs: {}", d.detail);
        let w = scan(&good.replace("\"PASS\"", "\"GREEN\""), &cmp, tok);
        assert!(w.per_side && !w.token_ok, "wrong token {}", w.detail);
        let reg = registry();
        let found = discovered_harnesses();
        assert!(!found.is_empty(), "empty harness enumeration is an ERROR");
        let gates: BTreeMap<&str, &GateRow> = reg.gate.iter().map(|g| (g.name.as_str(), g)).collect();
        let rows: BTreeMap<&str, &HarnessRow> = reg.harness.iter().map(|h| (h.file.as_str(), h)).collect();
        let mut carrying = 0usize;
        let mut remaining = Vec::new();
        for file in &found {
            let row = rows[file.as_str()];
            let src = std::fs::read_to_string(tests_dir().join(file)).unwrap();
            if census_of(&src, &row.comparators).cases == 0 { continue; }
            let token = gates.get(row.gate.as_str()).and_then(|g| g.success_token.as_deref()).unwrap_or("");
            assert!(!token.is_empty() || !row.verdict_shape_leg, "{file}: no registered token");
            let s = scan(&src, &row.comparators, token);
            if row.verdict_shape_leg {
                if s.per_side && s.token_ok { carrying += 1; } else { remaining.push(format!("{file} {}", s.detail)); }
            } else {
                assert!(row.verdict_shape_skip_reason.as_deref().is_some_and(|r| !r.is_empty()), "{file}: skip without a reason is a schema error");
            }
        }
        assert!(remaining.is_empty(), "HARNESS META-GATE names: {}", remaining.join(", "));
        assert!(carrying >= reg.verdict_shape.carrying_floor, "carrying={carrying} < floor {}", reg.verdict_shape.carrying_floor);
    }
}
