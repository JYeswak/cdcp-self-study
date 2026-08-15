//! Differential oracle: `scripts/verify_doc_consistency.py` vs
//! `cdcp_gate verify-doc-consistency`.
//!
//! Every case here runs BOTH implementations on the same input and compares the
//! streams byte for byte. The Python is the reference for its own replacement,
//! so "did I port this correctly" is measured here, not judged in review.
//!
//! # What is compared, exactly
//!
//! **All three streams, byte for byte, on every case: stdout, stderr, and the
//! exit code.** No normalisation, no trimming, no line filtering, no
//! equivalence map. The port reproduces the oracle's exit 1 with an empty
//! stderr on every failure path rather than routing through `GateError` — see
//! the gate's module header and `crate_exit_code` (bd-2m9). This is what makes
//! the suite able to distinguish a port BUG from an intended FIX: any delta at
//! all is a delta.
//!
//! EVERY input class now has a byte-exact target. The one that did not — a
//! ragged milestone row, on which the oracle printed `PASS` and then raised
//! `TypeError`, while the port completed the report and rendered `None` — was
//! repaired in both implementations under bd-hw3. Its recorded-divergence pin
//! was DELETED rather than amended (a pin exists to make a repair deliberate,
//! not permanent) and replaced by the ordinary equality case
//! `ragged_row_is_red_in_both`. If a future divergence is ever recorded here,
//! it must assert both sides separately and say why no byte-exact target exists.
//!
//! # Anti-vacuous
//!
//! A missing `python3` is a HARD FAILURE, never a skip. A differential suite
//! that silently stops running reports exactly like one that passed.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");

/// The oracle's own codes: 0 on agreement, 1 on any failure. The port matches
/// both while the oracle exists.
const OK: i32 = 0;
const FAIL: i32 = 1;
/// The dispatcher's usage code, which is NOT part of the differential: the
/// oracle's argparse surface is not a verdict on the tree.
const USAGE: i32 = 3;

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn python_script() -> PathBuf {
    let p = engine_root().join("scripts/verify_doc_consistency.py");
    assert!(
        p.is_file(),
        "the oracle {} is gone — this suite cannot honestly run without it",
        p.display()
    );
    p
}

fn require_python3() {
    let ok = Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(
        ok,
        "python3 is required: the byte-exactness oracle cannot be skipped into green"
    );
}

struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_python(root: Option<&Path>) -> Run {
    require_python3();
    let mut c = Command::new("python3");
    c.current_dir(engine_root()).arg(python_script());
    if let Some(r) = root {
        c.arg("--root").arg(r);
    }
    let out = c.output().expect("python3");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

fn run_rust(root: Option<&Path>) -> Run {
    let mut c = Command::new(BIN);
    c.current_dir(engine_root()).arg("verify-doc-consistency");
    if let Some(r) = root {
        c.arg("--repo-root").arg(r);
    }
    let out = c.output().expect("cdcp_gate");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

/// Findings the report lists, including the ones the 40-item cap elided.
fn finding_count(stdout: &str) -> usize {
    let listed = stdout.lines().filter(|l| l.starts_with("    - ")).count();
    let elided = stdout
        .lines()
        .find_map(|l| {
            l.strip_prefix("    ... +")?
                .strip_suffix(" more")?
                .parse::<usize>()
                .ok()
        })
        .unwrap_or(0);
    listed + elided
}

/// bd-2m9: the anti-vacuous class must stay DISTINGUISHABLE even though both
/// implementations currently exit 1 for it. The port keeps the split alive in
/// `crate_exit_code`, which the later commit flips this whole crate onto; here
/// we assert the observable half — the run is RED and says, in the report the
/// operator reads, that it could not honestly evaluate. A gate that fell back
/// to "the docs disagree" wording for an unscanned tree would pass equality and
/// still be wrong.
fn assert_anti_vacuous(root: &Path) {
    let rs = run_rust(Some(root));
    assert_eq!(rs.code, FAIL, "an unscanned tree must be RED");
    let out = String::from_utf8_lossy(&rs.stdout);
    let vacuous_wording = [
        "empty scan set is an ERROR, not a pass",
        "(vacuous)",
        "roadmap doc missing",
        "refusing to pass unscanned",
    ];
    assert!(
        vacuous_wording.iter().any(|w| out.contains(w)),
        "a vacuous scan must SAY it could not evaluate, not merely fail: {out}"
    );
}

/// The whole acceptance bar, applied to one input.
fn assert_byte_exact(label: &str, root: Option<&Path>) -> String {
    let py = run_python(root);
    let rs = run_rust(root);
    compare_streams(label, &py, &rs)
}

/// The comparator, split out so it can be driven with a PLANTED mismatch and
/// shown to trip — see `the_comparator_is_proven_to_trip_on_each_stream`.
/// Weakening or deleting ANY ONE of the three byte-for-byte assertions below
/// turns that test RED, so this suite cannot be quietly hollowed out into one
/// that compares nothing.
fn compare_streams(label: &str, py: &Run, rs: &Run) -> String {
    let py_out = String::from_utf8_lossy(&py.stdout).into_owned();
    let rs_out = String::from_utf8_lossy(&rs.stdout).into_owned();
    let py_err = String::from_utf8_lossy(&py.stderr).into_owned();
    let rs_err = String::from_utf8_lossy(&rs.stderr).into_owned();

    assert_eq!(
        py.stdout, rs.stdout,
        "[{label}] stdout differs\n--- python ---\n{py_out}\n--- rust ---\n{rs_out}"
    );
    assert_eq!(
        py.stderr, rs.stderr,
        "[{label}] stderr differs\n--- python ---\n{py_err}\n--- rust ---\n{rs_err}"
    );
    assert_eq!(
        py.code, rs.code,
        "[{label}] exit code differs: python {} vs rust {}\n--- stdout ---\n{py_out}\n--- rust stderr ---\n{rs_err}",
        py.code, rs.code
    );

    // Beyond equality: neither side may agree its way into a vacuous verdict.
    if py.code == OK {
        assert!(
            py.stderr.is_empty(),
            "[{label}] a passing run must be stderr-silent: {py_err}"
        );
        assert!(
            py_out.contains("roadmap GREEN"),
            "[{label}] a pass must carry its receipt: {py_out}"
        );
    } else {
        assert_eq!(
            py.code, FAIL,
            "[{label}] the oracle failed with an unmodelled code"
        );
        assert!(
            py.stderr.is_empty(),
            "[{label}] the oracle wrote to stderr — that input class is not \
             byte-exactly portable and belongs in the recorded-divergence test, \
             not here: {py_err}"
        );
        assert!(
            py_out.starts_with("FAIL\n"),
            "[{label}] a failure must say so on line one: {py_out}"
        );
        assert!(
            finding_count(&py_out) > 0,
            "[{label}] a FAIL with zero listed findings is vacuous: {py_out}"
        );
    }
    py_out
}

// ───────────────────────────── specimen builder ────────────────────────────

struct Spec {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Spec {
    /// The clean roadmap specimen `scripts/selftest_doc_consistency.sh` writes,
    /// byte for byte.
    fn clean() -> Self {
        let s = Self::bare();
        s.write(
            "CHARTER.md",
            "# Specimen charter\n\n\
             ## 9. Milestones\n\n\
             | ID | Milestone | Status |\n\
             |----|-----------|--------|\n\
             | **M0–M2** | scaffold · registries · bank | **DONE** |\n\
             | **V11** | stretch surfaces | **DONE** |\n\
             | **M8** | learn v2 | **GREEN** |\n\
             | **M9** | publicize | **DONE** |\n",
        );
        s.write(
            "README.md",
            "# Specimen readme\n\n\
             ## Roadmap\n\n\
             | ID | Milestone | Status |\n\
             |---|---|---|\n\
             | M0–M2 | scaffold · registries · bank | **done** |\n\
             | V11 | stretch surfaces | **done** |\n\
             | M8 | learn v2 | **done** |\n\
             | M9 | publicize | **DONE** (2026-08-12) |\n",
        );
        s.write(
            "course-engine/docs/PHASE-NEXT.md",
            "# Specimen phase-next\n\n\
             ## Done (do not re-plan)\n\n\
             | Wave | Outcome |\n\
             |------|---------|\n\
             | **V11** | stretch surfaces |\n\
             | **M8** | learn v2 |\n\
             | **M9-S1/S2** | bar + OSS meta |\n",
        );
        s
    }

    fn bare() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonicalize");
        Spec { _dir: dir, root }
    }

    fn write(&self, rel: &str, body: &str) {
        self.write_bytes(rel, body.as_bytes());
    }

    fn write_bytes(&self, rel: &str, body: &[u8]) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    fn append(&self, rel: &str, body: &str) {
        let p = self.root.join(rel);
        let mut cur = std::fs::read_to_string(&p).unwrap_or_default();
        cur.push_str(body);
        std::fs::write(p, cur).unwrap();
    }

    fn replace(&self, rel: &str, from: &str, to: &str) {
        let p = self.root.join(rel);
        let cur = std::fs::read_to_string(&p).unwrap();
        assert!(cur.contains(from), "injection target {from:?} not in {rel}");
        std::fs::write(p, cur.replace(from, to)).unwrap();
    }

    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.root.join(rel)).unwrap();
    }

    /// Turn the specimen into a real git repo so BOTH implementations take the
    /// `git ls-files` branch rather than the filesystem fallback.
    fn git_init(&self) {
        let run = |args: &[&str]| {
            let out = Command::new("git")
                .current_dir(&self.root)
                .args(args)
                .output()
                .expect("git");
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        run(&["init", "-q"]);
        run(&["add", "-A"]);
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

// ══════════════════════════ 1. the live repo tree ══════════════════════════

/// Whatever the working tree's verdict is right now — GREEN because the roadmap
/// agrees, or RED because a sibling agent is mid-edit — both implementations
/// must reach it identically. A shared RED is as good a byte-exactness datum as
/// a shared GREEN.
#[test]
fn live_tree_agrees_byte_for_byte() {
    let out = assert_byte_exact("live-tree", None);
    // Agreement is not enough (bd-diff-remaining-agreement-only-qgy9). The
    // live tree can be GREEN or RED (a sibling may be mid-edit); pin content
    // that survives either: a leading verdict word, the root it scanned, and
    // the roadmap the gate exists to check.
    assert!(
        out.starts_with("PASS\n") || out.starts_with("FAIL\n"),
        "live tree must lead with a verdict: {out}"
    );
    assert!(
        out.contains("  root="),
        "live tree report must name the root it scanned: {out}"
    );
    assert!(
        out.contains("roadmap"),
        "live tree report must mention the roadmap: {out}"
    );
}

/// The default root is the ENGINE's PARENT, not the engine directory. Getting
/// this wrong reads two of the three roadmap docs from nowhere and still prints
/// a plausible report.
#[test]
fn live_tree_default_root_is_the_repo_root() {
    let out = assert_byte_exact("live-tree-root", None);
    let repo = engine_root().parent().unwrap().to_path_buf();
    assert!(
        out.contains(&format!("  root={}\n", repo.display())),
        "default root is not the repo root: {out}"
    );
}

// ═══════════════ 2. every injection the shell selftest exercises ═══════════

/// (a) clean specimen → GREEN.
#[test]
fn a_clean_specimen_is_green_in_both() {
    let s = Spec::clean();
    let out = assert_byte_exact("a-clean-specimen", Some(s.path()));
    assert!(out.starts_with("PASS\n"), "{out}");
    assert!(out.contains("roadmap GREEN"), "{out}");
}

/// (b) one milestone twice inside one table → RED.
#[test]
fn b_duplicate_row_is_red_in_both() {
    let s = Spec::clean();
    s.append(
        "CHARTER.md",
        "| **M9** | publicize (stale copy) | **open** |\n",
    );
    let out = assert_byte_exact("b-duplicate-row", Some(s.path()));
    assert!(out.contains("appears twice in the same table"), "{out}");
}

/// (c) the same milestone carrying two statuses across docs → RED.
#[test]
fn c_cross_doc_conflict_is_red_in_both() {
    let s = Spec::clean();
    s.replace(
        "README.md",
        "| M8 | learn v2 | **done** |",
        "| M8 | learn v2 | **ongoing** |",
    );
    let out = assert_byte_exact("c-cross-doc-conflict", Some(s.path()));
    assert!(
        out.contains("conflicting status across the roadmap docs"),
        "{out}"
    );
    assert!(out.contains("conflicts=1"), "{out}");
}

/// (d) a status cell in vocabulary the gate cannot read → RED, fail-closed.
#[test]
fn d_unreadable_status_is_red_in_both() {
    let s = Spec::clean();
    s.replace(
        "README.md",
        "| M8 | learn v2 | **done** |",
        "| M8 | learn v2 | mostly there |",
    );
    let out = assert_byte_exact("d-unreadable-status", Some(s.path()));
    assert!(out.contains("unrecognised status vocabulary"), "{out}");
    assert!(
        out.contains("'mostly there'"),
        "the !r form must survive: {out}"
    );
}

/// (e) a doc that still calls publication pending → RED.
#[test]
fn e_publication_pending_is_red_in_both() {
    let s = Spec::clean();
    s.append(
        "course-engine/docs/PHASE-NEXT.md",
        "\nThe visibility flip is blocked pending a human decision.\n",
    );
    let out = assert_byte_exact("e-publication-pending", Some(s.path()));
    assert!(out.contains("publication described as not done"), "{out}");
}

/// (f) anti-vacuous: a root with zero markdown is an ERROR, not a pass.
#[test]
fn f_zero_markdown_is_an_error_in_both() {
    let s = Spec::bare();
    let out = assert_byte_exact("f-zero-markdown", Some(s.path()));
    assert!(out.contains("zero markdown files scanned"), "{out}");
    assert!(out.contains("markdown_scanned=0"), "{out}");
    assert_anti_vacuous(s.path());
}

/// (g) a missing roadmap doc is an ERROR, not a silent skip.
#[test]
fn g_missing_roadmap_doc_is_an_error_in_both() {
    let s = Spec::clean();
    s.remove("course-engine/docs/PHASE-NEXT.md");
    let out = assert_byte_exact("g-missing-roadmap-doc", Some(s.path()));
    assert!(out.contains("roadmap doc missing"), "{out}");
    assert_anti_vacuous(s.path());
}

// ═══════════════════════ 3. anti-vacuous, beyond (f) ═══════════════════════

/// A roadmap doc that exists but declares no milestone table at all.
#[test]
fn zero_milestone_tables_is_an_error_in_both() {
    let s = Spec::clean();
    s.write(
        "course-engine/docs/PHASE-NEXT.md",
        "# Specimen phase-next\n\nNo tables here at all.\n",
    );
    let out = assert_byte_exact("zero-tables", Some(s.path()));
    assert!(out.contains("zero milestone tables parsed"), "{out}");
    assert_anti_vacuous(s.path());
}

/// Every roadmap doc gone: zero rows across the whole scan.
#[test]
fn zero_rows_across_all_docs_is_an_error_in_both() {
    let s = Spec::bare();
    s.write("notes.md", "# just a note, no roadmap anywhere\n");
    let out = assert_byte_exact("zero-rows", Some(s.path()));
    assert!(
        out.contains("zero milestone rows parsed across all roadmap docs"),
        "{out}"
    );
    assert!(out.contains("milestone_rows=0"), "{out}");
    assert_anti_vacuous(s.path());
}

/// A milestone table whose rows all parse to no milestone id: tables > 0 but
/// rows == 0. The "vacuous" wording is a different message from the one above.
#[test]
fn tables_yielding_zero_rows_is_an_error_in_both() {
    let s = Spec::clean();
    s.write(
        "course-engine/docs/PHASE-NEXT.md",
        "# Specimen phase-next\n\n\
         ## Done (do not re-plan)\n\n\
         | Wave | Outcome |\n\
         |------|---------|\n\
         | alpha | nothing milestone-shaped |\n\
         | beta | still nothing |\n",
    );
    let out = assert_byte_exact("tables-zero-rows", Some(s.path()));
    assert!(
        out.contains("milestone tables yielded zero rows (vacuous)"),
        "{out}"
    );
}

/// A markdown file that is not valid UTF-8 must be reported unreadable in both,
/// with CPython's own `UnicodeDecodeError` text.
#[test]
fn non_utf8_markdown_is_refused_identically() {
    let s = Spec::clean();
    s.write_bytes("broken.md", b"# doc\n\xff\xfe bad\n");
    let out = assert_byte_exact("non-utf8-markdown", Some(s.path()));
    assert!(out.contains("refusing to pass unscanned"), "{out}");
    assert!(
        out.contains("'utf-8' codec can't decode byte 0xff"),
        "{out}"
    );
    assert_anti_vacuous(s.path());
}

// ═══════════════════════ 4. ordering must not drift ════════════════════════

/// Findings are emitted in a fixed order. A `HashMap` would reorder the
/// milestone summary; sorting paths as raw bytes rather than as CPython's parts
/// tuple would reorder the publication findings (`a/b.md` sorts BEFORE
/// `a-b/c.md`, the opposite of a byte comparison).
#[test]
fn publication_findings_keep_cpython_path_order() {
    let s = Spec::clean();
    let bad = "Publication is deferred.\n";
    for rel in ["a/b.md", "a-b/c.md", "a.md", "zz/aa.md", "zz-1/aa.md"] {
        s.write(rel, bad);
    }
    s.git_init();
    let out = assert_byte_exact("path-order", Some(s.path()));

    let order: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("publication described as not done"))
        .collect();
    assert_eq!(order.len(), 5, "{out}");
    let names: Vec<String> = order
        .iter()
        .map(|l| {
            l.trim_start_matches("    - ")
                .split(':')
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(
        names,
        vec!["a/b.md", "a-b/c.md", "a.md", "zz/aa.md", "zz-1/aa.md"],
        "path ordering drifted from CPython's parts-tuple comparison: {out}"
    );
}

/// The milestone summary block is ordered by id, not by first appearance.
#[test]
fn milestone_summary_is_ordered_by_id() {
    let s = Spec::clean();
    let out = assert_byte_exact("summary-order", Some(s.path()));
    let ids: Vec<&str> = out
        .lines()
        .filter(|l| l.starts_with("    ") && l.contains(": DONE ("))
        .map(|l| l.trim().split(':').next().unwrap())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "summary is not id-sorted: {out}");
    // string sort, not numeric: M10 before M2
    assert!(ids.contains(&"M0") && ids.contains(&"M9-S1"), "{out}");
}

/// More findings than the 40-item cap: the tail must be counted, not dropped.
#[test]
fn report_cap_and_overflow_line_match() {
    let s = Spec::clean();
    let mut body = String::new();
    for i in 0..45 {
        body.push_str(&format!("Line {i}: publication is blocked for now\n"));
    }
    s.write("many.md", &body);
    let out = assert_byte_exact("report-cap", Some(s.path()));
    assert_eq!(
        out.lines().filter(|l| l.starts_with("    - ")).count(),
        40,
        "{out}"
    );
    assert!(out.contains("    ... +5 more"), "{out}");
}

// ═══════════════════════ 5. parser corners of the port ═════════════════════

/// `!r` is CPython's `repr`, quote selection and all.
#[test]
fn repr_quoting_in_status_errors_matches() {
    let s = Spec::clean();
    s.replace(
        "README.md",
        "| M8 | learn v2 | **done** |",
        "| M8 | learn v2 | it's murky |",
    );
    let out = assert_byte_exact("repr-single-quote", Some(s.path()));
    assert!(out.contains("\"it's murky\""), "{out}");

    let s2 = Spec::clean();
    s2.replace(
        "README.md",
        "| M8 | learn v2 | **done** |",
        "| M8 | learn v2 | it's \"murky\" |",
    );
    let out2 = assert_byte_exact("repr-both-quotes", Some(s2.path()));
    assert!(out2.contains("'it\\'s \"murky\"'"), "{out2}");
}

/// DONE and OPEN in one cell is a contradiction, not a coin flip.
#[test]
fn done_and_open_at_once_is_red_in_both() {
    let s = Spec::clean();
    s.replace(
        "README.md",
        "| M8 | learn v2 | **done** |",
        "| M8 | learn v2 | done but blocked |",
    );
    let out = assert_byte_exact("done-and-open", Some(s.path()));
    assert!(
        out.contains("status asserts DONE and OPEN at once"),
        "{out}"
    );
}

/// An empty status cell is unreadable, not permission.
#[test]
fn empty_status_cell_is_red_in_both() {
    let s = Spec::clean();
    s.replace(
        "README.md",
        "| M8 | learn v2 | **done** |",
        "| M8 | learn v2 |  |",
    );
    let out = assert_byte_exact("empty-status", Some(s.path()));
    assert!(out.contains("empty status cell"), "{out}");
}

/// A milestone-keyed table with neither a Status column nor a status heading.
#[test]
fn table_declaring_no_status_is_reported_in_both() {
    let s = Spec::clean();
    s.replace(
        "course-engine/docs/PHASE-NEXT.md",
        "## Done (do not re-plan)",
        "## Notes on the waves",
    );
    let out = assert_byte_exact("no-status-declared", Some(s.path()));
    assert!(out.contains("declares no status"), "{out}");
}

/// Ranges, sub-milestone runs, and the deliberate case-sensitivity that keeps
/// prose like "Learn v2" from minting a phantom milestone V2.
#[test]
fn ranges_sub_runs_and_case_sensitivity_match() {
    let s = Spec::clean();
    s.write(
        "CHARTER.md",
        "# Specimen charter\n\n\
         ## 9. Milestones\n\n\
         | ID | Milestone | Status |\n\
         |----|-----------|--------|\n\
         | **M0–M2** | scaffold | **DONE** |\n\
         | M3-M5 | middle | **DONE** |\n\
         | M6 — M7 | dash spacing | **DONE** |\n\
         | **M9-S1/S2/S3** | sub runs | **DONE** |\n\
         | learn v2 and phase m4 | lowercase prose is not a milestone | **DONE** |\n\
         | **V11** | stretch | **DONE** |\n\
         | **M8** | learn v2 | **GREEN** |\n\
         | **M9** | publicize | **DONE** |\n",
    );
    s.write(
        "README.md",
        "# Specimen readme\n\n\
         ## Roadmap\n\n\
         | ID | Milestone | Status |\n\
         |---|---|---|\n\
         | M0–M7 | scaffold | **done** |\n\
         | M9-S1/S2/S3 | sub runs | **done** |\n\
         | V11 | stretch | **done** |\n\
         | M8 | learn v2 | **done** |\n\
         | M9 | publicize | **done** |\n",
    );
    let out = assert_byte_exact("ranges-and-runs", Some(s.path()));
    for id in ["M0", "M4", "M7", "M9-S3", "V11"] {
        assert!(
            out.contains(&format!("    {id}: DONE (")),
            "missing {id}: {out}"
        );
    }
    assert!(!out.contains("    V2:"), "prose minted a phantom V2: {out}");
    assert!(!out.contains("    M4: DONE (1 row"), "{out}");
}

/// The heading of a status-bearing section supplies the status for a table with
/// no Status column — the PHASE-NEXT shape.
#[test]
fn heading_supplied_status_matches() {
    let s = Spec::clean();
    let out = assert_byte_exact("heading-status", Some(s.path()));
    assert!(out.contains("    M9-S1: DONE (1 row(s))"), "{out}");
    assert!(out.contains("    V11: DONE (3 row(s))"), "{out}");
}

/// A git-backed specimen exercises the `git ls-files` branch in both, not the
/// filesystem fallback — the branch the live tree actually takes.
#[test]
fn git_backed_specimen_agrees_byte_for_byte() {
    let s = Spec::clean();
    s.write("docs/extra.md", "# extra\n");
    s.git_init();
    let out = assert_byte_exact("git-backed", Some(s.path()));
    assert!(out.contains("markdown_scanned=4"), "{out}");
}

/// A .gitignore'd markdown file is invisible to the git branch in both.
#[test]
fn gitignored_markdown_is_skipped_identically() {
    let s = Spec::clean();
    s.write(".gitignore", "ignored/\n");
    s.write("ignored/secret.md", "Publication is deferred.\n");
    s.git_init();
    let out = assert_byte_exact("gitignored", Some(s.path()));
    assert!(out.starts_with("PASS\n"), "{out}");
}

// ═════════════════════════ 6. dispatcher contract ══════════════════════════

/// A typo'd flag is USAGE, never a silent pass.
#[test]
fn unknown_flag_is_usage_not_silence() {
    let out = Command::new(BIN)
        .current_dir(engine_root())
        .args(["verify-doc-consistency", "--repo-rot", "/tmp"])
        .output()
        .expect("run");
    assert_eq!(out.status.code().unwrap_or(-1), USAGE);
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("unknown argument"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `--repo-root` without a value is USAGE, not a scan of the wrong tree.
#[test]
fn repo_root_without_a_value_is_usage() {
    let out = Command::new(BIN)
        .current_dir(engine_root())
        .args(["verify-doc-consistency", "--repo-root"])
        .output()
        .expect("run");
    assert_eq!(out.status.code().unwrap_or(-1), USAGE);
}

// ══ 7. THE REPAIRED DEFECT — was the one recorded divergence (bd-hw3) ═══════

/// A milestone row too short to reach its table's Status column is RED in both,
/// byte for byte.
///
/// This case used to be the suite's single recorded divergence: `parse_doc` fell
/// back to the heading's status for a short row, that fallback was `None` under a
/// non-status heading, the row was recorded with `status=None` and NO error, and
/// `main` then died in `",".join(sorted({r["status"] for r in rows}))` with
/// `TypeError: sequence item 0: expected str instance, NoneType found` — AFTER
/// printing `PASS` and most of the summary. The port meanwhile completed the
/// report and rendered the missing status as the string `None`. The divergence
/// was pinned rather than fixed because fixing it is a behaviour change.
///
/// bd-hw3 made that change deliberately in both implementations (see the gate's
/// module header for the argument), so the pin is DELETED, not amended — it
/// existed to force the repair to be a decision, and the decision has been made.
/// What replaces it is an ordinary equality case, which is the point: the input
/// class now has a byte-exact target like every other.
#[test]
fn ragged_row_is_red_in_both() {
    let s = Spec::bare();
    let doc = "# Notes on the waves\n\n\
               | ID | Milestone | Status |\n\
               |----|-----------|--------|\n\
               | M1 | a | DONE |\n\
               | M2 | ragged row |\n";
    s.write("CHARTER.md", doc);
    s.write("README.md", doc);
    s.write("course-engine/docs/PHASE-NEXT.md", doc);

    let out = assert_byte_exact("ragged-row", Some(s.path()));

    // The verdict is FAIL, and it is on line one — never a PASS that a later
    // path retracts.
    assert!(out.starts_with("FAIL\n"), "{out}");
    assert!(!out.contains("roadmap GREEN"), "{out}");

    // The finding names the file, the line, and the row itself.
    assert!(
        out.contains(
            "    - CHARTER.md:6: row is shorter than its Status column \
             (has 2 cell(s), Status is column 3): '| M2 | ragged row |'\n"
        ),
        "{out}"
    );

    // `None` is never rendered as a status, and the unread row is not counted
    // among the rows the gate claims to have read.
    assert!(
        !out.contains("None"),
        "a status was rendered as None: {out}"
    );
    assert!(out.contains("  milestone_rows=3\n"), "{out}");
    assert!(out.contains("    M1: DONE (3 row(s))\n"), "{out}");
    assert!(!out.contains("    M2:"), "{out}");
}

/// The same shortfall under a status-BEARING heading. The table still declares a
/// Status column, so the row still owes one; borrowing the heading's status would
/// be a guess, and a guess is what fail-closed forbids. This is the leg that
/// would silently survive if the fix had only special-cased a `None` heading.
#[test]
fn ragged_row_under_a_status_heading_is_also_red_in_both() {
    let s = Spec::clean();
    s.append("CHARTER.md", "| M10 | ragged under a DONE heading |\n");
    let out = assert_byte_exact("ragged-row-status-heading", Some(s.path()));
    assert!(
        out.contains("row is shorter than its Status column"),
        "{out}"
    );
    assert!(
        !out.contains("    M10:"),
        "the unread row was counted: {out}"
    );
}

/// KNOWN-GOOD, and the reason the fix is a scalpel rather than a hammer: a
/// milestone table with NO Status column at all, under a status-bearing heading,
/// is untouched. Every row there is legitimately "short" — there is no Status
/// column to fall short of — and the heading declares the status for the table.
/// An over-strict gate gets routed around, which is a slower death than no gate.
#[test]
fn short_rows_in_a_table_without_a_status_column_stay_green() {
    let s = Spec::clean();
    // PHASE-NEXT's `| Wave | Outcome |` table has two columns and no Status.
    s.append(
        "course-engine/docs/PHASE-NEXT.md",
        "| **M12** | one cell short |\n",
    );
    let out = assert_byte_exact("no-status-column-short-rows", Some(s.path()));
    assert!(out.starts_with("PASS\n"), "{out}");
    assert!(out.contains("roadmap GREEN"), "{out}");
    assert!(out.contains("    M12: DONE (1 row(s))"), "{out}");
}

// ══════════════════ 8. L4: the comparator, proven to trip ══════════════════

/// Green here means the comparator demonstrably trips — on EACH of the three
/// streams independently, not just in aggregate.
///
/// A real passing run is taken as the baseline, then a partner is synthesised
/// that differs in exactly ONE stream. So the assertion under test is the only
/// thing in `compare_streams` that can possibly notice; delete or weaken any one
/// of the three and this goes RED. That matters more than it looks: a
/// differential suite whose comparison has been hollowed out reports exactly
/// like one that found no differences, and the stderr and exit-code legs are the
/// easiest to lose, because for a long while every case produced identical empty
/// stderr and identical exit 1 whether or not anyone was checking.
///
/// The identical-pair leg first proves the comparator does NOT trip on a true
/// match, so "it trips" cannot be satisfied by a comparator that always panics.
#[test]
fn the_comparator_is_proven_to_trip_on_each_stream() {
    let s = Spec::clean();
    let base = run_python(Some(s.path()));
    assert_eq!(base.code, OK, "the baseline fixture must pass");

    let mk = |code: i32, stdout: &[u8], stderr: &[u8]| Run {
        code,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    };

    // A true match must NOT trip — otherwise "it trips" proves nothing.
    let twin = mk(base.code, &base.stdout, &base.stderr);
    compare_streams("identical-pair", &base, &twin);

    let planted = [
        (
            "stdout",
            mk(
                base.code,
                b"PASS\n  roadmap GREEN (tampered)\n",
                &base.stderr,
            ),
        ),
        (
            "stderr",
            mk(base.code, &base.stdout, b"one byte of noise\n"),
        ),
        ("exit code", mk(7, &base.stdout, &base.stderr)),
    ];

    for (stream, partner) in planted {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compare_streams("planted-mismatch", &base, &partner)
        }));
        std::panic::set_hook(prev);
        assert!(
            caught.is_err(),
            "the {stream} comparison did not trip on a planted mismatch — \
             this suite would report green while comparing nothing"
        );
    }
}

/// The gate is registered and self-describing.
#[test]
fn the_gate_is_listed() {
    let out = Command::new(BIN)
        .current_dir(engine_root())
        .arg("list")
        .output()
        .expect("run");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("verify-doc-consistency"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
