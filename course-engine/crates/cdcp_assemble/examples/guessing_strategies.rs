//! Product-level guessing-strategy measurement.
//!
//! This example deliberately scores the `AssembledExam` returned by the real
//! `cdcp_assemble::assemble` path. It does not resample `bank/items` itself.
//! Seeds 0 through 99 are the predeclared denominator; seed 42 is included.

use cdcp_assemble::{assemble, rng_from_seed, AssembleConfig, AssembledExam};
use cdcp_bank::Bank;
use rand::Rng;
use std::collections::HashSet;
use std::path::PathBuf;

const SEED_FIRST: u64 = 0;
const SEED_COUNT: u64 = 100;
const PASS_BAR: u32 = 27;
const HEDGED_MEAN_MIN: f64 = 9.0;
const HEDGED_MEAN_MAX: f64 = 11.0;
const HEDGED_APPLICABLE_HIT_MIN: f64 = 0.20;
const HEDGED_APPLICABLE_HIT_MAX: f64 = 0.30;
const STEM_OVERLAP_MEAN_MIN: f64 = 9.0;
const STEM_OVERLAP_MEAN_MAX: f64 = 11.5;
const STEM_OVERLAP_APPLICABLE_HIT_MIN: f64 = 0.20;
const STEM_OVERLAP_APPLICABLE_HIT_MAX: f64 = 0.30;
const HEDGES: &[&str] = &[
    "can",
    "could",
    "generally",
    "may",
    "might",
    "often",
    "usually",
    "typically",
    "tends",
    "sometimes",
];
const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "because", "by", "can", "for", "from", "how", "if",
    "in", "into", "is", "it", "its", "more", "of", "on", "or", "should", "that", "the", "their",
    "then", "this", "to", "under", "was", "what", "when", "which", "with",
];

#[derive(Clone, Copy)]
enum Strategy {
    Longest,
    Letter(usize),
    Hedged,
    AvoidHedged,
    StemOverlap,
    UniformRandom,
}

impl Strategy {
    fn name(self) -> &'static str {
        match self {
            Self::Longest => "longest",
            Self::Letter(0) => "always-A",
            Self::Letter(1) => "always-B",
            Self::Letter(2) => "always-C",
            Self::Letter(3) => "always-D",
            Self::Letter(_) => unreachable!(),
            Self::Hedged => "hedged",
            Self::AvoidHedged => "avoid-hedged",
            Self::StemOverlap => "stem-overlap",
            Self::UniformRandom => "uniform-random",
        }
    }
}

fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect()
}

fn content_words(text: &str) -> HashSet<String> {
    tokens(text)
        .into_iter()
        .filter(|word| !STOP_WORDS.contains(&word.as_str()))
        .collect()
}

fn longest(item: &cdcp_assemble::AssembledItem) -> usize {
    // Ties go to the first occurrence. This is fixed for every seed.
    item.choices
        .iter()
        .enumerate()
        .max_by_key(|(index, choice)| (choice.chars().count(), std::cmp::Reverse(*index)))
        .map(|(index, _)| index)
        .expect("assembled item has choices")
}

fn hedged(item: &cdcp_assemble::AssembledItem) -> usize {
    item.choices
        .iter()
        .position(|choice| {
            tokens(choice)
                .iter()
                .any(|word| HEDGES.contains(&word.as_str()))
        })
        .unwrap_or(0)
}

fn first_hedge_option(item: &cdcp_assemble::AssembledItem) -> Option<usize> {
    item.choices.iter().position(|choice| {
        tokens(choice)
            .iter()
            .any(|word| HEDGES.contains(&word.as_str()))
    })
}

fn hedge_option_index(item: &cdcp_assemble::AssembledItem) -> Option<usize> {
    let matches: Vec<_> = item
        .choices
        .iter()
        .enumerate()
        .filter(|(_, choice)| {
            tokens(choice)
                .iter()
                .any(|word| HEDGES.contains(&word.as_str()))
        })
        .map(|(index, _)| index)
        .collect();
    if matches.len() == 1 {
        Some(matches[0])
    } else {
        None
    }
}

fn avoid_hedged(item: &cdcp_assemble::AssembledItem) -> usize {
    hedge_option_index(item)
        .and_then(|hedge_index| {
            item.choices
                .iter()
                .enumerate()
                .find(|(index, _)| *index != hedge_index)
                .map(|(index, _)| index)
        })
        .unwrap_or(0)
}

