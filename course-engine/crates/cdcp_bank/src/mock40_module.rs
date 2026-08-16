//! `mock40-*` module-vs-stem audit (bd-mock40-q37-cross-module-topic-76vs).
//!
//! The practice-exam import numbered files by question order and copied that
//! number into `module`. `verify_orphans` checks that every `topic_ids` entry
//! resolves; it does not check that `module = 13` and a stem about MOPs agree.
//!
//! DECISION (2026-08-15): a `topic_ids` prefix that does not match `module` is
//! LEGAL. Topics may name a secondary domain (removing a floor tile is module 4
//! and still cites `m09-raised-floor-cooling`). A prefix-equality gate would
//! have PASSED mock40-q37 (it carried `m13-safety-components`) and FAILED
//! legitimate cross-cuts.
//!
//! What is NOT legal: an *approved* item whose `module` disagrees with the
//! stem's subject. That is what this ledger asserts. Retired items may keep a
//! wrong `module` — retiring is how the duplicate left the drawable pool; the
//! misfile stays on the record (mock40-q37: filed 13, content 15).
//!
//! Anti-vacuous: zero `mock40-*` items is an ERROR. A new `mock40-*` id that
//! is not in [`MOCK40_CONTENT_MODULE`] is RED. A ledger row whose id is gone
//! is RED. An empty finding set is therefore a finding only if the scan ran.

use crate::{BankItem, ItemStatus};
use std::collections::{BTreeMap, BTreeSet};

/// Stem-derived home module for every `mock40-*` id in the shipped bank.
///
/// `content_module` is what the stem is *about*, not the filename digit and
/// not the first `topic_ids` prefix. Cross-cutting extras stay on the item.
pub const MOCK40_CONTENT_MODULE: &[(&str, u32)] = &[
    ("mock40-q01", 1),  // mission-critical definition
    ("mock40-q02", 1),  // availability vs reliability (m15-mtbf-mttr is a pointer)
    ("mock40-q03", 1),  // colo customer role
    ("mock40-q04", 1),  // outage contributors
    ("mock40-q05", 2),  // code vs standard
    ("mock40-q06", 2),  // TIA-942 vs Uptime Tier
    ("mock40-q07", 3),  // dual-utility site
    ("mock40-q08", 3),  // supporting facilities
    ("mock40-q09", 4),  // raised-floor plenum
    ("mock40-q10", 4),  // rolling load
    ("mock40-q11", 4),  // tile removal (m09 cooling is secondary)
    ("mock40-q12", 5),  // emergency lighting
    ("mock40-q13", 6),  // critical power path
    ("mock40-q14", 6),  // STS vs ATS
    ("mock40-q15", 6),  // N+1 UPS
    ("mock40-q16", 6),  // 2N
    ("mock40-q17", 6),  // dual-cord
    ("mock40-q18", 6),  // double-conversion UPS
    ("mock40-q19", 6),  // generators
    ("mock40-q20", 6),  // busway
    ("mock40-q21", 6),  // PUE definition (m06-sustainability)
    ("mock40-q22", 10), // WUE
    ("mock40-q23", 7),  // magnetic-field source
    ("mock40-q24", 8),  // one RU
    ("mock40-q25", 8),  // blanking plates (m09 containment is secondary)
    ("mock40-q26", 9),  // CRAC vs CRAH
    ("mock40-q27", 9),  // hot-aisle containment
    ("mock40-q28", 9),  // IT heat
    ("mock40-q29", 9),  // CDU
    ("mock40-q30", 9),  // ASHRAE thermal guidelines
    ("mock40-q31", 10), // process water
    ("mock40-q32", 11), // meet-me room
    ("mock40-q33", 11), // scalable network
    ("mock40-q34", 12), // aspirating detection
    ("mock40-q35", 12), // clean-agent suppression
    ("mock40-q36", 13), // mantrap
    ("mock40-q37", 15), // MOP — filed as 13; retired. THE 76vs specimen.
    ("mock40-q38", 14), // BMS vs DCIM
    ("mock40-q39", 14), // leak detection
    ("mock40-q40", 14), // IST
];

/// Receipt the live-bank test prints so an empty scan cannot look like a pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mock40Audit {
    pub checked: usize,
    pub live_misfiles: usize,
    pub retired_misfiles: usize,
}

