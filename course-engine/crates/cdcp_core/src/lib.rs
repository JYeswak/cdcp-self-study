//! cdcp_core — types + byte-exact canonical digests (Assessment-System).
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

pub const SCHEMA_VERSION: u32 = 1;

/// Domain-separation tag for `cdcp_bank::compute_bank_hash`.
///
/// **This tag names a DEFINITION, not a bank snapshot.** Two hashes carrying
/// the same tag are comparable: a difference between them means the bank's
/// content moved. Two hashes carrying different tags are NOT comparable, and a
/// reader must not read their difference as drift.
///
/// | Tag | What the hash covers |
/// |-----|----------------------|
/// | `cdcp-bank-v1` | `id`, `module`, `stem`, `choices`, `correct`, `explanation`, `bloom`, `source_class`, `quantity_evidence`, `topic_ids`. `status` was excluded, and `objective_ids` / `citation_ids` / `tags` were not modelled at all — serde discarded them on load. A `status` flip therefore did not move a v1 hash, while it *did* move what a learner could be assessed on (C1). |
/// | `cdcp-bank-v2` | v1's fields **plus** `objective_ids`, `citation_ids`, `tags` and `status`, over a payload that is total across every modelled field (`deny_unknown_fields` + `hash_payload_covers_every_modelled_field`). An empty bank is an ERROR, not a hash. |
/// | `cdcp-bank-v3` | v2's fields **plus** `kind`. The 804-item bank is `single-select` (G1). A kind flip changes what assemble will admit. |
///
/// Bumping this constant is a THREE-SITE change that must land in one commit —
/// this constant, `content.lock` `canonical`, and `scripts/gen_content_lock.py`
/// `CANONICAL`. A partial bump creates a third state and is worse than none.
/// `crates/cdcp_core/tests/bank_hash_domain_agreement.rs` keys on this constant
/// (never on a grep for the literal) and goes RED naming both sides.
pub const BANK_HASH_DOMAIN: &[u8] = b"cdcp-bank-v3\0";

/// The domain tag as a label, DERIVED from [`BANK_HASH_DOMAIN`] — the single
/// source the other two sites are checked against.
///
/// Anti-vacuous: an empty, non-NUL-terminated, non-UTF-8, or interior-NUL
/// domain is an **ERROR**, never a silent default. A domain that parsed to `""`
/// would make every hash comparable to every other hash, which is exactly the
/// confusion the tag exists to prevent.
pub fn bank_hash_domain_label() -> Result<&'static str, CoreError> {
    let raw = BANK_HASH_DOMAIN;
    let Some((&last, head)) = raw.split_last() else {
        return Err(CoreError::InvalidDomain(
            "BANK_HASH_DOMAIN is empty — a domain tag must name a definition".into(),
        ));
    };
    if last != 0 {
        return Err(CoreError::InvalidDomain(
            "BANK_HASH_DOMAIN must be NUL-terminated".into(),
        ));
    }
    if head.is_empty() {
        return Err(CoreError::InvalidDomain(
            "BANK_HASH_DOMAIN is only a NUL — the label is empty".into(),
        ));
    }
    if head.contains(&0) {
        return Err(CoreError::InvalidDomain(
            "BANK_HASH_DOMAIN has an interior NUL".into(),
        ));
    }
    std::str::from_utf8(head)
        .map_err(|e| CoreError::InvalidDomain(format!("BANK_HASH_DOMAIN is not UTF-8: {e}")))
}