fn stem_overlap(item: &cdcp_assemble::AssembledItem) -> usize {
    let stem = content_words(&item.stem);
    item.choices
        .iter()
        .enumerate()
        .max_by_key(|(index, choice)| {
            let overlap = content_words(choice).intersection(&stem).count();
            (overlap, std::cmp::Reverse(*index))
        })
        .map(|(index, _)| index)
        .expect("assembled item has choices")
}

fn unique_stem_overlap_option(item: &cdcp_assemble::AssembledItem) -> Option<usize> {
    let stem = content_words(&item.stem);
    let scores: Vec<_> = item
        .choices
        .iter()
        .map(|choice| content_words(choice).intersection(&stem).count())
        .collect();
    let max = scores.iter().copied().max()?;
    if max == 0 || scores.iter().filter(|&&score| score == max).count() != 1 {
        return None;
    }
    scores.iter().position(|&score| score == max)
}

fn choice(strategy: Strategy, item: &cdcp_assemble::AssembledItem, rng: &mut impl Rng) -> usize {
    match strategy {
        Strategy::Longest => longest(item),
        Strategy::Letter(index) => index,
        Strategy::Hedged => hedged(item),
        Strategy::AvoidHedged => avoid_hedged(item),
        Strategy::StemOverlap => stem_overlap(item),
        Strategy::UniformRandom => rng.gen_range(0..4),
    }
}

fn correct_index(item: &cdcp_assemble::AssembledItem) -> usize {
    match item.correct.as_str() {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        other => panic!("assembled item {} has invalid correct={other:?}", item.id),
    }
}

fn score(exam: &AssembledExam, strategy: Strategy, seed: u64) -> (u32, Vec<usize>) {
    let mut rng = rng_from_seed(seed ^ 0xBADC_0FFE);
    let mut score = 0;
    let mut right = Vec::new();
    for (index, item) in exam.items.iter().enumerate() {
        if choice(strategy, item, &mut rng) == correct_index(item) {
            score += 1;
            right.push(index);
        }
    }
    (score, right)
}

