//! Load bank items and compute bank_hash (OQ-03).
//!
//! Item schema floors match `scripts/verify_bank.py` (see docs/TESTING.md parity table).
#![forbid(unsafe_code)]

use cdcp_core::{canonical_json, sha256_hex, ChoiceLetter, BANK_HASH_DOMAIN};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Allowed `correct` letters — same as verify_bank.py `ALLOWED_CORRECT` (uppercase only).
const ALLOWED_CORRECT: &[&str] = &["A", "B", "C", "D"];

/// Bloom taxonomy — same as verify_bank.py `ALLOWED_BLOOM`.
const ALLOWED_BLOOM: &[&str] = &[
    "remember",
    "understand",
    "apply",
    "analyze",
    "evaluate",
    "create",
];

/// Default quantity_evidence set — same as verify_bank.py / fact_policy.toml.
const ALLOWED_QUANTITY_EVIDENCE: &[&str] = &[
    "free_url",
    "licensed_note",
    "qualitative_only",
    "exam_form_public",
];

/// Minimum explanation length — same as verify_bank.py.
const MIN_EXPLANATION_LEN: usize = 12;

#[derive(Debug, Error)]
pub enum BankError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml: {0}")]
    Toml(String),
    #[error("json: {0}")]
    Json(String),
    #[error("empty bank")]
    Empty,
    #[error("item {0}: {1}")]
    Item(String, String),
    #[error("core: {0}")]
    Core(#[from] cdcp_core::CoreError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankItem {
    pub id: String,
    pub module: u32,
    pub stem: String,
    pub choices: Vec<String>,
    pub correct: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub topic_ids: Vec<String>,
    #[serde(default)]
    pub bloom: String,
    #[serde(default)]
    pub source_class: String,
    #[serde(default)]
    pub quantity_evidence: String,
}

impl BankItem {
    pub fn correct_letter(&self) -> Result<ChoiceLetter, BankError> {
        ChoiceLetter::parse(&self.correct)
            .map_err(|e| BankError::Item(self.id.clone(), e.to_string()))
    }

    /// Per-item schema floors aligned with `scripts/verify_bank.py`.
    pub fn validate(&self) -> Result<(), BankError> {
        let label = if self.id.is_empty() {
            "<missing-id>".to_string()
        } else {
            self.id.clone()
        };

        if self.id.trim().is_empty() {
            return Err(BankError::Item(label, "missing id".into()));
        }

        if self.stem.trim().is_empty() {
            return Err(BankError::Item(self.id.clone(), "empty stem".into()));
        }

        if self.choices.len() != 4 {
            return Err(BankError::Item(
                self.id.clone(),
                "choices must be length 4".into(),
            ));
        }
        if self.choices.iter().any(|c| c.trim().is_empty()) {
            return Err(BankError::Item(self.id.clone(), "empty choice text".into()));
        }

        // Uppercase A–D only (verify_bank ALLOWED_CORRECT); reject lowercase.
        if !ALLOWED_CORRECT.contains(&self.correct.as_str()) {
            return Err(BankError::Item(
                self.id.clone(),
                format!("correct must be A-D, got {:?}", self.correct),
            ));
        }
        // Keep ChoiceLetter parse as a second check (should always pass for A–D).
        let _ = self.correct_letter()?;

        if self.explanation.trim().len() < MIN_EXPLANATION_LEN {
            return Err(BankError::Item(
                self.id.clone(),
                "explanation too short".into(),
            ));
        }

        if self.topic_ids.is_empty() {
            return Err(BankError::Item(
                self.id.clone(),
                "topic_ids required".into(),
            ));
        }

        if self.source_class != "original" {
            return Err(BankError::Item(
                self.id.clone(),
                format!("source_class must be original, got {:?}", self.source_class),
            ));
        }

        if !ALLOWED_QUANTITY_EVIDENCE.contains(&self.quantity_evidence.as_str()) {
            return Err(BankError::Item(
                self.id.clone(),
                format!("bad quantity_evidence {:?}", self.quantity_evidence),
            ));
        }

        if !ALLOWED_BLOOM.contains(&self.bloom.as_str()) {
            return Err(BankError::Item(
                self.id.clone(),
                format!("bad bloom {:?}", self.bloom),
            ));
        }

        Ok(())
    }

