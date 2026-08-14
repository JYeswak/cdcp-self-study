//! C5 — every module a learner can be ASSESSED on must also be TAUGHT.
//!
//! # The defect this gate closes
//!
//! `knowledge/domains.toml` declared `15-ops-adjacent`; the bank held 39
//! approved items for it; `web/content/modules/` had fourteen modules and no
//! module 15. `cdcp_assemble` samples across every module present in the
//! approved pool, so a learner could be scored on material the course did not
//! contain. Disclosed in CHARTER 2026-08-14, resolved on this tick by TEACHING
//! the module (see CHARTER §3 and §11 row 8) rather than retiring the items.
//!
//! # What this suite claims (FLOOR-RAISE, and what it cannot decide)
//!
//! RAISES THE FLOOR: the set of modules reachable by `cdcp_assemble` (i.e.
//! modules holding at least one `approved` bank item) is a SUBSET of the set of
//! modules carrying a navigable Learn surface — an index entry that is not
//! `empty`, has a relative `href`, and whose `web/learn/{id}.html` page and
//! `web/content/modules/{id}.md` copy both exist on disk. The decision recorded
//! in the CHARTER is therefore enforced by a test rather than by prose, in
//! either direction: retiring every item in a module also satisfies it, and
//! adding items to an untaught module breaks the build.
//!
//! CANNOT DECIDE: whether the Learn surface actually teaches what the items
//! assess. Module→module correspondence is structural. Topic-level alignment is
//! `web/data/topic_anchors.json` (checked by `smoke_feedback_links.py`), and
//! item-level pedagogical adequacy is not machine-checkable at all.
//!
//! # ANTI-VACUOUS (CHARTER decalogue §3)
//!
//! Zero modules compared is an ERROR, not a pass. A deliverable that was never
//! checked must never report like one that passed. Three separate emptiness
//! guards below (`assessed.is_empty()`, `taught.is_empty()`, and a missing
//! artifact) each panic rather than skip, and `anti_vacuous_*` exercise them.
//!
//! # Gate proven to trip (L4)
//!
//! `check_coverage` is a pure function over (assessed set, index rows), so the
//! known-bad legs inject an untaught assessed module, an `empty: true` row, an
//! empty assessed set and an empty index directly — no fixture files, no
//! mutation of the shipped tree.

use cdcp_bank::Bank;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// ───────────────────────────────────────────────────────────────────────────
// Pure core (so the known-bad legs can inject states the real tree cannot hold)
// ───────────────────────────────────────────────────────────────────────────

/// One row of `web/data/modules_index.json`, reduced to what this gate needs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LearnRow {
    id: String,
    order: u32,
    /// `build_learn.py` sets this when `primary_notes` is blank — the module is
    /// declared but has no teaching content. This is exactly the state that
    /// produced the C5 defect, so it does NOT count as a Learn surface.
    empty: bool,
    href: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum CoverageError {
    /// Anti-vacuous: nothing was compared.
    NoAssessedModules,
    /// Anti-vacuous: the taught side was empty or unreadable.
    NoLearnRows,
    /// The defect itself: assessed but not taught.
    Untaught { modules: Vec<u32> },
}

impl std::fmt::Display for CoverageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAssessedModules => write!(
                f,
                "zero assessed modules compared — an empty comparison is an ERROR, not a pass"
            ),
            Self::NoLearnRows => write!(
                f,
                "zero navigable Learn surfaces — an empty comparison is an ERROR, not a pass"
            ),
            Self::Untaught { modules } => write!(
                f,
                "modules are ASSESSED but never TAUGHT: {modules:?} — assembly can draw items \
                 from them and no Learn surface exists. Either give the module a Learn surface \
                 or retire its items so they leave the approved pool."
            ),
        }
    }
}

/// The whole gate, as a function of two sets.
fn check_coverage(assessed: &BTreeSet<u32>, rows: &[LearnRow]) -> Result<usize, CoverageError> {
    if assessed.is_empty() {
        return Err(CoverageError::NoAssessedModules);
    }
    let taught: BTreeSet<u32> = rows
        .iter()
        .filter(|r| !r.empty && r.href.as_deref().is_some_and(|h| !h.is_empty()))
        .map(|r| r.order)
        .collect();
    if taught.is_empty() {
        return Err(CoverageError::NoLearnRows);
    }
    let untaught: Vec<u32> = assessed.difference(&taught).copied().collect();
    if !untaught.is_empty() {
        return Err(CoverageError::Untaught { modules: untaught });
    }
    Ok(assessed.len())
}