fn main() {
    let bank_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items");
    let bank = Bank::load_dir(&bank_path).expect("load live approved bank");
    let cfg = AssembleConfig::default();
    let strategies = [
        Strategy::Longest,
        Strategy::Letter(0),
        Strategy::Letter(1),
        Strategy::Letter(2),
        Strategy::Letter(3),
        Strategy::Hedged,
        Strategy::AvoidHedged,
        Strategy::StemOverlap,
        Strategy::UniformRandom,
    ];
    let mut scores = vec![Vec::<u32>::new(); strategies.len()];
    let mut failures = Vec::new();
    let mut any_hedged_total = 0;
    let mut any_hedged_hits = 0;
    let mut exact_hedged_total = 0;
    let mut exact_hedged_hits = 0;
    let mut exact_avoid_hits = 0;
    let mut stem_applicable_total = 0;
    let mut stem_applicable_hits = 0;

    println!(
        "PREDECLARED_SEEDS first={SEED_FIRST} count={SEED_COUNT} inclusive_end={} pass_bar={PASS_BAR}",
        SEED_FIRST + SEED_COUNT - 1
    );
    println!("RULE longest=tie-first; hedged=no-match=A; overlap=tie-first; random=ChaCha12(seed^0xBADC0FFE)");

    for seed in SEED_FIRST..SEED_FIRST + SEED_COUNT {
        let exam = match assemble(&bank, seed, cfg) {
            Ok(exam) => exam,
            Err(error) => {
                failures.push((seed, error.to_string()));
                for row in &mut scores {
                    row.push(0);
                }
                continue;
            }
        };
        assert_eq!(
            exam.items.len(),
            cfg.n_items,
            "assembly returned non-40 form"
        );
        for item in &exam.items {
            if let Some(index) = first_hedge_option(item) {
                any_hedged_total += 1;
                if index == correct_index(item) {
                    any_hedged_hits += 1;
                }
            }
            if let Some(index) = hedge_option_index(item) {
                exact_hedged_total += 1;
                if index == correct_index(item) {
                    exact_hedged_hits += 1;
                }
                if avoid_hedged(item) == correct_index(item) {
                    exact_avoid_hits += 1;
                }
            }
            if let Some(index) = unique_stem_overlap_option(item) {
                stem_applicable_total += 1;
                if index == correct_index(item) {
                    stem_applicable_hits += 1;
                }
            }
        }
        for (index, strategy) in strategies.iter().copied().enumerate() {
            let (value, right) = score(&exam, strategy, seed);
            scores[index].push(value);
            if value >= PASS_BAR {
                println!(
                    "RESIDUAL seed={seed} strategy={} score={value}",
                    strategy.name()
                );
                for item_index in right {
                    let item = &exam.items[item_index];
                    println!(
                        "  id={} correct={} stem={:?} chosen={:?}",
                        item.id,
                        item.correct,
                        item.stem,
                        item.choices[correct_index(item)]
                    );
                }
            }
        }
    }

    if !failures.is_empty() {
        for (seed, error) in &failures {
            println!("ASSEMBLY_FAILURE seed={seed} reason={error}");
        }
    }
    for (index, strategy) in strategies.iter().copied().enumerate() {
        let values = &scores[index];
        let sum: u32 = values.iter().sum();
        let passes = values.iter().filter(|&&value| value >= PASS_BAR).count();
        let min = values.iter().copied().min().expect("seed denominator");
        let max = values.iter().copied().max().expect("seed denominator");
        println!(
            "TABLE strategy={} mean={:.2} min={} max={} pass_count={} pass_share={:.1}% denominator={} assembled={} failures={}",
            strategy.name(),
            sum as f64 / values.len() as f64,
            min,
            max,
            passes,
            passes as f64 * 100.0 / values.len() as f64,
            values.len(),
            values.len() - failures.len(),
            failures.len()
        );
    }

    assert!(failures.is_empty(), "all predeclared seeds must assemble");
    let hedged_values = &scores[5];
    let hedged_mean = hedged_values.iter().sum::<u32>() as f64 / hedged_values.len() as f64;
    let stem_values = &scores[7];
    let stem_mean = stem_values.iter().sum::<u32>() as f64 / stem_values.len() as f64;
    let any_hedged_rate = any_hedged_hits as f64 / any_hedged_total as f64;
    assert!(
        (HEDGED_MEAN_MIN..=HEDGED_MEAN_MAX).contains(&hedged_mean),
        "hedged mean {hedged_mean:.2} is outside control band {HEDGED_MEAN_MIN:.1}..={HEDGED_MEAN_MAX:.1}"
    );
    let exact_hedged_rate = exact_hedged_hits as f64 / exact_hedged_total as f64;
    assert!(
        (HEDGED_APPLICABLE_HIT_MIN..=HEDGED_APPLICABLE_HIT_MAX).contains(&exact_hedged_rate),
        "exactly-one hedged hit rate {exact_hedged_rate:.1} is outside {HEDGED_APPLICABLE_HIT_MIN:.1}..={HEDGED_APPLICABLE_HIT_MAX:.1}"
    );
    println!(
        "HEDGED_ANY total={any_hedged_total} key_hits={any_hedged_hits} hit_rate={:.1}% diagnostic_multi_hedge",
        any_hedged_rate * 100.0
    );
    println!(
        "HEDGED_APPLICABLE_EXACTLY_ONE total={exact_hedged_total} key_hits={exact_hedged_hits} hit_rate={:.1}% target=20-30%",
        exact_hedged_rate * 100.0
    );
    let exact_avoid_rate = exact_avoid_hits as f64 / exact_hedged_total as f64;
    assert!(
        (HEDGED_APPLICABLE_HIT_MIN..=HEDGED_APPLICABLE_HIT_MAX).contains(&exact_avoid_rate),
        "exactly-one avoid-hedged hit rate {exact_avoid_rate:.1} is outside {HEDGED_APPLICABLE_HIT_MIN:.1}..={HEDGED_APPLICABLE_HIT_MAX:.1}"
    );
    println!(
        "HEDGED_AVOID_EXACTLY_ONE total={exact_hedged_total} key_hits={exact_avoid_hits} hit_rate={:.1}% target=20-30%",
        exact_avoid_rate * 100.0
    );
    let stem_applicable_rate = stem_applicable_hits as f64 / stem_applicable_total as f64;
    assert!(
        (STEM_OVERLAP_MEAN_MIN..=STEM_OVERLAP_MEAN_MAX).contains(&stem_mean),
        "stem-overlap mean {stem_mean:.2} is outside control band {STEM_OVERLAP_MEAN_MIN:.1}..={STEM_OVERLAP_MEAN_MAX:.1}"
    );
    assert!(
        (STEM_OVERLAP_APPLICABLE_HIT_MIN..=STEM_OVERLAP_APPLICABLE_HIT_MAX)
            .contains(&stem_applicable_rate),
        "stem-overlap applicable hit rate {stem_applicable_rate:.1} is outside {STEM_OVERLAP_APPLICABLE_HIT_MIN:.1}..={STEM_OVERLAP_APPLICABLE_HIT_MAX:.1}"
    );
    println!(
        "STEM_OVERLAP_APPLICABLE total={stem_applicable_total} key_hits={stem_applicable_hits} hit_rate={:.1}% target=20-30%",
        stem_applicable_rate * 100.0
    );
}
