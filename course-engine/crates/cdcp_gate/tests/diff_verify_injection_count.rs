//! Differential: the Rust `verify-injection-count` gate against the Python it
//! replaces, `scripts/verify_injection_count.py`.
//!
//! Every case runs BOTH programs on the same input and asserts stdout, stderr and
//! the exit code are byte-identical. The Python is the oracle: if they disagree,
//! the Rust is wrong. That is the whole contract of a port — a Rust gate that
//! "improves" on the Python is an unreviewed behaviour change, not a migration.
//!
//! The comparator returns the compared Run (code + stdout + stderr), not just
//! the exit code. An exit-only case cannot see what the gate said, so deleting
//! a drift message would stay green. Anti-vacuous and floor cases pin a named
//! substring of that Run; when the oracle is retired those assertions remain.
//!
//! Coverage, in this file:
//!   * the LIVE repo tree, against the real README, both green and drifted;
//!   * every known-bad case `scripts/selftest_injection_count.sh` injects (a..h);
//!   * anti-vacuous: an empty log and a missing log must be RED in both, because
//!     a drift guard that reports green on zero receipts is the exact failure it
//!     exists to prevent;
//!   * drift in both directions — advertised above actual and below actual;
//!   * the three bd-wf2 holes: a duplicate `--require` entry, a finding naming a
//!     file other than the one scanned, and a count spelled in words — plus the
//!     partial-coverage floor that catches a site dropping out for any other
//!     reason;
//!   * `--write-readme` regeneration, which cannot use `assert_identical` (each
//!     side rewrites its own copy) and so has its own comparator below.
//!
//! The harness is itself anti-vacuous: it fails if `python3` or the oracle script
//! is missing (a differential test that silently skips reports exactly like one
//! that passed), if the case list is empty, and if the cases do not include both
//! greens and reds.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_cdcp_gate");
const ORACLE: &str = "scripts/verify_injection_count.py";

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

