//! Independent verdict for the wave-8 export-anki agreement-only case
//! (`bd-wave8-ports-agreement-only-debt-idns`).
//!
//! EXTRACT-THEN-DELETE (jhd.13) retired `scripts/export_anki.py` and the
//! `cdcp_gate` port. Product is `cdcp_anki` / `cdcp export-anki`. The leftover
//! call site was a bare `compare` against a green bank: both sides agreed,
//! and a shared fallback to the live approved pool would have shipped 779
//! cards while naming the planted all-retired tree.
//!
//! The converted case points `--root` at a PLANTED all-retired bank and
//! asserts the resolved path's item COUNT (`scanned=3`), the named finding,
//! and that nothing was written. Known-bad plants the fallback in both
//! implementations.

use cdcp_anki::{evaluate, Request, ITEMS_DIR_REL};
use std::path::{Path, PathBuf};

fn engine_root() -> PathBuf {
    cdcp_gate::root::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
    files: usize,
    scanned: usize,
}

fn compare(label: &str, root: &Path) -> Run {
    let mut req = Request::default_for(root);
    req.format = "tsv".into();
    let o = evaluate(&req);
    assert!(
        o.files.is_empty() || o.code == 0,
        "[{label}] a RED export must write nothing: {} files, code {}",
        o.files.len(),
        o.code
    );
    Run {
        code: o.code,
        stdout: o.stdout,
        stderr: o.stderr,
        files: o.files.len(),
        scanned: o.scanned,
    }
}

fn planted_all_retired() -> (tempfile::TempDir, PathBuf) {
    let td = tempfile::tempdir().unwrap();
    let root = td.path().join("r");
    let items = root.join(ITEMS_DIR_REL);
    std::fs::create_dir_all(&items).unwrap();
    for i in 0..3 {
        std::fs::write(
            items.join(format!("r{i}.toml")),
            format!(
                "id = \"r{i}\"\nstatus = \"retired\"\nmodule = 1\n\
                 stem = \"retired-only-planted-{i}\"\nchoices = [\"a\"]\ncorrect = \"A\"\n"
            ),
        )
        .unwrap();
    }
    (td, root)
}

/// THE converted case. Three retired cards, zero approved. The count is
/// what catches a silent fallback: the live bank reports scanned=804.
#[test]
fn planted_all_retired_is_red_names_the_finding_and_reports_this_banks_count() {
    let (_td, root) = planted_all_retired();
    let rs = compare("all retired bank", &root);

    assert_ne!(rs.code, 0, "all-retired must never ship a deck");
    // 1. named finding
    assert_eq!(
        rs.stderr, "FAIL: zero approved items to export\n",
        "the detector must name the planted finding: {:?}",
        rs.stderr
    );
    // 2. item COUNT of THIS bank, not of a default it fell back to
    assert_eq!(
        rs.scanned, 3,
        "scanned={} — the spelling reached a different tree than the planted 3",
        rs.scanned
    );
    // 3. resolved path produced no artifact
    assert_eq!(rs.files, 0, "must not write a retired deck");
    assert!(rs.stdout.is_empty(), "nothing on stdout: {:?}", rs.stdout);
}

/// Known-bad: both implementations ignore the named root and export the
/// live approved pool. They agree, they are GREEN, scanned is hundreds.
/// The converted verdict (scanned=3 + named finding) does not trip.
#[test]
fn known_bad_shared_fallback_passes_agreement_and_fails_the_converted_verdict() {
    let (_td, root) = planted_all_retired();
    let live = engine_root();
    let py = {
        let mut req = Request::default_for(&live);
        req.format = "tsv".into();
        evaluate(&req)
    };
    let rs = {
        let mut req = Request::default_for(&live);
        req.format = "tsv".into();
        evaluate(&req)
    };

    assert_eq!(py.code, rs.code);
    assert_eq!(py.stdout, rs.stdout);
    assert_eq!(py.stderr, rs.stderr);
    assert_eq!(
        py.code, 0,
        "live fallback is GREEN — the old assertion would pass:\n{}{}",
        py.stdout, py.stderr
    );
    assert!(
        py.scanned > 3,
        "live scanned={} must dwarf the planted 3 so the count is the detector",
        py.scanned
    );

    let converted_trips =
        rs.code != 0 && rs.stderr == "FAIL: zero approved items to export\n" && rs.scanned == 3;
    assert!(
        !converted_trips,
        "shared fallback must fail the converted verdict; scanned={} stderr={:?}",
        rs.scanned, rs.stderr
    );

    let honest = compare("control: honest scan of the plant", &root);
    assert_ne!(honest.code, 0);
    assert_eq!(honest.scanned, 3);
    assert_eq!(honest.stderr, "FAIL: zero approved items to export\n");
}
