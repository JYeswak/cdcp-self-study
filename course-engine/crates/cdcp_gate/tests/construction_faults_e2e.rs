//! Dispatcher coverage for the construction-fault gate.

mod support;

use std::path::Path;

#[test]
fn the_gate_is_registered_and_reports_both_populations() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, output) = support::run_gate(&root, &["construction-faults"]);
    assert_eq!(code, 2, "{output}");
    assert!(output.contains("live-approved: items=931"), "{output}");
    assert!(output.contains("damaged-corpus: items=448"), "{output}");
    assert!(output.contains("length-rank-uniformity"), "{output}");
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, output) = support::run_gate(&root, &["construction-faults", "--threshold=0"]);
    assert_eq!(code, 3, "{output}");
}
