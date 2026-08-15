//! ORACLE-GAUNTLET L2 — seeded stratified assemble + choice shuffle remap.
//!
//! # PRNG (C4 / OQ-01) — named, pinned, portable
//!
//! The sampler and the choice-shuffle remapper share one generator:
//!
//! * **Algorithm:** ChaCha12 (`rand_chacha::ChaCha12Rng`).
//! * **Seeding:** `rand::SeedableRng::seed_from_u64` (rand 0.8 / rand_core 0.6
//!   splitmix-style u64 → 32-byte seed). Same seed ⇒ same stream.
//! * **Crate pins:** workspace `rand = "=0.8.7"` (owns `SliceRandom::shuffle`)
//!   and `rand_chacha = "=0.3.1"` (owns the stream). Neither is a caret.
//! * **Not used:** `rand::rngs::StdRng`. Rand documents that type as free to
//!   change across releases; goldens must not sit on it.
//!
//! v1 (pre-C4) froze seed-42 under `StdRng` as of rand 0.8.7, which *happened*
//! to be ChaCha12 with this seeder. v2 names that algorithm. The stream is
//! identical — `item_ids` did not move; see `goldens/PROVENANCE.md` §PRNG.
//! A swap to ChaCha20 (or a future `StdRng`) is the known-bad: the pinned
//! stream in `prng_stream_seed42_is_pinned` goes RED.
//!
//! Stratification: group by module → shuffle modules → one-per-module first
//! pass → round-robin fill → final order shuffle. Choice shuffle uses a
//! dedicated generator `seed ^ 0xCDC5_FF1E`. Byte streams will not match
//! CPython `random.Random` (MT19937); the L2 contract is this crate's stream.
//!
//! Typed `cdcp_assess` kinds are **not** flattened to A–D. The product
//! assemble path ([`assemble`] / [`assemble_with`]) presents through
//! [`assemble_input`]; a non-letter kind is [`AssembleError::NotLetterMcq`].
#![forbid(unsafe_code)]

mod kind;

pub use kind::{admit_assemble_kind, assemble_input, AssembleInput, LETTER_ASSEMBLE_KINDS};

use cdcp_bank::{Bank, BankItem};
use cdcp_core::ChoiceLetter;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

/// Documented PRNG for L2 assemble / shuffle (C4).
///
/// Named ChaCha12, not `StdRng`. See the crate-level PRNG section.
pub type AssembleRng = ChaCha12Rng;

