//! Dispatcher coverage for the construction-fault gate.

mod support;

use std::path::Path;

#[test]
fn the_gate_is_registered_and_reports_both_populations() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, output) = support::run_gate(&root, &["construction-faults"]);
    assert_eq!(code, 0, "{output}");
    let approved =
        cdcp_registry_check::count_pins::expected_value(&root, "bank.approved-count").unwrap();
    assert!(
        output.contains(&format!("live-approved: items={approved}")),
        "{output}"
    );
    assert!(output.contains("damaged-corpus: items=448"), "{output}");
    assert!(output.contains("length-rank-uniformity"), "{output}");
    assert!(
        output.contains("live-approved verdict=PASS")
            && output.contains("damaged-corpus verdict=EXPECTED-RED"),
        "{output}"
    );
    assert!(
        output.contains(
            "GREEN-DOES-NOT-PROVE: exit 0 here means four named option-set cues are absent."
        ),
        "{output}"
    );
    assert!(
        output.contains("longest-option-correct is observational only"),
        "{output}"
    );
}

#[test]
fn empty_live_scan_is_error_exit_four() {
    let fixture = support::Fixture::empty();
    std::fs::create_dir_all(fixture.path("bank/items")).unwrap();
    let policy = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registries/construction_faults.toml"),
    )
    .unwrap();
    fixture.write("registries/construction_faults.toml", &policy);

    let (code, output) = support::run_gate(&fixture.root, &["construction-faults"]);
    assert_eq!(code, 4, "{output}");
    assert_ne!(code, 0, "empty live scan must not pass: {output}");
    assert!(
        output.contains("zero approved single-select items"),
        "{output}"
    );
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, output) = support::run_gate(&root, &["construction-faults", "--threshold=0"]);
    assert_eq!(code, 3, "{output}");
}
