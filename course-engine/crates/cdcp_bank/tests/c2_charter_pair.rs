//! Named C2 assertion for the CHARTER mutate/delete pair.
//!
//! Beads: `bd-hardening-c-status-hzs.2` (the assertion),
//! `bd-metatest-rerun-blocked-yhg6` (re-run under restore_safe).
//!
//! This file is the assertion half of the pair required by `.flywheel/CHARTER.md`:
//!
//!   1. MUTATE `BankItem::hash_payload` so it omits `status` → this suite
//!      goes non-zero.
//!   2. With that mutation STILL IN PLACE, delete this assertion → the suite
//!      returns to zero.
//!
//! The driver is `scripts/selftest_reconstructed.sh`. Restore of the
//! cargo-compiled sources goes through `scripts/restore_safe.inc.sh`
//! (`cdcp_restore_safe`); never `mv` a backup over dest.
//!
//! FLOOR-RAISE: this test asserts that flipping `approved` → `draft` moves
//! `bank_hash`. It cannot decide that any other field is in the payload
//! (`hash_payload_covers_every_modelled_field` in `src/lib.rs` is that
//! structural pin).
//!
//! Re-run 2026-08-15 (private `CARGO_TARGET_DIR=target/yhg6-rerun`,
//! `cdcp_restore_safe`, true exit codes to files): mutate 101 (hashes
//! identical `c740c0de…`), restore 0, artifact mtime moved.

use cdcp_bank::Bank;
use std::fs;

fn hash_of(tag: &str, status: &str) -> String {
    let dir =
        std::env::temp_dir().join(format!("cdcp-c2-pair-{tag}-{}-{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp bank dir");
    fs::write(
        dir.join("t.toml"),
        format!(
            r#"
id = "s1"
module = 3
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "B"
explanation = "because reasons here"
topic_ids = ["t1"]
objective_ids = []
citation_ids = []
tags = []
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "{status}"
"#
        ),
    )
    .expect("write item");
    let h = Bank::load_dir(&dir).expect("bank should load").bank_hash;
    let _ = fs::remove_dir_all(&dir);
    h
}

#[test]
fn status_flip_moves_bank_hash() {
    let approved = hash_of("approved", "approved");
    let draft = hash_of("draft", "draft");
    assert_ne!(
        approved, draft,
        "approved -> draft MUST move bank_hash: assembly draws approved items only (C1), \
         so this flip changes what a learner can be assessed on. A content address that \
         cannot see it is not a content address."
    );
}
