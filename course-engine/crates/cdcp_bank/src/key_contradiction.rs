//! Cross-item contradiction floor for the approved bank.
//!
//! This gate is deliberately narrow. It clusters approved items by their
//! `topic_ids`, then compares only two mechanically visible contradiction
//! shapes: the same quantity/threshold with different numeric values, and
//! answer text that differs by an explicit negation. It does not infer the
//! truth of either proposition from the bank.
//!
//! a bank that does not contradict itself can still be uniformly wrong.
//! This raises a floor; it does not establish correctness.
//!
//! Thresholds and the accepted numeric units live in
//! `registries/key_contradiction.toml`. This module owns product logic; the
//! `cdcp_gate` command is only a dispatcher.

use crate::near_duplicate::{self, Sim};
use crate::{Bank, BankItem};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

pub const NAME: &str = "key-contradiction";
pub const SUMMARY: &str = "detect narrow cross-item numeric and negation contradictions";
const POLICY: &str = "registries/key_contradiction.toml";
const NUMERIC_NAME: &str = "numeric";
const NEGATION_NAME: &str = "explicit-negation";

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
    stem_similarity_pct: Option<u32>,
    #[serde(default)]
    anchor_similarity_pct: Option<u32>,
    #[serde(default)]
    answer_similarity_pct: Option<u32>,
    #[serde(default)]
    units: Vec<String>,
    #[serde(default)]
    negation_tokens: Vec<String>,
}

#[derive(Debug)]
struct Policy {
    numeric_stem_similarity_pct: u32,
    numeric_anchor_similarity_pct: u32,
    numeric_units: BTreeSet<String>,
    negation_stem_similarity_pct: u32,
    negation_answer_similarity_pct: u32,
    negation_tokens: BTreeSet<String>,
}

fn detector<'a>(registry: &'a Registry, name: &str) -> Result<&'a DetectorRow, String> {
    let mut rows = registry.detector.iter().filter(|row| row.name == name);
    let row = rows
        .next()
        .ok_or_else(|| format!("{POLICY}: missing detector row {name:?}"))?;
    if rows.next().is_some() {
        return Err(format!("{POLICY}: duplicate detector row {name:?}"));
    }
    Ok(row)
}

fn percentage(value: Option<u32>, field: &str) -> Result<u32, String> {
    let value = value.ok_or_else(|| format!("{POLICY}: {field} is required"))?;
    if !(1..=100).contains(&value) {
        return Err(format!("{POLICY}: {field} must be 1..=100"));
    }
    Ok(value)
}

fn words(values: &[String], field: &str) -> Result<BTreeSet<String>, String> {
    let out = values
        .iter()
        .flat_map(|value| near_duplicate::tokens(value))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if out.is_empty() {
        return Err(format!("{POLICY}: {field} must not be empty"));
    }
    Ok(out)
}

