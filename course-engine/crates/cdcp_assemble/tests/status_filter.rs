//! C1 — APPROVED-ONLY assembly, gate proven to trip.
//!
//! # What this suite claims (FLOOR-RAISE, and what it cannot decide)
//!
//! RAISES THE FLOOR: an item whose `status` is `draft` or `retired` cannot be
//! drawn into an assembled exam by `cdcp_assemble`, at any seed, through any of
//! the three selection paths (first pass, round-robin fill, `max_per_module`
//! relaxation fallback); and an eligible pool that is empty or too small is an
//! ERROR rather than a short/empty exam.
//!
//! CANNOT DECIDE: whether an `approved` item is any *good*. Status is an
//! editorial assertion by whoever wrote the line; this gate only enforces that
//! the assertion exists and is honoured. It also cannot detect a status flip
//! after the fact — `bank_hash` does not cover `status` (see C2, blocked on B2).
//!
//! # Known-bad injections (each has a matching GOOD control that flips only
//! `status`, so a green result cannot come from the test being vacuous)
//!
//! 1. planted DRAFT that the unfiltered pool would certainly sample -> RED
//! 2. planted RETIRED, same shape -> RED
//! 3. whole pool draft -> `NoApprovedItems` (empty set is an ERROR, §3)
//! 4. whole pool retired -> `NoApprovedItems`
//! 5. approved pool starved of module breadth -> error NAMES the shortfall
//! 6. approved-only pool narrows breadth the full pool satisfied (C1 x C6)
//!
//! # Meta-test
//!
//! Deleting the `approved` filter in `sample_item_ids` turns this file RED.
//! Verified by hand on the C1 tick (assertion deleted -> `cargo test` exit 101;
//! restored -> exit 0).

use cdcp_assemble::{assemble, sample_item_ids, AssembleConfig, AssembleError};
use cdcp_bank::{Bank, BankItem, ItemStatus};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Fixed seed for every planted case: the same seed the shipped goldens use.
const SEED: u64 = 42;

fn item(id: &str, module: u32, status: ItemStatus) -> BankItem {
    BankItem {
        id: id.to_string(),
        module,
        stem: format!("stem for {id}"),
        choices: vec!["a".into(), "b".into(), "c".into(), "d".into()],
        correct: "A".into(),
        explanation: "because reasons here".into(),
        topic_ids: vec![format!("topic-{id}")],
        // C2 (bd-hardening-c-status-hzs.2): these three fields joined BankItem
        // and the hash payload. Empty here — this fixture varies `status` only.
        objective_ids: Vec::new(),
        citation_ids: Vec::new(),
        tags: Vec::new(),
        bloom: "understand".into(),
        source_class: "original".into(),
        quantity_evidence: "qualitative_only".into(),
        status,
    }
}

/// `count` approved items spread across `modules` modules (round-robin), so the
/// breadth precondition is satisfied by construction.
fn approved_spread(count: usize, modules: u32) -> Vec<BankItem> {
    (0..count)
        .map(|i| {
            item(
                &format!("ok-{i:03}"),
                (i as u32 % modules) + 1,
                ItemStatus::Approved,
            )
        })
        .collect()
}

fn cfg(n_items: usize, min_modules: usize) -> AssembleConfig {
    AssembleConfig {
        n_items,
        max_per_module: 8,
        min_modules,
        shuffle_choices: false,
    }
}

// ───────────────────────────────────────────────────────────────────────────
// 1 + 2. A planted draft / retired item is never assembled.
//
// Shape: 39 approved + 1 planted, n = 40. The planted item is the 40th and
// last eligible item, so with NO filter it is drawn at EVERY seed — there is no
// seed at which the unfiltered run could accidentally exclude it. Flipping only
// its `status` therefore isolates the filter and nothing else.
// ───────────────────────────────────────────────────────────────────────────

fn bank_with_planted(status: ItemStatus) -> Bank {
    let mut items = approved_spread(39, 8);
    items.push(item("planted-zzz", 8, status));
    Bank::from_items(items).expect("planted bank loads")
}

#[test]
fn control_planted_item_is_sampled_when_approved() {
    // GOOD leg. Without this, cases 1 and 2 could pass because the item was
    // never sampleable in the first place.
    let bank = bank_with_planted(ItemStatus::Approved);
    let ids = sample_item_ids(&bank, 40, SEED, 8, 8).expect("40 approved, n=40");
    assert_eq!(ids.len(), 40);
    assert!(
        ids.iter().any(|i| i == "planted-zzz"),
        "control: the planted item MUST be sampled when approved, else the \
         draft/retired cases prove nothing"
    );
}

