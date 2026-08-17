//! First-class OSHA facts bound to the vendored eCFR snapshots.
//!
//! 29 CFR 1910.147(a)(1)(ii)(D) EXCLUDES electrical-hazard exposure on
//! electric-utilization installations (switchgear / UPS / PDU). That work
//! is Subpart S, principally 1910.333. Concurrent maintainability is a
//! LEGAL isolation constraint, not a reliability slogan.

use crate::{load_compiled, DataError, LoadedSnapshot};
use std::fmt;
use std::path::Path;

/// Token interpolated inside the exclusion check.
pub const EXCLUSION_147: &str = "Exposure to electrical hazards from work on, near, or with conductors or equipment in electric-utilization installations, which is covered by subpart S of this part";

/// Token interpolated inside the deenergize-first constraint.
pub const DEENERGIZE_FIRST: &str = "Live parts to which an employee may be exposed shall be deenergized before the employee works on or near them";

/// Token interpolated inside the control-device constraint.
pub const CONTROL_DEVICES_NOT_ISOLATION: &str =
    "Control circuit devices, such as push buttons, selector switches, and interlocks, may not be used as the sole means for deenergizing circuits or equipment.";

/// Token interpolated inside the backfeed-test constraint.
pub const BACKFEED_TEST: &str = "The test shall also determine if any energized condition exists as a result of inadvertently induced voltage or unrelated voltage backfeed";

/// Pin id for the 1910.147 extract.
pub const SNAP_147: &str = "src-osha-29cfr-1910-147";
/// Pin id for the 1910.269 extract.
pub const SNAP_269: &str = "src-osha-29cfr-1910-269";
/// Pin id for the 1910.333 / Subpart S extract.
pub const SNAP_333: &str = "src-osha-29cfr-1910-333";

/// A legal isolation constraint. Concurrent maintainability is these
/// three sentences, not a marketing claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsolationConstraint {
    /// Stable id.
    pub id: &'static str,
    /// One-line rule.
    pub rule: &'static str,
    /// CFR citation.
    pub citation: &'static str,
    /// Official quote that MUST appear in the vendored snapshot.
    pub quote: &'static str,
    /// Snapshot pin that must carry the quote.
    pub snapshot_id: &'static str,
}

/// The three isolation constraints 1910.333 imposes on utilization work.
pub const ISOLATION_CONSTRAINTS: &[IsolationConstraint] = &[
    IsolationConstraint {
        id: "deenergize-first",
        rule: "deenergise-first",
        citation: "29 CFR 1910.333(a)(1)",
        quote: DEENERGIZE_FIRST,
        snapshot_id: SNAP_333,
    },
    IsolationConstraint {
        id: "control-devices-not-isolation",
        rule: "control-circuit devices and interlocks may not be the sole means of deenergizing",
        citation: "29 CFR 1910.333(b)(2)(ii)(B)",
        quote: CONTROL_DEVICES_NOT_ISOLATION,
        snapshot_id: SNAP_333,
    },
    IsolationConstraint {
        id: "backfeed-test",
        rule: "backfeed testing mandatory",
        citation: "29 CFR 1910.333(b)(2)(iv)(B)",
        quote: BACKFEED_TEST,
        snapshot_id: SNAP_333,
    },
];

/// Why an OSHA check went RED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OshaFault {
    /// A required snapshot was not in the load report.
    MissingSnapshot {
        /// Pin id.
        id: String,
    },
    /// A required official quote is not in the vendored body.
    QuoteMissing {
        /// Constraint or fact id.
        id: String,
        /// Snapshot that should have carried it.
        snapshot: String,
    },
    /// A curriculum unit or bank item cites 1910.147 as the electrical
    /// LOTO authority for utilization equipment.
    Cite147AsElectricalLoto {
        /// Path, engine-root-relative.
        path: String,
    },
}

impl fmt::Display for OshaFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OshaFault::MissingSnapshot { id } => {
                write!(f, "OSHA snapshot {id} did not load")
            }
            OshaFault::QuoteMissing { id, snapshot } => {
                write!(f, "official quote {id} missing from {snapshot}")
            }
            OshaFault::Cite147AsElectricalLoto { path } => {
                write!(f, "cites 1910.147 as the electrical-LOTO authority: {path}")
            }
        }
    }
}

/// Outcome of [`check_osha`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OshaReport {
    /// Units / items scanned.
    pub scanned: usize,
    /// Findings. Empty is clean.
    pub faults: Vec<OshaFault>,
}

impl OshaReport {
    /// True when there are no faults.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.faults.is_empty()
    }
}

