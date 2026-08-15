//! Product CLI for the F3 differential oracle (`bd-hardening-f-oracle-qly.5`).
//!
//! BUILT ≠ WIRED: `cdcp_data::check_oracle` is cargo-test only until this
//! verb exists. No network — compiled references + vendored snapshots.

use cdcp_data::{
    check_oracle, check_oracle_with, compiled_pins, compiled_references, engine_root,
    perturb_one_tolerance, Comparison, OracleError, ANTI_VACUOUS_REFS, DISAGREEMENT,
};
use std::path::{Path, PathBuf};

/// `cdcp oracle-check` / `cdcp oracle`.
pub(crate) fn run(root: Option<&Path>, selftest: bool) -> Result<(), String> {
    let resolved = resolve_root(root)?;
    if selftest {
        return run_selftest(&resolved);
    }
    match check_oracle(&resolved) {
        Ok(report) => {
            print!("{report}");
            if !report.is_clean() {
                return Err(format!(
                    "oracle RED (compared={})",
                    report.comparisons.len()
                ));
            }
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

fn resolve_root(root: Option<&Path>) -> Result<PathBuf, String> {
    match root {
        Some(p) => Ok(p.to_path_buf()),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            engine_root(&start).map_err(|e| e.to_string())
        }
    }
}

/// A pair we can plant against. Live GREEN yields comparisons; live
/// honest RED yields findings. Other errors stay structural failures.
///
/// qly.6 made F3 honestly RED. Requiring GREEN here was the theater
/// that blocked `--selftest` (`requires a green live oracle first`).
fn pair_to_plant(root: &Path) -> Result<Comparison, String> {
    match check_oracle(root) {
        Ok(report) => report.comparisons.into_iter().next().ok_or_else(|| {
            "oracle-check --selftest: live oracle compared nothing — an empty set is not a plant"
                .to_string()
        }),
        Err(OracleError::Disagreement { findings }) => {
            findings.into_iter().next().ok_or_else(|| {
                "oracle-check --selftest: Disagreement with zero findings — not a plant".to_string()
            })
        }
        Err(e) => Err(format!(
            "oracle-check --selftest: live oracle failed structurally: {e}"
        )),
    }
}

/// Plant: perturb one published ref, then delete all refs. Both must RED
/// and the disagreement must name location / computed / reference / delta.
///
/// Live may already be honestly RED (EPA eGRID SRCO2RTA vs plant-subset).
/// That is the product working — not a reason to refuse the plants.
/// Exit 0 means the plants tripped. A plant that stays GREEN is the failure.
fn run_selftest(root: &Path) -> Result<(), String> {
    let pins = compiled_pins().map_err(|e| e.to_string())?;
    let pair = pair_to_plant(root)?;

    let mut ledger = compiled_references().map_err(|e| e.to_string())?;
    let planted = perturb_one_tolerance(pair.computed, pair.reference, pair.tolerance);
    let mut planted_n = 0usize;
    for r in &mut ledger.references {
        if r.location == pair.location && r.quantity == pair.quantity {
            r.value = planted;
            planted_n += 1;
            break;
        }
    }
    if planted_n == 0 {
        return Err(format!(
            "oracle-check --selftest: no ledger row for {} {}",
            pair.location,
            pair.quantity.as_str()
        ));
    }

    let perturb_err = match check_oracle_with(root, &ledger, &pins) {
        Err(e) => e,
        Ok(report) => {
            return Err(format!(
                "oracle-check --selftest: perturb one published ref stayed GREEN (compared={})",
                report.comparisons.len()
            ));
        }
    };
    let perturb_text = perturb_err.to_string();
    match &perturb_err {
        OracleError::Disagreement { findings } if !findings.is_empty() => {
            // Live eGRID is already RED. Prove *this* plant landed: the
            // planted pair must appear with the planted reference, not
            // just that some other official miss is still disagreeing.
            let hit = findings
                .iter()
                .find(|f| f.location == pair.location && f.quantity == pair.quantity);
            let Some(f) = hit else {
                return Err(format!(
                    "oracle-check --selftest: planted {} {} missing from findings: {perturb_text}",
                    pair.location,
                    pair.quantity.as_str()
                ));
            };
            if f.ok || f.reference != planted {
                return Err(format!(
                    "oracle-check --selftest: planted reference={planted} not what findings report (reference={} ok={})",
                    f.reference, f.ok
                ));
            }
        }
        OracleError::Disagreement { .. } => {
            return Err("oracle-check --selftest: Disagreement with zero findings".into());
        }
        other => {
            return Err(format!(
                "oracle-check --selftest: expected Disagreement, got {other}"
            ));
        }
    }
    for needle in [
        "location=",
        "computed=",
        "reference=",
        "delta=",
        DISAGREEMENT,
        pair.location.as_str(),
    ] {
        if !perturb_text.contains(needle) {
            return Err(format!(
                "oracle-check --selftest: disagreement must name {needle}: {perturb_text}"
            ));
        }
    }
    print!("{perturb_text}\n");

    ledger.references.clear();
    let empty_err = match check_oracle_with(root, &ledger, &pins) {
        Err(e) => e,
        Ok(_) => return Err("oracle-check --selftest: empty refs stayed GREEN".into()),
    };
    if !matches!(empty_err, OracleError::EmptyReferences) {
        return Err(format!(
            "oracle-check --selftest: expected EmptyReferences, got {empty_err}"
        ));
    }
    let empty_text = empty_err.to_string();
    if !empty_text.contains(ANTI_VACUOUS_REFS) {
        return Err(format!(
            "oracle-check --selftest: empty-refs must name the anti-vacuous token: {empty_text}"
        ));
    }
    println!("{empty_text}");
    println!("oracle-check --selftest: PASS (perturb RED, empty-refs RED)");
    Ok(())
}
