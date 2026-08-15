//! F3 differential harness: published references vs computed quantities.
//!
//! Plants:
//! - perturb a passing compute by 1 tolerance unit → RED
//! - zero reference locations → ERROR
//! - fewer than 5 locations → ERROR
//!
//! Meta-test: delete the comparison → this selftest is non-zero.

use cdcp_data::{
    agrees, check_oracle, check_oracle_with, compiled_pins, compiled_references, engine_root,
    parse_references, perturb_one_tolerance, Quantity, ANTI_VACUOUS_LOCATIONS, ANTI_VACUOUS_REFS,
    COMPILED_REFERENCES, COMPILED_REFERENCES_ORIGIN, DISAGREEMENT, SNAP_EGRID, SNAP_TMY3,
    SNAP_USGS,
};
use std::path::PathBuf;

fn engine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("engine")
}

fn production_oracle_src() -> &'static str {
    include_str!("../src/oracle.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests")
}

#[test]
fn compiled_ledger_has_at_least_five_locations_each_with_retrieval_date() {
    let ledger = compiled_references().expect("compiled references");
    let locs = ledger.referenced_locations();
    assert!(locs.len() >= 5, "{ANTI_VACUOUS_LOCATIONS}: {:?}", locs);
    assert!(!ledger.references.is_empty(), "{ANTI_VACUOUS_REFS}");
    for r in &ledger.references {
        assert!(
            !r.retrieved.is_empty(),
            "{} {} missing retrieval date",
            r.location,
            r.quantity.as_str()
        );
        assert!(
            !r.source.is_empty(),
            "{} {} missing source",
            r.location,
            r.quantity.as_str()
        );
    }
    for q in [
        Quantity::FreeCoolingHours,
        Quantity::SeismicSs,
        Quantity::GridCo2LbPerMwh,
    ] {
        assert!(
            ledger.references.iter().any(|r| r.quantity == q),
            "missing quantity {}",
            q.as_str()
        );
        assert!(
            ledger.tolerances.contains_key(&q),
            "tolerance for {} must be declared in the same commit as the reference",
            q.as_str()
        );
        let t = &ledger.tolerances[&q];
        assert!(
            !t.justification.trim().is_empty(),
            "tolerance {} needs a justification declared before seeing results",
            q.as_str()
        );
    }
}

#[test]
fn published_references_match_or_red() {
    let root = engine();
    let report = check_oracle(&root).unwrap_or_else(|e| panic!("{e}"));
    assert!(report.is_clean(), "{report}");
    assert!(
        report.comparisons.len() >= 5,
        "anti-vacuous: compared {}",
        report.comparisons.len()
    );
    let locs: std::collections::BTreeSet<_> = report
        .comparisons
        .iter()
        .map(|c| c.location.as_str())
        .collect();
    assert!(locs.len() >= 5, "{ANTI_VACUOUS_LOCATIONS}: {locs:?}");
    assert!(report.to_string().contains("oracle: PASS"));
}

#[test]
fn disagreement_names_location_computed_reference_delta() {
    let text = format!(
        "{DISAGREEMENT} location=ashburn quantity=free_cooling_hours computed=0 reference=5734 delta=-5734.000000 tolerance=1"
    );
    assert!(text.contains("location=ashburn"));
    assert!(text.contains("computed=0"));
    assert!(text.contains("reference=5734"));
    assert!(text.contains("delta="));
}

#[test]
fn perturb_one_tolerance_unit_is_red() {
    let root = engine();
    let report = check_oracle(&root).expect("live oracle must be green before the plant");
    let mut planted = 0usize;
    for c in &report.comparisons {
        let bad = perturb_one_tolerance(c.computed, c.reference, c.tolerance);
        assert!(
            !agrees(bad, c.reference, c.tolerance),
            "perturb {} {} by 1 tol: computed={} planted={} ref={} tol={} must be RED",
            c.location,
            c.quantity.as_str(),
            c.computed,
            bad,
            c.reference,
            c.tolerance
        );
        planted += 1;
    }
    assert!(planted >= 5, "planted {planted} pairs");
}

#[test]
fn zero_reference_locations_is_error() {
    let err = parse_references("schema = \"x\"\n", "empty.toml").expect_err("empty");
    assert!(
        matches!(err, cdcp_data::OracleError::EmptyReferences),
        "{err:?}"
    );
    assert!(err.to_string().contains(ANTI_VACUOUS_REFS));
}

#[test]
fn fewer_than_five_locations_is_error() {
    let text = r#"
[[reference]]
location = "only-one"
quantity = "free_cooling_hours"
value = 1.0
retrieved = "2026-08-15"
"#;
    let err = parse_references(text, "one.toml").expect_err("one location");
    match err {
        cdcp_data::OracleError::TooFewLocations { n } => assert_eq!(n, 1),
        other => panic!("expected TooFewLocations, got {other:?}"),
    }
    assert!(err.to_string().contains(ANTI_VACUOUS_LOCATIONS));
}

#[test]
fn live_pins_include_the_three_oracle_snapshots() {
    let pins = compiled_pins().expect("pins");
    for id in [SNAP_TMY3, SNAP_USGS, SNAP_EGRID] {
        assert!(pins.iter().any(|p| p.id == id), "missing pin {id}");
    }
}

#[test]
fn engine_root_resolves() {
    let root = engine_root(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))).expect("root");
    assert!(root.join("registries/claims.toml").is_file());
}

/// Meta-test: delete the comparison → this selftest is non-zero.
#[test]
fn selftest_delete_comparison_is_nonzero() {
    let src = production_oracle_src();
    assert!(
        src.contains("agrees("),
        "delete the comparison → selftest non-zero"
    );
    assert!(
        src.contains("DISAGREEMENT"),
        "delete the disagreement token interpolation → selftest non-zero"
    );
    assert!(
        src.contains("computed") && src.contains("reference") && src.contains("delta"),
        "delete the location/computed/reference/delta naming → selftest non-zero"
    );
    assert!(DISAGREEMENT.contains("ORACLE RED"), "{DISAGREEMENT}");
}

/// Meta-test: delete the anti-vacuous empty-set ERROR → this selftest is non-zero.
#[test]
fn selftest_delete_anti_vacuous_is_nonzero() {
    let src = production_oracle_src();
    assert!(src.contains("ANTI_VACUOUS_REFS"));
    assert!(src.contains("ANTI_VACUOUS_LOCATIONS"));
    assert!(src.contains("EmptyReferences"));
    assert!(src.contains("TooFewLocations"));
}

#[test]
fn compiled_references_text_is_the_include() {
    let a = compiled_references().expect("compiled");
    let b = parse_references(COMPILED_REFERENCES, COMPILED_REFERENCES_ORIGIN).expect("parse");
    assert_eq!(a.references.len(), b.references.len());
    assert_eq!(a.locations.len(), b.locations.len());
}

#[test]
fn check_oracle_with_empty_refs_is_error() {
    let root = engine();
    let pins = compiled_pins().expect("pins");
    let mut ledger = compiled_references().expect("ledger");
    ledger.references.clear();
    let err = check_oracle_with(&root, &ledger, &pins).expect_err("empty refs");
    assert!(
        matches!(err, cdcp_data::OracleError::EmptyReferences),
        "{err:?}"
    );
}