impl fmt::Display for OshaReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_clean() {
            writeln!(
                f,
                "check_osha: PASS scanned={} faults=0 exclusion=1910.147(a)(1)(ii)(D) isolation={}",
                self.scanned,
                ISOLATION_CONSTRAINTS.len()
            )?;
        } else {
            writeln!(
                f,
                "check_osha: FAIL scanned={} faults={}",
                self.scanned,
                self.faults.len()
            )?;
            for fault in &self.faults {
                writeln!(f, "  {fault}")?;
            }
        }
        Ok(())
    }
}

/// True when `text` treats 1910.147 as the electrical-LOTO authority
/// for utilization equipment (switchgear / UPS / PDU) without the
/// (a)(1)(ii)(D) exclusion. Mentioning Subpart S or the exclusion is
/// the lawful shape (M15).
#[must_use]
pub fn cites_147_as_electrical_loto_authority(text: &str) -> bool {
    let t = text.to_ascii_lowercase();
    if !t.contains("1910.147") {
        return false;
    }
    // Short tokens ("ups", "pdu") must be whole words: "upstream" is not UPS
    // (measured 2026-08-17: m15-q228 CRAH/147 isolation tripped on "upstream").
    let electrical = [
        "switchgear",
        "busway",
        "electric-utilization",
        "electric utilization",
        "electrical loto",
        "electrical-loto",
        "electrical-hazard",
        "electrical hazard",
    ]
    .iter()
    .any(|k| t.contains(k))
        || has_word(&t, "ups")
        || has_word(&t, "pdu");
    if !electrical {
        return false;
    }
    let exclusion = t.contains("(a)(1)(ii)(d)")
        || t.contains("a)(1)(ii)(d")
        || t.contains("does not cover")
        || t.contains("excludes exposure to electrical")
        || t.contains("expressly excludes")
        || t.contains("subpart s");
    !exclusion
}

fn has_word(hay: &str, word: &str) -> bool {
    let w = word.as_bytes();
    let h = hay.as_bytes();
    let mut i = 0;
    while i + w.len() <= h.len() {
        if &h[i..i + w.len()] == w {
            let left_ok = i == 0 || !h[i - 1].is_ascii_alphanumeric();
            let right_ok = i + w.len() == h.len() || !h[i + w.len()].is_ascii_alphanumeric();
            if left_ok && right_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Load compiled snapshots and enforce the OSHA facts against the
/// vendored bodies and the live curriculum (modules + bank).
pub fn check_osha(root: &Path) -> Result<OshaReport, DataError> {
    let loaded = load_compiled(root)?;
    check_osha_with(&loaded.loaded, root)
}

/// Same check with a caller-supplied load (tests inject plants).
pub fn check_osha_with(loaded: &[LoadedSnapshot], root: &Path) -> Result<OshaReport, DataError> {
    let mut faults = Vec::new();
    let by_id = |id: &str| loaded.iter().find(|s| s.id == id);

    for id in [SNAP_147, SNAP_269, SNAP_333] {
        if by_id(id).is_none() {
            faults.push(OshaFault::MissingSnapshot { id: id.into() });
        }
    }

    if let Some(s) = by_id(SNAP_147) {
        let body = String::from_utf8_lossy(&s.bytes);
        if !body.contains(EXCLUSION_147) {
            faults.push(OshaFault::QuoteMissing {
                id: "exclusion-147-a-1-ii-d".into(),
                snapshot: SNAP_147.into(),
            });
        }
    }
    for c in ISOLATION_CONSTRAINTS {
        if let Some(s) = by_id(c.snapshot_id) {
            let body = String::from_utf8_lossy(&s.bytes);
            if !body.contains(c.quote) {
                faults.push(OshaFault::QuoteMissing {
                    id: c.id.into(),
                    snapshot: c.snapshot_id.into(),
                });
            }
        }
    }

    let mut scanned = 0usize;
    for rel in ["../modules", "web/content/modules", "bank/items"] {
        let dir = root.join(rel);
        if dir.is_dir() {
            scanned += scan_tree(&dir, root, &mut faults);
        }
    }

    Ok(OshaReport { scanned, faults })
}

fn scan_tree(dir: &Path, engine: &Path, faults: &mut Vec<OshaFault>) -> usize {
    let mut n = 0usize;
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return 0,
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            n += scan_tree(&p, engine, faults);
            continue;
        }
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !(name.ends_with(".md") || name.ends_with(".toml")) {
            continue;
        }
        n += 1;
        let text = match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if cites_147_as_electrical_loto_authority(&text) {
            let rel = p
                .strip_prefix(engine)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            faults.push(OshaFault::Cite147AsElectricalLoto { path: rel });
        }
    }
    n
}