fn policy(root: &Path) -> Result<Policy, String> {
    let path = root.join(POLICY);
    let raw = std::fs::read_to_string(&path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let registry: Registry =
        toml::from_str(&raw).map_err(|error| format!("parse {}: {error}", path.display()))?;
    if registry.detector.is_empty() {
        return Err(format!("{POLICY}: zero detector rows is an ERROR"));
    }
    let numeric = detector(&registry, NUMERIC_NAME)?;
    let negation = detector(&registry, NEGATION_NAME)?;
    let units = words(&numeric.units, "numeric.units")?;
    let negation_tokens = words(
        &negation.negation_tokens,
        "explicit-negation.negation_tokens",
    )?;
    Ok(Policy {
        numeric_stem_similarity_pct: percentage(
            numeric.stem_similarity_pct,
            "numeric.stem_similarity_pct",
        )?,
        numeric_anchor_similarity_pct: percentage(
            numeric.anchor_similarity_pct,
            "numeric.anchor_similarity_pct",
        )?,
        numeric_units: units,
        negation_stem_similarity_pct: percentage(
            negation.stem_similarity_pct,
            "explicit-negation.stem_similarity_pct",
        )?,
        negation_answer_similarity_pct: percentage(
            negation.answer_similarity_pct,
            "explicit-negation.answer_similarity_pct",
        )?,
        negation_tokens,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NumericClaim {
    value: String,
    unit: String,
    anchor: BTreeSet<String>,
}

fn is_digits(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit())
}

fn inline_number_unit(token: &str, units: &BTreeSet<String>) -> Option<(String, String)> {
    let split = token
        .char_indices()
        .find(|(_, character)| !character.is_ascii_digit())
        .map(|(index, _)| index)?;
    let (value, unit) = token.split_at(split);
    units
        .contains(unit)
        .then(|| (value.to_string(), unit.to_string()))
}

fn ordered_tokens(text: &str) -> Vec<String> {
    near_duplicate::normalize(text)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn numeric_claims(text: &str, units: &BTreeSet<String>) -> Vec<NumericClaim> {
    let tokens = ordered_tokens(text);
    let mut claims = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let (value, unit, end) =
            if let Some((value, unit)) = inline_number_unit(&tokens[index], units) {
                (value, unit, index + 1)
            } else if is_digits(&tokens[index])
                && index + 2 < tokens.len()
                && is_digits(&tokens[index + 1])
                && tokens[index].len() == 1
                && units.contains(&tokens[index + 2])
            {
                (
                    format!("{}{}", tokens[index], tokens[index + 1]),
                    tokens[index + 2].clone(),
                    index + 3,
                )
            } else if is_digits(&tokens[index])
                && index + 1 < tokens.len()
                && units.contains(&tokens[index + 1])
            {
                (tokens[index].clone(), tokens[index + 1].clone(), index + 2)
            } else {
                index += 1;
                continue;
            };

        let start = index.saturating_sub(5);
        let anchor = tokens[start..index]
            .iter()
            .filter(|token| !is_digits(token) && !units.contains(*token))
            .cloned()
            .collect::<BTreeSet<_>>();
        if !anchor.is_empty() {
            claims.push(NumericClaim {
                value,
                unit,
                anchor,
            });
        }
        index = end;
    }
    claims
}

fn key_text(item: &BankItem) -> Result<&str, String> {
    let index = match item.correct.as_str() {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        other => return Err(format!("{}: correct key is not A-D: {other:?}", item.id)),
    };
    item.choices
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{}: correct key has no corresponding choice", item.id))
}

struct ItemView<'a> {
    item: &'a BankItem,
    topics: BTreeSet<String>,
    stem: BTreeSet<String>,
    answer: BTreeSet<String>,
    answer_without_negation: BTreeSet<String>,
    numeric: Vec<NumericClaim>,
}

fn view<'a>(item: &'a BankItem, policy: &Policy) -> Result<ItemView<'a>, String> {
    let answer = key_text(item)?;
    let answer_tokens = near_duplicate::tokens(answer);
    let answer_without_negation = answer_tokens
        .iter()
        .filter(|token| !policy.negation_tokens.contains(*token))
        .cloned()
        .collect();
    Ok(ItemView {
        item,
        topics: item.topic_ids.iter().cloned().collect(),
        stem: near_duplicate::tokens(&item.stem),
        answer: answer_tokens,
        answer_without_negation,
        numeric: numeric_claims(answer, &policy.numeric_units),
    })
}

fn shared_topics(left: &ItemView<'_>, right: &ItemView<'_>) -> Vec<String> {
    left.topics.intersection(&right.topics).cloned().collect()
}

fn has_negation(answer: &BTreeSet<String>, policy: &Policy) -> bool {
    answer
        .iter()
        .any(|token| policy.negation_tokens.contains(token))
}

fn numeric_conflict(
    left: &ItemView<'_>,
    right: &ItemView<'_>,
    policy: &Policy,
    stem_similarity: Sim,
) -> Option<(NumericClaim, NumericClaim, Sim)> {
    if !stem_similarity.at_least(policy.numeric_stem_similarity_pct) {
        return None;
    }
    left.numeric.iter().find_map(|left_claim| {
        right.numeric.iter().find_map(|right_claim| {
            let anchor_similarity = Sim::of(&left_claim.anchor, &right_claim.anchor);
            (left_claim.unit == right_claim.unit
                && left_claim.value != right_claim.value
                && anchor_similarity.at_least(policy.numeric_anchor_similarity_pct))
            .then(|| (left_claim.clone(), right_claim.clone(), anchor_similarity))
        })
    })
}

