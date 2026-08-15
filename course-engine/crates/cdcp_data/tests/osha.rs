//! OSHA / eCFR facts: exclusion, isolation constraints, known-bad 147 plant.

use cdcp_data::{
    check_osha, check_osha_with, cites_147_as_electrical_loto_authority, engine_root,
    load_compiled, IsolationConstraint, OshaFault, BACKFEED_TEST, CONTROL_DEVICES_NOT_ISOLATION,
    DEENERGIZE_FIRST, EXCLUSION_147, ISOLATION_CONSTRAINTS, SNAP_147, SNAP_269, SNAP_333,
};
use std::path::PathBuf;

fn engine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("engine")
}

#[test]
fn isolation_constraints_are_the_three_named_in_the_bead() {
    assert_eq!(ISOLATION_CONSTRAINTS.len(), 3);
    let ids: Vec<_> = ISOLATION_CONSTRAINTS.iter().map(|c| c.id).collect();
    assert_eq!(
        ids,
        [
            "deenergize-first",
            "control-devices-not-isolation",
            "backfeed-test"
        ]
    );
    for c in ISOLATION_CONSTRAINTS {
        assert!(!c.quote.is_empty());
        assert_eq!(c.snapshot_id, SNAP_333);
    }
    let _ = IsolationConstraint {
        id: ISOLATION_CONSTRAINTS[0].id,
        rule: ISOLATION_CONSTRAINTS[0].rule,
        citation: ISOLATION_CONSTRAINTS[0].citation,
        quote: ISOLATION_CONSTRAINTS[0].quote,
        snapshot_id: ISOLATION_CONSTRAINTS[0].snapshot_id,
    };
}

#[test]
fn live_osha_snapshots_load_and_carry_the_quotes() {
    let root = engine();
    let report = load_compiled(&root).expect("load");
    for id in [SNAP_147, SNAP_269, SNAP_333] {
        assert!(
            report.loaded.iter().any(|s| s.id == id),
            "missing {id} in {report}"
        );
    }
    let s147 = report.loaded.iter().find(|s| s.id == SNAP_147).unwrap();
    let body = String::from_utf8_lossy(&s147.bytes);
    assert!(
        body.contains(EXCLUSION_147),
        "147 snapshot must carry the (a)(1)(ii)(D) exclusion"
    );

    let s333 = report.loaded.iter().find(|s| s.id == SNAP_333).unwrap();
    let body = String::from_utf8_lossy(&s333.bytes);
    assert!(body.contains(DEENERGIZE_FIRST));
    assert!(body.contains(CONTROL_DEVICES_NOT_ISOLATION));
    assert!(body.contains(BACKFEED_TEST));
}

#[test]
fn live_curriculum_does_not_cite_147_as_electrical_loto() {
    let root = engine();
    let report = check_osha(&root).expect("check");
    assert!(
        report.is_clean(),
        "live tree must not treat 1910.147 as electrical-LOTO authority:\n{report}"
    );
    assert!(
        report.scanned >= 1,
        "zero units scanned is vacuous: scanned={}",
        report.scanned
    );
}

#[test]
fn plant_item_citing_147_as_electrical_loto_is_red() {
    let plant =
        "The electrical LOTO authority for data-centre switchgear and UPS is 29 CFR 1910.147.";
    assert!(
        cites_147_as_electrical_loto_authority(plant),
        "plant must trip"
    );
}

#[test]
fn m15_shape_is_not_red() {
    let lawful = r#"
29 CFR 1910.147(a)(1)(ii)(D) expressly excludes exposure to electrical
hazards in electric-utilization installations. Switchgear / UPS / PDU
work is Subpart S — 1910.333, not 1910.147.
"#;
    assert!(
        !cites_147_as_electrical_loto_authority(lawful),
        "the exclusion + Subpart S shape must stay green"
    );
}

#[test]
fn machine_loto_without_electrical_context_is_not_red() {
    let machines = "1910.147 covers unexpected energisation of chillers and pumps.";
    assert!(
        !cites_147_as_electrical_loto_authority(machines),
        "machine LOTO is what 147 actually covers"
    );
}

#[test]
fn injected_plant_file_is_red_on_the_check() {
    let root = engine();
    let loaded = load_compiled(&root).expect("load").loaded;
    let dir = std::env::temp_dir().join(format!(
        "cdcp-osha-plant-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(dir.join("bank/items")).unwrap();
    std::fs::write(
        dir.join("bank/items/plant-147.toml"),
        r#"
stem = "Who is the electrical LOTO authority for switchgear?"
correct = "29 CFR 1910.147"
explanation = "1910.147 is the electrical LOTO rule for UPS and PDU work."
"#,
    )
    .unwrap();
    // Point scan at the plant by using check_osha_with + a fake root that
    // has bank/items. engine_root is unused; we pass `dir` as root so
    // scan_tree finds the plant. Snapshots still come from `loaded`.
    let report = check_osha_with(&loaded, &dir).expect("check plant");
    assert!(
        report.faults.iter().any(|f| matches!(
            f,
            OshaFault::Cite147AsElectricalLoto { path } if path.contains("plant-147.toml")
        )),
        "plant item must be RED:\n{report}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn engine_root_still_resolves() {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(engine_root(&start)
        .unwrap()
        .join("registries/claims.toml")
        .is_file());
}

#[test]
fn selftest_quotes_are_interpolated() {
    let src = include_str!("../src/osha.rs");
    assert!(src.contains("EXCLUSION_147"));
    assert!(src.contains("DEENERGIZE_FIRST"));
    assert!(src.contains("CONTROL_DEVICES_NOT_ISOLATION"));
    assert!(src.contains("BACKFEED_TEST"));
    assert!(src.contains("cites_147_as_electrical_loto_authority"));
}
