//! Known-bad plants: omit boundary → ERROR. Empty plant set is itself RED.
//!
//! Meta-test: delete the boundary check (`require_boundary` /
//! `doc.get("boundary")` / `MISSING_BOUNDARY`) → this file is non-zero.

use cdcp_metrics::{
    parse_metric, Boundary, CarbonAccounting, EnergyWaterMix, Ewif, HydroReservoir, ItMeter,
    Metric, MetricKind, Ratio, ReusePolicy, ScopeItem, BARE_NUMBER, EMPTY_BOUNDARY,
    EWIF_EXCLUDES_HYDRO_TWICE, KINDS, MISSING_BOUNDARY,
};

struct Plant {
    kind: &'static str,
    why: &'static str,
    doc: &'static str,
}

/// One planted failure per kind plus the bare-number class. An empty
/// list is itself a test failure.
fn plants() -> Vec<Plant> {
    vec![
        Plant {
            kind: "pue",
            why: "omit boundary",
            doc: "kind = \"pue\"\nvalue = { num = 6, den = 5 }\n",
        },
        Plant {
            kind: "pue",
            why: "empty boundary table",
            doc: "kind = \"pue\"\nvalue = { num = 6, den = 5 }\n[boundary]\n",
        },
        Plant {
            kind: "pue",
            why: "bare marketing number",
            doc: "1.2",
        },
        Plant {
            kind: "wue",
            why: "omit boundary",
            doc: "kind = \"wue\"\nvalue = { num = 9, den = 5 }\n",
        },
        Plant {
            kind: "wue",
            why: "1.8 L/kWh with no site/source declaration",
            doc: "1.8",
        },
        Plant {
            kind: "wue",
            why: "source WUE omits hydro",
            doc: "\
kind = \"wue\"
value = { num = 9, den = 5 }
[boundary]
it_meter = \"it-input\"
includes = [\"cooling-tower-evaporation\", \"energy-water\"]
excludes = [\"fire-water\"]
water_scope = \"source\"
",
        },
        Plant {
            kind: "cue",
            why: "omit boundary",
            doc: "kind = \"cue\"\nvalue = { num = 1, den = 4 }\n",
        },
        Plant {
            kind: "cue",
            why: "CUE without carbon accounting",
            doc: "\
kind = \"cue\"
value = { num = 1, den = 4 }
[boundary]
it_meter = \"ups-output\"
includes = [\"scope-1-on-site\", \"scope-2-purchased-energy\"]
excludes = [\"scope-3-upstream\"]
",
        },
        Plant {
            kind: "ere",
            why: "omit boundary",
            doc: "kind = \"ere\"\nvalue = { num = 1, den = 1 }\n",
        },
        Plant {
            kind: "ere",
            why: "ERE without reuse policy",
            doc: "\
kind = \"ere\"
value = { num = 1, den = 1 }
[boundary]
it_meter = \"ups-output\"
includes = [\"it-energy\", \"district-heat\"]
excludes = [\"office-hvac\"]
",
        },
        Plant {
            kind: "pue",
            why: "float value is not a rational",
            doc: "\
kind = \"pue\"
value = 1.2
[boundary]
it_meter = \"ups-output\"
includes = [\"it-energy\", \"cooling\"]
excludes = [\"office-hvac\"]
",
        },
    ]
}

#[test]
fn known_bad_set_is_non_empty() {
    let p = plants();
    assert!(!p.is_empty(), "empty known-bad set is an ERROR, not a pass");
}

#[test]
fn every_kind_has_at_least_one_omit_boundary_plant() {
    let p = plants();
    for kind in KINDS {
        assert!(
            p.iter()
                .any(|x| x.kind == *kind && x.why.contains("omit boundary")),
            "kind {kind} has no omit-boundary plant"
        );
    }
}

#[test]
fn every_known_bad_is_red() {
    for plant in plants() {
        let result = parse_metric(plant.doc);
        assert!(
            result.is_err(),
            "planted {} ({}) scored green: {:?}",
            plant.kind,
            plant.why,
            result
        );
    }
}

#[test]
fn omit_boundary_is_the_named_schema_error() {
    for plant in plants().into_iter().filter(|p| p.why == "omit boundary") {
        let err = parse_metric(plant.doc).expect_err(plant.kind);
        assert_eq!(err, cdcp_metrics::MetricsError::MissingBoundary);
        assert!(
            err.to_string().contains(MISSING_BOUNDARY),
            "{kind}: {err}",
            kind = plant.kind
        );
    }
}

#[test]
fn bare_number_is_the_named_schema_error() {
    for raw in ["1.8", "1.2", "9/5", "18"] {
        let err = parse_metric(raw).expect_err(raw);
        assert_eq!(err, cdcp_metrics::MetricsError::BareNumber);
        assert!(err.to_string().contains(BARE_NUMBER), "{raw}: {err}");
    }
}

#[test]
fn empty_boundary_table_is_named() {
    let err = parse_metric("kind = \"pue\"\nvalue = { num = 6, den = 5 }\n[boundary]\n")
        .expect_err("empty");
    assert_eq!(err, cdcp_metrics::MetricsError::EmptyBoundary);
    assert!(err.to_string().contains(EMPTY_BOUNDARY));
}

#[test]
fn rust_constructors_cannot_drop_the_boundary() {
    // There is no Metric::from_int / from_f64. A missing water_scope on
    // WUE and a missing carbon on CUE are schema errors.
    let pue_b = Boundary::pue(
        ItMeter::UpsOutput,
        [ScopeItem::ItEnergy, ScopeItem::Cooling],
        [ScopeItem::OfficeHvac],
    )
    .unwrap();
    assert!(Metric::declared(MetricKind::Wue, Ratio::from_int(1), pue_b.clone()).is_err());
    assert!(Metric::declared(MetricKind::Cue, Ratio::from_int(1), pue_b).is_err());

    let cue_b = Boundary::cue(
        ItMeter::UpsOutput,
        CarbonAccounting::Scope1,
        [ScopeItem::Scope1OnSite],
        [ScopeItem::Scope3Upstream],
    )
    .unwrap();
    assert!(Metric::declared(MetricKind::Pue, Ratio::new(6, 5).unwrap(), cue_b).is_err());
}

#[test]
fn tgg_unknown_ewif_cannot_be_constructed() {
    let err = Ewif::tgg_unknown_default().expect_err("tgg default");
    assert_eq!(err, cdcp_metrics::MetricsError::EwifExcludesHydroTwice);
    assert!(err.to_string().contains(EWIF_EXCLUDES_HYDRO_TWICE));

    let err = Ewif::new(
        Ratio::new(9, 5).unwrap(),
        HydroReservoir::Excluded,
        EnergyWaterMix::NationalAverage,
    )
    .expect_err("1.8 as national average");
    assert_eq!(err, cdcp_metrics::MetricsError::EwifExcludesHydroTwice);
}

#[test]
fn recovered_not_consumed_cannot_score_as_ere() {
    let b = Boundary::ere(
        ItMeter::UpsOutput,
        ReusePolicy::RecoveredNotConsumed,
        [ScopeItem::ItEnergy, ScopeItem::DistrictHeat],
        [ScopeItem::OfficeHvac],
    )
    .unwrap();
    assert!(Metric::ere(120, 20, 100, b).is_err());
}

#[test]
fn selftest_delete_boundary_check_is_red() {
    let parse_src = include_str!("../src/parse.rs");
    let prod = parse_src
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(
        prod.contains("pub fn require_boundary"),
        "delete require_boundary → selftest non-zero"
    );
    assert!(
        prod.contains("MISSING_BOUNDARY"),
        "delete the missing-boundary token → selftest non-zero"
    );
    assert!(
        prod.contains("doc.get(\"boundary\")"),
        "delete the boundary lookup → selftest non-zero"
    );
    assert!(
        prod.contains("require_boundary(&doc)"),
        "delete the require_boundary call from parse_metric → selftest non-zero"
    );
    assert!(
        prod.contains("BARE_NUMBER"),
        "delete the bare-number token → selftest non-zero"
    );

    let metric_src = include_str!("../src/metric.rs");
    assert!(
        metric_src.contains("boundary: Boundary"),
        "Metric dropped the Boundary field"
    );
    assert!(
        !metric_src.contains("boundary: Option<"),
        "Boundary must not be optional — a missing boundary is a schema ERROR"
    );
}

#[test]
fn scoring_modules_contain_no_floating_types() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = 0usize;
    let mut hits = Vec::new();
    for ent in std::fs::read_dir(&root).expect("src") {
        let ent = ent.unwrap();
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        files += 1;
        let name = path.file_name().unwrap().to_string_lossy();
        // f64 exists only at the consume boundary in free_cooling.rs
        // (upstream cdcp_data / cdcp_site store a count as f64).
        if name == "free_cooling.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            if code.contains("f32") || code.contains("f64") {
                hits.push(format!("{}:{}:{code}", path.display(), i + 1));
            }
        }
    }
    assert!(files >= 6, "empty scan of src/ is an ERROR (found {files})");
    assert!(
        hits.is_empty(),
        "floating-point type in scoring/comparison:\n  {}",
        hits.join("\n  ")
    );
}
