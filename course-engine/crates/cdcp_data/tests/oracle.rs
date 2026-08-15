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
    parse_references, perturb_one_tolerance, OracleError, Quantity, ANTI_VACUOUS_LOCATIONS,
    ANTI_VACUOUS_REFS, COMPILED_REFERENCES, COMPILED_REFERENCES_ORIGIN, DISAGREEMENT, SNAP_EGRID,
    SNAP_TMY3, SNAP_USGS,
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
    assert!(
        ledger
            .references
            .iter()
            .all(|r| r.quantity != Quantity::FreeCoolingHours),
        "free-cooling hours are MMS-only; STAT/ASHRAE/NREL do not publish this hour count"
    );
    for q in [
        Quantity::HeatingDegreeDays18c,
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
    match check_oracle(&root) {
        Ok(report) => {
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
        Err(e) => {
            let text = e.to_string();
            assert!(
                matches!(e, OracleError::Disagreement { .. }),
                "live miss must be Disagreement, not a structural error: {e:?}"
            );
            for needle in [
                "location=",
                "computed=",
                "reference=",
                "delta=",
                DISAGREEMENT,
            ] {
                assert!(
                    text.contains(needle),
                    "honest RED must name {needle}: {text}"
                );
            }
            // Official published refs are kept. Do not rewrite them to match us.
            panic!("oracle honestly RED (published refs kept):\n{text}");
        }
    }
}

#[test]
fn disagreement_names_location_computed_reference_delta() {
    let root = engine();
    let pins = compiled_pins().expect("pins");
    let mut ledger = compiled_references().expect("ledger");
    let idx = ledger
        .references
        .iter()
        .position(|r| r.location == "ashburn" && r.quantity == Quantity::SeismicSs)
        .expect("ashburn seismic_ss is in the ledger");
    let planted_ref = 99.0;
    ledger.references[idx].value = planted_ref;

    let err =
        check_oracle_with(&root, &ledger, &pins).expect_err("known-bad published ref must be RED");
    let OracleError::Disagreement { findings } = &err else {
        panic!("expected OracleError::Disagreement, got {err:?}");
    };
    let f = findings
        .iter()
        .find(|c| c.location == "ashburn" && c.quantity == Quantity::SeismicSs)
        .expect("Disagreement must name the planted ashburn seismic_ss pair");
    assert!(!f.ok, "planted pair must be outside its band");
    assert_eq!(f.reference, planted_ref);
    assert_eq!(f.location, "ashburn");
    assert_eq!(f.delta, f.computed - planted_ref);

    // Production Display interpolates format_disagreements. A tautological
    // format!(...) of a local string would not carry the live computed.
    let text = err.to_string();
    assert!(text.contains(DISAGREEMENT), "{text}");
    assert!(text.contains("location=ashburn"), "{text}");
    assert!(
        text.contains(&format!("computed={}", f.computed)),
        "Display must name live computed={}: {text}",
        f.computed
    );
    assert!(text.contains("reference=99"), "{text}");
    assert!(
        text.contains(&format!("delta={:.6}", f.delta)),
        "Display must name delta={:.6}: {text}",
        f.delta
    );
    assert!(text.contains("quantity=seismic_ss"), "{text}");
}

#[test]
fn perturb_one_tolerance_unit_is_red() {
    let root = engine();
    let pins = compiled_pins().expect("pins");
    let ledger = compiled_references().expect("ledger");

    // Recover live computed values even when the official ledger is already
    // RED: plant every published number at 1e12, then read findings.
    let mut probe = ledger.clone();
    for r in &mut probe.references {
        r.value = 1.0e12;
    }
    let probe_err =
        check_oracle_with(&root, &probe, &pins).expect_err("absurd 1e12 refs must be RED");
    let OracleError::Disagreement {
        findings: probe_findings,
    } = &probe_err
    else {
        panic!("expected Disagreement from 1e12 plant, got {probe_err:?}");
    };
    assert_eq!(
        probe_findings.len(),
        ledger.references.len(),
        "every pair must disagree with 1e12 so we recover every computed"
    );

    let mut planted_ledger = ledger.clone();
    let mut expected = Vec::new();
    for f in probe_findings {
        let original = ledger
            .references
            .iter()
            .find(|r| r.location == f.location && r.quantity == f.quantity)
            .expect("probe finding matches a ledger row")
            .value;
        let planted = perturb_one_tolerance(f.computed, original, f.tolerance);
        assert!(
            !agrees(f.computed, planted, f.tolerance),
            "1-tol plant {} {} computed={} planted={} tol={} must be outside the open band",
            f.location,
            f.quantity.as_str(),
            f.computed,
            planted,
            f.tolerance
        );
        for r in &mut planted_ledger.references {
            if r.location == f.location && r.quantity == f.quantity {
                r.value = planted;
            }
        }
        expected.push((
            f.location.clone(),
            f.quantity,
            f.computed,
            planted,
            f.tolerance,
        ));
    }

    let err = check_oracle_with(&root, &planted_ledger, &pins)
        .expect_err("one-tolerance-unit plant must be OracleError::Disagreement");
    let text = err.to_string();
    let OracleError::Disagreement { findings } = &err else {
        panic!("expected OracleError::Disagreement, got {err:?}");
    };
    assert!(
        findings.len() >= 5,
        "planted {} pairs through check_oracle_with",
        findings.len()
    );
    for (loc, qty, computed, planted, _tol) in &expected {
        let f = findings
            .iter()
            .find(|c| c.location == *loc && c.quantity == *qty)
            .unwrap_or_else(|| panic!("Disagreement must name {loc} {}", qty.as_str()));
        assert!(!f.ok);
        assert_eq!(f.computed, *computed);
        assert_eq!(f.reference, *planted);
        assert!(
            text.contains(&format!("location={loc}")),
            "Display must name location={loc}: {text}"
        );
    }
    for needle in ["computed=", "reference=", "delta=", DISAGREEMENT] {
        assert!(text.contains(needle), "Display must name {needle}: {text}");
    }
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