// ───────────────────────────────────────────────────────────────────────────
// Loaders — every failure is a panic, never a skip
// ───────────────────────────────────────────────────────────────────────────

fn engine_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn must_exist(p: &Path, what: &str) {
    assert!(
        p.exists(),
        "{what} missing at {} — this gate must never pass by being unable to look",
        p.display()
    );
}

/// Modules `cdcp_assemble` can actually draw from: those holding ≥1 approved item.
fn assessed_modules() -> BTreeSet<u32> {
    let dir = engine_root().join("bank/items");
    must_exist(&dir, "bank/items");
    let bank = Bank::load_dir(&dir).expect("bank loads");
    assert!(!bank.items.is_empty(), "bank loaded zero items (vacuous)");
    bank.items
        .values()
        .filter(|i| i.is_approved())
        .map(|i| i.module)
        .collect()
}

/// Learn rows, cross-checked against the files they claim to have produced.
fn learn_rows() -> Vec<LearnRow> {
    let root = engine_root();
    let index_path = root.join("web/data/modules_index.json");
    must_exist(&index_path, "web/data/modules_index.json");
    let raw = std::fs::read_to_string(&index_path).expect("read modules_index.json");
    let doc: Value = serde_json::from_str(&raw).expect("modules_index.json parses");
    let mods = doc
        .get("modules")
        .and_then(Value::as_array)
        .expect("modules_index.json has a modules array");
    assert!(!mods.is_empty(), "modules_index.json lists zero modules");

    let mut rows = Vec::with_capacity(mods.len());
    for m in mods {
        let id = m
            .get("id")
            .and_then(Value::as_str)
            .expect("module row has id")
            .to_string();
        let order = u32::try_from(
            m.get("order")
                .and_then(Value::as_u64)
                .expect("module row has order"),
        )
        .expect("order fits u32");
        let empty = m.get("empty").and_then(Value::as_bool).unwrap_or(false);
        let href = m
            .get("href")
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string);

        // A row is only a Learn surface if the artifacts it advertises exist.
        // Without this, deleting every learn page would still report green.
        if !empty {
            must_exist(
                &root.join(format!("web/learn/{id}.html")),
                &format!("learn page for {id}"),
            );
            must_exist(
                &root.join(format!("web/content/modules/{id}.md")),
                &format!("module content copy for {id}"),
            );
        }
        rows.push(LearnRow {
            id,
            order,
            empty,
            href,
        });
    }
    rows
}

// ───────────────────────────────────────────────────────────────────────────
// The shipped assertion
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn every_assessed_module_has_a_learn_surface() {
    let assessed = assessed_modules();
    let rows = learn_rows();
    let compared = check_coverage(&assessed, &rows).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        compared > 0,
        "anti-vacuous: zero modules compared is an ERROR"
    );
    // Sanity floor: the fourteen public EPI domains plus the ops-adjacent
    // supplement. A collapse to a handful of modules is itself a defect.
    assert!(
        compared >= 14,
        "expected ≥14 assessed modules, compared {compared} — pool looks starved"
    );
}