    /// Canonical fields for hashing (stable subset).
    pub fn hash_payload(&self) -> BTreeMap<String, serde_json::Value> {
        let mut m = BTreeMap::new();
        m.insert("id".into(), serde_json::json!(self.id));
        m.insert("module".into(), serde_json::json!(self.module));
        m.insert("stem".into(), serde_json::json!(self.stem));
        m.insert("choices".into(), serde_json::json!(self.choices));
        m.insert("correct".into(), serde_json::json!(self.correct));
        m.insert("explanation".into(), serde_json::json!(self.explanation));
        let mut topics = self.topic_ids.clone();
        topics.sort();
        m.insert("topic_ids".into(), serde_json::json!(topics));
        m.insert("bloom".into(), serde_json::json!(self.bloom));
        m.insert("source_class".into(), serde_json::json!(self.source_class));
        m.insert(
            "quantity_evidence".into(),
            serde_json::json!(self.quantity_evidence),
        );
        m
    }
}

#[derive(Debug, Clone)]
pub struct Bank {
    pub items: BTreeMap<String, BankItem>,
    pub bank_hash: String,
}

impl Bank {
    pub fn load_dir(dir: &Path) -> Result<Self, BankError> {
        let mut items: BTreeMap<String, BankItem> = BTreeMap::new();
        if !dir.is_dir() {
            return Err(BankError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("not a directory: {}", dir.display()),
            )));
        }
        // Fail closed on read_dir entry errors (do not filter_map e.ok()).
        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) == Some("toml") {
                paths.push(path);
            }
        }
        paths.sort();
        for path in paths {
            let text = fs::read_to_string(&path)?;
            let item: BankItem = toml::from_str(&text)
                .map_err(|e| BankError::Toml(format!("{}: {e}", path.display())))?;
            item.validate()?;
            if items.insert(item.id.clone(), item.clone()).is_some() {
                return Err(BankError::Item(item.id, "duplicate id".into()));
            }
        }
        if items.is_empty() {
            return Err(BankError::Empty);
        }
        let bank_hash = compute_bank_hash(&items)?;
        Ok(Self { items, bank_hash })
    }

    pub fn get(&self, id: &str) -> Option<&BankItem> {
        self.items.get(id)
    }

    /// Build a bank from an in-memory item list (WASM / JSON dual-path).
    /// Validates each item, rejects duplicates and empty sets, recomputes `bank_hash`.
    pub fn from_items(items: impl IntoIterator<Item = BankItem>) -> Result<Self, BankError> {
        let mut map: BTreeMap<String, BankItem> = BTreeMap::new();
        for item in items {
            item.validate()?;
            if map.insert(item.id.clone(), item.clone()).is_some() {
                return Err(BankError::Item(item.id, "duplicate id".into()));
            }
        }
        if map.is_empty() {
            return Err(BankError::Empty);
        }
        let bank_hash = compute_bank_hash(&map)?;
        Ok(Self {
            items: map,
            bank_hash,
        })
    }

    /// Deserialize bank items from JSON (array of items, or `{"items":[...]}`).
    pub fn from_json_str(json: &str) -> Result<Self, BankError> {
        #[derive(Deserialize)]
        struct Wrapper {
            items: Vec<BankItem>,
        }
        let items: Vec<BankItem> = match serde_json::from_str::<Wrapper>(json) {
            Ok(w) => w.items,
            Err(_) => serde_json::from_str(json)
                .map_err(|e| BankError::Json(format!("bank json: {e}")))?,
        };
        Self::from_items(items)
    }

    /// Export items as a JSON array (stable BTreeMap key order).
    pub fn to_json_items(&self) -> Result<String, BankError> {
        let list: Vec<&BankItem> = self.items.values().collect();
        serde_json::to_string(&list).map_err(|e| BankError::Json(format!("bank json encode: {e}")))
    }
}