pub const STUDY_PASS_CORRECT: u32 = 27;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("invalid choice letter: {0}")]
    InvalidChoice(String),
    #[error("json: {0}")]
    Json(String),
    #[error("bank hash domain: {0}")]
    InvalidDomain(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ChoiceLetter {
    A,
    B,
    C,
    D,
}

impl ChoiceLetter {
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        match s.trim() {
            "A" | "a" => Ok(Self::A),
            "B" | "b" => Ok(Self::B),
            "C" | "c" => Ok(Self::C),
            "D" | "d" => Ok(Self::D),
            other => Err(CoreError::InvalidChoice(other.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
        }
    }

    /// Deterministic wrong letter for fixtures (cycles A→B→C→D→A, never equals self).
    pub fn wrong_letter(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::C,
            Self::C => Self::D,
            Self::D => Self::A,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemResult {
    pub item_id: String,
    pub chosen: ChoiceLetter,
    pub correct: ChoiceLetter,
    pub is_correct: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleScore {
    pub module: u32,
    pub correct: u32,
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GradeReport {
    pub schema_version: u32,
    pub bank_hash: String,
    pub exam_id: String,
    pub seed: u64,
    pub item_results: Vec<ItemResult>,
    pub score_correct: u32,
    pub score_total: u32,
    pub by_module: Vec<ModuleScore>,
    pub weak_modules: Vec<u32>,
    pub passed_study_signal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnsweredItem {
    pub item_id: String,
    pub chosen: ChoiceLetter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExamAttempt {
    pub exam_id: String,
    pub seed: u64,
    pub bank_hash: String,
    pub answers: Vec<AnsweredItem>,
}

/// Compact JSON with BTreeMap ensuring sorted keys for maps.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CoreError> {
    // Serialize via Value then re-emit with sorted keys
    let v = serde_json::to_value(value).map_err(|e| CoreError::Json(e.to_string()))?;
    let sorted = sort_value(v);
    serde_json::to_vec(&sorted).map_err(|e| CoreError::Json(e.to_string()))
}

fn sort_value(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut bt = BTreeMap::new();
            for (k, val) in map {
                bt.insert(k, sort_value(val));
            }
            // serde_json::Map from BTreeMap keeps sorted iteration on serialize
            let mut out = serde_json::Map::new();
            for (k, val) in bt {
                out.insert(k, val);
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_value).collect())
        }
        other => other,
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

pub fn digest_report(report: &GradeReport) -> Result<String, CoreError> {
    let bytes = canonical_json(report)?;
    Ok(sha256_hex(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_order_stability() {
        let a = json!({"z": 1, "a": {"y": 2, "b": 3}});
        let b = json!({"a": {"b": 3, "y": 2}, "z": 1});
        let ca = sort_value(a);
        let cb = sort_value(b);
        assert_eq!(
            serde_json::to_vec(&ca).unwrap(),
            serde_json::to_vec(&cb).unwrap()
        );
    }

    #[test]
    fn report_digest_floor_zero() {
        let r = GradeReport {
            schema_version: 1,
            bank_hash: "abc".into(),
            exam_id: "mock40".into(),
            seed: 42,
            item_results: vec![],
            score_correct: 0,
            score_total: 0,
            by_module: vec![],
            weak_modules: vec![],
            passed_study_signal: false,
        };
        let d1 = digest_report(&r).unwrap();
        let d2 = digest_report(&r).unwrap();
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 64);
    }

    #[test]
    fn choice_parse() {
        assert_eq!(ChoiceLetter::parse("B").unwrap(), ChoiceLetter::B);
        assert!(ChoiceLetter::parse("E").is_err());
    }

    // --- proptest (bd-334): digest stability + ChoiceLetter roundtrip ---

    use proptest::prelude::*;

    fn arb_choice() -> impl Strategy<Value = ChoiceLetter> {
        prop_oneof![
            Just(ChoiceLetter::A),
            Just(ChoiceLetter::B),
            Just(ChoiceLetter::C),
            Just(ChoiceLetter::D),
        ]
    }

    fn arb_item_result() -> impl Strategy<Value = ItemResult> {
        (
            "[a-z0-9_-]{1,16}",
            arb_choice(),
            arb_choice(),
            any::<bool>(),
        )
            .prop_map(|(item_id, chosen, correct, is_correct)| ItemResult {
                item_id,
                chosen,
                correct,
                is_correct,
            })
    }

    fn arb_module_score() -> impl Strategy<Value = ModuleScore> {
        (1u32..20, 0u32..40, 1u32..40).prop_map(|(module, correct, total)| ModuleScore {
            module,
            correct: correct.min(total),
            total,
        })
    }

    fn arb_grade_report() -> impl Strategy<Value = GradeReport> {
        (
            0u32..4,
            "[a-f0-9]{0,64}",
            "[a-z0-9_-]{0,32}",
            any::<u64>(),
            proptest::collection::vec(arb_item_result(), 0..8),
            0u32..40,
            0u32..40,
            proptest::collection::vec(arb_module_score(), 0..6),
            proptest::collection::vec(1u32..20, 0..6),
            any::<bool>(),
        )
            .prop_map(
                |(
                    schema_version,
                    bank_hash,
                    exam_id,
                    seed,
                    item_results,
                    score_correct,
                    score_total,
                    by_module,
                    mut weak_modules,
                    passed_study_signal,
                )| {
                    weak_modules.sort_unstable();
                    weak_modules.dedup();
                    GradeReport {
                        schema_version,
                        bank_hash,
                        exam_id,
                        seed,
                        item_results,
                        score_correct,
                        score_total,
                        by_module,
                        weak_modules,
                        passed_study_signal,
                    }
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Equal field values ⇒ identical digest (and double-run floor 0).
        #[test]
        fn equal_reports_same_digest(r in arb_grade_report()) {
            let r2 = r.clone();
            assert_eq!(r, r2);
            let d1 = digest_report(&r).expect("digest");
            let d2 = digest_report(&r2).expect("digest");
            assert_eq!(d1, d2);
            assert_eq!(d1, digest_report(&r).expect("digest again"));
            assert_eq!(d1.len(), 64);
        }

        /// ChoiceLetter::parse(as_str) is identity for all letters.
        #[test]
        fn choice_letter_as_str_roundtrip(letter in arb_choice()) {
            assert_eq!(ChoiceLetter::parse(letter.as_str()).unwrap(), letter);
        }
    }
}