/// The C5 decision, stated as an assertion rather than as prose: module 15 is
/// assessed AND taught. If a later tick retires the 39 items instead, this
/// flips to asserting it is neither — but it may never be assessed-only again.
#[test]
fn module_15_is_taught_because_it_is_assessed() {
    let assessed = assessed_modules();
    let rows = learn_rows();
    let m15 = rows.iter().find(|r| r.order == 15);

    if assessed.contains(&15) {
        let row = m15.expect("module 15 is assessed, so it must appear in modules_index.json");
        assert!(
            !row.empty,
            "module 15 holds approved bank items but its Learn row is empty-ok — this is \
             exactly the C5 fairness defect"
        );
        assert_eq!(row.id, "15-ops-adjacent");
        assert!(
            row.href.as_deref().is_some_and(|h| h.starts_with("learn/")),
            "module 15 must be reachable offline, got {:?}",
            row.href
        );
        // The six topics the 39 items assess must be teachable from this page.
        let md = engine_root().join("web/content/modules/15-ops-adjacent.md");
        let text = std::fs::read_to_string(&md).expect("module 15 content copy readable");
        for heading in [
            "### Labelling",
            "### Documentation",
            "### Cleaning",
            "### MTBF / MTTR",
            "### Maintenance contracts / SLA",
            "### Operational security and safety practices",
        ] {
            assert!(
                text.contains(heading),
                "module 15 must teach the topics its items assess; missing heading {heading:?}"
            );
        }
    } else {
        // EXCLUDE branch: if the items were retired, no learner can reach them.
        assert!(
            m15.is_none_or(|r| r.empty),
            "module 15 has a Learn surface but no approved items — harmless, yet the \
             CHARTER decision row must be updated to match"
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// L4 — the gate is proven to trip
// ───────────────────────────────────────────────────────────────────────────

fn row(order: u32, empty: bool) -> LearnRow {
    LearnRow {
        id: format!("{order:02}-mod"),
        order,
        empty,
        href: if empty {
            None
        } else {
            Some(format!("learn/{order:02}-mod.html"))
        },
    }
}

#[test]
fn control_matched_sets_pass() {
    // GOOD leg. Without it, every known-bad below could be passing for the
    // wrong reason (e.g. check_coverage returning Err unconditionally).
    let assessed: BTreeSet<u32> = (1..=15).collect();
    let rows: Vec<LearnRow> = (1..=15).map(|o| row(o, false)).collect();
    assert_eq!(check_coverage(&assessed, &rows), Ok(15));
}

#[test]
fn known_bad_assessed_module_with_no_learn_surface_is_red() {
    // The literal C5 defect: module 15 in the pool, absent from the index.
    let assessed: BTreeSet<u32> = (1..=15).collect();
    let rows: Vec<LearnRow> = (1..=14).map(|o| row(o, false)).collect();
    assert_eq!(
        check_coverage(&assessed, &rows),
        Err(CoverageError::Untaught { modules: vec![15] })
    );
}

#[test]
fn known_bad_empty_ok_row_does_not_count_as_taught() {
    // The pre-C5 shipped state exactly: the row EXISTS but carries no content.
    // A gate that only checked "is the id present" would have reported green
    // for the entire life of the defect.
    let assessed: BTreeSet<u32> = (1..=15).collect();
    let mut rows: Vec<LearnRow> = (1..=14).map(|o| row(o, false)).collect();
    rows.push(row(15, true));
    assert_eq!(
        check_coverage(&assessed, &rows),
        Err(CoverageError::Untaught { modules: vec![15] })
    );
}

#[test]
fn known_bad_multiple_untaught_modules_are_all_named() {
    let assessed: BTreeSet<u32> = (1..=16).collect();
    let rows: Vec<LearnRow> = (1..=14).map(|o| row(o, false)).collect();
    assert_eq!(
        check_coverage(&assessed, &rows),
        Err(CoverageError::Untaught {
            modules: vec![15, 16]
        })
    );
}

#[test]
fn anti_vacuous_empty_assessed_set_is_an_error() {
    let rows: Vec<LearnRow> = (1..=15).map(|o| row(o, false)).collect();
    assert_eq!(
        check_coverage(&BTreeSet::new(), &rows),
        Err(CoverageError::NoAssessedModules),
        "an empty assessed set must be an ERROR, not a pass"
    );
}

#[test]
fn anti_vacuous_empty_index_is_an_error() {
    let assessed: BTreeSet<u32> = (1..=15).collect();
    assert_eq!(
        check_coverage(&assessed, &[]),
        Err(CoverageError::NoLearnRows),
        "an empty Learn index must be an ERROR, not a pass"
    );
    // …and an index of nothing but empty-ok rows is the same vacuum.
    let all_empty: Vec<LearnRow> = (1..=15).map(|o| row(o, true)).collect();
    assert_eq!(
        check_coverage(&assessed, &all_empty),
        Err(CoverageError::NoLearnRows)
    );
}

/// Retiring every item in a module is the OTHER lawful resolution, and it must
/// pass — otherwise the gate would forbid the exclude branch the CHARTER keeps
/// open, and a future tick could not choose it without editing the test.
#[test]
fn excluding_a_module_from_the_pool_also_satisfies_the_gate() {
    let assessed: BTreeSet<u32> = (1..=14).collect();
    let mut rows: Vec<LearnRow> = (1..=14).map(|o| row(o, false)).collect();
    rows.push(row(15, true));
    assert_eq!(check_coverage(&assessed, &rows), Ok(14));
}

/// Order→id mapping must be injective; two rows claiming order 15 would let a
/// taught module launder an untaught one.
#[test]
fn learn_rows_have_unique_orders() {
    let rows = learn_rows();
    let mut by_order: BTreeMap<u32, &str> = BTreeMap::new();
    for r in &rows {
        assert!(
            by_order.insert(r.order, &r.id).is_none(),
            "duplicate module order {} (ids collide)",
            r.order
        );
    }
    assert!(!by_order.is_empty(), "anti-vacuous: zero rows inspected");
}
