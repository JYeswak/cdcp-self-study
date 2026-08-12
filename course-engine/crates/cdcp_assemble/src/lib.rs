//! ORACLE-GAUNTLET L2 — seeded stratified assemble + choice shuffle remap.
//!
//! # PRNG (OQ-01 ASSUMED)
//!
//! Uses `rand::rngs::StdRng` seeded via `SeedableRng::seed_from_u64`.
//! `StdRng` is a ChaCha-based CSPRNG (currently ChaCha12). This freezes the
//! **native** L2 algorithm for later L4 WASM dual-path parity work (`bd-hdj`).
//!
//! Stratification mirrors `scripts/sample_mock.py` (module group → shuffle
//! modules → one-per-module first pass → round-robin fill → final order
//! shuffle). Byte streams will not match CPython `random.Random` (MT19937);
//! structure + seed stability within this crate is the L2 contract.
#![forbid(unsafe_code)]

use cdcp_bank::{Bank, BankItem};
use cdcp_core::ChoiceLetter;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

/// Documented PRNG for L2 assemble / shuffle (OQ-01).
pub type AssembleRng = StdRng;

/// Create the assemble PRNG from a u64 seed (StdRng / ChaCha family).
pub fn rng_from_seed(seed: u64) -> AssembleRng {
    StdRng::seed_from_u64(seed)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssembleError {
    #[error("pool size {pool} < requested n={n}")]
    PoolTooSmall { pool: usize, n: usize },
    #[error("item {0}: choices must be length 4")]
    BadChoices(String),
    #[error("item {0}: {1}")]
    Item(String, String),
    #[error("could not fill exam: need {n}, got {got}")]
    Undersampled { n: usize, got: usize },
}

/// Defaults aligned with `knowledge/bank_policy.toml` + `exam_form.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssembleConfig {
    pub n_items: usize,
    pub max_per_module: usize,
    pub min_modules: usize,
    /// When true, apply choice shuffle + correct-letter remap per item.
    pub shuffle_choices: bool,
}

impl Default for AssembleConfig {
    fn default() -> Self {
        Self {
            n_items: 40,
            max_per_module: 8,
            min_modules: 8,
            shuffle_choices: true,
        }
    }
}

/// One item as presented on a mock form (possibly choice-shuffled).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssembledItem {
    pub id: String,
    pub module: u32,
    pub stem: String,
    pub choices: Vec<String>,
    /// Correct letter **relative to `choices` as presented**.
    pub correct: String,
    /// Original bank correct letter (pre-shuffle).
    pub original_correct: String,
}

/// Seeded mock exam form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssembledExam {
    pub exam_id: String,
    pub seed: u64,
    pub n_items: usize,
    pub bank_hash: String,
    pub item_ids: Vec<String>,
    pub modules: Vec<u32>,
    pub items: Vec<AssembledItem>,
}