/// Compare every `mock40-*` item against [`MOCK40_CONTENT_MODULE`].
pub fn mock40_module_audit<'a, I>(items: I) -> Result<Mock40Audit, String>
where
    I: IntoIterator<Item = &'a BankItem>,
{
    let ledger: BTreeMap<&str, u32> = MOCK40_CONTENT_MODULE.iter().copied().collect();
    if ledger.len() != MOCK40_CONTENT_MODULE.len() {
        return Err("MOCK40_CONTENT_MODULE contains a duplicate id".to_string());
    }

    let mut present: BTreeMap<&str, &BankItem> = BTreeMap::new();
    for item in items {
        if item.id.starts_with("mock40-") {
            present.insert(item.id.as_str(), item);
        }
    }

    if present.is_empty() {
        return Err(
            "zero mock40-* items scanned — an empty module-audit is an ERROR, not a pass"
                .to_string(),
        );
    }

    let have: BTreeSet<&str> = present.keys().copied().collect();
    let listed: BTreeSet<&str> = ledger.keys().copied().collect();
    let missing_from_ledger: Vec<&str> = have.difference(&listed).copied().collect();
    let missing_from_bank: Vec<&str> = listed.difference(&have).copied().collect();
    if !missing_from_ledger.is_empty() || !missing_from_bank.is_empty() {
        return Err(format!(
            "mock40 module ledger disagrees with the bank\n  \
             in bank, not in MOCK40_CONTENT_MODULE: {missing_from_ledger:?}\n  \
             in ledger, not in bank: {missing_from_bank:?}"
        ));
    }

    let mut live_misfiles = 0usize;
    let mut retired_misfiles = 0usize;
    let mut live_names: Vec<String> = Vec::new();
    for (id, item) in &present {
        let content = *ledger.get(id).expect("set equality above");
        if item.module == content {
            continue;
        }
        match item.status {
            ItemStatus::Approved => {
                live_misfiles += 1;
                live_names.push(format!(
                    "{id} filed module={} stem-subject={}",
                    item.module, content
                ));
            }
            ItemStatus::Retired | ItemStatus::Draft => retired_misfiles += 1,
        }
    }
    if !live_names.is_empty() {
        return Err(format!(
            "approved mock40-* item(s) filed under the wrong module (stem subject ≠ module): {}",
            live_names.join("; ")
        ));
    }

    // The 76vs specimen must stay visible: retired, filed 13, content 15.
    // Approving it, "fixing" the filed module without a ledger edit, or
    // deleting the file all go RED so the misfile cannot vanish silently.
    let q37 = present.get("mock40-q37").ok_or_else(|| {
        "mock40-q37 missing — the 76vs misfile must stay on the record (retire, never delete)"
            .to_string()
    })?;
    if q37.status != ItemStatus::Retired || q37.module != 13 || ledger["mock40-q37"] != 15 {
        return Err(format!(
            "mock40-q37 is the recorded MOP-vs-module-13 misfile \
             (must stay retired, filed module=13, content_module=15); got status={} module={}",
            q37.status, q37.module
        ));
    }

    Ok(Mock40Audit {
        checked: present.len(),
        live_misfiles,
        retired_misfiles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BankItem;

    fn item(id: &str, module: u32, status: ItemStatus) -> BankItem {
        BankItem {
            id: id.to_string(),
            module,
            stem: "a stem long enough".into(),
            choices: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            correct: "A".into(),
            explanation: "because reasons here".into(),
            topic_ids: vec!["m01-importance".into()],
            objective_ids: vec![],
            citation_ids: vec![],
            tags: vec![],
            bloom: "understand".into(),
            source_class: "original".into(),
            quantity_evidence: "qualitative_only".into(),
            status,
            kind: crate::ItemKind::SingleSelect,
        }
    }

    fn full_set() -> Vec<BankItem> {
        MOCK40_CONTENT_MODULE
            .iter()
            .map(|(id, content)| {
                if *id == "mock40-q37" {
                    item(id, 13, ItemStatus::Retired)
                } else {
                    item(id, *content, ItemStatus::Approved)
                }
            })
            .collect()
    }

    #[test]
    fn zero_mock40_items_is_an_error() {
        let only = item("bank-m01-q001", 1, ItemStatus::Approved);
        let err = mock40_module_audit([&only]).unwrap_err();
        assert!(err.contains("zero mock40"), "{err}");
    }

    #[test]
    fn unlisted_mock40_id_is_red() {
        let mut set = full_set();
        set.push(item("mock40-q99", 1, ItemStatus::Approved));
        let err = mock40_module_audit(set.iter()).unwrap_err();
        assert!(err.contains("mock40-q99"), "{err}");
    }

    #[test]
    fn approved_misfile_is_red() {
        let mut set = full_set();
        for it in &mut set {
            if it.id == "mock40-q01" {
                it.module = 15;
            }
        }
        let err = mock40_module_audit(set.iter()).unwrap_err();
        assert!(err.contains("mock40-q01"), "{err}");
        assert!(err.contains("approved"), "{err}");
    }

    #[test]
    fn retired_q37_misfile_is_the_recorded_specimen() {
        let set = full_set();
        let report = mock40_module_audit(set.iter()).expect("q37 retired misfile is legal");
        assert_eq!(report.checked, MOCK40_CONTENT_MODULE.len());
        assert_eq!(report.live_misfiles, 0);
        assert_eq!(report.retired_misfiles, 1);
    }

    #[test]
    fn approving_q37_without_fixing_module_is_red() {
        let mut set = full_set();
        for it in &mut set {
            if it.id == "mock40-q37" {
                it.status = ItemStatus::Approved;
            }
        }
        let err = mock40_module_audit(set.iter()).unwrap_err();
        assert!(
            err.contains("mock40-q37") || err.contains("approved"),
            "{err}"
        );
    }
}