pub fn compute_bank_hash(items: &BTreeMap<String, BankItem>) -> Result<String, BankError> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(BANK_HASH_DOMAIN);
    // BTreeMap values() iteration is sorted by key (item id)
    for item in items.values() {
        let payload = item.hash_payload();
        let bytes = canonical_json(&payload)?;
        buf.extend_from_slice(&bytes);
        buf.push(0);
    }
    Ok(sha256_hex(&buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn valid_item_toml(id: &str, correct: &str) -> String {
        format!(
            r#"
id = "{id}"
module = 1
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "{correct}"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#
        )
    }

    fn write_item(dir: &Path, name: &str, body: &str) {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{body}").unwrap();
    }

    #[test]
    fn empty_bank_errors() {
        let dir = std::env::temp_dir().join(format!("cdcp-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Empty));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hash_changes_when_correct_flips() {
        let dir = std::env::temp_dir().join(format!("cdcp-hash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(&dir, "t.toml", &valid_item_toml("t1", "A"));
        let b1 = Bank::load_dir(&dir).unwrap();
        write_item(&dir, "t.toml", &valid_item_toml("t1", "B"));
        let b2 = Bank::load_dir(&dir).unwrap();
        assert_ne!(b1.bank_hash, b2.bank_hash);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_empty_stem() {
        let dir = std::env::temp_dir().join(format!("cdcp-stem-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "bad-stem"
module = 1
stem = "   "
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("empty stem")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_empty_choice() {
        let dir = std::env::temp_dir().join(format!("cdcp-choice-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "bad-choice"
module = 1
stem = "stem"
choices = ["a","","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("empty choice")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_lowercase_correct() {
        // verify_bank.py ALLOWED_CORRECT is uppercase-only
        let dir = std::env::temp_dir().join(format!("cdcp-lc-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(&dir, "t.toml", &valid_item_toml("t1", "a"));
        // valid_item_toml uses correct as-is; force lowercase
        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "a"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("correct must be A-D")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_bad_bloom_and_quantity_evidence() {
        let dir = std::env::temp_dir().join(format!("cdcp-bloom-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "stem text"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "not-a-level"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("bad bloom")));

        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "stem text"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "made_up"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("bad quantity_evidence")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_empty_topic_ids() {
        let dir = std::env::temp_dir().join(format!("cdcp-topics-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "stem text"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = []
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("topic_ids")));
        let _ = fs::remove_dir_all(&dir);
    }

    /// Integration: load the real repo bank/items (path via CARGO_MANIFEST_DIR).
    #[test]
    fn load_real_bank_items() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items");
        assert!(
            dir.is_dir(),
            "expected real bank at {} (run tests from repo checkout)",
            dir.display()
        );
        let bank = Bank::load_dir(&dir).expect("real bank should load under unified validation");
        assert!(
            bank.items.len() >= 40,
            "expected a real pool, got {}",
            bank.items.len()
        );
        assert_eq!(bank.bank_hash.len(), 64);
        // Spot-check fields that verify_bank also requires
        for item in bank.items.values() {
            assert!(!item.stem.trim().is_empty());
            assert_eq!(item.choices.len(), 4);
            assert!(ALLOWED_CORRECT.contains(&item.correct.as_str()));
            assert!(ALLOWED_BLOOM.contains(&item.bloom.as_str()));
            assert!(ALLOWED_QUANTITY_EVIDENCE.contains(&item.quantity_evidence.as_str()));
            assert!(!item.topic_ids.is_empty());
            assert_eq!(item.source_class, "original");
        }
    }

    // --- proptest (bd-334): bank_hash independent of BTreeMap insert order ---

    use proptest::prelude::*;

    fn sample_item(id: &str, module: u32, correct: &str) -> BankItem {
        BankItem {
            id: id.to_string(),
            module,
            stem: format!("stem-{id}"),
            choices: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            correct: correct.to_string(),
            explanation: "because reasons here".into(),
            topic_ids: vec![format!("t-{id}")],
            bloom: "understand".into(),
            source_class: "original".into(),
            quantity_evidence: "qualitative_only".into(),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// compute_bank_hash depends only on sorted item ids/payloads, not insert order.
        #[test]
        fn bank_hash_independent_of_insert_order(
            mut ids in proptest::collection::vec("[a-z][a-z0-9]{0,7}", 1..12),
            seed in any::<u64>(),
        ) {
            ids.sort();
            ids.dedup();
            prop_assume!(!ids.is_empty());

            let letters = ["A", "B", "C", "D"];
            let items: Vec<BankItem> = ids
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    let correct = letters[(seed as usize + i) % 4];
                    sample_item(id, ((i as u32) % 14) + 1, correct)
                })
                .collect();

            let mut forward = BTreeMap::new();
            for it in &items {
                forward.insert(it.id.clone(), it.clone());
            }
            let mut reverse = BTreeMap::new();
            for it in items.iter().rev() {
                reverse.insert(it.id.clone(), it.clone());
            }
            // Third order: even then odd indices
            let mut interleaved = BTreeMap::new();
            for it in items.iter().step_by(2).chain(items.iter().skip(1).step_by(2)) {
                interleaved.insert(it.id.clone(), it.clone());
            }

            let h1 = compute_bank_hash(&forward).unwrap();
            let h2 = compute_bank_hash(&reverse).unwrap();
            let h3 = compute_bank_hash(&interleaved).unwrap();
            assert_eq!(h1, h2);
            assert_eq!(h1, h3);
            assert_eq!(h1.len(), 64);
        }
    }
}
