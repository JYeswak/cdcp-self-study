//! Named C1 assertion for the CHARTER mutate/delete pair.
//!
//! Bead: `bd-single-leg-metatest-closes-illw`.
//!
//! This file is the assertion half of the pair required by `.flywheel/CHARTER.md`:
//!
//!   1. MUTATE the C1 filter in `cdcp_assemble::sample_item_ids`
//!      (`filter(|i| i.is_approved())` → admit everything) → this suite goes
//!      non-zero.
//!   2. With that mutation STILL IN PLACE, delete this assertion → the suite
//!      returns to zero.
//!
//! The driver is `scripts/selftest_reconstructed.sh`. Restore of the
//! cargo-compiled sources goes through `scripts/restore_safe.inc.sh`
//! (`cdcp_restore_safe`); never `mv` a backup over dest.
//!
//! Shape matches `status_filter.rs`: 39 approved + 1 planted draft that is the
//! 40th and last eligible item, so with NO filter it is drawn at every seed.
//! Flipping only `status` isolates the filter.
//!
//! FLOOR-RAISE: this test asserts a draft cannot fill the approved pool. It
//! cannot decide that an `approved` item is any good.

use cdcp_assemble::{sample_item_ids, AssembleError};
use cdcp_bank::{Bank, BankItem, ItemStatus};

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
        objective_ids: Vec::new(),
        citation_ids: Vec::new(),
        tags: Vec::new(),
        bloom: "understand".into(),
        source_class: "original".into(),
        quantity_evidence: "qualitative_only".into(),
        status,
    }
}

#[test]
fn planted_draft_is_refused_by_the_approved_only_filter() {
    let mut items: Vec<BankItem> = (0..39)
        .map(|i| {
            item(
                &format!("ok-{i:03}"),
                (i as u32 % 8) + 1,
                ItemStatus::Approved,
            )
        })
        .collect();
    items.push(item("planted-zzz", 8, ItemStatus::Draft));
    let bank = Bank::from_items(items).expect("planted bank loads");

    match sample_item_ids(&bank, 40, SEED, 8, 8) {
        Err(AssembleError::PoolTooSmall {
            approved,
            n,
            total,
            not_approved,
        }) => {
            assert_eq!(
                (approved, n, total, not_approved),
                (39, 40, 40, 1),
                "draft must not enter the eligible pool"
            );
        }
        other => panic!("draft item leaked into the eligible pool: {other:?}"),
    }
}
