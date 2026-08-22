//! Dispatcher coverage for the absolute/universal plausibility measurement.

mod support;

use std::path::Path;

#[test]
fn live_gate_runs_product_detector_and_names_measured_branch() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, output) = support::run_gate(&root, &["plausibility-detector"]);
    assert_eq!(
        code, 2,
        "the measured live defect must be visible: {output}"
    );
    assert!(output.contains("bank-wide: scanned=957"), "{output}");
    assert!(
        output.contains("applicable_exactly_three=47")
            && output.contains("key_is_lone_plausible=38")
            && output.contains("rate=80.9%"),
        "{output}"
    );
    assert!(output.contains(cdcp_bank::plausibility::BRANCH), "{output}");
    assert!(
        output.contains("LIMITATION: lexical absolute/universal sub-case of F-01 only"),
        "{output}"
    );
}

#[test]
fn an_empty_bank_is_error_exit_four_not_a_silent_pass() {
    let fixture = support::Fixture::empty();
    let (code, output) = support::run_gate(&fixture.root, &["plausibility-detector"]);
    assert_eq!(code, 4, "{output}");
    assert_ne!(code, 0, "empty bank must not pass: {output}");
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, output) = support::run_gate(&root, &["plausibility-detector", "--approved-only"]);
    assert_eq!(code, 3, "{output}");
}
