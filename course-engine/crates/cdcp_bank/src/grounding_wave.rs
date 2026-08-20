//! Grounding-wave regression detectors over the approved item pool.
//!
//! This is bank-product logic, not gate plumbing. The paired gate dispatcher
//! keeps the public cdcp_gate grounding-wave command stable while this module
//! owns the two fingerprints that the grounding wave made visible:
//! near-identical template stems with collapsed correct-answer text, and
//! document-index recall stems. Stem shape alone is deliberately insufficient:
//! the approved containment teaching family uses parallel stems while teaching
//! different answer propositions.
//!
//! A detector finding may be adjudicated only by a per-item, per-detector
//! exclusion in `registries/grounding_wave.toml`. Exclusions require a reason,
//! stale rows are errors, and an all-excluded scan reports that fact loudly.

use crate::near_duplicate::{self, Sim};
use crate::{Bank, BankItem};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

pub const NAME: &str = "grounding-wave";
pub const SUMMARY: &str = "detect template and recall-only stems in the approved pool";
const POLICY: &str = "registries/grounding_wave.toml";
const TEMPLATE_NAME: &str = "template-stem";
const RECALL_NAME: &str = "recall-only-stem";

/// The bank-product result mapped to process behavior by the gate dispatcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eval {
    Ok(String),
    Violation(Vec<String>),
    Error(String),
}

#[derive(Debug, Deserialize)]
struct Registry {
    detector: Vec<DetectorRow>,
    #[serde(default)]
    exclude: Vec<ExcludeRow>,
}

#[derive(Debug, Deserialize)]
struct DetectorRow {
    name: String,
    #[serde(default)]
    similarity_pct: Option<u32>,
    #[serde(default)]
    answer_similarity_pct: Option<u32>,
    #[serde(default)]
    min_peers: Option<usize>,
    #[serde(default)]
    required_tokens: Vec<String>,
    #[serde(default)]
    question_tokens: Vec<String>,
    #[serde(default)]
    action_tokens: Vec<String>,
    #[serde(default)]
    document_tokens: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExcludeRow {
    #[serde(default)]
    id: String,
    #[serde(default)]
    detector: String,
    #[serde(default)]
    reason: String,
}

#[derive(Debug)]
struct Policy {
    template_stem_similarity_pct: u32,
    template_answer_similarity_pct: u32,
    template_min_peers: usize,
    recall_required: BTreeSet<String>,
    recall_questions: BTreeSet<String>,
    recall_actions: BTreeSet<String>,
    recall_documents: BTreeSet<String>,
    exclusions: Vec<ExcludeRow>,
}

fn row<'a>(registry: &'a Registry, name: &str) -> Result<&'a DetectorRow, String> {
    let mut rows = registry.detector.iter().filter(|r| r.name == name);
    let first = rows
        .next()
        .ok_or_else(|| format!("{POLICY}: missing detector row {name:?}"))?;
    if rows.next().is_some() {
        return Err(format!("{POLICY}: duplicate detector row {name:?}"));
    }
    Ok(first)
}

fn words(values: &[String], field: &str) -> Result<BTreeSet<String>, String> {
    let out = values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if out.is_empty() {
        return Err(format!("{POLICY}: {field} must not be empty"));
    }
    Ok(out)
}

fn known_detector(name: &str) -> bool {
    matches!(name, TEMPLATE_NAME | RECALL_NAME)
}

fn exclusions(registry: &Registry) -> Result<Vec<ExcludeRow>, String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(registry.exclude.len());
    for (index, row) in registry.exclude.iter().enumerate() {
        let id = row.id.trim();
        if id.is_empty() {
            return Err(format!(
                "{POLICY}: [[exclude]] #{} is missing an id",
                index + 1
            ));
        }
        let detector = row.detector.trim();
        if detector.is_empty() {
            return Err(format!(
                "{POLICY}: [[exclude]] {id:?} is missing a detector"
            ));
        }
        if !known_detector(detector) {
            return Err(format!(
                "{POLICY}: [[exclude]] {id:?} names unknown detector {detector:?}"
            ));
        }
        let reason = row.reason.trim();
        if reason.is_empty() {
            return Err(format!(
                "{POLICY}: [[exclude]] {id:?} for {detector:?} has a missing or empty reason — SCHEMA ERROR, not permission"
            ));
        }
        let key = (id.to_string(), detector.to_string());
        if !seen.insert(key) {
            return Err(format!(
                "{POLICY}: duplicate [[exclude]] for id {id:?} and detector {detector:?}"
            ));
        }
        out.push(ExcludeRow {
            id: id.to_string(),
            detector: detector.to_string(),
            reason: reason.to_string(),
        });
    }
    Ok(out)
}

