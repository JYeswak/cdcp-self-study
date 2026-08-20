//! Live-tree leftover-cartoon honesty (bd-curriculum-truth-ebrr.30 / .32).
//!
//! The in-tree check is `cdcp_bank::leftover_honesty`. A TEMP restore of the
//! old q154 stem or the old mock40-q04 explanation is refused here — in
//! memory, not via a `.patch` + worktree harness.

use cdcp_bank::leftover_honesty::{
    self, Q04_ID, Q04_OLD_EXPLANATION, Q04_OLD_STEM, Q154_ID, Q154_OLD_STEM, Q210_ID,
};
use cdcp_bank::{Bank, BankItem};
use std::path::PathBuf;

fn bank_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items")
}

fn practice_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../practice/PRACTICE-EXAM.md")
}

fn live_bank() -> Bank {
    let dir = bank_dir();
    assert!(
        dir.is_dir(),
        "live bank missing at {} — an empty scan is an ERROR, not a skip",
        dir.display()
    );
    Bank::load_dir(&dir).expect("live bank loads")
}

#[test]
fn live_bank_refuses_the_two_leftover_cartoons() {
    let bank = live_bank();
    leftover_honesty::audit_bank(bank.items.values()).expect("live bank leftover-honesty");
}

#[test]
fn live_q154_stem_is_not_the_old_singular() {
    let bank = live_bank();
    let item = bank
        .get(Q154_ID)
        .expect("bank-m15-q154 must stay in the bank");
    assert!(item.is_approved(), "q154 must stay approved");
    assert_ne!(item.stem, Q154_OLD_STEM);
    assert!(
        !item
            .stem
            .to_ascii_lowercase()
            .contains("timeline, root cause"),
        "q154 stem still treats root cause singular as the object: {}",
        item.stem
    );
    assert!(
        item.stem
            .to_ascii_lowercase()
            .contains("contributing factors"),
        "q154 stem must name contributing factors, plural: {}",
        item.stem
    );
}

#[test]
fn live_q04_explanation_is_not_the_peer_bucket() {
    let bank = live_bank();
    let item = bank.get(Q04_ID).expect("mock40-q04 must stay in the bank");
    assert!(item.is_approved(), "q04 rewritten in place, not deleted");
    assert_ne!(item.explanation, Q04_OLD_EXPLANATION);
    assert!(
        !item
            .explanation
            .to_ascii_lowercase()
            .contains("classic major outage driver"),
        "q04 explanation still asserts the peer-bucket cartoon: {}",
        item.explanation
    );
    assert_ne!(item.stem, Q04_OLD_STEM);
}

#[test]
fn live_q210_stays_approved_and_kills_the_cartoon() {
    let bank = live_bank();
    let item = bank.get(Q210_ID).expect("m01-q210 must stay");
    assert!(item.is_approved(), "m01-q210 must stay approved");
    let key_index = match item.correct.as_str() {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        other => panic!("q210 has an invalid correct letter: {other}"),
    };
    let key = item.choices[key_index].to_ascii_lowercase();
    for marker in ["power path", "cooling", "human"] {
        assert!(
            key.contains(marker),
            "q210 keyed text must retain the anti-cartoon marker {marker:?}: {key}"
        );
    }
    assert!(
        item.choices[0]
            .to_ascii_lowercase()
            .contains("three equal root-cause buckets")
            || item
                .stem
                .to_ascii_lowercase()
                .contains("three equal root-cause buckets"),
        "q210 must still present the cartoon as the wrong proposition"
    );
}

#[test]
fn temp_restore_of_old_q154_stem_is_red() {
    let bank = live_bank();
    let mut planted: BankItem = bank.get(Q154_ID).expect("q154").clone();
    planted.stem = Q154_OLD_STEM.into();
    let err = leftover_honesty::audit_item(&planted).expect_err("old singular stem must be RED");
    assert!(err.contains(Q154_ID), "{err}");
    assert!(err.contains("root cause"), "{err}");
}

#[test]
fn temp_restore_of_old_q04_explanation_is_red() {
    let bank = live_bank();
    let mut planted: BankItem = bank.get(Q04_ID).expect("q04").clone();
    planted.explanation = Q04_OLD_EXPLANATION.into();
    let err = leftover_honesty::audit_item(&planted)
        .expect_err("old peer-bucket explanation must be RED");
    assert!(err.contains(Q04_ID), "{err}");
}

#[test]
fn practice_exam_q4_matches_rewritten_item() {
    let path = practice_path();
    assert!(
        path.is_file(),
        "PRACTICE-EXAM.md missing at {} — empty scan is ERROR",
        path.display()
    );
    let text = std::fs::read_to_string(&path).expect("read PRACTICE-EXAM.md");
    leftover_honesty::audit_practice_exam(&text).expect("practice Q4 leftover-honesty");
}

#[test]
fn temp_restore_of_old_practice_stem_is_red() {
    let err = leftover_honesty::audit_practice_exam(Q04_OLD_STEM)
        .expect_err("old practice stem must be RED");
    assert!(err.contains("cartoon stem"), "{err}");
}
