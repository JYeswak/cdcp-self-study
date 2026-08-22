//! W1b — census whether the shipped item bank is taught by each module page.
//!
//! This is deliberately a measurement tool, not a semantic oracle.  It uses a
//! predeclared lexical floor to find rows that require a human read, then
//! applies the small set of already-recorded human adjudications from the W1b
//! sample and the two known F-04 cases.  A page mentioning a word is not, by
//! itself, evidence that the page teaches the decision.

use cdcp_bank::{Bank, BankItem};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const MODULE_NAMES: [&str; 15] = [
    "mission-critical",
    "standards",
    "site-building",
    "floor-ceiling",
    "lighting",
    "power",
    "emf",
    "racks",
    "cooling",
    "water",
    "network",
    "fire",
    "security",
    "auxiliary",
    "ops-adjacent",
];

/// The denominator and lexical thresholds are fixed before the census result
/// is read.  Approved rows are what a learner can receive; retired rows stay
/// in the 957-file inventory but are not part of the mismatch rate.
const EXPECTED_TOTAL_FILES: usize = 957;
const EXPECTED_APPROVED: usize = 931;
const TOPIC_SUPPORT_TAUGHT: f64 = 0.50;
const EVIDENCE_SUPPORT_TAUGHT: f64 = 0.55;
const TOPIC_SUPPORT_REVIEW: f64 = 0.40;
const EVIDENCE_SUPPORT_REVIEW: f64 = 0.30;

const STOP: &[&str] = &[
    "the", "a", "an", "and", "or", "of", "to", "in", "on", "for", "with", "from", "by", "is",
    "are", "was", "were", "be", "as", "at", "this", "that", "it", "its", "their", "they", "them",
    "into", "over", "under", "which", "what", "when", "where", "how", "why", "does", "do", "did",
    "can", "could", "should", "would", "will", "may", "might", "than", "then", "also", "only",
    "more", "most", "some", "such", "each", "both", "same", "other", "about", "after", "before",
    "while", "there", "those", "these", "been", "being", "have", "has", "had", "not", "no", "very",
    "just", "like", "because", "through", "during", "without", "within", "your", "you", "we",
    "our",
];

#[derive(Debug, Deserialize)]
struct TopicRegistry {
    topic: Vec<TopicRow>,
}

#[derive(Debug, Deserialize)]
struct TopicRow {
    id: String,
    label: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Classification {
    Taught,
    Shallow,
    Absent,
    Contradicted,
}

impl Classification {
    fn is_mismatch(self) -> bool {
        !matches!(self, Self::Taught)
    }
}

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Taught => "TAUGHT",
            Self::Shallow => "SHALLOW",
            Self::Absent => "ABSENT",
            Self::Contradicted => "CONTRADICTED",
        })
    }
}

#[derive(Debug)]
struct Finding {
    id: String,
    module: u32,
    classification: Classification,
    topic_support: f64,
    evidence_support: f64,
    stem_support: f64,
    reason: &'static str,
}

#[derive(Debug, Default, Clone, Copy)]
struct Counts {
    taught: usize,
    shallow: usize,
    absent: usize,
    contradicted: usize,
}

impl Counts {
    fn add(&mut self, classification: Classification) {
        match classification {
            Classification::Taught => self.taught += 1,
            Classification::Shallow => self.shallow += 1,
            Classification::Absent => self.absent += 1,
            Classification::Contradicted => self.contradicted += 1,
        }
    }

    fn total(self) -> usize {
        self.taught + self.shallow + self.absent + self.contradicted
    }

    fn mismatch(self) -> usize {
        self.shallow + self.absent + self.contradicted
    }
}

fn engine_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tokenize(text: &str) -> BTreeSet<String> {
    let mut words = BTreeSet::new();
    let mut current = String::new();

    let flush = |current: &mut String, words: &mut BTreeSet<String>| {
        if current.len() > 2 && !STOP.contains(&current.as_str()) {
            words.insert(std::mem::take(current));
        } else {
            current.clear();
        }
    };

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else {
            flush(&mut current, &mut words);
        }
    }
    flush(&mut current, &mut words);
    words
}