fn policy(root: &Path) -> Result<Policy, String> {
    let path = root.join(POLICY);
    let raw =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let registry: Registry =
        toml::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let template = row(&registry, TEMPLATE_NAME)?;
    let similarity = template
        .similarity_pct
        .ok_or_else(|| format!("{POLICY}: {TEMPLATE_NAME}.similarity_pct is required"))?;
    let answer_similarity = template
        .answer_similarity_pct
        .ok_or_else(|| format!("{POLICY}: {TEMPLATE_NAME}.answer_similarity_pct is required"))?;
    let min_peers = template
        .min_peers
        .ok_or_else(|| format!("{POLICY}: {TEMPLATE_NAME}.min_peers is required"))?;
    if similarity == 0 || similarity > 100 {
        return Err(format!(
            "{POLICY}: {TEMPLATE_NAME}.similarity_pct must be 1..=100"
        ));
    }
    if answer_similarity == 0 || answer_similarity > 100 {
        return Err(format!(
            "{POLICY}: {TEMPLATE_NAME}.answer_similarity_pct must be 1..=100"
        ));
    }
    if min_peers == 0 {
        return Err(format!(
            "{POLICY}: {TEMPLATE_NAME}.min_peers must be positive"
        ));
    }
    let recall = row(&registry, RECALL_NAME)?;
    Ok(Policy {
        template_stem_similarity_pct: similarity,
        template_answer_similarity_pct: answer_similarity,
        template_min_peers: min_peers,
        recall_required: words(&recall.required_tokens, "recall-only-stem.required_tokens")?,
        recall_questions: words(&recall.question_tokens, "recall-only-stem.question_tokens")?,
        recall_actions: words(&recall.action_tokens, "recall-only-stem.action_tokens")?,
        recall_documents: words(&recall.document_tokens, "recall-only-stem.document_tokens")?,
        exclusions: exclusions(&registry)?,
    })
}

fn correct_text(item: &BankItem) -> Result<&str, String> {
    let index = match item.correct.as_str() {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        other => return Err(format!("{}: correct key is not A-D: {other:?}", item.id)),
    };
    item.choices.get(index).map(String::as_str).ok_or_else(|| {
        format!(
            "{}: correct key {} has no corresponding choice",
            item.id, item.correct
        )
    })
}

fn template_ids(items: &[&BankItem], policy: &Policy) -> Result<Vec<String>, String> {
    let stems = items
        .iter()
        .map(|item| near_duplicate::tokens(&item.stem))
        .collect::<Vec<_>>();
    let answers = items
        .iter()
        .map(|item| correct_text(item).map(near_duplicate::tokens))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let peers = stems
                .iter()
                .enumerate()
                .filter(|(other, stem)| {
                    *other != index
                        && Sim::of(&stems[index], stem)
                            .at_least(policy.template_stem_similarity_pct)
                        && Sim::of(&answers[index], &answers[*other])
                            .at_least(policy.template_answer_similarity_pct)
                })
                .count();
            (peers >= policy.template_min_peers).then(|| item.id.clone())
        })
        .collect())
}

fn recall_ids(items: &[&BankItem], policy: &Policy) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| {
            let tokens = near_duplicate::tokens(&item.stem);
            let required = policy
                .recall_required
                .iter()
                .all(|word| tokens.contains(word));
            let question = policy
                .recall_questions
                .iter()
                .any(|word| tokens.contains(word));
            let action = policy
                .recall_actions
                .iter()
                .any(|word| tokens.contains(word));
            let document = policy
                .recall_documents
                .iter()
                .any(|word| tokens.contains(word));
            (required && question && action && document).then(|| item.id.clone())
        })
        .collect()
}

fn ids(values: &[String]) -> String {
    values.join(",")
}

fn filtered_ids(flagged: &[String], detector: &str, exclusions: &[ExcludeRow]) -> Vec<String> {
    flagged
        .iter()
        .filter(|id| {
            !exclusions
                .iter()
                .any(|row| row.detector == detector && row.id.as_str() == id.as_str())
        })
        .cloned()
        .collect()
}

fn exclusion_receipt(exclusions: &[ExcludeRow]) -> String {
    exclusions
        .iter()
        .map(|row| format!("{}:{} reason={:?}", row.detector, row.id, row.reason))
        .collect::<Vec<_>>()
        .join("; ")
}

fn reject_stale_exclusions(
    exclusions: &[ExcludeRow],
    template: &[String],
    recall: &[String],
) -> Result<(), String> {
    for row in exclusions {
        let flagged = match row.detector.as_str() {
            TEMPLATE_NAME => template,
            RECALL_NAME => recall,
            _ => unreachable!("validated detector name"),
        };
        if !flagged.iter().any(|id| id == &row.id) {
            return Err(format!(
                "{POLICY}: stale [[exclude]] id {:?} is not currently flagged by detector {:?} — remove the exception or fix the item",
                row.id, row.detector
            ));
        }
    }
    Ok(())
}

