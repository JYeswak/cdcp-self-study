//! Tick-ledger reconciliation: every bead closed since the baseline must appear
//! in a tick receipt.
//!
//! # Why this exists
//!
//! `tick_emitter` was built, tested, closed against an acceptance clause demanding
//! it be "reachable from the actual local workflow" — and then invoked zero times
//! for 154 commits and 18 bead closes. BUILT is not WIRED. `watchdog.sh` observed
//! the stall correctly and, by explicit design, never dispatches; it wrote 111
//! consecutive STALL rows and one `URGENT_JOSH.md` that nobody read.
//!
//! A file nobody reads is not a gate. This gate sits on the edge where a claim of
//! done becomes permanent: you may not hold a green chain while closed beads have
//! no receipt.
//!
//! # What it mechanically enforces
//!
//! Every bead whose `closed_at` is at or after `baseline_utc` is named by the
//! `bead` field of some row in the ledger.
//!
//! # What it CANNOT decide
//!
//! Whether the receipt is TRUE. A row naming a bead satisfies this gate no matter
//! how worthless its `value_added` is. This raises the floor from "closes may be
//! invisible" to "closes must be declared" — an author who writes a hollow receipt
//! still passes. That judgement stays with a human reading the ledger.
//!
//! It also cannot see a bead closed and then reopened, nor work that shipped
//! without any bead at all.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const NAME: &str = "tick-reconcile";
pub const SUMMARY: &str = "every bead closed since the baseline has a tick receipt";

/// One-time amnesty anchor. Closes before this instant predate the gate and are
/// accounted for in aggregate by the T12 reconciliation row, not individually.
const BASELINE_REGISTRY: &str = "registries/tick_reconcile.toml";

pub enum ReconcileError {
    /// Closed beads with no receipt. One string per bead.
    Unreconciled(Vec<String>),
    /// The gate could not honestly evaluate.
    Error(String),
}

fn read_baseline(root: &Path) -> Result<String, ReconcileError> {
    let path = root.join(BASELINE_REGISTRY);
    let text = fs::read_to_string(&path)
        .map_err(|e| ReconcileError::Error(format!("{BASELINE_REGISTRY}: {e}")))?;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if let Some(rest) = line.strip_prefix("baseline_utc") {
            if let Some(v) = rest.split('=').nth(1) {
                let v = v.trim().trim_matches('"').to_string();
                if !v.is_empty() {
                    return Ok(v);
                }
            }
        }
    }
    Err(ReconcileError::Error(format!(
        "{BASELINE_REGISTRY}: no baseline_utc — a ratchet with no anchor is not a ratchet"
    )))
}

/// Bead IDs named by any ledger row. The `bead` field may carry a comma-separated
/// list, so it is split rather than compared whole.
fn ledger_beads(root: &Path) -> Result<BTreeSet<String>, ReconcileError> {
    let path = root.join(".flywheel/tick-ledger.jsonl");
    let text = fs::read_to_string(&path)
        .map_err(|e| ReconcileError::Error(format!("tick-ledger.jsonl: {e}")))?;
    let mut out = BTreeSet::new();
    let mut rows = 0usize;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        rows += 1;
        if let Some(field) = json_string_field(line, "bead") {
            for id in field.split(',') {
                let id = id.trim();
                if !id.is_empty() {
                    out.insert(id.to_string());
                }
            }
        }
    }
    if rows == 0 {
        return Err(ReconcileError::Error(
            "tick-ledger.jsonl is empty — an empty ledger is an ERROR, never a pass".into(),
        ));
    }
    Ok(out)
}

/// Minimal extractor for a top-level `"key":"value"` pair. The ledger and the
/// beads export are both machine-written single-line JSON, so this avoids taking
/// a parser dependency for two string fields. Returns None if the key is absent
/// or its value is not a plain string.
fn json_string_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let mut from = 0usize;
    while let Some(idx) = line[from..].find(&needle) {
        let start = from + idx + needle.len();
        let rest = line[start..].trim_start();
        if let Some(body) = rest.strip_prefix('"') {
            let mut out = String::new();
            let mut chars = body.chars();
            while let Some(c) = chars.next() {
                match c {
                    '\\' => {
                        if let Some(esc) = chars.next() {
                            out.push(esc);
                        }
                    }
                    '"' => return Some(out),
                    other => out.push(other),
                }
            }
            return None;
        }
        from = start;
    }
    None
}