fn ratio(needle: &BTreeSet<String>, haystack: &BTreeSet<String>) -> f64 {
    if needle.is_empty() {
        return 0.0;
    }
    needle.intersection(haystack).count() as f64 / needle.len() as f64
}

fn module_text(root: &Path, module: u32) -> String {
    let name = MODULE_NAMES
        .get(module.saturating_sub(1) as usize)
        .unwrap_or_else(|| panic!("item module {module} is outside the 15-module census"));
    let path = root.join(format!("web/content/modules/{module:02}-{name}.md"));
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(
        text.len() > 1_000,
        "module {module} is too small to teach a bank"
    );
    text
}

fn topic_labels(root: &Path) -> BTreeMap<String, String> {
    let path = root.join("knowledge/topics.toml");
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let registry: TopicRegistry = toml::from_str(&raw).expect("knowledge/topics.toml parses");
    registry
        .topic
        .into_iter()
        .map(|row| (row.id, row.label))
        .collect()
}

/// These are not tuned after the census.  They are the already documented
/// human reads: the two original F-04 examples and three named marginal rows
/// from the earlier 120-item sample.  Everything else comes from the fixed
/// lexical floor and remains a proxy, not an expert claim.
fn prior_adjudication(id: &str) -> Option<(Classification, &'static str)> {
    match id {
        "m10-q300" => Some((
            Classification::Absent,
            "the module does not teach the applied leak-response sequence",
        )),
        "m15-q350" => Some((
            Classification::Absent,
            "the module does not teach sanitization or recording a reused device's destination",
        )),
        "m15-q385" => Some((
            Classification::Absent,
            "the module does not teach the OSHA 1904.39 amputation reporting timeline",
        )),
        "m15-q363" => Some((
            Classification::Shallow,
            "the page teaches role/competence matrices, not job-based IDPs or DOE O 360.1D",
        )),
        "m15-q376" => Some((
            Classification::Shallow,
            "the page teaches service-level reporting, not the specific support-demand decision",
        )),
        _ => None,
    }
}

fn classify(
    item: &BankItem,
    module_tokens: &BTreeSet<String>,
    labels: &BTreeMap<String, String>,
) -> Finding {
    let topic_text = item
        .topic_ids
        .iter()
        .filter_map(|id| labels.get(id))
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let topic_tokens = tokenize(&topic_text);
    let key = item
        .correct
        .as_bytes()
        .first()
        .and_then(|b| b.checked_sub(b'A'))
        .map(usize::from)
        .unwrap_or_else(|| panic!("{} has invalid correct letter", item.id));
    let key_text = item
        .choices
        .get(key)
        .unwrap_or_else(|| panic!("{} correct letter has no choice", item.id));
    let evidence_tokens = tokenize(&format!("{key_text} {}", item.explanation));
    let stem_tokens = tokenize(&item.stem);
    let topic_support = ratio(&topic_tokens, module_tokens);
    let evidence_support = ratio(&evidence_tokens, module_tokens);
    let stem_support = ratio(&stem_tokens, module_tokens);

    let (classification, reason) = if let Some((classification, reason)) =
        prior_adjudication(&item.id)
    {
        (classification, reason)
    } else if topic_support >= TOPIC_SUPPORT_TAUGHT && evidence_support >= EVIDENCE_SUPPORT_TAUGHT {
        (
            Classification::Taught,
            "topic and answer/explanation vocabulary clear the predeclared support floor",
        )
    } else if topic_support >= TOPIC_SUPPORT_REVIEW && evidence_support >= EVIDENCE_SUPPORT_REVIEW {
        (
            Classification::Shallow,
            "topic is present but answer/explanation support is below the taught floor",
        )
    } else {
        (
            Classification::Absent,
            "the lexical floor found neither enough topic nor answer/explanation support",
        )
    };

    Finding {
        id: item.id.clone(),
        module: item.module,
        classification,
        topic_support,
        evidence_support,
        stem_support,
        reason,
    }
}

