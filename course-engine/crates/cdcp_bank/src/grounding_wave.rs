//! Grounding-wave regression detectors over the approved item pool.
//!
//! This is bank-product logic, not gate plumbing. The paired gate dispatcher
//! keeps the public cdcp_gate grounding-wave command stable while this module
//! owns the two fingerprints that the grounding wave made visible:
//! near-identical template stems and document-index recall stems.

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
}

#[derive(Debug, Deserialize)]
struct DetectorRow {
    name: String,
    #[serde(default)]
    similarity_pct: Option<u32>,
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

#[derive(Debug)]
struct Policy {
    template_similarity_pct: u32,
    template_min_peers: usize,
    recall_required: BTreeSet<String>,
    recall_questions: BTreeSet<String>,
    recall_actions: BTreeSet<String>,
    recall_documents: BTreeSet<String>,
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
    let min_peers = template
        .min_peers
        .ok_or_else(|| format!("{POLICY}: {TEMPLATE_NAME}.min_peers is required"))?;
    if similarity == 0 || similarity > 100 {
        return Err(format!(
            "{POLICY}: {TEMPLATE_NAME}.similarity_pct must be 1..=100"
        ));
    }
    if min_peers == 0 {
        return Err(format!(
            "{POLICY}: {TEMPLATE_NAME}.min_peers must be positive"
        ));
    }
    let recall = row(&registry, RECALL_NAME)?;
    Ok(Policy {
        template_similarity_pct: similarity,
        template_min_peers: min_peers,
        recall_required: words(&recall.required_tokens, "recall-only-stem.required_tokens")?,
        recall_questions: words(&recall.question_tokens, "recall-only-stem.question_tokens")?,
        recall_actions: words(&recall.action_tokens, "recall-only-stem.action_tokens")?,
        recall_documents: words(&recall.document_tokens, "recall-only-stem.document_tokens")?,
    })
}

fn template_ids(items: &[&BankItem], policy: &Policy) -> Vec<String> {
    let stems = items
        .iter()
        .map(|item| near_duplicate::tokens(&item.stem))
        .collect::<Vec<_>>();
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let peers = stems
                .iter()
                .enumerate()
                .filter(|(other, stem)| {
                    *other != index
                        && Sim::of(&stems[index], stem).at_least(policy.template_similarity_pct)
                })
                .count();
            (peers >= policy.template_min_peers).then(|| item.id.clone())
        })
        .collect()
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
    let template = template_ids(&approved, &policy);
    let recall = recall_ids(&approved, &policy);
    let report = format!(
        "approved={}; template-stem(similarity>={}% peers>={})={} ids=[{}]; recall-only-stem={} ids=[{}]",
        approved.len(),
        policy.template_similarity_pct,
        policy.template_min_peers,
        template.len(),
        ids(&template),
        recall.len(),
        ids(&recall),
    );
    if template.is_empty() && recall.is_empty() {
        Eval::Ok(format!("{NAME}: PASS: {report}"))
    } else {
        Eval::Violation(vec![report])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/grounding_wave")
            .join(name)
    }

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
    fn zero_approved_items_is_an_error() {
        match evaluate(&fixture("empty")) {
            Eval::Error(message) => assert!(message.contains("zero approved items")),
            other => panic!("empty approved pool was not an ERROR: {other:?}"),
        }
    }
}
