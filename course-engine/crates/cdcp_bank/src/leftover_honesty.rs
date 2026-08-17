//! Leftover-cartoon honesty (bd-curriculum-truth-ebrr.30 / .32).
//!
//! Approved items may not teach two retired sentences as the correct path:
//!
//! * **q154** — post-incident object is "root cause" singular
//!   (`timeline, root cause, corrective actions`).
//! * **q04** — human error as a peer outage-cause bucket
//!   (`classic major outage driver`).
//!
//! `m01-q210` stays approved: that item treats the three-bucket cartoon as
//! the *wrong* answer. This module does not retire it.
//!
//! # What this cannot decide
//!
//! It cannot decide that a rewritten stem is *good*. It refuses two exact
//! leftover sentences and requires the two live ids to stay present. A
//! paraphrase of the cartoon that avoids those strings is out of scope.

use crate::{BankItem, ItemStatus};

pub const Q154_ID: &str = "bank-m15-q154";
pub const Q154_OLD_STEM: &str =
    "Post-incident documentation (timeline, root cause, corrective actions) is valuable because it:";
pub const Q154_OLD_OBJECT: &str = "timeline, root cause";

pub const Q04_ID: &str = "mock40-q04";
pub const Q04_OLD_STEM: &str =
    "Which factor is frequently cited as a major contributor to data-centre outages in industry analyses?";
pub const Q04_OLD_EXPLANATION: &str =
    "Process and human error during change/maintenance work is a classic major outage driver.";
pub const Q04_OLD_KEY: &str = "Human error during change/maintenance";
pub const Q04_CARTOON_MARKER: &str = "classic major outage driver";

pub const Q210_ID: &str = "m01-q210";

/// Audit one item. Draft/retired copies of the old text are out of the
/// drawable pool (C1) and are not this check's job.
pub fn audit_item(item: &BankItem) -> Result<(), String> {
    if !item.is_approved() {
        return Ok(());
    }

    let stem = item.stem.as_str();
    if stem == Q154_OLD_STEM || contains_ci(stem, Q154_OLD_OBJECT) {
        return Err(format!(
            "{}: approved stem treats \"root cause\" singular as the post-incident object",
            item.id
        ));
    }

    if item.explanation == Q04_OLD_EXPLANATION || contains_ci(&item.explanation, Q04_CARTOON_MARKER)
    {
        return Err(format!(
            "{}: approved explanation asserts human-error as a peer outage bucket",
            item.id
        ));
    }

    if stem == Q04_OLD_STEM {
        return Err(format!(
            "{}: approved stem is the retired industry-analyses cartoon",
            item.id
        ));
    }

    if item.id == Q154_ID {
        audit_q154(item)?;
    }
    if item.id == Q04_ID {
        audit_q04(item)?;
    }
    Ok(())
}

fn audit_q154(item: &BankItem) -> Result<(), String> {
    let stem = item.stem.to_ascii_lowercase();
    let key = correct_text(item).to_ascii_lowercase();
    if !stem.contains("contributing factors") && !key.contains("contributing factors") {
        return Err(format!(
            "{Q154_ID}: approved item must name contributing factors, plural, in the stem or key"
        ));
    }
    if item.source_class != "original" {
        return Err(format!(
            "{Q154_ID}: source_class must stay original, got {:?}",
            item.source_class
        ));
    }
    Ok(())
}

fn audit_q04(item: &BankItem) -> Result<(), String> {
    let key = correct_text(item);
    if key == Q04_OLD_KEY {
        return Err(format!(
            "{Q04_ID}: approved key is still the human-error peer-bucket cartoon"
        ));
    }
    let blob = format!("{} {}", item.stem, key).to_ascii_lowercase();
    if !blob.contains("contributing factors") {
        return Err(format!(
            "{Q04_ID}: rewritten approved item must name contributing factors, plural"
        ));
    }
    if item.source_class != "original" {
        return Err(format!(
            "{Q04_ID}: source_class must stay original, got {:?}",
            item.source_class
        ));
    }
    Ok(())
}

