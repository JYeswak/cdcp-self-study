//! Product CLI for the F3 oracle (`bd-hardening-f-oracle-qly.5` / `.8`).
//!
//! Live `oracle-check` is GREEN against official EPA eGRID 2023 Rev 2
//! SRCO2RTA once the full PLNT23 population is the compute input
//! (`bd-hardening-e-data-4up.5`). Plants (perturb one published ref /
//! empty refs) must still fire, and must name location + computed +
//! reference + delta. No network — compiled references + vendored snapshots.

use assert_cmd::Command;
use cdcp_data::{
    check_oracle, check_oracle_with, compiled_pins, compiled_references, perturb_one_tolerance,
    Comparison, OracleError, ANTI_VACUOUS_REFS, DISAGREEMENT,
};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve course-engine workspace root")
}

fn cdcp() -> Command {
    let mut cmd = Command::cargo_bin("cdcp").expect("cdcp binary");
    cmd.current_dir(workspace_root());
    cmd
}

fn combined(assert: &assert_cmd::assert::Assert) -> String {
    let out = assert.get_output();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Live F3 is GREEN: full PLNT23 roll-up vs official SRCO2RTA inside
/// the declared 0.5 lb/MWh band. Do not rewrite official numbers.
fn assert_live_green(out: &str) {
    assert!(
        out.contains("oracle: PASS"),
        "live oracle must report PASS: {out}"
    );
    assert!(
        !out.contains(DISAGREEMENT),
        "live GREEN must not name {DISAGREEMENT}: {out}"
    );
    assert!(
        out.contains("grid_co2_lb_per_mwh") || out.contains("compared="),
        "live PASS must name a comparison: {out}"
    );
}

/// A pair to plant against. Live GREEN yields comparisons; live honest
/// RED yields findings. Requiring GREEN here is the qly.8 regression.
fn pair_to_plant() -> (PathBuf, Comparison) {
    let root = workspace_root();
    let pair = match check_oracle(&root) {
        Ok(report) => {
            assert!(
                !report.comparisons.is_empty(),
                "anti-vacuous: live oracle compared nothing"
            );
            report.comparisons[0].clone()
        }
        Err(OracleError::Disagreement { findings }) => {
            assert!(
                !findings.is_empty(),
                "anti-vacuous: Disagreement with zero findings"
            );
            findings[0].clone()
        }
        Err(e) => panic!("live oracle must run (GREEN or honest RED), not a structural error: {e}"),
    };
    (root, pair)
}

#[test]
fn help_lists_oracle_check() {
    let assert = cdcp().env("CDCP_DEV", "1").arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("oracle-check"),
        "cdcp --help must list oracle-check: {stdout}"
    );
    assert!(
        stdout.contains("oracle"),
        "cdcp --help must list the oracle alias: {stdout}"
    );
}

#[test]
fn oracle_check_live_tree_is_green() {
    let assert = cdcp().arg("oracle-check").assert().success();
    assert_live_green(&combined(&assert));
}

#[test]
fn oracle_alias_live_tree_is_green() {
    let assert = cdcp().arg("oracle").assert().success();
    assert_live_green(&combined(&assert));
}

#[test]
fn oracle_check_selftest_plants_trip_and_name_fields() {
    let assert = cdcp()
        .arg("oracle-check")
        .arg("--selftest")
        .assert()
        .success();
    let out = combined(&assert);
    assert!(
        !out.contains("requires a green live oracle first"),
        "selftest must still plant when live is already RED: {out}"
    );
    for needle in [
        "location=",
        "computed=",
        "reference=",
        "delta=",
        DISAGREEMENT,
    ] {
        assert!(
            out.contains(needle),
            "selftest perturb must name {needle}: {out}"
        );
    }
    assert!(
        out.contains(ANTI_VACUOUS_REFS),
        "selftest empty-refs must name the anti-vacuous token: {out}"
    );
    assert!(
        out.contains("oracle-check --selftest: PASS"),
        "selftest must report the plants tripped: {out}"
    );
}