/// Evaluate both grounding-wave fingerprints over approved items.
pub fn evaluate(root: &Path) -> Eval {
    let policy = match policy(root) {
        Ok(policy) => policy,
        Err(message) => return Eval::Error(message),
    };
    let bank = match Bank::load_dir(&root.join("bank/items")) {
        Ok(bank) => bank,
        Err(error) => return Eval::Error(format!("load bank/items: {error}")),
    };
    let approved = bank
        .items
        .values()
        .filter(|item| item.is_approved())
        .collect::<Vec<_>>();
    if approved.is_empty() {
        return Eval::Error("zero approved items (vacuous grounding-wave scan)".to_string());
    }
    let template = match template_ids(&approved, &policy) {
        Ok(ids) => ids,
        Err(message) => return Eval::Error(message),
    };
    let recall = recall_ids(&approved, &policy);
    if let Err(message) = reject_stale_exclusions(&policy.exclusions, &template, &recall) {
        return Eval::Error(message);
    }
    let template_unexcepted = filtered_ids(&template, TEMPLATE_NAME, &policy.exclusions);
    let recall_unexcepted = filtered_ids(&recall, RECALL_NAME, &policy.exclusions);
    let report = format!(
        "approved={}; template-stem(stem>={}% answer>={}% peers>={})={} ids=[{}] unexcepted={} ids=[{}]; recall-only-stem={} ids=[{}] unexcepted={} ids=[{}]; adjudicated-exceptions={} [{}]",
        approved.len(),
        policy.template_stem_similarity_pct,
        policy.template_answer_similarity_pct,
        policy.template_min_peers,
        template.len(),
        ids(&template),
        template_unexcepted.len(),
        ids(&template_unexcepted),
        recall.len(),
        ids(&recall),
        recall_unexcepted.len(),
        ids(&recall_unexcepted),
        policy.exclusions.len(),
        exclusion_receipt(&policy.exclusions),
    );
    if template_unexcepted.is_empty() && recall_unexcepted.is_empty() {
        let suffix = if template.is_empty() && recall.is_empty() {
            "no detector findings"
        } else {
            "all currently flagged ids are adjudicated exceptions"
        };
        Eval::Ok(format!("{NAME}: PASS: {report}; {suffix}"))
    } else {
        Eval::Violation(vec![report])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/grounding_wave")
            .join(name)
    }

    fn copy_tree(source: &Path, target: &Path) {
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let from = entry.path();
            let to = target.join(entry.file_name());
            // ABSENT-OK: fixture copier; a non-directory entry is copied as a
            // file below, and no detector verdict is skipped by this branch.
            if from.is_dir() {
                copy_tree(&from, &to);
            } else {
                fs::copy(from, to).unwrap();
            }
        }
    }

    fn fixture_with_policy(name: &str, suffix: &str) -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        copy_tree(&fixture(name), temp.path());
        if !suffix.is_empty() {
            let policy_path = temp.path().join(POLICY);
            let mut policy = fs::read_to_string(&policy_path).unwrap();
            policy.push_str(suffix);
            fs::write(policy_path, policy).unwrap();
        }
        let root = temp.path().to_path_buf();
        (temp, root)
    }

    fn preserved_peak_fixture() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/damaged_corpus_2026_08_18");
        let item_dir = temp.path().join("bank/items");
        let policy_dir = temp.path().join("registries");
        fs::create_dir_all(&item_dir).unwrap();
        fs::create_dir_all(&policy_dir).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            if entry.path().extension().and_then(|value| value.to_str()) == Some("toml") {
                fs::copy(entry.path(), item_dir.join(entry.file_name())).unwrap();
            }
        }
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registries/grounding_wave.toml"),
            policy_dir.join("grounding_wave.toml"),
        )
        .unwrap();
        let root = temp.path().to_path_buf();
        (temp, root)
    }

    const REASON: &str =
        "Reviewed containment teaching family; this item is pre-wave and substantively sound.";

    #[test]
    fn known_bad_table_of_contents_fixture_is_red() {
        match evaluate(&fixture("bad")) {
            Eval::Violation(items) => {
                assert!(items[0].contains("template-stem"));
                assert!(items[0].contains("toc-a,toc-b,toc-c"));
                assert!(items[0].contains("recall-only-stem"));
            }
            other => panic!("known-bad fixture did not go RED: {other:?}"),
        }
    }

    #[test]
    fn known_good_substantive_fixture_is_green() {
        match evaluate(&fixture("good")) {
            Eval::Ok(text) => assert!(text.contains("approved=2")),
            other => panic!("substantive fixture did not pass: {other:?}"),
        }
    }

    #[test]
    fn parallel_containment_family_is_green_without_an_exception() {
        match evaluate(&fixture("containment")) {
            Eval::Ok(text) => {
                assert!(text.contains("template-stem("));
                assert!(text.contains(")=0 ids=[]"));
                assert!(text.contains("adjudicated-exceptions=0"));
            }
            other => panic!("containment family was misclassified: {other:?}"),
        }
    }

    #[test]
    fn preserved_peak_damage_fixture_is_red() {
        let (_temp, root) = preserved_peak_fixture();
        match evaluate(&root) {
            Eval::Violation(items) => {
                assert!(items[0].contains("template-stem("));
                assert!(items[0].contains("bank-m15-q149"));
                assert!(items[0].contains("m15-q212"));
                assert!(items[0].contains("recall-only-stem"));
            }
            other => panic!("preserved peak corpus did not go RED: {other:?}"),
        }
    }

    #[test]
    fn empty_exception_reason_is_a_schema_error() {
        let (_temp, root) = fixture_with_policy(
            "bad",
            "\n[[exclude]]\nid = \"toc-a\"\ndetector = \"template-stem\"\nreason = \"\"\n",
        );
        match evaluate(&root) {
            Eval::Error(message) => {
                assert!(message.contains("missing or empty reason"));
                assert!(message.contains("SCHEMA ERROR"));
            }
            other => panic!("empty reason was not a schema error: {other:?}"),
        }
    }

    #[test]
    fn missing_exception_reason_is_a_schema_error() {
        let (_temp, root) = fixture_with_policy(
            "bad",
            "\n[[exclude]]\nid = \"toc-a\"\ndetector = \"template-stem\"\n",
        );
        match evaluate(&root) {
            Eval::Error(message) => {
                assert!(message.contains("missing or empty reason"));
                assert!(message.contains("SCHEMA ERROR"));
            }
            other => panic!("missing reason was not a schema error: {other:?}"),
        }
    }

    #[test]
    fn stale_exception_is_an_error() {
        let (_temp, root) = fixture_with_policy(
            "bad",
            &format!(
                "\n[[exclude]]\nid = \"not-flagged\"\ndetector = \"template-stem\"\nreason = {REASON:?}\n"
            ),
        );
        match evaluate(&root) {
            Eval::Error(message) => assert!(message.contains("stale [[exclude]]")),
            other => panic!("stale exception was not an error: {other:?}"),
        }
    }

    #[test]
    fn an_exception_is_scoped_to_its_detector() {
        let (_temp, root) = fixture_with_policy(
            "bad",
            &format!(
                "\n[[exclude]]\nid = \"toc-a\"\ndetector = \"template-stem\"\nreason = {REASON:?}\n"
            ),
        );
        match evaluate(&root) {
            Eval::Violation(items) => {
                assert!(items[0].contains("adjudicated-exceptions=1"));
                assert!(items[0].contains("recall-only-stem=3"));
                assert!(items[0].contains("unexcepted=3 ids=[toc-a,toc-b,toc-c]"));
            }
            other => panic!("detector-scoped exception silenced too much: {other:?}"),
        }
    }

    #[test]
    fn planted_toc_items_remain_red_when_another_item_is_excepted() {
        let (_temp, root) = fixture_with_policy(
            "bad",
            &format!(
                "\n[[exclude]]\nid = \"toc-a\"\ndetector = \"template-stem\"\nreason = {REASON:?}\n"
            ),
        );
        match evaluate(&root) {
            Eval::Violation(items) => {
                assert!(items[0].contains("unexcepted=2 ids=[toc-b,toc-c]"));
                assert!(items[0].contains("recall-only-stem"));
            }
            other => panic!("one exception blanketed the planted TOC fixture: {other:?}"),
        }
    }

    #[test]
    fn all_flagged_items_excepted_is_a_loud_pass() {
        let mut suffix = String::new();
        for detector in [TEMPLATE_NAME, RECALL_NAME] {
            for id in ["toc-a", "toc-b", "toc-c"] {
                suffix.push_str(&format!(
                    "\n[[exclude]]\nid = {id:?}\ndetector = {detector:?}\nreason = {REASON:?}\n"
                ));
            }
        }
        let (_temp, root) = fixture_with_policy("bad", &suffix);
        match evaluate(&root) {
            Eval::Ok(text) => {
                assert!(text.contains("adjudicated-exceptions=6"));
                assert!(text.contains("all currently flagged ids are adjudicated exceptions"));
                assert!(text.contains("reason="));
            }
            other => panic!("all-excepted scan was not a loud pass: {other:?}"),
        }
    }

    #[test]
    fn zero_approved_items_is_an_error() {
        match evaluate(&fixture("empty")) {
            Eval::Error(message) => assert!(message.contains("zero approved items")),
            other => panic!("empty approved pool was not an ERROR: {other:?}"),
        }
    }
}