pub fn run(root: &Path) -> Result<String, ReconcileError> {
    let baseline = read_baseline(root)?;
    let receipted = ledger_beads(root)?;

    let beads_path = root.join(".beads/issues.jsonl");
    let text = fs::read_to_string(&beads_path)
        .map_err(|e| ReconcileError::Error(format!(".beads/issues.jsonl: {e}")))?;

    let mut scanned = 0usize;
    let mut closed_since = 0usize;
    let mut missing: Vec<String> = Vec::new();

    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        scanned += 1;
        if json_string_field(line, "status").as_deref() != Some("closed") {
            continue;
        }
        let closed_at = match json_string_field(line, "closed_at") {
            Some(v) if !v.is_empty() => v,
            // A closed bead with no timestamp cannot be placed relative to the
            // baseline. Fail closed: unplaceable is not the same as old.
            _ => {
                let id = json_string_field(line, "id").unwrap_or_else(|| "<no id>".into());
                missing.push(format!("{id}: status=closed with no closed_at — cannot be placed against the baseline"));
                continue;
            }
        };
        if closed_at.as_str() < baseline.as_str() {
            continue;
        }
        closed_since += 1;
        let id = match json_string_field(line, "id") {
            Some(v) => v,
            None => continue,
        };
        if !receipted.contains(&id) {
            let title = json_string_field(line, "title").unwrap_or_default();
            let short: String = title.chars().take(60).collect();
            missing.push(format!("{id} closed {closed_at} with no tick receipt — {short}"));
        }
    }

    if scanned == 0 {
        return Err(ReconcileError::Error(
            ".beads/issues.jsonl scanned 0 rows — an empty scan set is an ERROR, never a pass".into(),
        ));
    }

    if !missing.is_empty() {
        missing.sort();
        return Err(ReconcileError::Unreconciled(missing));
    }

    Ok(format!(
        "tick-reconcile: {closed_since} bead(s) closed since {baseline}, all receipted ({scanned} rows scanned, {} receipted ids in ledger). Does NOT check that any receipt is true.",
        receipted.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an isolated root. Tests live in `cdcp_bank` rather than
    /// `cdcp_gate/tests` on purpose: `gate_shrink` counts cdcp_gate's src AND
    /// tests against a ceiling with ~90 lines of headroom, so putting them there
    /// would spend the whole margin. The gate file stays a thin dispatcher.
    fn root(baseline: &str, ledger: &str, beads: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path();
        fs::create_dir_all(p.join("registries")).unwrap();
        fs::create_dir_all(p.join(".flywheel")).unwrap();
        fs::create_dir_all(p.join(".beads")).unwrap();
        fs::write(
            p.join(BASELINE_REGISTRY),
            format!("baseline_utc = \"{baseline}\"\n"),
        )
        .unwrap();
        fs::write(p.join(".flywheel/tick-ledger.jsonl"), ledger).unwrap();
        fs::write(p.join(".beads/issues.jsonl"), beads).unwrap();
        dir
    }

    const RECEIPTED: &str = r#"{"tick":"T1","bead":"bd-aaa,bd-bbb"}"#;

    #[test]
    fn known_good_closed_bead_with_a_receipt_passes() {
        let beads = r#"{"id":"bd-aaa","status":"closed","closed_at":"2026-08-20T12:00:00Z","title":"t"}"#;
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, beads);
        let out = run(dir.path()).unwrap_or_else(|_| panic!("known-good must pass"));
        assert!(out.contains("1 bead(s) closed"), "{out}");
    }

    #[test]
    fn known_bad_closed_bead_without_a_receipt_fails() {
        let beads = r#"{"id":"bd-zzz","status":"closed","closed_at":"2026-08-20T12:00:00Z","title":"unreceipted"}"#;
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, beads);
        match run(dir.path()) {
            Err(ReconcileError::Unreconciled(v)) => {
                assert_eq!(v.len(), 1);
                assert!(v[0].contains("bd-zzz"), "{:?}", v);
            }
            _ => panic!("known-bad must be Unreconciled"),
        }
    }

    #[test]
    fn a_close_before_the_baseline_is_out_of_scope() {
        let beads = r#"{"id":"bd-zzz","status":"closed","closed_at":"2026-08-01T00:00:00Z","title":"old"}"#;
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, beads);
        assert!(run(dir.path()).is_ok(), "pre-baseline closes are amnestied");
    }

    /// Fail closed, not open: a close with no timestamp cannot be placed against
    /// the baseline, and "unplaceable" must never be silently read as "old".
    #[test]
    fn a_close_with_no_timestamp_is_a_violation_not_a_pass() {
        let beads = r#"{"id":"bd-zzz","status":"closed","closed_at":"","title":"no stamp"}"#;
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, beads);
        match run(dir.path()) {
            Err(ReconcileError::Unreconciled(v)) => {
                assert!(v[0].contains("cannot be placed"), "{:?}", v)
            }
            _ => panic!("an unplaceable close must not pass"),
        }
    }

    #[test]
    fn an_empty_ledger_is_an_error_never_a_pass() {
        let dir = root("2026-08-20T00:00:00Z", "", "");
        assert!(
            matches!(run(dir.path()), Err(ReconcileError::Error(_))),
            "an empty ledger must ERROR, not pass vacuously"
        );
    }

    #[test]
    fn an_empty_scan_set_is_an_error_never_a_pass() {
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, "");
        assert!(
            matches!(run(dir.path()), Err(ReconcileError::Error(_))),
            "zero scanned beads must ERROR, not pass vacuously"
        );
    }

    #[test]
    fn a_missing_baseline_is_an_error_never_a_pass() {
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, "{}");
        fs::write(dir.path().join(BASELINE_REGISTRY), "# no anchor here\n").unwrap();
        assert!(matches!(run(dir.path()), Err(ReconcileError::Error(_))));
    }

    /// The `bead` field carries a comma-separated list; a member of that list
    /// must count as receipted or multi-bead ticks would read as unreconciled.
    #[test]
    fn a_bead_named_inside_a_comma_list_counts_as_receipted() {
        let beads = r#"{"id":"bd-bbb","status":"closed","closed_at":"2026-08-20T12:00:00Z","title":"t"}"#;
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, beads);
        assert!(run(dir.path()).is_ok());
    }

    #[test]
    fn an_open_bead_is_not_in_scope() {
        let beads = r#"{"id":"bd-zzz","status":"open","closed_at":"","title":"still open"}"#;
        let dir = root("2026-08-20T00:00:00Z", RECEIPTED, beads);
        assert!(run(dir.path()).is_ok(), "only closes are adjudicated");
    }
}