#[test]
fn planted_draft_is_never_assembled() {
    let bank = bank_with_planted(ItemStatus::Draft);
    match sample_item_ids(&bank, 40, SEED, 8, 8) {
        Err(AssembleError::PoolTooSmall {
            approved,
            n,
            total,
            not_approved,
        }) => {
            assert_eq!((approved, n, total, not_approved), (39, 40, 40, 1));
        }
        other => panic!("draft item leaked into the eligible pool: {other:?}"),
    }
    // and the whole-exam path refuses too
    let err = assemble(&bank, SEED, cfg(40, 8)).unwrap_err();
    assert!(
        !format!("{err}").is_empty() && matches!(err, AssembleError::PoolTooSmall { .. }),
        "assemble() must refuse rather than return a 39-item exam, got {err:?}"
    );
}

#[test]
fn planted_retired_is_never_assembled() {
    let bank = bank_with_planted(ItemStatus::Retired);
    assert!(
        matches!(
            sample_item_ids(&bank, 40, SEED, 8, 8),
            Err(AssembleError::PoolTooSmall { approved: 39, .. })
        ),
        "retired item leaked into the eligible pool"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// Absence inside a *successful* assembly: the exam still fills, and the
// ineligible items are simply not on it.
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn draft_and_retired_absent_from_a_full_40_item_exam() {
    let mut items = approved_spread(40, 8);
    items.push(item("planted-draft", 3, ItemStatus::Draft));
    items.push(item("planted-retired", 5, ItemStatus::Retired));
    let bank = Bank::from_items(items).unwrap();
    assert_eq!(bank.items.len(), 42, "42 loaded, 40 eligible");

    let exam = assemble(&bank, SEED, cfg(40, 8)).expect("40 approved items fill a 40-item form");
    let on_form: BTreeSet<&str> = exam.item_ids.iter().map(|s| s.as_str()).collect();

    assert!(!on_form.contains("planted-draft"), "draft item on the form");
    assert!(
        !on_form.contains("planted-retired"),
        "retired item on the form"
    );
    // Exactly the approved set — nothing eligible dropped, nothing ineligible added.
    let expected: BTreeSet<&str> = bank
        .items
        .values()
        .filter(|i| i.is_approved())
        .map(|i| i.id.as_str())
        .collect();
    assert_eq!(on_form, expected);
    // And the rendered items agree with the id list.
    assert_eq!(exam.items.len(), 40);
    for it in &exam.items {
        assert_eq!(
            bank.get(&it.id).unwrap().status,
            ItemStatus::Approved,
            "assembled item {} is not approved",
            it.id
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// 3 + 4. Anti-vacuous: an empty approved pool is an ERROR, not an empty exam.
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn empty_approved_pool_is_an_error_not_an_empty_exam() {
    for status in [ItemStatus::Draft, ItemStatus::Retired] {
        let items: Vec<BankItem> = (0..45)
            .map(|i| item(&format!("x-{i:03}"), (i % 8) + 1, status))
            .collect();
        let bank = Bank::from_items(items).unwrap();
        assert_eq!(bank.items.len(), 45, "non-empty bank, zero eligible");

        match assemble(&bank, SEED, cfg(40, 8)) {
            Err(AssembleError::NoApprovedItems { total }) => assert_eq!(total, 45),
            Ok(exam) => panic!(
                "all-{status} bank produced an exam of {} items — an empty eligible \
                 pool must be an ERROR (CHARTER decalogue §3)",
                exam.n_items
            ),
            Err(other) => panic!("expected NoApprovedItems for all-{status} bank, got {other:?}"),
        }
    }
}

#[test]
fn empty_approved_pool_error_is_distinct_from_small_pool_error() {
    // A zero-eligible bank must not report as "pool too small" — the two are
    // different defects and a learner-facing message that conflates them hides
    // an editorial mistake behind a capacity one.
    let items: Vec<BankItem> = (0..45)
        .map(|i| item(&format!("d-{i:03}"), (i % 8) + 1, ItemStatus::Draft))
        .collect();
    let bank = Bank::from_items(items).unwrap();
    let err = sample_item_ids(&bank, 40, SEED, 8, 8).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("no approved items") && msg.contains("45"),
        "error must name the condition and the loaded count, got: {msg}"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// 5 + 6. C1 x C6: the approved-only pool can starve module breadth that the
// FULL pool satisfied. The error names the shortfall.
//
// NOTE (C6 remains open): `min_modules` is enforced here as a POOL precondition
// only. Whether the 40 SELECTED items span `min_modules` is still unchecked —
// that is bead C6 and this tick deliberately does not fix it.
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn approved_only_pool_starved_of_modules_names_the_shortfall() {
    // Full pool spans 10 modules — satisfies min_modules=8.
    // Approved subset is 60 items but spans only modules 1..=3 — it does not.
    // Size is deliberately generous so the failure can ONLY be breadth.
    let mut items: Vec<BankItem> = Vec::new();
    for module in 1..=10u32 {
        let (count, status) = if module <= 3 {
            (20, ItemStatus::Approved)
        } else {
            (6, ItemStatus::Draft)
        };
        for i in 0..count {
            items.push(item(&format!("m{module:02}-{i:03}"), module, status));
        }
    }
    let bank = Bank::from_items(items).unwrap();

    let full_modules: BTreeSet<u32> = bank.items.values().map(|i| i.module).collect();
    let approved_modules: BTreeSet<u32> = bank
        .items
        .values()
        .filter(|i| i.is_approved())
        .map(|i| i.module)
        .collect();
    assert_eq!(full_modules.len(), 10, "full pool satisfies min_modules=8");
    assert_eq!(approved_modules.len(), 3, "approved pool does not");
    assert!(
        bank.items.values().filter(|i| i.is_approved()).count() >= 40,
        "the approved pool is large enough — breadth, not size, is what fails"
    );

    let err = sample_item_ids(&bank, 40, SEED, 8, 8).unwrap_err();
    match err {
        AssembleError::ApprovedTooFewModules {
            modules,
            min_modules,
            shortfall,
            ..
        } => {
            assert_eq!((modules, min_modules, shortfall), (3, 8, 5));
        }
        other => panic!("expected a named module shortfall, got {other:?}"),
    }
    let msg = err.to_string();
    assert!(
        msg.contains("min_modules=8") && msg.contains("shortfall 5"),
        "the message must name the shortfall a human has to act on, got: {msg}"
    );
}

#[test]
fn c6_min_modules_is_still_unenforced_over_the_selected_items() {
    // DOCUMENTS A KNOWN DEFECT (C6), it does not bless it. The approved pool
    // spans 8 modules so the C1 precondition passes, but max_per_module=8 with
    // n=40 means the selection can legitimately come from as few as 5 modules
    // and nothing checks that. When C6 lands, this test SHOULD go red and be
    // rewritten — that is the point of pinning current behaviour.
    let bank = Bank::from_items(approved_spread(64, 8)).unwrap();
    let ids = sample_item_ids(&bank, 40, SEED, 8, 8).expect("pool precondition satisfied");
    let modules: BTreeSet<u32> = ids.iter().map(|id| bank.get(id).unwrap().module).collect();
    assert!(
        modules.len() <= 8,
        "sanity: cannot exceed the 8 modules planted"
    );
    // No assertion that modules.len() >= min_modules — because the code makes
    // no such promise. C6 owns that.
}

// ───────────────────────────────────────────────────────────────────────────
// Anchoring: the real, shipped 804-item bank.
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn real_bank_is_all_approved_and_seed42_holds() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items");
    // Anti-vacuous: this test's whole value is that it reads the REAL corpus.
    // A missing bank used to print "skip" and return green, which is the same
    // observable outcome as having checked it.
    assert!(
        dir.is_dir(),
        "the real bank must be present for this anchoring test; expected {} \
         (a missing corpus is an ERROR here, never a skip)",
        dir.display()
    );
    let bank = Bank::load_dir(&dir).expect("real bank loads under the status schema");

    // Approved-only EXCEPT the deliberate retirements, adjudicated by the one
    // predicate cdcp_bank owns — see cdcp_bank::SANCTIONED_RETIRED.
    if let Err(msg) = cdcp_bank::sanctioned_retirement_report(&bank) {
        panic!("{msg}");
    }

    let exam = assemble(&bank, 42, AssembleConfig::default()).expect("seed 42 assembles");
    assert_eq!(exam.n_items, 40);
    for id in &exam.item_ids {
        assert!(
            bank.get(id).unwrap().is_approved(),
            "seed 42 drew a non-approved item: {id}"
        );
    }
}