fn main() {
    let root = engine_root();
    let bank = Bank::load_dir(&root.join("bank/items")).expect("bank/items loads");
    let labels = topic_labels(&root);
    let module_tokens: BTreeMap<u32, BTreeSet<String>> = (1..=15)
        .map(|module| (module, tokenize(&module_text(&root, module))))
        .collect();

    assert_eq!(bank.items.len(), EXPECTED_TOTAL_FILES, "denominator drift");
    let approved: Vec<&BankItem> = bank
        .items
        .values()
        .filter(|item| item.is_approved())
        .collect();
    assert_eq!(
        approved.len(),
        EXPECTED_APPROVED,
        "approved denominator drift"
    );

    let mut counts = Counts::default();
    let mut by_module = BTreeMap::<u32, Counts>::new();
    let mut findings = Vec::new();
    for item in &approved {
        let finding = classify(
            item,
            module_tokens
                .get(&item.module)
                .expect("module tokens exist"),
            &labels,
        );
        counts.add(finding.classification);
        by_module
            .entry(finding.module)
            .or_default()
            .add(finding.classification);
        if finding.classification.is_mismatch() {
            findings.push(finding);
        }
    }

    println!("W1B teaching/test census");
    println!(
        "DENOMINATOR total_files={} approved_shipped={} retired_or_not_shipped={} modules=15 prior_human_adjudications=5",
        bank.items.len(),
        approved.len(),
        bank.items.len() - approved.len()
    );
    println!(
        "RUBRIC taught=topic_support>={:.2} AND evidence_support>={:.2}; shallow=topic_support>={:.2} AND evidence_support>={:.2}; absent=otherwise; contradicted=semantic human-only state",
        TOPIC_SUPPORT_TAUGHT,
        EVIDENCE_SUPPORT_TAUGHT,
        TOPIC_SUPPORT_REVIEW,
        EVIDENCE_SUPPORT_REVIEW
    );
    println!(
        "BANK taught={} shallow={} absent={} contradicted={} mismatch={} mismatch_rate={:.1}%",
        counts.taught,
        counts.shallow,
        counts.absent,
        counts.contradicted,
        counts.mismatch(),
        100.0 * counts.mismatch() as f64 / counts.total() as f64
    );
    println!("MODULE module taught shallow absent contradicted mismatch rate");
    for module in 1..=15 {
        let c = by_module.get(&module).copied().unwrap_or_default();
        println!(
            "MODULE m{module:02} {} {} {} {} {} {:.1}%",
            c.taught,
            c.shallow,
            c.absent,
            c.contradicted,
            c.mismatch(),
            100.0 * c.mismatch() as f64 / c.total().max(1) as f64
        );
    }
    println!("FINDINGS non_taught_rows={}", findings.len());
    for finding in findings {
        println!(
            "FINDING {} m{:02} {} topic={:.2} evidence={:.2} stem={:.2} {}",
            finding.id,
            finding.module,
            finding.classification,
            finding.topic_support,
            finding.evidence_support,
            finding.stem_support,
            finding.reason
        );
    }
    println!("CONTRADICTED none — lexical text cannot establish a contradiction or name both conflicting artifact ids");
    println!("SHOULD_FAIL m03-q217 — an initial absence search missed the dedicated BTM section; a closer read finds land, fuel/gas path, permits, and walk-away conditions, so it remains TAUGHT");
    println!("LIMITATION this is a lexical review floor plus five prior human adjudications, not a semantic teaching oracle; a page mentioning a term is not proof that a learner is taught the decision, and semantic contradiction requires a human read");
}
