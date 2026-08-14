//! Differential: the Rust `verify-injection-count` gate against the Python it
//! replaces, `scripts/verify_injection_count.py`.
//!
//! Every case runs BOTH programs on the same input and asserts stdout, stderr and
//! the exit code are byte-identical. The Python is the oracle: if they disagree,
//! the Rust is wrong. That is the whole contract of a port — a Rust gate that
//! "improves" on the Python is an unreviewed behaviour change, not a migration.
//!
//! Coverage, in this file:
//!   * the LIVE repo tree, against the real README, both green and drifted;
//!   * every known-bad case `scripts/selftest_injection_count.sh` injects (a..h);
//!   * anti-vacuous: an empty log and a missing log must be RED in both, because
//!     a drift guard that reports green on zero receipts is the exact failure it
//!     exists to prevent;
//!   * drift in both directions — advertised above actual and below actual.
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
/// Returns the (shared) exit code so a case can additionally pin GREEN or RED.
#[must_use]
fn assert_identical(label: &str, args: &[&str]) -> i32 {
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
    py.code
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

#[test]
fn live_tree_green_case_is_identical_and_green() {
    let d = scratch("live_green");
    let total = live_advertised_total();
    let log = write(&d, "live.log", &live_log_body(total));
    // No --readme and no --require: both implementations must resolve the same
    // default README (the repo root's) and the same registered-suite roster.
    let code = assert_identical("live-green", &["--log", &log]);
    assert_eq!(
        code, 0,
        "the live tree must be GREEN when the receipts match what README advertises"
    );
}

#[test]
fn live_tree_drift_is_identical_and_red() {
    let d = scratch("live_drift");
    let total = live_advertised_total();
    let log = write(&d, "live.log", &live_log_body(total + 1));
    let code = assert_identical("live-drift", &["--log", &log]);
    assert_eq!(
        code, 1,
        "one extra injection against the live README must be RED"
    );
}

// ───────────── the eight cases selftest_injection_count.sh injects ──────────

/// The specimen README `write_readme` in the selftest writes: `$2` injections
/// across `$3` suites.
fn specimen_readme(injections: u32, suites: u32) -> String {
    format!(
        "# Specimen readme\n\
         \n\
         [![known-bad: {injections} injections](https://img.shields.io/badge/known--bad-{injections}_injections_all_RED-success.svg)](#x)\n\
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

fn check(label: &str, s: &Specimen) -> i32 {
    assert_identical(
        label,
        &["--log", &s.log, "--readme", &s.readme, "--require", REQUIRE],
    )
}

#[test]
fn case_a_baseline_agreement_is_identical_and_green() {
    let s = specimen("a_baseline", GOOD_LOG, 7, 2);
    assert_eq!(check("a-baseline", &s), 0, "log and README agree → GREEN");
}

#[test]
fn case_b_readme_off_by_one_is_identical_and_red() {
    let s = specimen("b_off_by_one", GOOD_LOG, 8, 2);
    assert_eq!(check("b-readme-off-by-one", &s), 1);
}

#[test]
fn case_c_deleted_suite_receipt_is_identical_and_red() {
    let s = specimen("c_missing", "INJECTIONS=3 SUITE=spec_alpha\n", 7, 2);
    assert_eq!(check("c-suite-receipt-missing", &s), 1);
}

#[test]
fn case_d_suite_reporting_zero_is_identical_and_red() {
    let s = specimen(
        "d_zero",
        "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=0 SUITE=spec_beta\n",
        7,
        2,
    );
    assert_eq!(check("d-suite-reports-zero", &s), 1);
}

#[test]
fn case_e_unregistered_suite_is_identical_and_red() {
    let s = specimen(
        "e_rogue",
        "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\nINJECTIONS=2 SUITE=spec_rogue\n",
        7,
        2,
    );
    assert_eq!(check("e-unregistered-suite", &s), 1);
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
    let code = assert_identical(
        "g-readme-silent",
        &["--log", &log, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(code, 1, "nothing to check is an ERROR, not a pass");
}

#[test]
fn case_h_readme_suite_count_wrong_is_identical_and_red() {
    let s = specimen("h_suites", GOOD_LOG, 7, 5);
    assert_eq!(check("h-readme-suite-count", &s), 1);
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
        assert_eq!(
            check("f-empty-log", &s),
            1,
            "{tag}: an empty scan set is an ERROR, not a pass"
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
    let code = assert_identical(
        "missing-log",
        &["--log", &absent, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(code, 1, "a missing log must fail closed");
}

#[test]
fn a_log_that_is_a_directory_is_identical_and_red() {
    let dir = scratch("dir_log");
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    let as_dir = dir.to_string_lossy().into_owned();
    let code = assert_identical(
        "log-is-a-directory",
        &["--log", &as_dir, "--readme", &readme, "--require", REQUIRE],
    );
    assert_eq!(code, 1);
}

#[test]
fn a_missing_readme_is_identical_and_red() {
    let dir = scratch("missing_readme");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let absent = dir.join("no-README.md").to_string_lossy().into_owned();
    let code = assert_identical(
        "missing-readme",
        &["--log", &log, "--readme", &absent, "--require", REQUIRE],
    );
    assert_eq!(code, 1);
}

#[test]
fn an_empty_require_registry_is_identical_and_red() {
    let dir = scratch("empty_require");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    for req in [",, ,", "", ","] {
        let code = assert_identical(
            "empty-require",
            &["--log", &log, "--readme", &readme, "--require", req],
        );
        assert_eq!(code, 1, "a gate over an empty registry is vacuous → RED");
    }
}

// ─────────────────────── drift, in both directions ──────────────────────────

#[test]
fn drift_advertised_above_actual_is_identical_and_red() {
    // README says 9, suites self-report 7.
    let s = specimen("drift_over_advertised", GOOD_LOG, 9, 2);
    assert_eq!(check("drift-advertised-high", &s), 1);
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
    assert_eq!(check("drift-advertised-low", &s), 1);
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
        let code = check("drift-boundary", &s);
        assert_eq!(code, i32::from(red), "{tag}");
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
    assert_eq!(check("double-report", &s), 1);
}

#[test]
fn a_repeated_identical_receipt_is_identical_and_not_double_counted() {
    let s = specimen(
        "repeat_receipt",
        "INJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=3 SUITE=spec_alpha\nINJECTIONS=4 SUITE=spec_beta\n",
        7,
        2,
    );
    assert_eq!(
        check("repeat-receipt", &s),
        0,
        "a receipt echoed twice must not inflate the total"
    );
}

#[test]
fn a_suite_named_twice_in_require_double_counts_in_both() {
    // The Python double-counts here. The port reproduces it rather than fixing
    // it; the assertion pins that they agree, not that the behaviour is good.
    let dir = scratch("dup_require");
    let log = write(&dir, "injections.log", GOOD_LOG);
    let readme = write(&dir, "README.md", &specimen_readme(7, 2));
    let code = assert_identical(
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
    assert_eq!(code, 1);
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
    assert_eq!(green, 0, "the agreeing case must actually be GREEN");
    assert_eq!(red, 1, "the drifted case must actually be RED");
    assert_ne!(green, red, "a differential over one verdict proves nothing");
}