/// Scan a loaded bank. An empty set is ERROR (a scan that never ran).
pub fn audit_bank<'a, I>(items: I) -> Result<(), String>
where
    I: IntoIterator<Item = &'a BankItem>,
{
    let items: Vec<&BankItem> = items.into_iter().collect();
    if items.is_empty() {
        return Err(
            "leftover-honesty: zero items scanned — an empty scan is an ERROR, not a pass"
                .to_string(),
        );
    }

    let mut by_id: std::collections::BTreeMap<&str, &BankItem> = std::collections::BTreeMap::new();
    for item in &items {
        if let Some(prev) = by_id.insert(item.id.as_str(), item) {
            return Err(format!("leftover-honesty: duplicate id {}", prev.id));
        }
        audit_item(item)?;
    }

    for required in [Q154_ID, Q04_ID, Q210_ID] {
        if !by_id.contains_key(required) {
            return Err(format!(
                "leftover-honesty: required id {required} missing from the scanned bank"
            ));
        }
    }

    let q210 = by_id[Q210_ID];
    if q210.status != ItemStatus::Approved {
        return Err(format!(
            "{Q210_ID} must stay approved (the three-bucket cartoon is the wrong answer there); got {}",
            q210.status
        ));
    }
    let q210_blob = format!("{} {}", q210.stem, q210.choices.join(" "));
    if !contains_ci(&q210_blob, "three equal root-cause buckets") {
        return Err(format!(
            "{Q210_ID} must still present the three-bucket cartoon as the wrong proposition"
        ));
    }
    if q210.correct != "B" {
        return Err(format!(
            "{Q210_ID} correct letter moved (expected B, the anti-cartoon reading); got {}",
            q210.correct
        ));
    }

    Ok(())
}

/// Capstone markdown must match the rewritten Q4. Missing/empty file is ERROR.
pub fn audit_practice_exam(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err(
            "leftover-honesty: PRACTICE-EXAM.md is empty — an empty scan is an ERROR, not a pass"
                .to_string(),
        );
    }
    if text.contains(Q04_OLD_STEM) {
        return Err(
            "leftover-honesty: PRACTICE-EXAM.md still carries the retired Q4 cartoon stem"
                .to_string(),
        );
    }
    if contains_ci(text, Q04_CARTOON_MARKER) {
        return Err(
            "leftover-honesty: PRACTICE-EXAM.md still asserts the classic-major-outage-driver cartoon"
                .to_string(),
        );
    }
    if !text
        .contains("When a hall outage is written up, human/process during change is treated as:")
    {
        return Err(
            "leftover-honesty: PRACTICE-EXAM.md Q4 does not match the rewritten mock40-q04 stem"
                .to_string(),
        );
    }
    if !contains_ci(text, "contributing factors") {
        return Err(
            "leftover-honesty: PRACTICE-EXAM.md Q4 must name contributing factors, plural"
                .to_string(),
        );
    }
    Ok(())
}

fn correct_text(item: &BankItem) -> String {
    match item.correct.as_str() {
        "A" => item.choices.first().cloned().unwrap_or_default(),
        "B" => item.choices.get(1).cloned().unwrap_or_default(),
        "C" => item.choices.get(2).cloned().unwrap_or_default(),
        "D" => item.choices.get(3).cloned().unwrap_or_default(),
        _ => String::new(),
    }
}