/// Stratified sample of item ids (presentation order), matching sample_mock flow.
pub fn sample_item_ids(
    bank: &Bank,
    n: usize,
    seed: u64,
    max_per_module: usize,
    _min_modules: usize,
) -> Result<Vec<String>, AssembleError> {
    let pool = bank.items.len();
    if pool < n {
        return Err(AssembleError::PoolTooSmall { pool, n });
    }

    // Group by module; sort each group by id (BTreeMap keys already sorted when collected).
    let mut by_mod: BTreeMap<u32, Vec<&BankItem>> = BTreeMap::new();
    for item in bank.items.values() {
        by_mod.entry(item.module).or_default().push(item);
    }
    // Ensure each module list is sorted by id (defensive; values() is id-ordered but group is not).
    for list in by_mod.values_mut() {
        list.sort_by(|a, b| a.id.cmp(&b.id));
    }

    let mut rng = rng_from_seed(seed);
    let mut modules: Vec<u32> = by_mod.keys().copied().collect();
    modules.shuffle(&mut rng);
    for m in &modules {
        if let Some(list) = by_mod.get_mut(m) {
            list.shuffle(&mut rng);
        }
    }

    let mut chosen: Vec<String> = Vec::with_capacity(n);
    let mut used: HashSet<String> = HashSet::new();
    let mut mod_counts: HashMap<u32, usize> = HashMap::new();

    // First pass: at least one from as many modules as possible.
    for &m in &modules {
        if chosen.len() >= n {
            break;
        }
        let Some(list) = by_mod.get(&m) else {
            continue;
        };
        let count = *mod_counts.get(&m).unwrap_or(&0);
        if count >= max_per_module {
            continue;
        }
        for it in list {
            if used.contains(&it.id) {
                continue;
            }
            chosen.push(it.id.clone());
            used.insert(it.id.clone());
            *mod_counts.entry(m).or_insert(0) += 1;
            break;
        }
    }

    // Second pass: round-robin fill under max_per_module.
    while chosen.len() < n {
        let mut progress = false;
        for &m in &modules {
            if chosen.len() >= n {
                break;
            }
            let count = *mod_counts.get(&m).unwrap_or(&0);
            if count >= max_per_module {
                continue;
            }
            let Some(list) = by_mod.get(&m) else {
                continue;
            };
            for it in list {
                if used.contains(&it.id) {
                    continue;
                }
                chosen.push(it.id.clone());
                used.insert(it.id.clone());
                *mod_counts.entry(m).or_insert(0) += 1;
                progress = true;
                break;
            }
        }
        if !progress {
            // Relax max_per_module: take remaining pool in bank id order.
            for it in bank.items.values() {
                if used.contains(&it.id) {
                    continue;
                }
                chosen.push(it.id.clone());
                used.insert(it.id.clone());
                if chosen.len() >= n {
                    break;
                }
            }
            break;
        }
    }

    if chosen.len() < n {
        return Err(AssembleError::Undersampled {
            n,
            got: chosen.len(),
        });
    }
    chosen.truncate(n);

    // Final presentation-order shuffle.
    let mut order: Vec<usize> = (0..chosen.len()).collect();
    order.shuffle(&mut rng);
    let ordered: Vec<String> = order.into_iter().map(|i| chosen[i].clone()).collect();
    Ok(ordered)
}

/// Fisher–Yates shuffle of 4 choices; remaps correct letter to the new index.
///
/// Invariant: `shuffled[new_correct] == choices[old_correct]` (same answer text).
pub fn shuffle_choices(
    choices: &[String],
    correct: ChoiceLetter,
    rng: &mut impl Rng,
) -> Result<(Vec<String>, ChoiceLetter), AssembleError> {
    if choices.len() != 4 {
        return Err(AssembleError::BadChoices("len≠4".into()));
    }
    let mut indices = [0usize, 1, 2, 3];
    indices.shuffle(rng);
    let shuffled: Vec<String> = indices.iter().map(|&i| choices[i].clone()).collect();
    let old_idx = letter_index(correct);
    let correct_text = &choices[old_idx];
    let new_idx = shuffled
        .iter()
        .position(|c| c == correct_text)
        .expect("correct text must appear in shuffled choices");
    Ok((shuffled, index_letter(new_idx)))
}

fn letter_index(c: ChoiceLetter) -> usize {
    match c {
        ChoiceLetter::A => 0,
        ChoiceLetter::B => 1,
        ChoiceLetter::C => 2,
        ChoiceLetter::D => 3,
    }
}

fn index_letter(i: usize) -> ChoiceLetter {
    match i {
        0 => ChoiceLetter::A,
        1 => ChoiceLetter::B,
        2 => ChoiceLetter::C,
        3 => ChoiceLetter::D,
        _ => unreachable!("choice index out of range"),
    }
}