/// A fresh directory under the target dir; unique per call, no cleanup races with
/// the sibling gate ports that also run in this crate's test binary set.
fn scratch(tag: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let d = std::env::temp_dir().join(format!(
        "cdcp_diff_injcount_{}_{}_{tag}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&d).expect("scratch dir");
    d
}

fn write(dir: &Path, name: &str, body: &str) -> String {
    let p = dir.join(name);
    std::fs::write(&p, body).expect("write fixture");
    p.to_string_lossy().into_owned()
}

struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Run {
    fn out(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
    fn err(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
    /// Named-substring verdicts read stdout *or* stderr — USAGE and FAIL
    /// findings do not share a stream.
    fn combined(&self) -> String {
        let mut s = self.out();
        s.push_str(&self.err());
        s
    }
}

fn exec(program: &str, args: &[String], cwd: &Path) -> Run {
    let out = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("spawn {program}: {e}"));
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: out.stdout,
        stderr: out.stderr,
    }
}

fn show(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Run both implementations on `args` and assert byte-for-byte identity.
///
/// Returns the compared **Rust** run so a case can pin *what* the gate said.
/// Returning the oracle's bytes instead would collapse the content assertion
/// onto the comparison: deleting a finding from the gate would stay green as
/// long as both sides still agreed (see `diff_verify_content_lock.rs`).
#[must_use]
fn assert_identical(label: &str, args: &[&str]) -> Run {
    let root = engine_root();

    let mut py_args: Vec<String> = vec![root.join(ORACLE).to_string_lossy().into_owned()];
    py_args.extend(args.iter().map(|s| (*s).to_string()));
    let py = exec("python3", &py_args, &root);

    let mut rs_args: Vec<String> = vec!["verify-injection-count".to_string()];
    rs_args.extend(args.iter().map(|s| (*s).to_string()));
    let rs = exec(BIN, &rs_args, &root);

    assert_eq!(
        show(&py.stdout),
        show(&rs.stdout),
        "[{label}] STDOUT differs.\n--- python ---\n{}\n--- rust ---\n{}",
        show(&py.stdout),
        show(&rs.stdout)
    );
    assert_eq!(
        py.stdout, rs.stdout,
        "[{label}] STDOUT differs at the byte level (same when lossily decoded)"
    );
    assert_eq!(
        show(&py.stderr),
        show(&rs.stderr),
        "[{label}] STDERR differs.\n--- python ---\n{}\n--- rust ---\n{}",
        show(&py.stderr),
        show(&rs.stderr)
    );
    assert_eq!(
        py.stderr, rs.stderr,
        "[{label}] STDERR differs at the byte level"
    );
    assert_eq!(
        py.code,
        rs.code,
        "[{label}] EXIT CODE differs: python {} vs rust {}\npython stdout:\n{}",
        py.code,
        rs.code,
        show(&py.stdout)
    );
    rs
}

/// Named-substring verdict. Exit-only is not a tick: deleting `needle` from
/// the gate must turn the case RED even after the Python oracle is retired.
fn assert_named(label: &str, run: &Run, needle: &str) {
    assert!(
        run.combined().contains(needle),
        "[{label}] named finding {needle:?} vanished from the compared run \
         (exit {}).\n--- stdout ---\n{}\n--- stderr ---\n{}",
        run.code,
        run.out(),
        run.err()
    );
}

// ─────────────────────── the harness must not be vacuous ────────────────────

#[test]
fn the_oracle_is_present_and_runnable() {
    let root = engine_root();
    assert!(
        root.join(ORACLE).is_file(),
        "{} is missing — without the oracle this whole file is vacuous, \
         which reports exactly like a passing differential",
        root.join(ORACLE).display()
    );
    let v = exec("python3", &["--version".to_string()], &root);
    assert_eq!(
        v.code, 0,
        "python3 is not runnable; a skipped differential is not a passed one"
    );
}

/// The selftest script is the enumeration this file mirrors. If a case is added
/// there, this assertion is the tripwire that says so.
#[test]
fn the_selftest_case_list_is_the_one_this_file_mirrors() {
    let root = engine_root();
    let body = std::fs::read_to_string(root.join("scripts/selftest_injection_count.sh"))
        .expect("selftest_injection_count.sh must exist — it is the case enumeration");
    for marker in [
        "(a) log and README agree",
        "(b) README off by one",
        "(c) suite receipt deleted",
        "(d) suite reports zero",
        "(e) unregistered suite in log",
        "(f) empty log",
        "(g) README advertises nothing",
        "(h) README suite count wrong",
        "(dd) per-suite cell disagrees LOW",
        "(ee) per-suite cell disagrees HIGH",
        "(ff) registered suite missing from the table",
        "(gg) unregistered suite row in the table",
        "(hh) table parses to zero suite rows",
        "(jj) advertisement says known-bad without a shell/selftest qualifier",
    ] {
        assert!(
            body.contains(marker),
            "selftest case {marker:?} vanished or was renamed; \
             re-derive this file's case list before trusting it"
        );
    }
}

// ────────────────────────── the live repo tree ──────────────────────────────

/// The real README, read through the gate's own scanner, so the fixture log can
/// be built to match whatever the tree currently advertises.
fn live_advertised_total() -> u128 {
    let root = engine_root();
    let readme = root.parent().unwrap_or(&root).join("README.md");
    let text =
        std::fs::read_to_string(&readme).unwrap_or_else(|e| panic!("{}: {e}", readme.display()));
    let mut claims: Vec<u128> = text
        .lines()
        .flat_map(cdcp_gate::gates::verify_injection_count::scan_advertised)
        .map(|d| d.parse::<u128>().expect("advertised count fits u128"))
        .collect();
    claims.sort_unstable();
    claims.dedup();
    assert_eq!(
        claims.len(),
        1,
        "the live README advertises {claims:?} — a single agreed number is required \
         to build a GREEN fixture; this is itself drift"
    );
    claims[0]
}

/// Receipts for the live registered suites summing to `total`, every suite ≥ 1.
fn live_log_body(total: u128) -> String {
    let suites = cdcp_gate::gates::verify_injection_count::REGISTERED_SUITES;
    let n = suites.len() as u128;
    assert!(
        total >= n,
        "live README advertises {total} injections across {n} registered suites; \
         a suite would have to report zero, which is RED by design"
    );
    let mut out = String::new();
    for (i, s) in suites.iter().enumerate() {
        let v = if i == 0 { total - (n - 1) } else { 1 };
        out.push_str(&format!("INJECTIONS={v} SUITE={s}\n"));
    }
    out
}

/// Receipts taken from the live per-suite `n` column so the column check and
/// the advertised total can both be green on the same fixture.
fn live_cell_log() -> (String, u128) {
    let root = engine_root();
    let readme = root.parent().unwrap_or(&root).join("README.md");
    let text =
        std::fs::read_to_string(&readme).unwrap_or_else(|e| panic!("{}: {e}", readme.display()));
    let mut out = String::new();
    let mut sum = 0u128;
    let mut seen = std::collections::BTreeSet::new();
    for line in text.lines() {
        if let Some((suite, digits, _, _)) =
            cdcp_gate::gates::verify_injection_count::parse_suite_row(line)
        {
            if seen.insert(suite.to_string()) {
                let n: u128 = digits.parse().expect("cell is digits");
                sum += n;
                out.push_str(&format!("INJECTIONS={n} SUITE={suite}\n"));
            }
        }
    }
    assert!(
        !out.is_empty(),
        "live README per-suite table parsed to zero rows — the column check \
         would be vacuous"
    );
    (out, sum)
}

#[test]
fn live_tree_green_case_is_identical_and_green() {
    let d = scratch("live_green");
    let total = live_advertised_total();
    let (body, cell_sum) = live_cell_log();
    assert_eq!(
        cell_sum, total,
        "live README per-suite cells sum to {cell_sum} but the advertised \
         total is {total} — run verify-injection-count --write-readme against \
         a real receipt log"
    );
    let log = write(&d, "live.log", &body);
    // No --readme and no --require: both implementations must resolve the same
    // default README (the repo root's) and the same registered-suite roster.
    let run = assert_identical("live-green", &["--log", &log]);
    assert_eq!(
        run.code, 0,
        "the live tree must be GREEN when the receipts match what README advertises"
    );
    assert_named(
        "live-green",
        &run,
        "injection count GREEN (README and the suites both say ",
    );
}

#[test]
fn live_tree_drift_is_identical_and_red() {
    let d = scratch("live_drift");
    let total = live_advertised_total();
    let log = write(&d, "live.log", &live_log_body(total + 1));
    let run = assert_identical("live-drift", &["--log", &log]);
    assert_eq!(
        run.code, 1,
        "one extra injection against the live README must be RED"
    );
    assert_named(
        "live-drift",
        &run,
        "known-bad injections; the suites self-reported",
    );
}

// ───────────── the eight cases selftest_injection_count.sh injects ──────────

/// The specimen README `write_readme` in the selftest writes: `$2` injections
/// across `$3` suites.
fn specimen_readme(injections: u32, suites: u32) -> String {
    format!(
        "# Specimen readme\n\
         \n\
         [![known-bad (shell selftest suites): {injections} injections](https://img.shields.io/badge/known--bad-{injections}_injections_all_RED-success.svg)](#x)\n\
         \n\
         | **Gate** | {suites} selftest suites; {injections} known-bad injections that must all go RED |\n\
         \n\
         Two selftest suites inject **{injections} known-bad faults** and assert the build fails.\n\
         \n\
         | **L4 — gates proven to trip** | ok | {suites} suites, {injections} injections, anti-vacuous |\n"
    )
}

const REQUIRE: &str = "spec_alpha,spec_beta";
const GOOD_LOG: &str = "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n";

struct Specimen {
    _dir: PathBuf,
    log: String,
    readme: String,
}

fn specimen(tag: &str, log_body: &str, injections: u32, suites: u32) -> Specimen {
    let dir = scratch(tag);
    let log = write(&dir, "injections.log", log_body);
    let readme = write(&dir, "README.md", &specimen_readme(injections, suites));
    Specimen {
        _dir: dir,
        log,
        readme,
    }
}

fn check(label: &str, s: &Specimen) -> Run {
    assert_identical(
        label,
        &["--log", &s.log, "--readme", &s.readme, "--require", REQUIRE],
    )
}

#[test]
fn case_a_baseline_agreement_is_identical_and_green() {
    let s = specimen("a_baseline", GOOD_LOG, 7, 2);
    let run = check("a-baseline", &s);
    assert_eq!(run.code, 0, "log and README agree → GREEN");
    assert_named(
        "a-baseline",
        &run,
        "injection count GREEN (README and the suites both say 7)",
    );
}

#[test]
fn case_b_readme_off_by_one_is_identical_and_red() {
    let s = specimen("b_off_by_one", GOOD_LOG, 8, 2);
    let run = check("b-readme-off-by-one", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "b-readme-off-by-one",
        &run,
        "advertises 8 known-bad injections; the suites self-reported 7",
    );
}

#[test]
fn case_c_deleted_suite_receipt_is_identical_and_red() {
    let s = specimen("c_missing", "INJECTIONS=3 SUITE=spec_alpha\n", 7, 2);
    let run = check("c-suite-receipt-missing", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "c-suite-receipt-missing",
        &run,
        "emitted no INJECTIONS= line — that is an ERROR, never a silent zero",
    );
}

#[test]
fn case_d_suite_reporting_zero_is_identical_and_red() {
    let s = specimen(
        "d_zero",
        "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=0 SUITE=spec_beta\n",
        7,
        2,
    );
    let run = check("d-suite-reports-zero", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "d-suite-reports-zero",
        &run,
        "self-reported 0 injections — a known-bad suite that asserts no RED is not a gate",
    );
}

#[test]
fn case_e_unregistered_suite_is_identical_and_red() {
    let s = specimen(
        "e_rogue",
        "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\nINJECTIONS=2 SUITE=spec_rogue\n",
        7,
        2,
    );
    let run = check("e-unregistered-suite", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "e-unregistered-suite",
        &run,
        "is not registered in REGISTERED_SUITES",
    );
}

#[test]
fn case_g_readme_advertising_nothing_is_identical_and_red() {
    let dir = scratch("g_silent");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let readme = write(
        &dir,
        "README.md",
        "# Specimen readme with no advertised count at all.\n",
    );
    let run = assert_identical(
        "g-readme-silent",
        &["--log", &log, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(run.code, 1, "nothing to check is an ERROR, not a pass");
    assert_named(
        "g-readme-silent",
        &run,
        "README advertises no known-bad injection count at all (nothing to check is an ERROR, not a pass)",
    );
}

#[test]
fn case_jj_unqualified_known_bad_is_identical_and_red() {
    let dir = scratch("jj_unqual");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let body = specimen_readme(7, 2).replace("known-bad (shell selftest suites):", "known-bad:");
    let readme = write(&dir, "README_unqual.md", &body);
    let run = assert_identical(
        "jj-unqualified-known-bad",
        &["--log", &log, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(run.code, 1);
    assert_named(
        "jj-unqualified-known-bad",
        &run,
        "advertises known-bad injections without a shell/selftest qualifier",
    );
}

#[test]
fn case_h_readme_suite_count_wrong_is_identical_and_red() {
    let s = specimen("h_suites", GOOD_LOG, 7, 5);
    let run = check("h-readme-suite-count", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "h-readme-suite-count",
        &run,
        "advertises 5 selftest suites; 2 are registered",
    );
}

// ───────────────────────────── ANTI-VACUOUS ─────────────────────────────────
//
// Case (f) plus the cases the selftest does not reach. A drift guard that goes
// green on zero receipts is a fooled certificate, so both "no receipts at all"
// shapes are pinned here explicitly rather than folded in with the rest.

#[test]
fn case_f_empty_log_is_identical_and_an_error_never_a_pass() {
    for (tag, body) in [
        ("f_empty", ""),
        ("f_blank_lines", "\n\n"),
        ("f_whitespace_only", "   \n\t\n"),
    ] {
        let s = specimen(tag, body, 7, 2);
        let run = check("f-empty-log", &s);
        assert_eq!(
            run.code, 1,
            "{tag}: an empty scan set is an ERROR, not a pass"
        );
        assert_named(
            "f-empty-log",
            &run,
            "injection log is empty — zero suites self-reported (empty scan set is an ERROR, not a pass)",
        );
    }
}

#[test]
fn a_missing_log_file_is_identical_and_an_error_never_a_pass() {
    let dir = scratch("missing_log");
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    let absent = dir
        .join("does-not-exist.log")
        .to_string_lossy()
        .into_owned();
    let run = assert_identical(
        "missing-log",
        &["--log", &absent, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(run.code, 1, "a missing log must fail closed");
    assert_named("missing-log", &run, "injection log missing:");
}

#[test]
fn a_log_that_is_a_directory_is_identical_and_red() {
    let dir = scratch("dir_log");
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    let as_dir = dir.to_string_lossy().into_owned();
    let run = assert_identical(
        "log-is-a-directory",
        &["--log", &as_dir, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(run.code, 1);
    // Path.is_file() is false for a directory, same as a missing path — the
    // named finding is the missing-log line. Silence here is the defect.
    assert_named("log-is-a-directory", &run, "injection log missing:");
}

#[test]
fn a_missing_readme_is_identical_and_red() {
    let dir = scratch("missing_readme");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let absent = dir.join("no-README.md").to_string_lossy().into_owned();
    let run = assert_identical(
        "missing-readme",
        &["--log", &log, "--readme", &absent, "--require", REQUIRE],
    );
    assert_eq!(run.code, 1);
    assert_named("missing-readme", &run, "README missing:");
}

#[test]
fn an_empty_require_registry_is_identical_and_red() {
    let dir = scratch("empty_require");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    for req in [",, ,", "", ","] {
        let run = assert_identical(
            "empty-require",
            &["--log", &log, "--readme", &readme, "--require", req],
        );
        assert_eq!(
            run.code, 1,
            "a gate over an empty registry is vacuous → RED"
        );
        assert_named(
            "empty-require",
            &run,
            "no suites required (a gate over an empty registry is vacuous)",
        );
    }
}

// ─────────────────────── drift, in both directions ──────────────────────────

#[test]
fn drift_advertised_above_actual_is_identical_and_red() {
    // README says 9, suites self-report 7.
    let s = specimen("drift_over_advertised", GOOD_LOG, 9, 2);
    let run = check("drift-advertised-high", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "drift-advertised-high",
        &run,
        "advertises 9 known-bad injections; the suites self-reported 7",
    );
}

#[test]
fn drift_advertised_below_actual_is_identical_and_red() {
    // README says 7, suites self-report 13.
    let s = specimen(
        "drift_under_advertised",
        "INJECTIONS=9 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n",
        7,
        2,
    );
    let run = check("drift-advertised-low", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "drift-advertised-low",
        &run,
        "advertises 7 known-bad injections; the suites self-reported 13",
    );
}

#[test]
fn drift_at_the_boundary_off_by_one_each_way_is_identical() {
    for (tag, alpha, red) in [
        ("boundary_exact", 3u32, false),
        ("boundary_minus_one", 2u32, true),
        ("boundary_plus_one", 4u32, true),
    ] {
        let s = specimen(
            tag,
            &format!("INJECTIONS={alpha} SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n"),
            7,
            2,
        );
        let run = check("drift-boundary", &s);
        assert_eq!(run.code, i32::from(red), "{tag}");
        if red {
            assert_named(
                "drift-boundary",
                &run,
                "known-bad injections; the suites self-reported",
            );
        } else {
            assert_named(
                "drift-boundary",
                &run,
                "injection count GREEN (README and the suites both say 7)",
            );
        }
    }
}

// ─────────────────── receipt-integrity shapes (port fidelity) ───────────────

#[test]
fn a_double_reported_suite_is_identical() {
    // Same suite, two different counts, in one run.
    let s = specimen(
        "double_report",
        "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=5 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n",
        7,
        2,
    );
    let run = check("double-report", &s);
    assert_eq!(run.code, 1);
    assert_named(
        "double-report",
        &run,
        "reported two different counts (3 then 5) in one run",
    );
}

#[test]
fn a_repeated_identical_receipt_is_identical_and_not_double_counted() {
    let s = specimen(
        "repeat_receipt",
        "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n",
        7,
        2,
    );
    let run = check("repeat-receipt", &s);
    assert_eq!(
        run.code, 0,
        "a receipt echoed twice must not inflate the total"
    );
    assert_named(
        "repeat-receipt",
        &run,
        "injection count GREEN (README and the suites both say 7)",
    );
}

// ──────────────────── bd-wf2: the three holes, both sides ───────────────────
//
// Each case is a hole the original port reproduced faithfully. The Python was
// fixed first and this file went red against the unpatched port; these cases are
// what stayed red until the Rust matched.

#[test]
fn a_suite_named_twice_in_require_is_identical_and_an_error() {
    // Formerly: summed twice, inflating measured_total — the one direction that
    // turns real drift GREEN. Now a usage ERROR in both, with no total printed.
    let dir = scratch("dup_require");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    let run = assert_identical(
        "duplicate-require",
        &[
            "--log",
            &log,
            "--readme",
            &readme,
            "--require",
            "spec_alpha,spec_alpha",
        ],
    );
    assert_eq!(run.code, 1);
    assert_named(
        "duplicate-require",
        &run,
        "--require names 'spec_alpha' more than once",
    );

    // The sharp shape: ONE suite reporting 3 must not certify a README that
    // advertises 6 across two suites.
    let solo = write(&dir, "solo.log", "INJECTIONS=3 SUITE=spec_alpha\n");
    let six = write(&dir, "SIX.md", &specimen_readme(6, 2));
    let run = assert_identical(
        "duplicate-require-inflation",
        &[
            "--log",
            &solo,
            "--readme",
            &six,
            "--require",
            "spec_alpha,spec_alpha",
        ],
    );
    assert_eq!(run.code, 1, "an inflated total must never certify a README");
    assert_named(
        "duplicate-require-inflation",
        &run,
        "--require names 'spec_alpha' more than once",
    );
}

#[test]
fn a_finding_names_the_file_that_was_scanned_in_both() {
    // Formerly: findings said `README.md:<lineno>` whatever --readme pointed at,
    // sending the next reader to an innocent file.
    let dir = scratch("finding_path");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let readme = write(&dir, "README_off.md", &specimen_readme(8, 2));
    let run = assert_identical(
        "finding-names-scanned-file",
        &["--log", &log, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(run.code, 1);
    // Read the PORT's bytes, not a second oracle spawn: deleting the finding
    // from the gate must go RED even if the comparison were removed.
    assert_named(
        "finding-names-scanned-file",
        &run,
        "README_off.md:3 advertises 8",
    );
    assert!(
        !run.combined().contains("\n    - README.md:"),
        "no finding may point at an innocent README.md:\n{}",
        run.combined()
    );
}

/// The selftest's specimen with the prose site's count spelled in words.
fn specimen_readme_prose(digits: u32, suites: u32, spelled: &str) -> String {
    specimen_readme(digits, suites).replace(
        &format!("**{digits} known-bad faults**"),
        &format!("**{spelled} known-bad faults**"),
    )
}

#[test]
fn a_word_spelled_count_is_identical_and_no_longer_invisible() {
    // The dangerous MIDDLE case: three sites in digits still parse, so the gate
    // reported GREEN while the fourth site advertised something else entirely.
    let dir = scratch("word_count");
    let log = write(&dir, "injections.log", GOOD_LOG);

    let drifted = write(
        &dir,
        "DRIFTED.md",
        &specimen_readme_prose(7, 2, "thirty-six"),
    );
    let run = assert_identical(
        "word-count-drifted",
        &["--log", &log, "--readme", &drifted, "--require", REQUIRE],
    );
    assert_eq!(
        run.code, 1,
        "a word-spelled site that disagrees must be RED"
    );
    assert_named(
        "word-count-drifted",
        &run,
        "advertises 36 known-bad injections; the suites self-reported 7",
    );

    let agreeing = write(&dir, "AGREEING.md", &specimen_readme_prose(7, 2, "seven"));
    let run = assert_identical(
        "word-count-agreeing",
        &["--log", &log, "--readme", &agreeing, "--require", REQUIRE],
    );
    assert_eq!(
        run.code, 0,
        "a word-spelled site that agrees must stay GREEN"
    );
    assert_named(
        "word-count-agreeing",
        &run,
        "injection count GREEN (README and the suites both say 7)",
    );

    // Every word shape both implementations must agree on, drift or not.
    for (i, spelled) in [
        "seven",
        "SEVEN",
        "Thirty-Six",
        "thirty six",
        "eighteen",
        "eight",
        "ninety-nine",
        "zero",
        "three dozen", // outside the vocabulary: the site drops out entirely
        "freighter",   // the word branch carries a \b
    ]
    .iter()
    .enumerate()
    {
        let r = write(
            &dir,
            &format!("W{i}.md"),
            &specimen_readme_prose(7, 2, spelled),
        );
        let _ = assert_identical(
            &format!("word-shape-{i}"),
            &["--log", &log, "--readme", &r, "--require", REQUIRE],
        );
    }
}

#[test]
fn a_readme_that_loses_an_advertisement_site_is_identical_and_red() {
    // Partial coverage must not report identically to full coverage. Dropping the
    // prose site leaves four parseable sites, all of which still agree.
    let dir = scratch("lost_site");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let body: String = specimen_readme(7, 2)
        .lines()
        .filter(|l| !l.contains("known-bad faults"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert!(
        !body.contains("known-bad faults"),
        "the fixture must actually drop the site"
    );
    let readme = write(&dir, "README.md", &body);
    let run = assert_identical(
        "lost-advertisement-site",
        &["--log", &log, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(
        run.code, 1,
        "four of five sites parsing is a loss of coverage"
    );
    assert_named(
        "lost-advertisement-site",
        &run,
        "only 4 advertisement site(s) parsed",
    );
}

// ────────────── bd-wf2: regenerated, never hand-maintained ──────────────────

/// Write mode cannot use [`assert_identical`]: each implementation must rewrite
/// its OWN copy, so the two runs necessarily print two different paths. This runs
/// both against byte-identical fixtures, compares stdout with the fixture path
/// elided, and then compares the two rewritten files byte-for-byte.
fn assert_identical_write_mode(label: &str, log: &str, body: &str, require: &str) -> (i32, String) {
    let root = engine_root();
    let dir = scratch(&format!("write_{label}"));
    let py_readme = write(&dir, "PY.md", body);
    let rs_readme = write(&dir, "RS.md", body);

    let py = exec(
        "python3",
        &[
            root.join(ORACLE).to_string_lossy().into_owned(),
            "--log".into(),
            log.to_string(),
            "--readme".into(),
            py_readme.clone(),
            "--require".into(),
            require.to_string(),
            "--write-readme".into(),
        ],
        &root,
    );
    let rs = exec(
        BIN,
        &[
            "verify-injection-count".into(),
            "--log".into(),
            log.to_string(),
            "--readme".into(),
            rs_readme.clone(),
            "--require".into(),
            require.to_string(),
            "--write-readme".into(),
        ],
        &root,
    );

    let norm = |b: &[u8], p: &str| show(b).replace(p, "<README>");
    assert_eq!(
        norm(&py.stdout, &py_readme),
        norm(&rs.stdout, &rs_readme),
        "[{label}] STDOUT differs once the fixture path is elided"
    );
    assert_eq!(
        norm(&py.stderr, &py_readme),
        norm(&rs.stderr, &rs_readme),
        "[{label}] STDERR differs"
    );
    assert_eq!(py.code, rs.code, "[{label}] EXIT CODE differs");

    let py_after = std::fs::read(&py_readme).expect("read python's rewrite");
    let rs_after = std::fs::read(&rs_readme).expect("read rust's rewrite");
    assert_eq!(
        String::from_utf8_lossy(&py_after),
        String::from_utf8_lossy(&rs_after),
        "[{label}] the two rewritten READMEs differ"
    );
    assert_eq!(
        py_after, rs_after,
        "[{label}] rewrites differ at byte level"
    );
    (py.code, String::from_utf8_lossy(&py_after).into_owned())
}

#[test]
fn write_readme_regenerates_identically_in_both() {
    let dir = scratch("write_good");
    let log = write(&dir, "injections.log", GOOD_LOG);

    // Drifted README, sound receipts: every site is rewritten to the measured 7.
    let (code, after) =
        assert_identical_write_mode("drifted", &log, &specimen_readme(9, 2), REQUIRE);
    assert_eq!(code, 0, "after regeneration the tree must be GREEN");
    assert_eq!(
        after,
        specimen_readme(7, 2),
        "regeneration must produce exactly the agreeing README"
    );

    // Word-spelled site: regeneration normalises it to the checkable digit form.
    let (code, after) = assert_identical_write_mode(
        "prose",
        &log,
        &specimen_readme_prose(7, 2, "thirty-six"),
        REQUIRE,
    );
    assert_eq!(code, 0);
    assert_eq!(after, specimen_readme(7, 2));

    // Already correct: idempotent, and it says so.
    let (code, after) =
        assert_identical_write_mode("idempotent", &log, &specimen_readme(7, 2), REQUIRE);
    assert_eq!(code, 0);
    assert_eq!(after, specimen_readme(7, 2));
}

#[test]
fn write_readme_refuses_to_launder_an_unsound_total_in_both() {
    let dir = scratch("write_unsound");
    let before = specimen_readme(9, 2);

    // spec_beta never reported — the total is not a number worth writing.
    let solo = write(&dir, "solo.log", "INJECTIONS=3 SUITE=spec_alpha\n");
    let (code, after) = assert_identical_write_mode("missing-receipt", &solo, &before, REQUIRE);
    assert_eq!(code, 1);
    assert_eq!(after, before, "an unsound run must leave README untouched");

    // Empty log — the anti-vacuous case, with a writer attached.
    let empty = write(&dir, "empty.log", "");
    let (code, after) = assert_identical_write_mode("empty-log", &empty, &before, REQUIRE);
    assert_eq!(code, 1);
    assert_eq!(after, before);
}

// ─────────── per-suite n column (bd-per-suite-injection-column-unguarded-aop9)

const COL_REQUIRE: &str = "selftest_orphan,selftest_known_bad";
const COL_LOG: &str = "INJECTIONS=6 SUITE=selftest_orphan\nINJECTIONS=4 SUITE=selftest_known_bad\n";

fn column_readme(orphan: u32, extra: &str) -> String {
    format!(
        "# Specimen readme\n\
         \n\
         [![known-bad (shell selftest suites): 10 injections](https://img.shields.io/badge/known--bad-10_injections_all_RED-success.svg)](#x)\n\
         \n\
         | **Gate** | 2 selftest suites; 10 known-bad injections that must all go RED |\n\
         \n\
         Two selftest suites inject **10 known-bad faults** and assert the build fails.\n\
         \n\
         | **L4 — gates proven to trip** | ok | 2 suites, 10 injections, anti-vacuous |\n\
         \n\
         | Suite | n | Injections |\n\
         |---|---|---|\n\
         | `selftest_known_bad` | 4 | planted |\n\
         | `selftest_orphan` | {orphan} | planted |\n\
         {extra}"
    )
}

#[test]
fn per_suite_column_shapes_are_identical() {
    let dir = scratch("col");
    let log = write(&dir, "col.log", COL_LOG);

    let good = write(&dir, "GOOD.md", &column_readme(6, ""));
    let run = assert_identical(
        "col-agree",
        &["--log", &log, "--readme", &good, "--require", COL_REQUIRE],
    );
    assert_eq!(run.code, 0);
    assert_named(
        "col-agree",
        &run,
        "injection count GREEN (README and the suites both say 10)",
    );

    let low = write(&dir, "LOW.md", &column_readme(5, ""));
    let run = assert_identical(
        "col-low",
        &["--log", &log, "--readme", &low, "--require", COL_REQUIRE],
    );
    assert_eq!(run.code, 1);
    assert_named(
        "col-low",
        &run,
        "suite selftest_orphan advertises 5 injections; the suite self-reported 6",
    );

    let high = write(&dir, "HIGH.md", &column_readme(7, ""));
    let run = assert_identical(
        "col-high",
        &["--log", &log, "--readme", &high, "--require", COL_REQUIRE],
    );
    assert_eq!(run.code, 1);
    assert_named(
        "col-high",
        &run,
        "suite selftest_orphan advertises 7 injections; the suite self-reported 6",
    );

    let miss_body = column_readme(6, "")
        .lines()
        .filter(|l| !l.contains("`selftest_orphan`"))
        .map(|l| format!("{l}\n"))
        .collect::<String>();
    let miss = write(&dir, "MISS.md", &miss_body);
    let run = assert_identical(
        "col-missing",
        &["--log", &log, "--readme", &miss, "--require", COL_REQUIRE],
    );
    assert_eq!(run.code, 1);
    assert_named(
        "col-missing",
        &run,
        "has no per-suite table row — that is an ERROR, never a silent skip",
    );

    let extra = write(
        &dir,
        "EXTRA.md",
        &column_readme(6, "| `selftest_not_a_real_suite` | 1 | planted |\n"),
    );
    let run = assert_identical(
        "col-extra",
        &["--log", &log, "--readme", &extra, "--require", COL_REQUIRE],
    );
    assert_eq!(run.code, 1);
    assert_named("col-extra", &run, "is not in REGISTERED_SUITES");

    let empty = write(&dir, "EMPTY.md", &specimen_readme(10, 2));
    let run = assert_identical(
        "col-empty",
        &["--log", &log, "--readme", &empty, "--require", COL_REQUIRE],
    );
    assert_eq!(run.code, 1);
    assert_named(
        "col-empty",
        &run,
        "README per-suite injection table parsed to zero suite rows (empty scan set is an ERROR, not a pass)",
    );

    let (code, after) =
        assert_identical_write_mode("col-regen", &log, &column_readme(5, ""), COL_REQUIRE);
    assert_eq!(code, 0, "regenerated cells must be GREEN");
    assert!(
        after.contains("| `selftest_orphan` | 6 |"),
        "cell must be rewritten from the receipt:\n{after}"
    );
    assert!(
        !after.contains("| `selftest_orphan` | 5 |"),
        "drifted cell survived regeneration:\n{after}"
    );
}

#[test]
fn unparseable_and_comment_lines_are_identical() {
    for (tag, body) in [
        ("junk_leading", "garbage line\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n"),
        ("junk_comment", "# INJECTIONS=99 SUITE=spec_alpha\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n"),
        ("junk_nondigit", "INJECTIONS=x SUITE=spec_alpha\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n"),
        ("junk_quotes", "it's a 'quoted' line\nline with \"double\" and 'single'\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n"),
        ("junk_backslash", "TABBED\\back\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n"),
        ("junk_nonascii", "\u{a0}nbsp\u{2028}sep é —\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n"),
        ("junk_spacey", "INJECTIONS=3\tSUITE=spec_alpha\n  INJECTIONS=4   SUITE=spec_beta  \n"),
        ("junk_cr", "INJECTIONS=3 SUITE=spec_alpha\rINJECTIONS=4 SUITE=spec_beta\r"),
        ("junk_leading_zeros", "INJECTIONS=003 SUITE=spec_alpha\nINJECTIONS=04 SUITE=spec_beta\n"),
    ] {
        let s = specimen(tag, body, 7, 2);
        let _ = check(tag, &s);
    }
}

#[test]
fn readme_shapes_that_exercise_the_scanners_are_identical() {
    let dir = scratch("readme_shapes");
    let log = write(&dir, "injections.log", GOOD_LOG);
    for (i, body) in [
        "7 injections\n",
        "7 injection\n",
        "7 faults\n",
        "7 known-bad injections\n",
        "7_injections\n",
        "07 injections and 8 known-bad faults and nine selftest suites\n",
        "seven injections\n", // word count: invisible to the scanner
        "twelve selftest suites; 7 faults\n",
        "SEVEN SUITES and 7 INJECTIONS\n",
        "2 suites, 7 injections\n",
        "2 suitesx, 7 injections\n", // `suites?\b` must not match `suitesx`
        "v1.7 injections\n",
        "7\tinjections\n",
        "no numbers at all here\n",
        "7 injections\r8 faults\r", // CPython splitlines splits on bare \r
    ]
    .iter()
    .enumerate()
    {
        let readme = write(&dir, &format!("R{i}.md"), body);
        let _ = assert_identical(
            &format!("readme-shape-{i}"),
            &["--log", &log, "--readme", &readme, "--require", REQUIRE],
        );
    }
}

#[test]
fn path_spellings_print_identically() {
    let dir = scratch("path_spellings");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    let doubled = log.replace("/injections.log", "//./injections.log");
    let trailing = format!("{log}/");
    for spelling in [doubled.as_str(), trailing.as_str(), ""] {
        let _ = assert_identical(
            "path-spelling",
            &["--log", spelling, "--readme", &readme, "--require", REQUIRE],
        );
    }
}

// ─────────────── the harness must exercise both verdicts ────────────────────

/// A differential suite in which nothing ever went green, or nothing ever went
/// red, agrees with the oracle for an uninteresting reason. This pins both.
#[test]
fn the_case_set_contains_both_greens_and_reds() {
    let dir = scratch("both_verdicts");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let good = write(&dir, "GOOD.md", &specimen_readme(7, 2));
    let bad = write(&dir, "BAD.md", &specimen_readme(8, 2));

    let green = assert_identical(
        "verdict-green",
        &["--log", &log, "--readme", &good, "--require", REQUIRE],
    );
    let red = assert_identical(
        "verdict-red",
        &["--log", &log, "--readme", &bad, "--require", REQUIRE],
    );
    assert_eq!(green.code, 0, "the agreeing case must actually be GREEN");
    assert_eq!(red.code, 1, "the drifted case must actually be RED");
    assert_ne!(
        green.code, red.code,
        "a differential over one verdict proves nothing"
    );
    assert_named(
        "verdict-green",
        &green,
        "injection count GREEN (README and the suites both say 7)",
    );
    assert_named(
        "verdict-red",
        &red,
        "advertises 8 known-bad injections; the suites self-reported 7",
    );
}
