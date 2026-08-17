//! Dispatcher smoke for `cdcp_gate near-duplicate-items`.
//! Product tests live in `cdcp_bank::near_duplicate` /
//! `cdcp_bank/tests/near_duplicate.rs`.

mod support;

use std::path::Path;

#[test]
fn the_gate_is_registered_in_the_dispatcher() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, out) = support::run_gate(&root, &["list"]);
    assert_eq!(code, 0);
    assert!(out.contains("near-duplicate-items"), "{out}");
}

#[test]
fn an_unknown_flag_is_usage_not_a_silent_pass() {
    let root =
        cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root");
    let (code, out) = support::run_gate(&root, &["near-duplicate-items", "--threshold=0"]);
    assert_eq!(code, 3, "{out}");
    assert!(!out
        .lines()
        .any(|l| l.starts_with("near-duplicate-items: ok:")));
}