#[test]
fn oracle_check_self_test_alias_also_runs() {
    let assert = cdcp()
        .arg("oracle-check")
        .arg("--self-test")
        .assert()
        .success();
    let out = combined(&assert);
    assert!(
        !out.contains("requires a green live oracle first"),
        "--self-test must still plant when live is already RED: {out}"
    );
    assert!(
        out.contains("oracle-check --selftest: PASS"),
        "--self-test alias must run the plants: {out}"
    );
}

/// cargo test plant: shift one published ref by one tolerance unit → RED
/// and the error names location + computed + reference + delta.
/// Live may already be honestly RED; the plant must still land.
#[test]
fn plant_perturb_one_published_ref_is_nonzero_and_names_fields() {
    let (root, pair) = pair_to_plant();
    let pins = compiled_pins().expect("compiled pins");
    let mut ledger = compiled_references().expect("compiled ledger");
    let planted = perturb_one_tolerance(pair.computed, pair.reference, pair.tolerance);
    let mut planted_n = 0usize;
    for r in &mut ledger.references {
        if r.location == pair.location && r.quantity == pair.quantity {
            r.value = planted;
            planted_n += 1;
            break;
        }
    }
    assert_eq!(
        planted_n,
        1,
        "must plant exactly one published ref ({} {})",
        pair.location,
        pair.quantity.as_str()
    );

    let err = check_oracle_with(&root, &ledger, &pins)
        .expect_err("perturb one published ref must be non-zero");
    let text = err.to_string();
    let OracleError::Disagreement { findings } = &err else {
        panic!("expected Disagreement, got {err:?}");
    };
    let hit = findings
        .iter()
        .find(|f| f.location == pair.location && f.quantity == pair.quantity)
        .unwrap_or_else(|| {
            panic!(
                "planted {} {} missing from findings: {text}",
                pair.location,
                pair.quantity.as_str()
            )
        });
    assert!(!hit.ok, "planted pair must be outside its band");
    assert_eq!(
        hit.reference, planted,
        "findings must report the planted reference, not only the live eGRID miss"
    );
    for needle in [
        "location=",
        "computed=",
        "reference=",
        "delta=",
        DISAGREEMENT,
        pair.location.as_str(),
    ] {
        assert!(text.contains(needle), "plant must name {needle}: {text}");
    }
}

/// cargo test plant: delete all refs → ERROR (anti-vacuous empty).
#[test]
fn plant_empty_refs_is_nonzero() {
    let root = workspace_root();
    let pins = compiled_pins().expect("compiled pins");
    let mut ledger = compiled_references().expect("compiled ledger");
    ledger.references.clear();
    let err = check_oracle_with(&root, &ledger, &pins).expect_err("empty refs must be non-zero");
    assert!(
        matches!(err, OracleError::EmptyReferences),
        "expected EmptyReferences, got {err:?}"
    );
    assert!(
        err.to_string().contains(ANTI_VACUOUS_REFS),
        "empty refs must name the anti-vacuous token: {err}"
    );
}

#[test]
fn oracle_check_missing_snapshots_is_nonzero() {
    let plant = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cdcp_data/tests/fixtures/good");
    let assert = cdcp()
        .args(["oracle-check", "--root"])
        .arg(&plant)
        .assert()
        .failure();
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        !out.is_empty(),
        "missing-snapshot plant must name the refusal: {out}"
    );
}

/// Meta: delete the check_oracle call or add a socket → this selftest is non-zero.
#[test]
fn cli_oracle_source_calls_check_oracle_and_has_no_network() {
    let src = include_str!("../src/oracle.rs");
    assert!(
        src.contains("check_oracle("),
        "delete the check_oracle call → selftest non-zero"
    );
    assert!(
        src.contains("check_oracle_with("),
        "delete the plant path → selftest non-zero"
    );
    for needle in [
        "TcpStream",
        "UdpSocket",
        "TcpListener",
        "std::net",
        "::net::",
        "reqwest",
        "ureq",
        "hyper::",
    ] {
        assert!(
            !src.contains(needle),
            "oracle CLI must not mention {needle}"
        );
    }
}