fn negation_conflict(
    left: &ItemView<'_>,
    right: &ItemView<'_>,
    policy: &Policy,
    stem_similarity: Sim,
) -> bool {
    stem_similarity.at_least(policy.negation_stem_similarity_pct)
        && has_negation(&left.answer, policy) != has_negation(&right.answer, policy)
        && Sim::of(
            &left.answer_without_negation,
            &right.answer_without_negation,
        )
        .at_least(policy.negation_answer_similarity_pct)
}

/// Evaluate the approved pool for bank-internal contradiction evidence.
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
        .filter(|item| item.is_approved() && item.kind.is_letter_form())
        .map(|item| view(item, &policy))
        .collect::<Result<Vec<_>, _>>();
    let approved = match approved {
        Ok(approved) => approved,
        Err(message) => return Eval::Error(message),
    };
    if approved.is_empty() {
        return Eval::Error(
            "zero approved single-select items (vacuous contradiction scan)".into(),
        );
    }
    if approved.len() < 2 {
        return Eval::Error(
            "fewer than two approved single-select items (zero comparisons)".into(),
        );
    }

    let mut comparisons = 0usize;
    let mut numeric_count = 0usize;
    let mut negation_count = 0usize;
    let mut findings = Vec::new();
    for left_index in 0..approved.len() {
        for right_index in (left_index + 1)..approved.len() {
            let left = &approved[left_index];
            let right = &approved[right_index];
            let topics = shared_topics(left, right);
            if topics.is_empty() {
                continue;
            }
            comparisons += 1;
            let stem_similarity = Sim::of(&left.stem, &right.stem);
            if let Some((left_claim, right_claim, anchor_similarity)) =
                numeric_conflict(left, right, &policy, stem_similarity)
            {
                numeric_count += 1;
                findings.push(format!(
                    "numeric topic={} ids={}<> {} unit={} values={} vs {} stem={}%, anchor={}%; same keyed quantity has different values",
                    topics.join(","),
                    left.item.id,
                    right.item.id,
                    left_claim.unit,
                    left_claim.value,
                    right_claim.value,
                    stem_similarity.percent(),
                    anchor_similarity.percent(),
                ));
            }
            if negation_conflict(left, right, &policy, stem_similarity) {
                negation_count += 1;
                findings.push(format!(
                    "explicit-negation topic={} ids={}<> {} stem={}%, answer={}%; keyed answers differ only by explicit negation",
                    topics.join(","),
                    left.item.id,
                    right.item.id,
                    stem_similarity.percent(),
                    Sim::of(&left.answer_without_negation, &right.answer_without_negation).percent(),
                ));
            }
        }
    }
    findings.sort();
    let report = format!(
        "approved single-select={}; compared topic-pairs={}; numeric-contradictions={}; explicit-negation-pairs={}",
        approved.len(), comparisons, numeric_count, negation_count
    );
    if findings.is_empty() {
        Eval::Ok(format!(
            "{NAME}: PASS: {report}; no bank-internal contradictions"
        ))
    } else {
        Eval::Violation(std::iter::once(report).chain(findings).collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/key_contradiction")
            .join(name)
    }

    #[test]
    fn known_bad_fixture_finds_both_contradiction_shapes() {
        match evaluate(&fixture("bad")) {
            Eval::Violation(items) => {
                let report = items.join("\n");
                assert!(report.contains("numeric-contradictions=1"));
                assert!(report.contains("explicit-negation-pairs=1"));
                assert!(report.contains("numeric-a<> numeric-b"));
                assert!(report.contains("neg-a<> neg-b"));
            }
            other => panic!("known-bad contradiction fixture did not go RED: {other:?}"),
        }
    }

    #[test]
    fn known_good_fixture_is_green() {
        match evaluate(&fixture("good")) {
            Eval::Ok(text) => {
                assert!(text.contains("approved single-select=3"));
                assert!(text.contains("numeric-contradictions=0"));
                assert!(text.contains("explicit-negation-pairs=0"));
            }
            other => panic!("known-good contradiction fixture did not pass: {other:?}"),
        }
    }

    #[test]
    fn zero_approved_items_are_an_error() {
        match evaluate(&fixture("empty")) {
            Eval::Error(message) => assert!(message.contains("zero approved single-select")),
            other => panic!("empty contradiction scan was not an ERROR: {other:?}"),
        }
    }
}
