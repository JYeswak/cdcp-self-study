//! Construction-quality cues that can make an item answerable by test-wise
//! inspection rather than domain knowledge.
//!
//! This is deliberately narrower than item-quality or discrimination. It can
//! find option-set construction signals; it cannot tell whether the keyed
//! answer is true, current, well grounded, or discriminating.
//!
//! GREEN-DOES-NOT-PROVE: uniform length rank removes length as a signal. It
//! says nothing about whether distractors are plausible to someone who knows
//! the material; that still needs response data.

use crate::{Bank, BankItem};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

pub const NAME: &str = "construction-faults";
pub const SUMMARY: &str = "detect option-set construction cues in live and damaged pools";
pub const POLICY: &str = "registries/construction_faults.toml";
pub const DAMAGED_REL: &str = "crates/cdcp_bank/tests/fixtures/damaged_corpus_2026_08_18";

const LONGEST: &str = "longest-option-correct";
const RANK_UNIFORMITY: &str = "length-rank-uniformity";
const GRAMMAR: &str = "grammatical-disagreement";
const ABSOLUTE: &str = "absolute-language-distractor";
const ALL_NONE: &str = "all-none-of-the-above";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eval {
    Ok(String),
    Violation(Vec<String>),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Counts {
    pub items_scanned: usize,
    pub longest_option_correct: usize,
    pub grammatical_disagreement: usize,
    pub absolute_language_distractor: usize,
    pub all_none_of_the_above: usize,
}

impl Counts {
    fn total(&self) -> usize {
        self.longest_option_correct
            + self.grammatical_disagreement
            + self.absolute_language_distractor
            + self.all_none_of_the_above
    }

    fn get(&self, name: &str) -> usize {
        match name {
            LONGEST => self.longest_option_correct,
            GRAMMAR => self.grammatical_disagreement,
            ABSOLUTE => self.absolute_language_distractor,
            ALL_NONE => self.all_none_of_the_above,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Findings {
    counts: Counts,
    samples: [Vec<String>; 4],
    rank_counts: [usize; 4],
    rank_uniformity_violation: bool,
}

impl Findings {
    fn new() -> Self {
        Self {
            counts: Counts::default(),
            samples: std::array::from_fn(|_| Vec::new()),
            rank_counts: [0; 4],
            rank_uniformity_violation: false,
        }
    }

    fn add(&mut self, index: usize, id: &str) {
        if self.samples[index].len() < 5 {
            self.samples[index].push(id.to_string());
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    longest_option_correct: LongestPolicy,
    rank_uniformity: RankUniformityPolicy,
    grammatical_disagreement: GrammarPolicy,
    absolute_language_distractors: AbsolutePolicy,
    all_none_of_the_above: AllNonePolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LongestPolicy {
    min_ratio_pct: u32,
    min_extra_words: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RankUniformityPolicy {
    max_deviation_pct: u32,
    min_items: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GrammarPolicy {
    min_disagreeing_distractors: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AbsolutePolicy {
    absolute_words: Vec<String>,
    hedge_words: Vec<String>,
    min_absolute_distractors: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllNonePolicy {
    min_matches: usize,
}

#[derive(Debug)]
struct Policy {
    longest: LongestPolicy,
    rank: RankUniformityPolicy,
    grammar: GrammarPolicy,
    absolute_words: BTreeSet<String>,
    hedge_words: BTreeSet<String>,
    min_absolute_distractors: usize,
    all_none_min_matches: usize,
}

fn load_policy(root: &Path) -> Result<Policy, String> {
    let path = root.join(POLICY);
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let raw: Registry =
        toml::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))?;
    if !(101..=500).contains(&raw.longest_option_correct.min_ratio_pct) {
        return Err(format!(
            "{POLICY}: longest_option_correct.min_ratio_pct must be 101..=500"
        ));
    }
    if raw.longest_option_correct.min_extra_words == 0 {
        return Err(format!(
            "{POLICY}: longest_option_correct.min_extra_words must be positive"
        ));
    }
    if raw.rank_uniformity.max_deviation_pct >= 25 {
        return Err(format!(
            "{POLICY}: rank_uniformity.max_deviation_pct must be 0..=24"
        ));
    }
    if raw.rank_uniformity.min_items < 4 {
        return Err(format!(
            "{POLICY}: rank_uniformity.min_items must be at least 4"
        ));
    }
    if raw.grammatical_disagreement.min_disagreeing_distractors == 0
        || raw.absolute_language_distractors.min_absolute_distractors == 0
        || raw.all_none_of_the_above.min_matches == 0
    {
        return Err(format!("{POLICY}: detector minimums must be positive"));
    }
    let absolute_words = configured_words(
        &raw.absolute_language_distractors.absolute_words,
        "absolute_words",
    )?;
    let hedge_words = configured_words(
        &raw.absolute_language_distractors.hedge_words,
        "hedge_words",
    )?;
    Ok(Policy {
        longest: raw.longest_option_correct,
        rank: raw.rank_uniformity,
        grammar: raw.grammatical_disagreement,
        absolute_words,
        hedge_words,
        min_absolute_distractors: raw.absolute_language_distractors.min_absolute_distractors,
        all_none_min_matches: raw.all_none_of_the_above.min_matches,
    })
}

fn configured_words(values: &[String], field: &str) -> Result<BTreeSet<String>, String> {
    let words = values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if words.is_empty() {
        return Err(format!("{POLICY}: {field} must not be empty"));
    }
    Ok(words)
}

fn words(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect()
}

fn word_count(text: &str) -> usize {
    words(text).len()
}

fn token_set(text: &str) -> BTreeSet<String> {
    words(text).into_iter().collect()
}

fn choice_index(item: &BankItem) -> Result<usize, String> {
    match item.correct.as_str() {
        "A" => Ok(0),
        "B" => Ok(1),
        "C" => Ok(2),
        "D" => Ok(3),
        other => Err(format!("{}: correct key is not A-D: {other:?}", item.id)),
    }
}

fn length_rank(item: &BankItem) -> Result<usize, String> {
    let key = choice_index(item)?;
    let key_length = item.choices[key].chars().count();
    // Equal-length options break in A-D order, making the rank a permutation
    // instead of silently assigning two options the same position.
    let rank = 1 + item
        .choices
        .iter()
        .enumerate()
        .filter(|(index, choice)| {
            *index != key
                && (choice.chars().count() > key_length
                    || (choice.chars().count() == key_length && *index < key))
        })
        .count();
    Ok(rank)
}

fn longest_option_correct(item: &BankItem, policy: &LongestPolicy) -> Result<bool, String> {
    let key = choice_index(item)?;
    let key_words = word_count(&item.choices[key]);
    let max_distractor_words = item
        .choices
        .iter()
        .enumerate()
        .filter_map(|(index, choice)| (index != key).then_some(word_count(choice)))
        .max()
        .ok_or_else(|| format!("{}: no distractors", item.id))?;
    Ok(key_words >= max_distractor_words + policy.min_extra_words
        && key_words * 100 >= max_distractor_words * policy.min_ratio_pct as usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrammarCue {
    ArticleA,
    ArticleAn,
    SingularPresent,
    SingularPast,
    PluralPresent,
    PluralPast,
}

fn grammar_cue(stem: &str) -> Option<GrammarCue> {
    match words(stem).last()?.as_str() {
        "a" => Some(GrammarCue::ArticleA),
        "an" => Some(GrammarCue::ArticleAn),
        "is" | "has" | "does" => Some(GrammarCue::SingularPresent),
        "was" => Some(GrammarCue::SingularPast),
        "are" | "have" | "do" => Some(GrammarCue::PluralPresent),
        "were" => Some(GrammarCue::PluralPast),
        _ => None,
    }
}

fn agrees_with_cue(choice: &str, cue: GrammarCue) -> bool {
    let choice_words = words(choice);
    let first = choice_words.first().map(String::as_str);
    match cue {
        GrammarCue::ArticleA => first == Some("a"),
        GrammarCue::ArticleAn => first == Some("an"),
        GrammarCue::SingularPresent => {
            first == Some("is") || first == Some("has") || first == Some("does")
        }
        GrammarCue::SingularPast => first == Some("was"),
        GrammarCue::PluralPresent => {
            first == Some("are") || first == Some("have") || first == Some("do")
        }
        GrammarCue::PluralPast => first == Some("were"),
    }
}

fn grammatical_disagreement(item: &BankItem, policy: &GrammarPolicy) -> Result<bool, String> {
    let Some(cue) = grammar_cue(&item.stem) else {
        return Ok(false);
    };
    let key = choice_index(item)?;
    if !agrees_with_cue(&item.choices[key], cue) {
        return Ok(false);
    }
    let disagreeing = item
        .choices
        .iter()
        .enumerate()
        .filter(|(index, choice)| *index != key && !agrees_with_cue(choice, cue))
        .count();
    Ok(disagreeing >= policy.min_disagreeing_distractors)
}

fn absolute_language_distractor(item: &BankItem, policy: &Policy) -> Result<bool, String> {
    let key = choice_index(item)?;
    let key_tokens = token_set(&item.choices[key]);
    if key_tokens.is_disjoint(&policy.hedge_words) {
        return Ok(false);
    }
    let absolute = item
        .choices
        .iter()
        .enumerate()
        .filter(|(index, choice)| {
            *index != key && !token_set(choice).is_disjoint(&policy.absolute_words)
        })
        .count();
    Ok(absolute >= policy.min_absolute_distractors)
}

fn all_none_of_the_above(item: &BankItem, policy: &AllNonePolicy) -> bool {
    let matches = item
        .choices
        .iter()
        .filter(|choice| {
            let normalized = words(choice).join(" ");
            matches!(
                normalized.as_str(),
                "all of the above"
                    | "all of these above"
                    | "all above"
                    | "none of the above"
                    | "none of these above"
                    | "none above"
            )
        })
        .count();
    matches >= policy.min_matches
}

fn rank_uniformity_violation(
    rank_counts: &[usize; 4],
    items_scanned: usize,
    policy: &RankUniformityPolicy,
) -> bool {
    rank_counts.iter().any(|count| {
        let share = pct(*count, items_scanned);
        (share - 25.0).abs() > policy.max_deviation_pct as f64
    })
}

fn analyze_dir(
    dir: &Path,
    policy: &Policy,
    label: &str,
    approved_only: bool,
) -> Result<Findings, String> {
    let mut toml_files = 0;
    for entry in std::fs::read_dir(dir)
        .map_err(|error| format!("read {label} {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("read {label} {}: {error}", dir.display()))?;
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml_files += 1;
        }
    }
    if toml_files == 0 {
        return Err(format!(
            "{label}: zero approved single-select items (vacuous scan)"
        ));
    }
    let bank =
        Bank::load_dir(dir).map_err(|error| format!("load {label} {}: {error}", dir.display()))?;
    let items = bank
        .items
        .values()
        .filter(|item| (!approved_only || item.is_approved()) && item.kind.is_letter_form())
        .collect::<Vec<_>>();
    if items.is_empty() {
        return Err(format!(
            "{label}: zero approved single-select items (vacuous scan)"
        ));
    }
    if items.len() < policy.rank.min_items {
        return Err(format!(
            "{label}: only {} approved single-select items; rank-uniformity requires at least {}",
            items.len(),
            policy.rank.min_items
        ));
    }
    let mut findings = Findings::new();
    findings.counts.items_scanned = items.len();
    for item in items {
        let rank = length_rank(item)?;
        findings.rank_counts[rank - 1] += 1;
        let checks = [
            longest_option_correct(item, &policy.longest)?,
            grammatical_disagreement(item, &policy.grammar)?,
            absolute_language_distractor(item, policy)?,
            all_none_of_the_above(
                item,
                &AllNonePolicy {
                    min_matches: policy.all_none_min_matches,
                },
            ),
        ];
        // Counters are selected per-index rather than held in an array of &mut:
        // borrowing four fields of `findings` mutably at once, then calling
        // findings.add(), is a second overlapping mutable borrow.
        for (index, flagged) in checks.into_iter().enumerate() {
            if !flagged {
                continue;
            }
            match index {
                0 => findings.counts.longest_option_correct += 1,
                1 => findings.counts.grammatical_disagreement += 1,
                2 => findings.counts.absolute_language_distractor += 1,
                _ => findings.counts.all_none_of_the_above += 1,
            }
            findings.add(index, &item.id);
        }
    }
    findings.rank_uniformity_violation = rank_uniformity_violation(
        &findings.rank_counts,
        findings.counts.items_scanned,
        &policy.rank,
    );
    Ok(findings)
}

fn pct(count: usize, total: usize) -> f64 {
    count as f64 * 100.0 / total as f64
}

fn population_line(label: &str, findings: &Findings, policy: &RankUniformityPolicy) -> String {
    let c = &findings.counts;
    let rank_shares = findings
        .rank_counts
        .iter()
        .map(|count| format!("{:.1}%", pct(*count, c.items_scanned)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{label}: items={}; {LONGEST}={} ({:.1}%); {GRAMMAR}={} ({:.1}%); {ABSOLUTE}={} ({:.1}%); {ALL_NONE}={} ({:.1}%); detector-hits={}; {RANK_UNIFORMITY}={} counts=[{},{},{},{}] shares=[{}] expected=25.0%±{}pp",
        c.items_scanned,
        c.longest_option_correct,
        pct(c.longest_option_correct, c.items_scanned),
        c.grammatical_disagreement,
        pct(c.grammatical_disagreement, c.items_scanned),
        c.absolute_language_distractor,
        pct(c.absolute_language_distractor, c.items_scanned),
        c.all_none_of_the_above,
        pct(c.all_none_of_the_above, c.items_scanned),
        c.total(),
        if findings.rank_uniformity_violation {
            "FAIL"
        } else {
            "PASS"
        },
        findings.rank_counts[0],
        findings.rank_counts[1],
        findings.rank_counts[2],
        findings.rank_counts[3],
        rank_shares,
        policy.max_deviation_pct,
    )
}

fn delta_line(live: &Findings, damaged: &Findings) -> String {
    let names = [LONGEST, GRAMMAR, ABSOLUTE, ALL_NONE];
    let deltas = names
        .iter()
        .map(|name| {
            let count_delta = damaged.counts.get(name) as isize - live.counts.get(name) as isize;
            let rate_delta = pct(damaged.counts.get(name), damaged.counts.items_scanned)
                - pct(live.counts.get(name), live.counts.items_scanned);
            format!("{name} count_delta={count_delta:+}; rate_delta={rate_delta:+.1}pp")
        })
        .collect::<Vec<_>>();
    let rank_deltas = damaged
        .rank_counts
        .iter()
        .zip(live.rank_counts.iter())
        .map(|(damaged, live)| format!("{:+}", *damaged as isize - *live as isize))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "delta damaged-minus-live: {}; {RANK_UNIFORMITY} count_delta=[{rank_deltas}]",
        deltas.join("; ")
    )
}

fn sample_lines(label: &str, findings: &Findings) -> Vec<String> {
    let names = [LONGEST, GRAMMAR, ABSOLUTE, ALL_NONE];
    names
        .iter()
        .enumerate()
        .filter(|(index, _)| !findings.samples[*index].is_empty())
        .map(|(index, name)| {
            format!(
                "{label} sample {name}: {}",
                findings.samples[index].join(", ")
            )
        })
        .collect()
}

fn evaluate_pair(root: &Path, require_damaged: bool) -> Eval {
    let policy = match load_policy(root) {
        Ok(policy) => policy,
        Err(message) => return Eval::Error(message),
    };
    let live = match analyze_dir(&root.join("bank/items"), &policy, "live-approved", true) {
        Ok(findings) => findings,
        Err(message) => return Eval::Error(message),
    };
    let damaged_path = root.join(DAMAGED_REL);
    let damaged = if damaged_path.is_dir() {
        match analyze_dir(&damaged_path, &policy, "damaged-corpus", false) {
            Ok(findings) => Some(findings),
            Err(message) => return Eval::Error(message),
        }
    } else if require_damaged {
        return Eval::Error(format!(
            "damaged-corpus: missing required fixture directory {}",
            damaged_path.display()
        ));
    } else {
        None
    };

    let mut report = vec![format!(
        "{NAME}: construction faults are option-set cues, not discrimination or truth"
    )];
    report.push(population_line("live-approved", &live, &policy.rank));
    if let Some(damaged) = &damaged {
        report.push(population_line("damaged-corpus", damaged, &policy.rank));
        report.push(delta_line(&live, damaged));
    }
    report.extend(sample_lines("live-approved", &live));
    if let Some(damaged) = &damaged {
        report.extend(sample_lines("damaged-corpus", damaged));
    }
    if live.counts.total() > 0 || live.rank_uniformity_violation {
        Eval::Violation(report)
    } else {
        Eval::Ok(report.join("\n"))
    }
}

/// Evaluate the live approved pool and the preserved damaged corpus.
pub fn evaluate(root: &Path) -> Eval {
    evaluate_pair(root, true)
}

/// Evaluate only a fixture's live pool. This keeps known-good and known-bad
/// fixture tests small while the production command requires both populations.
pub fn evaluate_live_only(root: &Path) -> Eval {
    evaluate_pair(root, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/construction_faults")
            .join(name)
    }

    #[test]
    fn known_bad_fixture_is_red() {
        match evaluate_live_only(&fixture("bad")) {
            Eval::Violation(lines) => {
                let report = lines.join("\n");
                assert!(report.contains(LONGEST));
                assert!(report.contains(GRAMMAR));
                assert!(report.contains(ABSOLUTE));
                assert!(report.contains(ALL_NONE));
            }
            other => panic!("known-bad construction fixture did not go RED: {other:?}"),
        }
    }

    #[test]
    fn known_good_fixture_is_green() {
        match evaluate_live_only(&fixture("good")) {
            Eval::Ok(report) => {
                assert!(report.contains("items=4"));
                assert!(report.contains("detector-hits=0"));
                assert!(report.contains("length-rank-uniformity=PASS counts=[1,1,1,1]"));
            }
            other => panic!("known-good construction fixture did not pass: {other:?}"),
        }
    }

    #[test]
    fn all_longest_rank_fixture_is_red() {
        match evaluate_live_only(&fixture("rank_longest")) {
            Eval::Violation(lines) => {
                let report = lines.join("\n");
                assert!(report.contains("length-rank-uniformity=FAIL"));
                assert!(report.contains("counts=[4,0,0,0]"));
            }
            other => panic!("all-longest rank fixture did not go RED: {other:?}"),
        }
    }

    #[test]
    fn all_shortest_rank_fixture_is_red() {
        match evaluate_live_only(&fixture("rank_shortest")) {
            Eval::Violation(lines) => {
                let report = lines.join("\n");
                assert!(report.contains("length-rank-uniformity=FAIL"));
                assert!(report.contains("counts=[0,0,0,4]"));
            }
            other => panic!("all-shortest rank fixture did not go RED: {other:?}"),
        }
    }

    #[test]
    fn zero_item_scan_is_an_error() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("bank/items")).unwrap();
        std::fs::create_dir_all(root.path().join("registries")).unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registries/construction_faults.toml"),
            root.path().join(POLICY),
        )
        .unwrap();
        assert!(
            matches!(evaluate_live_only(root.path()), Eval::Error(message) if message.contains("zero"))
        );
    }

    #[test]
    fn detector_rules_are_individually_exercised() {
        let root = fixture("bad");
        let policy = load_policy(&root).unwrap();
        let findings = analyze_dir(&root.join("bank/items"), &policy, "bad", true).unwrap();
        assert_eq!(findings.counts.longest_option_correct, 1);
        assert_eq!(findings.counts.grammatical_disagreement, 1);
        assert_eq!(findings.counts.absolute_language_distractor, 1);
        assert_eq!(findings.counts.all_none_of_the_above, 1);
    }
}