fn contains_ci(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ItemKind;

    fn item(id: &str, stem: &str, explanation: &str) -> BankItem {
        BankItem {
            id: id.to_string(),
            module: 1,
            stem: stem.to_string(),
            choices: vec![
                "A third peer root-cause bucket next to power and cooling".into(),
                "One of several contributing factors on a power, cooling, or network object — not a pie slice and not a memorized survey percentage".into(),
                "Using LED lighting".into(),
                "Over-documenting MOPs".into(),
            ],
            correct: "B".into(),
            explanation: explanation.to_string(),
            topic_ids: vec!["m01-unavailability".into()],
            objective_ids: vec![],
            citation_ids: vec![],
            tags: vec![],
            bloom: "understand".into(),
            source_class: "original".into(),
            quantity_evidence: "qualitative_only".into(),
            status: ItemStatus::Approved,
            kind: ItemKind::SingleSelect,
        }
    }

    fn q210() -> BankItem {
        BankItem {
            id: Q210_ID.into(),
            module: 1,
            stem: "A slide still lists power, cooling, and human error as three equal root-cause buckets and asks you to memorize this year’s survey share. The taught 2026 reading is:".into(),
            choices: vec![
                "Recite the three buckets and treat the largest published percentage as law".into(),
                "Power path leads; cooling is a cascade; human/process is contributing".into(),
                "Cooling is one-third of all outages, so start at the chiller".into(),
                "Human error is the unverifiable majority, so train harder and stop".into(),
            ],
            correct: "B".into(),
            explanation: "The three-bucket cartoon is retired.".into(),
            topic_ids: vec!["m01-unavailability".into()],
            objective_ids: vec!["obj-m01-cause-framing-2026".into()],
            citation_ids: vec![],
            tags: vec![],
            bloom: "understand".into(),
            source_class: "original".into(),
            quantity_evidence: "qualitative_only".into(),
            status: ItemStatus::Approved,
            kind: ItemKind::SingleSelect,
        }
    }

    fn rewritten_q154() -> BankItem {
        let mut it = item(
            Q154_ID,
            "Post-incident documentation (timeline, contributing factors, corrective actions) is valuable because it:",
            "Learning loops turn incidents into durable ops improvements. Record contributing factors, plural — never a single root cause.",
        );
        it.module = 15;
        it.choices = vec![
            "Only blames individuals without system learning".into(),
            "Converts experience into prevention—procedure, design, and monitoring improvements"
                .into(),
            "Is optional if the outage was short".into(),
            "Replaces the need for MOPs on future similar work".into(),
        ];
        it
    }

    fn rewritten_q04() -> BankItem {
        item(
            Q04_ID,
            "When a hall outage is written up, human/process during change is treated as:",
            "People and process contribute; they are not a peer root-cause bucket. Cite surveys as surveys. Refuse a fake percentage.",
        )
    }

    #[test]
    fn empty_scan_is_an_error() {
        let err = audit_bank(std::iter::empty()).unwrap_err();
        assert!(err.contains("zero items"), "{err}");
    }

    #[test]
    fn planted_old_q154_stem_is_red() {
        let mut planted = rewritten_q154();
        planted.stem = Q154_OLD_STEM.into();
        let err = audit_item(&planted).unwrap_err();
        assert!(err.contains(Q154_ID), "{err}");
        assert!(err.contains("root cause"), "{err}");
    }

    #[test]
    fn planted_old_q04_explanation_is_red() {
        let mut planted = rewritten_q04();
        planted.explanation = Q04_OLD_EXPLANATION.into();
        let err = audit_item(&planted).unwrap_err();
        assert!(err.contains(Q04_ID), "{err}");
        assert!(err.contains("peer"), "{err}");
    }

    #[test]
    fn planted_old_q04_key_is_red() {
        let mut planted = rewritten_q04();
        planted.choices[1] = Q04_OLD_KEY.into();
        let err = audit_item(&planted).unwrap_err();
        assert!(err.contains("peer-bucket"), "{err}");
    }

    #[test]
    fn rewritten_trio_is_green() {
        let items = [rewritten_q154(), rewritten_q04(), q210()];
        audit_bank(items.iter()).expect("rewritten trio must pass");
    }

    #[test]
    fn retiring_q210_is_red() {
        let mut q = q210();
        q.status = ItemStatus::Retired;
        let items = [rewritten_q154(), rewritten_q04(), q];
        let err = audit_bank(items.iter()).unwrap_err();
        assert!(err.contains(Q210_ID), "{err}");
        assert!(err.contains("approved"), "{err}");
    }

    #[test]
    fn planted_old_practice_stem_is_red() {
        let err = audit_practice_exam(&format!(
            "# practice\n{Q04_OLD_STEM}\n{Q04_OLD_EXPLANATION}\n"
        ))
        .unwrap_err();
        assert!(err.contains("cartoon stem"), "{err}");
    }

    #[test]
    fn empty_practice_is_an_error() {
        let err = audit_practice_exam("   \n").unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }
}
