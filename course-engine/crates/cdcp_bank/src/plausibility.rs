//! Corpus-derived absolute/universal plausibility cue.
//!
//! This is deliberately a floor-raising lexical detector, not a semantic
//! plausibility grader.  It detects one narrow form of the F-01 defect:
//! exactly three options contain an absolute, universal, or totalising marker
//! and the one unmarked option is the key.  The marker vocabulary is filtered
//! through the actual bank corpus at runtime; a candidate that never occurs in
//! a bank option is not part of the inventory.
//!
//! The detector cannot decide semantic absurdity (for example, whether an
//! edge-data-centre option means "serve only as a diesel storage yard") and it
//! cannot decide all off-topic distractors.  Stem overlap covers part of the
//! latter, but this module does not pretend to cover the rest.

use crate::{Bank, BankItem};
use std::collections::BTreeMap;

pub const BRANCH: &str = "absolute-universal-lone-plausible";

/// Candidate phrases came from the quoted UX findings and their corpus
/// generalisations.  Membership in the shipped inventory still requires an
/// occurrence in an option in the bank; this is not a desk-invented marker
/// list applied wholesale to the data.
const CANDIDATES: &[(&str, &str)] = &[
    // Absolutes.
    ("all", "absolute"),
    ("every", "absolute"),
    ("always", "absolute"),
    ("never", "absolute"),
    ("none", "absolute"),
    ("zero", "absolute"),
    ("any", "absolute"),
    ("only", "absolute"),
    ("no", "absolute"),
    // Universals.
    ("regardless of", "universal"),
    ("in all cases", "universal"),
    ("without exception", "universal"),
    // Totalising language, including the observed inflections.
    ("immunity", "totalising"),
    ("immunity from all", "totalising"),
    ("eliminates entirely", "totalising"),
    ("guarantee", "totalising"),
    ("guarantees", "totalising"),
    ("guaranteed", "totalising"),
    ("universally", "totalising"),
    ("permanently", "totalising"),
    ("entirely", "totalising"),
    ("automatic", "totalising"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerTerm {
    pub phrase: String,
    pub category: String,
    /// Number of option texts in the derivation corpus containing this term.
    pub option_occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerInventory {
    pub corpus_options: usize,
    terms: Vec<MarkerTerm>,
}

impl MarkerInventory {
    pub fn terms(&self) -> &[MarkerTerm] {
        &self.terms
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

/// Derive the marker inventory from option text in the supplied corpus.
pub fn derive_marker_inventory<'a>(
    items: impl IntoIterator<Item = &'a BankItem>,
) -> MarkerInventory {
    let choices = items
        .into_iter()
        .flat_map(|item| item.choices.iter())
        .collect::<Vec<_>>();
    let terms = CANDIDATES
        .iter()
        .filter_map(|(phrase, category)| {
            let option_occurrences = choices
                .iter()
                .filter(|choice| contains_phrase(choice, phrase))
                .count();
            (option_occurrences > 0).then(|| MarkerTerm {
                phrase: (*phrase).to_string(),
                category: (*category).to_string(),
                option_occurrences,
            })
        })
        .collect();
    MarkerInventory {
        corpus_options: choices.len(),
        terms,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceClassification {
    pub marked: [bool; 4],
    pub marker_count: usize,
    pub applicable: bool,
    pub lone_unmarked: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub id: String,
    pub module: u32,
    pub correct_index: usize,
    pub marker_options: [usize; 3],
    pub marker_terms: Vec<String>,
}

impl Finding {
    pub fn branch_marker(&self) -> String {
        format!(
            "{BRANCH}: item={} module={} key={} unmarked={} marked_options={} markers={}",
            self.id,
            self.module,
            option_letter(self.correct_index),
            option_letter(remaining_index(&self.marker_options)),
            self.marker_options
                .iter()
                .map(|index| option_letter(*index).to_string())
                .collect::<Vec<_>>()
                .join(","),
            self.marker_terms.join("|")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PopulationCounts {
    pub scanned: usize,
    pub marker_distribution: [usize; 5],
    pub applicable: usize,
    pub key_hits: usize,
}

impl PopulationCounts {
    pub fn excluded_zero(&self) -> usize {
        self.marker_distribution[0]
    }

    pub fn excluded_all_four(&self) -> usize {
        self.marker_distribution[4]
    }

    pub fn excluded_one_or_two(&self) -> usize {
        self.marker_distribution[1] + self.marker_distribution[2]
    }

    pub fn rate_pct(&self) -> f64 {
        percentage(self.key_hits, self.applicable)
    }

    pub fn rate_label(&self) -> String {
        if self.applicable == 0 {
            "n/a".to_string()
        } else {
            format!("{:.1}%", self.rate_pct())
        }
    }

    fn add(&mut self, classification: &ChoiceClassification) {
        self.scanned += 1;
        self.marker_distribution[classification.marker_count] += 1;
        if classification.applicable {
            self.applicable += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankAudit {
    pub overall: PopulationCounts,
    pub by_module: BTreeMap<u32, PopulationCounts>,
    pub findings: Vec<Finding>,
}

/// Analyze every item in the supplied bank.  The bank is expected to be a
/// four-choice single-select corpus; a different kind is an explicit error,
/// not a silent denominator change.
pub fn audit_bank(bank: &Bank, inventory: &MarkerInventory) -> Result<BankAudit, String> {
    if bank.items.is_empty() {
        return Err("plausibility: zero bank items (vacuous scan)".to_string());
    }
    let mut audit = BankAudit {
        overall: PopulationCounts::default(),
        by_module: BTreeMap::new(),
        findings: Vec::new(),
    };
    for item in bank.items.values() {
        if !item.kind.is_letter_form() {
            return Err(format!(
                "{}: kind {} is not a four-choice single-select row",
                item.id, item.kind
            ));
        }
        let classification = classify_choices(&item.choices, inventory)?;
        audit.overall.add(&classification);
        audit
            .by_module
            .entry(item.module)
            .or_default()
            .add(&classification);
        if let Some(finding) = detect_item(item, inventory)? {
            audit.overall.key_hits += 1;
            audit
                .by_module
                .get_mut(&item.module)
                .expect("module was inserted with the row")
                .key_hits += 1;
            audit.findings.push(finding);
        }
    }
    Ok(audit)
}

/// Run the production detector for one bank item.
pub fn detect_item(
    item: &BankItem,
    inventory: &MarkerInventory,
) -> Result<Option<Finding>, String> {
    let classification = classify_choices(&item.choices, inventory)?;
    Ok(finding_for(item, &classification, inventory))
}

pub fn classify_choices(
    choices: &[String],
    inventory: &MarkerInventory,
) -> Result<ChoiceClassification, String> {
    if choices.len() != 4 {
        return Err(format!(
            "plausibility: expected four choices, got {}",
            choices.len()
        ));
    }
    let marked = std::array::from_fn(|index| {
        inventory
            .terms
            .iter()
            .any(|term| contains_phrase(&choices[index], &term.phrase))
    });
    let marker_count = marked.iter().filter(|&&value| value).count();
    let lone_unmarked = (marker_count == 3)
        .then(|| marked.iter().position(|&value| !value))
        .flatten();
    Ok(ChoiceClassification {
        marked,
        marker_count,
        // One- and two-marker rows have lexical evidence but cannot produce
        // the declared cue.  The applicable population is exactly three
        // marked options plus one unmarked option.
        applicable: marker_count == 3,
        lone_unmarked,
    })
}

pub fn finding_for(
    item: &BankItem,
    classification: &ChoiceClassification,
    inventory: &MarkerInventory,
) -> Option<Finding> {
    let correct_index = correct_index(&item.correct).ok()?;
    let lone_unmarked = classification.lone_unmarked?;
    if lone_unmarked != correct_index {
        return None;
    }
    let marker_options = classification
        .marked
        .iter()
        .enumerate()
        .filter_map(|(index, marked)| (*marked).then_some(index))
        .collect::<Vec<_>>();
    let marker_options: [usize; 3] = marker_options.try_into().ok()?;
    let marker_terms = inventory
        .terms
        .iter()
        .filter(|term| {
            item.choices
                .iter()
                .any(|choice| contains_phrase(choice, &term.phrase))
        })
        .map(|term| term.phrase.clone())
        .collect();
    Some(Finding {
        id: item.id.clone(),
        module: item.module,
        correct_index,
        marker_options,
        marker_terms,
    })
}

pub fn percentage(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

fn correct_index(correct: &str) -> Result<usize, String> {
    match correct {
        "A" => Ok(0),
        "B" => Ok(1),
        "C" => Ok(2),
        "D" => Ok(3),
        other => Err(format!("plausibility: correct key is not A-D: {other:?}")),
    }
}

fn option_letter(index: usize) -> char {
    char::from(b'A' + index as u8)
}

fn remaining_index(marked: &[usize; 3]) -> usize {
    (0..4).find(|index| !marked.contains(index)).unwrap_or(0)
}

fn tokens(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect()
}

fn contains_phrase(text: &str, phrase: &str) -> bool {
    let text_tokens = tokens(text);
    let phrase_tokens = tokens(phrase);
    if phrase_tokens.is_empty() || phrase_tokens.len() > text_tokens.len() {
        return false;
    }
    text_tokens
        .windows(phrase_tokens.len())
        .any(|window| window == phrase_tokens.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ItemKind, ItemStatus};

    fn item(id: &str, choices: [&str; 4], correct: &str) -> BankItem {
        BankItem {
            id: id.to_string(),
            module: 1,
            stem: "Which option is correct?".to_string(),
            choices: choices.into_iter().map(str::to_string).collect(),
            correct: correct.to_string(),
            explanation: "The explanation names the bounded reason.".to_string(),
            topic_ids: vec!["t".to_string()],
            objective_ids: Vec::new(),
            citation_ids: Vec::new(),
            tags: Vec::new(),
            bloom: "understand".to_string(),
            source_class: "original".to_string(),
            quantity_evidence: "qualitative_only".to_string(),
            status: ItemStatus::Approved,
            kind: ItemKind::SingleSelect,
        }
    }

    fn inventory(items: &[BankItem]) -> MarkerInventory {
        derive_marker_inventory(items.iter())
    }

    #[test]
    fn inventory_is_derived_and_absent_candidates_are_not_active() {
        let corpus = vec![item(
            "corpus",
            ["always safe", "bounded", "bounded", "bounded"],
            "B",
        )];
        let inventory = inventory(&corpus);
        assert!(inventory.terms().iter().any(|term| term.phrase == "always"));
        assert!(!inventory
            .terms()
            .iter()
            .any(|term| term.phrase == "without exception"));
    }

    #[test]
    fn known_bad_three_markers_key_is_named_by_branch() {
        let bad = item(
            "known-bad-absolute-universal",
            [
                "The bounded design response",
                "Always remove every safeguard",
                "No human factor ever matters",
                "Guarantees immunity from all failures",
            ],
            "A",
        );
        let inventory = inventory(std::slice::from_ref(&bad));
        let classification = classify_choices(&bad.choices, &inventory).unwrap();
        let finding = finding_for(&bad, &classification, &inventory).expect("named branch fires");
        assert_eq!(classification.marker_count, 3);
        assert_eq!(finding.correct_index, 0);
        assert!(finding.branch_marker().contains(BRANCH));
        assert!(finding
            .branch_marker()
            .contains("known-bad-absolute-universal"));
    }

    #[test]
    fn known_good_evenly_marked_set_does_not_fire() {
        let good = item(
            "known-good-even-markers",
            [
                "Always choose A",
                "Every choice is bounded",
                "No design is universal",
                "Only context decides",
            ],
            "A",
        );
        let inventory = inventory(std::slice::from_ref(&good));
        let classification = classify_choices(&good.choices, &inventory).unwrap();
        assert_eq!(classification.marker_count, 4);
        assert!(finding_for(&good, &classification, &inventory).is_none());
    }

    #[test]
    fn causal_counterfactual_is_intact_red_and_bypass_pass() {
        let bad = item(
            "causal-absolute-universal",
            [
                "A bounded response",
                "Always remove every safeguard",
                "No human factor ever matters",
                "Guarantees immunity from all failures",
            ],
            "A",
        );
        let inventory = inventory(std::slice::from_ref(&bad));
        type Detector = fn(&BankItem, &MarkerInventory) -> Result<Option<Finding>, String>;
        fn bypass_detector(
            _item: &BankItem,
            _inventory: &MarkerInventory,
        ) -> Result<Option<Finding>, String> {
            Ok(None)
        }
        fn branch_fired(item: &BankItem, inventory: &MarkerInventory, detector: Detector) -> bool {
            detector(item, inventory)
                .expect("detector evaluation")
                .is_some()
        }

        // The production function is the intact leg. The test-only injected
        // function is the scratch-source bypass; there is no production CLI
        // bypass or environment escape hatch.
        let intact = branch_fired(&bad, &inventory, detect_item);
        let bypassed = branch_fired(&bad, &inventory, bypass_detector);
        assert!(intact, "intact known-bad fixture must RED");
        assert!(!bypassed, "bypassed detector must PASS");
        assert_ne!(intact, bypassed, "bypass must change verdict");
    }

    #[test]
    fn zero_markers_are_not_applicable() {
        let good = item("zero-markers", ["A", "B", "C", "D"], "A");
        let inventory = inventory(std::slice::from_ref(&good));
        let classification = classify_choices(&good.choices, &inventory).unwrap();
        assert_eq!(classification.marker_count, 0);
        assert!(!classification.applicable);
    }
}
