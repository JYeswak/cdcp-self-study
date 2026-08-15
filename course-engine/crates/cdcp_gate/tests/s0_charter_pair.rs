//! Named S0 assertion for the CHARTER mutate/delete pair.
//!
//! Bead: `bd-single-leg-metatest-closes-illw`.
//!
//! This file is the assertion half of the pair required by `.flywheel/CHARTER.md`:
//!
//!   1. MUTATE the S0 detector (`scan_reason` in
//!      `crates/cdcp_gate/src/gates/substrate_guard.rs`) so it no longer
//!      identifies a scanned extension → this suite goes non-zero.
//!   2. With that mutation STILL IN PLACE, delete this assertion → the suite
//!      returns to zero.
//!
//! The driver is `scripts/selftest_reconstructed.sh`. Restore of the
//! cargo-compiled sources goes through `scripts/restore_safe.inc.sh`
//! (`cdcp_restore_safe`); never `mv` a backup over dest.
//!
//! FLOOR-RAISE: this test asserts that `scan_reason` identifies an unlisted
//! `scripts/foo.py` as a scanned `.py` extension. It cannot decide that a
//! `.rs` file is not secretly shelling out to Python.

use cdcp_gate::gates::substrate_guard::{scan_reason, Entry, ScanCfg, ScanReason};

fn scan() -> ScanCfg {
    ScanCfg {
        roots: vec!["scripts".into(), "crates".into()],
        extensions: vec!["py".into(), "sh".into()],
        include_engine_root_files: true,
    }
}

#[test]
fn unlisted_python_is_identified_as_a_scanned_extension() {
    let e = Entry {
        path: "scripts/foo.py".into(),
        mode: "100644".into(),
        shebang: None,
    };
    let reason = scan_reason(&e, &scan());
    assert!(
        matches!(reason, Some(ScanReason::Extension(ref ext)) if ext == "py"),
        "S0 detector must identify scripts/foo.py as a scanned .py extension, got {reason:?}"
    );
}
