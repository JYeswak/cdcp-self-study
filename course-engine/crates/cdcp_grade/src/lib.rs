//! Pure grade() — GradeExact oracle.
#![forbid(unsafe_code)]

use cdcp_bank::Bank;
use cdcp_core::{
    digest_report, ExamAttempt, GradeReport, ItemResult, ModuleScore, STUDY_PASS_CORRECT,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GradeError {
    #[error("bank_hash mismatch: attempt has {attempt}, bank has {bank}")]
    BankHashMismatch { attempt: String, bank: String },
    #[error("unknown item_id: {0}")]
    UnknownItem(String),
    #[error("empty attempt")]
    EmptyAttempt,
    #[error("core: {0}")]
    Core(String),
}

/// Weak module: correctness rate strictly below 3/5 (0.6).
///
/// Integer-only compare (no f64): `5 * correct < 3 * total`.
/// Multiplies use `u64` intermediates to avoid u32 overflow on large totals.
#[inline]
pub fn is_weak_module(correct: u32, total: u32) -> bool {
    if total == 0 {
        false
    } else {
        5u64 * u64::from(correct) < 3u64 * u64::from(total)
    }
}

pub fn grade(bank: &Bank, attempt: &ExamAttempt) -> Result<GradeReport, GradeError> {
    if attempt.answers.is_empty() {
        return Err(GradeError::EmptyAttempt);
    }
    if attempt.bank_hash != bank.bank_hash {
        return Err(GradeError::BankHashMismatch {
            attempt: attempt.bank_hash.clone(),
            bank: bank.bank_hash.clone(),
        });
    }

    let mut item_results: Vec<ItemResult> = Vec::with_capacity(attempt.answers.len());
    let mut mod_correct: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
    let mut score_correct: u32 = 0;

    for ans in &attempt.answers {
        let item = bank
            .get(&ans.item_id)
            .ok_or_else(|| GradeError::UnknownItem(ans.item_id.clone()))?;
        let correct = item
            .correct_letter()
            .map_err(|e| GradeError::Core(e.to_string()))?;
        let is_correct = ans.chosen == correct;
        if is_correct {
            score_correct += 1;
        }
        let entry = mod_correct.entry(item.module).or_insert((0, 0));
        entry.1 += 1;
        if is_correct {
            entry.0 += 1;
        }
        item_results.push(ItemResult {
            item_id: ans.item_id.clone(),
            chosen: ans.chosen,
            correct,
            is_correct,
        });
    }

    let score_total = item_results.len() as u32;
    let by_module: Vec<ModuleScore> = mod_correct
        .iter()
        .map(|(m, (c, t))| ModuleScore {
            module: *m,
            correct: *c,
            total: *t,
        })
        .collect();

    let mut weak_modules: Vec<u32> = mod_correct
        .iter()
        .filter(|(_, (c, t))| is_weak_module(*c, *t))
        .map(|(m, _)| *m)
        .collect();
    weak_modules.sort_unstable();

    let passed_study_signal = score_correct >= STUDY_PASS_CORRECT;

    Ok(GradeReport {
        schema_version: cdcp_core::SCHEMA_VERSION,
        bank_hash: bank.bank_hash.clone(),
        exam_id: attempt.exam_id.clone(),
        seed: attempt.seed,
        item_results,
        score_correct,
        score_total,
        by_module,
        weak_modules,
        passed_study_signal,
    })
}

pub fn grade_digest(bank: &Bank, attempt: &ExamAttempt) -> Result<String, GradeError> {
    let report = grade(bank, attempt)?;
    digest_report(&report).map_err(|e| GradeError::Core(e.to_string()))
}

/// Build attempt answering every listed item with the item's correct letter.
pub fn all_correct_attempt(
    bank: &Bank,
    exam_id: &str,
    seed: u64,
    item_ids: &[String],
) -> Result<ExamAttempt, GradeError> {
    let mut answers = Vec::new();
    for id in item_ids {
        let item = bank
            .get(id)
            .ok_or_else(|| GradeError::UnknownItem(id.clone()))?;
        let correct = item
            .correct_letter()
            .map_err(|e| GradeError::Core(e.to_string()))?;
        answers.push(cdcp_core::AnsweredItem {
            item_id: id.clone(),
            chosen: correct,
        });
    }
    Ok(ExamAttempt {
        exam_id: exam_id.into(),
        seed,
        bank_hash: bank.bank_hash.clone(),
        answers,
    })
}

pub fn all_wrong_attempt(
    bank: &Bank,
    exam_id: &str,
    seed: u64,
    item_ids: &[String],
) -> Result<ExamAttempt, GradeError> {
    let mut answers = Vec::new();
    for id in item_ids {
        let item = bank
            .get(id)
            .ok_or_else(|| GradeError::UnknownItem(id.clone()))?;
        let correct = item
            .correct_letter()
            .map_err(|e| GradeError::Core(e.to_string()))?;
        answers.push(cdcp_core::AnsweredItem {
            item_id: id.clone(),
            chosen: correct.wrong_letter(),
        });
    }
    Ok(ExamAttempt {
        exam_id: exam_id.into(),
        seed,
        bank_hash: bank.bank_hash.clone(),
        answers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdcp_bank::Bank;
    use std::path::PathBuf;

    fn bank_path() -> PathBuf {
        if let Ok(p) = std::env::var("CDCP_BANK") {
            return PathBuf::from(p);
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items")
    }

    #[test]
    fn weak_module_boundaries_integer_threshold() {
        // strict rate < 0.6 via 5*correct < 3*total
        assert!(is_weak_module(2, 5), "2/5=0.4 is weak");
        assert!(!is_weak_module(3, 5), "3/5=0.6 is not weak (strict <)");
        assert!(!is_weak_module(6, 10), "6/10=0.6 is not weak (strict <)");
        assert!(is_weak_module(5, 10), "5/10=0.5 is weak");
        assert!(!is_weak_module(0, 0), "empty module is not weak");
        assert!(is_weak_module(0, 1), "0/1 is weak");
        assert!(!is_weak_module(1, 1), "1/1 is not weak");
    }

    #[test]
    fn load_real_bank_and_grade_idempotent() {
        let path = bank_path();
        assert!(
            path.is_dir(),
            "bank/items required at {} (set CDCP_BANK or check out bank/)",
            path.display()
        );
        let bank = Bank::load_dir(&path).expect("load bank");
        // take first 5 ids
        let ids: Vec<String> = bank.items.keys().take(5).cloned().collect();
        let att = all_correct_attempt(&bank, "test5", 1, &ids).unwrap();
        let d1 = grade_digest(&bank, &att).unwrap();
        let d2 = grade_digest(&bank, &att).unwrap();
        assert_eq!(d1, d2);
        let r = grade(&bank, &att).unwrap();
        assert_eq!(r.score_correct, 5);
        assert_eq!(r.score_total, 5);
    }

    #[test]
    fn bank_hash_mismatch() {
        let path = bank_path();
        assert!(
            path.is_dir(),
            "bank/items required at {}",
            path.display()
        );
        let bank = Bank::load_dir(&path).unwrap();
        let ids: Vec<String> = bank.items.keys().take(1).cloned().collect();
        let mut att = all_correct_attempt(&bank, "t", 0, &ids).unwrap();
        att.bank_hash = "deadbeef".into();
        assert!(matches!(
            grade(&bank, &att),
            Err(GradeError::BankHashMismatch { .. })
        ));
    }

    #[test]
    fn all_wrong_zero() {
        let path = bank_path();
        assert!(
            path.is_dir(),
            "bank/items required at {}",
            path.display()
        );
        let bank = Bank::load_dir(&path).unwrap();
        let ids: Vec<String> = bank.items.keys().take(3).cloned().collect();
        let att = all_wrong_attempt(&bank, "t", 0, &ids).unwrap();
        let r = grade(&bank, &att).unwrap();
        assert_eq!(r.score_correct, 0);
        assert!(!r.passed_study_signal);
    }

    #[test]
    fn study_pass_const() {
        assert_eq!(STUDY_PASS_CORRECT, 27);
    }
}