/// Assemble a full mock exam: stratified sample + optional choice shuffle remap.
pub fn assemble(
    bank: &Bank,
    seed: u64,
    cfg: AssembleConfig,
) -> Result<AssembledExam, AssembleError> {
    let ids = sample_item_ids(
        bank,
        cfg.n_items,
        seed,
        cfg.max_per_module,
        cfg.min_modules,
    )?;

    // Derive per-item shuffle stream from the same seed family:
    // after sampling consumes rng draws, we re-seed a dedicated shuffle rng
    // as seed XOR a fixed domain tag so shuffle is deterministic & independent
    // of sampling draw count drift.
    let mut shuffle_rng = rng_from_seed(seed ^ 0xC_DCP_5UFF_1Eu64);

    let mut items: Vec<AssembledItem> = Vec::with_capacity(ids.len());
    let mut modules_set: BTreeMap<u32, ()> = BTreeMap::new();

    for id in &ids {
        let bank_item = bank
            .get(id)
            .ok_or_else(|| AssembleError::Item(id.clone(), "missing from bank".into()))?;
        if bank_item.choices.len() != 4 {
            return Err(AssembleError::BadChoices(id.clone()));
        }
        let original = bank_item
            .correct_letter()
            .map_err(|e| AssembleError::Item(id.clone(), e.to_string()))?;

        let (choices, correct) = if cfg.shuffle_choices {
            shuffle_choices(&bank_item.choices, original, &mut shuffle_rng)?
        } else {
            (bank_item.choices.clone(), original)
        };

        modules_set.insert(bank_item.module, ());
        items.push(AssembledItem {
            id: bank_item.id.clone(),
            module: bank_item.module,
            stem: bank_item.stem.clone(),
            choices,
            correct: correct.as_str().to_string(),
            original_correct: original.as_str().to_string(),
        });
    }

    let modules: Vec<u32> = modules_set.keys().copied().collect();
    Ok(AssembledExam {
        exam_id: format!("mock{}", cfg.n_items),
        seed,
        n_items: items.len(),
        bank_hash: bank.bank_hash.clone(),
        item_ids: ids,
        modules,
        items,
    })
}

/// Compact JSON payload for CLI (`item_ids` primary).
pub fn exam_ids_json(exam: &AssembledExam) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct IdsOnly<'a> {
        exam_id: &'a str,
        seed: u64,
        n_items: usize,
        bank_hash: &'a str,
        item_ids: &'a [String],
        modules: &'a [u32],
    }
    serde_json::to_string_pretty(&IdsOnly {
        exam_id: &exam.exam_id,
        seed: exam.seed,
        n_items: exam.n_items,
        bank_hash: &exam.bank_hash,
        item_ids: &exam.item_ids,
        modules: &exam.modules,
    })
}

/// Build answers that select every item's **remapped** correct letter.
pub fn remapped_all_correct_answers(
    exam: &AssembledExam,
) -> Result<Vec<cdcp_core::AnsweredItem>, AssembleError> {
    let mut out = Vec::with_capacity(exam.items.len());
    for it in &exam.items {
        let chosen = ChoiceLetter::parse(&it.correct)
            .map_err(|e| AssembleError::Item(it.id.clone(), e.to_string()))?;
        out.push(cdcp_core::AnsweredItem {
            item_id: it.id.clone(),
            chosen,
        });
    }
    Ok(out)
}