/// Create the assemble PRNG from a u64 seed (ChaCha12Rng / rand_chacha 0.3.1).
pub fn rng_from_seed(seed: u64) -> AssembleRng {
    ChaCha12Rng::seed_from_u64(seed)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssembleError {
    #[error("approved pool {approved} < requested n={n} ({total} items loaded, {not_approved} not approved)")]
    PoolTooSmall {
        approved: usize,
        n: usize,
        total: usize,
        not_approved: usize,
    },
    /// Anti-vacuous (CHARTER decalogue §3): an empty eligible pool is an
    /// ERROR, never an empty exam.
    #[error("no approved items in bank ({total} items loaded, 0 approved) — an empty approved pool is an ERROR, not an empty exam")]
    NoApprovedItems { total: usize },
    /// The approved-only pool cannot span the required breadth. This is a
    /// POOL precondition introduced by C1: filtering to `approved` can starve
    /// module coverage that the full pool satisfied.
    #[error("approved pool spans {modules} modules < min_modules={min_modules} ({approved} approved of {total} items) — shortfall {shortfall} modules")]
    ApprovedTooFewModules {
        modules: usize,
        min_modules: usize,
        approved: usize,
        total: usize,
        shortfall: usize,
    },
    /// The approved pool spanned enough modules, but the selected draw does
    /// not. Distinct from `ApprovedTooFewModules`: that is a pool
    /// precondition, this is a form-coverage failure. First-pass
    /// one-per-module usually prevents it when `n >= min_modules`; it fires
    /// when `n < min_modules` or when the `max_per_module` relaxation fills
    /// from a narrow id-ordered slice. C6 (`bd-hardening-c-status-hzs.5`).
    #[error("selected draw spans {modules} modules < min_modules={min_modules} (n={n}) — shortfall {shortfall} modules")]
    SelectedTooFewModules {
        modules: usize,
        min_modules: usize,
        n: usize,
        shortfall: usize,
    },
    #[error("item {0}: choices must be length 4")]
    BadChoices(String),
    #[error("item {0}: {1}")]
    Item(String, String),
    #[error("could not fill exam: need {n}, got {got}")]
    Undersampled { n: usize, got: usize },
    /// Typed assess kind that is not letter-mcq / single-select. Assemble
    /// must not flatten these back to four shuffled A–D strings.
    #[error(
        "item {id}: kind {kind} is not letter-mcq/single-select — assemble will not flatten it to A–D"
    )]
    NotLetterMcq { id: String, kind: String },
    /// Anti-vacuous: an empty typed assemble input is an ERROR, never an
    /// empty exam (same floor as [`AssembleError::NoApprovedItems`]).
    #[error("empty assemble input is an ERROR, not an empty exam")]
    EmptyInput,
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
///
/// # APPROVED ONLY (C1)
///
/// The eligible pool is `status == approved`. A `draft` or `retired` item is
/// never drawn — not in the first pass, not in the round-robin fill, and not
/// in the `max_per_module` relaxation fallback (that last path is the one a
/// naive filter misses, because it re-reads the whole bank).
///
/// CHARTER pair (bd-single-leg-metatest-closes-illw): mutating the `approved`
/// filter below turns `crates/cdcp_assemble/tests/c1_charter_pair.rs` non-zero;
/// deleting that assertion with the mutation still in place returns it to zero.
/// Driven by `scripts/selftest_reconstructed.sh`; restore via `cdcp_restore_safe`.
pub fn sample_item_ids(
    bank: &Bank,
    n: usize,
    seed: u64,
    max_per_module: usize,
    min_modules: usize,
) -> Result<Vec<String>, AssembleError> {
    let total = bank.items.len();

    // ── C1 APPROVED-ONLY FILTER (the gate) ──────────────────────────────────
    let approved: Vec<&BankItem> = bank.items.values().filter(|i| i.is_approved()).collect();

    if approved.is_empty() {
        return Err(AssembleError::NoApprovedItems { total });
    }
    let pool = approved.len();
    if pool < n {
        return Err(AssembleError::PoolTooSmall {
            approved: pool,
            n,
            total,
            not_approved: total - pool,
        });
    }

    // Group by module; sort each group by id (BTreeMap keys already sorted when collected).
    let mut by_mod: BTreeMap<u32, Vec<&BankItem>> = BTreeMap::new();
    for item in &approved {
        by_mod.entry(item.module).or_default().push(item);
    }

    // Breadth precondition: the approved-only pool may span fewer modules than
    // the full pool did. Report the shortfall rather than quietly assembling a
    // narrower exam than the form asks for.
    if by_mod.len() < min_modules {
        return Err(AssembleError::ApprovedTooFewModules {
            modules: by_mod.len(),
            min_modules,
            approved: pool,
            total,
            shortfall: min_modules - by_mod.len(),
        });
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
            // Relax max_per_module: take the remaining APPROVED pool in bank id
            // order. `approved` is collected from `bank.items.values()`, which
            // BTreeMap yields in id order, so this preserves the previous
            // ordering contract while staying inside the eligible pool.
            for it in &approved {
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

    // C6: min_modules is a property of the SELECTED form, not only of the
    // approved pool. The pool check above can pass while this draw still
    // under-covers (n < min_modules, or the max_per_module relaxation
    // taking a narrow id-ordered slice). Fail closed; never return a quiet
    // under-covered exam.
    let mut selected_modules: HashSet<u32> = HashSet::new();
    for id in &chosen {
        let item = bank
            .get(id)
            .ok_or_else(|| AssembleError::Item(id.clone(), "missing from bank".into()))?;
        selected_modules.insert(item.module);
    }
    if selected_modules.len() < min_modules {
        return Err(AssembleError::SelectedTooFewModules {
            modules: selected_modules.len(),
            min_modules,
            n: chosen.len(),
            shortfall: min_modules - selected_modules.len(),
        });
    }

    // Final presentation-order shuffle.
    let mut order: Vec<usize> = (0..chosen.len()).collect();
    order.shuffle(&mut rng);
    let ordered: Vec<String> = order.into_iter().map(|i| chosen[i].clone()).collect();
    Ok(ordered)
}

/// Fisher–Yates shuffle of 4 choices; remaps correct letter to the new index.
///
/// Letter-MCQ only. Typed assess kinds (multi-select, numeric-range, …)
/// must go through [`assemble_input`], which refuses to flatten them here.
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
///
/// Presentation goes through [`assemble_input`]. Bank rows are
/// [`AssembleInput::LetterMcq`]. Extra typed rows (CLI `--assess`) are
/// checked for kind **before** any shuffle — a multi-select is
/// [`AssembleError::NotLetterMcq`], never four shuffled A–D strings.
///
/// `extra` empty is the historical letter-MCQ path: same sample, same
/// shuffle stream (`seed ^ 0xCDC5_FF1E`), same `item_ids`.
pub fn assemble(
    bank: &Bank,
    seed: u64,
    cfg: AssembleConfig,
) -> Result<AssembledExam, AssembleError> {
    assemble_with(bank, seed, cfg, &[])
}

/// Product assemble: sample the bank, then present sample + `extra` via
/// [`assemble_input`].
pub fn assemble_with(
    bank: &Bank,
    seed: u64,
    cfg: AssembleConfig,
    extra: &[AssembleInput<'_>],
) -> Result<AssembledExam, AssembleError> {
    let ids = sample_item_ids(bank, cfg.n_items, seed, cfg.max_per_module, cfg.min_modules)?;

    let mut sampled: Vec<&BankItem> = Vec::with_capacity(ids.len());
    for id in &ids {
        let bank_item = bank
            .get(id)
            .ok_or_else(|| AssembleError::Item(id.clone(), "missing from bank".into()))?;
        sampled.push(bank_item);
    }

    let mut input: Vec<AssembleInput<'_>> = sampled
        .iter()
        .copied()
        .map(AssembleInput::LetterMcq)
        .collect();
    input.extend_from_slice(extra);

    let items = assemble_input(&input, seed, cfg)?;

    let mut modules_set: BTreeMap<u32, ()> = BTreeMap::new();
    for it in &items {
        modules_set.insert(it.module, ());
    }
    let modules: Vec<u32> = modules_set.keys().copied().collect();
    Ok(AssembledExam {
        exam_id: format!("mock{}", cfg.n_items),
        seed,
        n_items: items.len(),
        bank_hash: bank.bank_hash.clone(),
        item_ids: items.iter().map(|i| i.id.clone()).collect(),
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

    /// First eight `u64`s from `ChaCha12Rng::seed_from_u64(42)` under
    /// `rand_chacha = 0.3.1`. This is the C4 stream contract. A `rand` 0.8 → 0.9
    /// bump that went through `StdRng` would have been allowed to change this
    /// without touching any file we own; pinning the algorithm here makes that
    /// RED. Measured 2026-08-14: `StdRng` as of rand 0.8.7 emits the same eight
    /// values (v1 == v2); `ChaCha20Rng` at the same seed does not.
    const SEED42_FIRST_8_U64: [u64; 8] = [
        0x86cc_7763_2227_24a2,
        0x8af0_0a13_3fad_517d,
        0xa2ef_6071_de51_34d1,
        0x67e9_2d78_fd76_30b2,
        0x08ca_b0df_f811_9fea,
        0x6a3a_9ca3_9e0f_81a8,
        0xbcc7_d8e8_5908_78fb,
        0xd968_8d9b_2f8e_b737,
    ];

    #[test]
    fn assemble_rng_is_chacha12_not_stdrng() {
        let name = std::any::type_name::<AssembleRng>();
        assert!(
            name.contains("ChaCha12"),
            "AssembleRng must be ChaCha12Rng, got {name}"
        );
        assert!(
            !name.contains("StdRng"),
            "AssembleRng must not be StdRng, got {name}"
        );
    }

    #[test]
    fn prng_stream_seed42_is_pinned() {
        let mut rng = rng_from_seed(42);
        let got: [u64; 8] = std::array::from_fn(|_| rng.gen::<u64>());
        if got != SEED42_FIRST_8_U64 {
            // Print so the first landing can copy the measured stream into the
            // pin. After the pin is real this branch is the known-bad.
            panic!("seed-42 ChaCha12 stream moved: {got:#x?}");
        }
    }

    #[test]
    fn chacha20_at_same_seed_is_a_different_stream() {
        use rand_chacha::ChaCha20Rng;
        let mut twelve = ChaCha12Rng::seed_from_u64(42);
        let mut twenty = ChaCha20Rng::seed_from_u64(42);
        assert_ne!(
            twelve.gen::<u64>(),
            twenty.gen::<u64>(),
            "ChaCha12 vs ChaCha20 must differ at seed 42 — that difference is the known-bad for swapping the algorithm"
        );
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
        let (shuffled, new_correct) = shuffle_choices(&choices, ChoiceLetter::B, &mut rng).unwrap();
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
            let (shuffled, new_c) = shuffle_choices(&choices, ChoiceLetter::A, &mut rng).unwrap();
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