/// Grade remapped answers against the **form** by overlaying remapped correct
/// letters onto bank items (bank_hash preserved as content-address of the pool).
///
/// Learner letters are presentation-relative; we map them back to the original
/// bank letter via the answer-text identity, then grade with `cdcp_grade`.
pub fn grade_remapped_score(
    bank: &Bank,
    exam: &AssembledExam,
    answers: &[cdcp_core::AnsweredItem],
) -> Result<u32, AssembleError> {
    use std::collections::HashMap;
    let by_id: HashMap<&str, &AssembledItem> =
        exam.items.iter().map(|i| (i.id.as_str(), i)).collect();
    let mut score = 0u32;
    for ans in answers {
        let form_item = by_id
            .get(ans.item_id.as_str())
            .ok_or_else(|| AssembleError::Item(ans.item_id.clone(), "not on form".into()))?;
        let bank_item = bank
            .get(&ans.item_id)
            .ok_or_else(|| AssembleError::Item(ans.item_id.clone(), "missing bank".into()))?;
        // Map presentation letter → answer text → match bank correct text.
        let idx = letter_index(ans.chosen);
        if idx >= form_item.choices.len() {
            continue;
        }
        let chosen_text = &form_item.choices[idx];
        let bank_correct = bank_item
            .correct_letter()
            .map_err(|e| AssembleError::Item(ans.item_id.clone(), e.to_string()))?;
        let bank_text = &bank_item.choices[letter_index(bank_correct)];
        if chosen_text == bank_text {
            score += 1;
        }
    }
    Ok(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdcp_grade::{all_correct_attempt, grade};
    use std::path::PathBuf;

    fn bank_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items")
    }

    fn load_bank() -> Option<Bank> {
        let p = bank_path();
        if !p.is_dir() {
            eprintln!("skip: bank missing at {}", p.display());
            return None;
        }
        Some(Bank::load_dir(&p).expect("load bank"))
    }

    #[test]
    fn same_seed_same_item_ids_order() {
        let Some(bank) = load_bank() else { return };
        let cfg = AssembleConfig::default();
        let a = assemble(&bank, 42, cfg).unwrap();
        let b = assemble(&bank, 42, cfg).unwrap();
        assert_eq!(a.item_ids, b.item_ids);
        assert_eq!(a.n_items, 40);
        // Full form presentation (choices + remapped correct) also stable.
        assert_eq!(a.items, b.items);
    }

    #[test]
    fn different_seed_usually_different_set() {
        let Some(bank) = load_bank() else { return };
        let cfg = AssembleConfig::default();
        let a = assemble(&bank, 42, cfg).unwrap();
        let b = assemble(&bank, 43, cfg).unwrap();
        // With a 798-item pool, seeds 42 vs 43 should almost always differ.
        assert_ne!(
            a.item_ids, b.item_ids,
            "expected different item_ids for seeds 42 vs 43"
        );
    }

    #[test]
    fn shuffle_preserves_answer_text() {
        let choices = vec![
            "alpha".into(),
            "bravo".into(),
            "charlie".into(),
            "delta".into(),
        ];
        let mut rng = rng_from_seed(7);
        let (shuffled, new_correct) =
            shuffle_choices(&choices, ChoiceLetter::B, &mut rng).unwrap();
        assert_eq!(shuffled.len(), 4);
        assert_eq!(
            &shuffled[letter_index(new_correct)],
            "bravo",
            "remapped letter must still point at original correct text"
        );
        // Same seed → identical shuffle (idempotent independent runs).
        let mut rng2 = rng_from_seed(7);
        let (shuffled2, new_correct2) =
            shuffle_choices(&choices, ChoiceLetter::B, &mut rng2).unwrap();
        assert_eq!(shuffled, shuffled2);
        assert_eq!(new_correct, new_correct2);
    }

    #[test]
    fn shuffle_can_change_letter() {
        // Exhaust seeds until order actually changes (or letter stays if fixed point).
        let choices = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let mut saw_change = false;
        for seed in 0..200u64 {
            let mut rng = rng_from_seed(seed);
            let (shuffled, new_c) =
                shuffle_choices(&choices, ChoiceLetter::A, &mut rng).unwrap();
            if shuffled != choices {
                saw_change = true;
                // If order changed, letter may or may not move; text identity holds.
                assert_eq!(&shuffled[letter_index(new_c)], "a");
            }
            if new_c != ChoiceLetter::A {
                // Letter remapped when A moved off index 0.
                assert_ne!(shuffled[0], "a");
            }
        }
        assert!(saw_change, "expected some seed to permute choices");
    }

    #[test]
    fn metamorphic_remapped_all_correct_score() {
        let Some(bank) = load_bank() else { return };
        let cfg = AssembleConfig::default();
        let exam = assemble(&bank, 42, cfg).unwrap();

        // Baseline: original-bank all-correct via grade oracle.
        let att = all_correct_attempt(&bank, &exam.exam_id, exam.seed, &exam.item_ids).unwrap();
        let report = grade(&bank, &att).unwrap();
        assert_eq!(report.score_correct, exam.n_items as u32);
        assert_eq!(report.score_total, exam.n_items as u32);

        // Remapped answers (presentation letters) still score all-correct.
        let remapped = remapped_all_correct_answers(&exam).unwrap();
        let score = grade_remapped_score(&bank, &exam, &remapped).unwrap();
        assert_eq!(
            score, report.score_correct,
            "remapped all-correct must match unshuffled all-correct score"
        );
        assert_eq!(score, 40);
    }

    #[test]
    fn stratified_respects_max_per_module_softly() {
        let Some(bank) = load_bank() else { return };
        let cfg = AssembleConfig {
            n_items: 40,
            max_per_module: 8,
            min_modules: 8,
            shuffle_choices: false,
        };
        let exam = assemble(&bank, 99, cfg).unwrap();
        let mut counts: HashMap<u32, usize> = HashMap::new();
        for it in &exam.items {
            *counts.entry(it.module).or_insert(0) += 1;
        }
        // With a healthy multi-module pool, soft cap should hold without relax.
        for (m, c) in &counts {
            assert!(
                *c <= 8,
                "module {m} has {c} items (max_per_module=8); pool may be starved"
            );
        }
        assert!(
            counts.len() >= cfg.min_modules,
            "expected ≥{} modules, got {}",
            cfg.min_modules,
            counts.len()
        );
    }
}
