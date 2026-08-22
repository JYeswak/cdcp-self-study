//! Measure the corpus-derived absolute/universal cue through the real
//! `cdcp_assemble::assemble` path.
//!
//! Seeds 0..99 are the predeclared form denominator.  This is a measurement
//! example, not a second sampler: the only forms counted here are the actual
//! `AssembledExam`s returned by the product assembler.

use cdcp_assemble::{assemble, rng_from_seed, AssembleConfig};
use cdcp_bank::plausibility::{self, BankAudit, ChoiceClassification, MarkerInventory};
use cdcp_bank::Bank;
use rand::Rng;
use std::path::PathBuf;

const SEED_FIRST: u64 = 0;
const SEED_COUNT: u64 = 100;
const RANDOM_CONTROL_SALT: u64 = 0xA11C_E55E;

fn correct_index(correct: &str) -> usize {
    match correct {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        other => panic!("assembled item has invalid correct={other:?}"),
    }
}

fn print_bank_report(inventory: &MarkerInventory, audit: &BankAudit) {
    let terms = inventory
        .terms()
        .iter()
        .map(|term| {
            format!(
                "{}:{}:{}",
                term.phrase, term.category, term.option_occurrences
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "MARKERS corpus_options={} terms={}",
        inventory.corpus_options, terms
    );
    println!(
        "BANK_WIDE scanned={} distribution=[0:{},1:{},2:{},3:{},4:{}] applicable_exactly_three={} non_pattern_one_or_two={} excluded_zero={} excluded_all_four={} key_is_lone_plausible={} rate={:.1}% chance_floor=25.0% delta={:+.1}pp",
        audit.overall.scanned,
        audit.overall.marker_distribution[0],
        audit.overall.marker_distribution[1],
        audit.overall.marker_distribution[2],
        audit.overall.marker_distribution[3],
        audit.overall.marker_distribution[4],
        audit.overall.applicable,
        audit.overall.excluded_one_or_two(),
        audit.overall.excluded_zero(),
        audit.overall.excluded_all_four(),
        audit.overall.key_hits,
        audit.overall.rate_pct(),
        audit.overall.rate_pct() - 25.0
    );
    println!(
        "BANK_EXCLUSIONS reason_zero=no lexical evidence reason_four=no unique unmarked option"
    );
    for (module, counts) in &audit.by_module {
        println!(
            "MODULE module={module:02} scanned={} applicable_exactly_three={} non_pattern_one_or_two={} excluded_zero={} excluded_all_four={} key_is_lone_plausible={} rate={}",
            counts.scanned,
            counts.applicable,
            counts.excluded_one_or_two(),
            counts.excluded_zero(),
            counts.excluded_all_four(),
            counts.key_hits,
            counts.rate_label()
        );
    }
}

fn form_classification(
    item: &cdcp_assemble::AssembledItem,
    inventory: &MarkerInventory,
) -> ChoiceClassification {
    plausibility::classify_choices(&item.choices, inventory)
        .unwrap_or_else(|error| panic!("{}: {error}", item.id))
}

fn main() {
    let bank_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items");
    let bank = Bank::load_dir(&bank_path).expect("load live bank");
    let inventory = plausibility::derive_marker_inventory(bank.items.values());
    assert!(
        !inventory.is_empty(),
        "corpus-derived marker inventory is empty"
    );
    let audit = plausibility::audit_bank(&bank, &inventory).expect("audit live bank");

    println!(
        "PREDECLARED_ASSEMBLER_SEEDS first={SEED_FIRST} count={SEED_COUNT} inclusive_end={}",
        SEED_FIRST + SEED_COUNT - 1
    );
    println!(
        "APPLICABLE_RULE marker_count=3 exactly; hit=lone_unmarked_option=key; chance_floor=25.0%"
    );
    print_bank_report(&inventory, &audit);

    let cfg = AssembleConfig::default();
    let mut assembled_forms = 0usize;
    let mut assembled_items = 0usize;
    let mut applicable = 0usize;
    let mut key_hits = 0usize;
    let mut random_hits = 0usize;
    let mut expected_random_hits = 0.0f64;

    for seed in SEED_FIRST..SEED_FIRST + SEED_COUNT {
        let exam = assemble(&bank, seed, cfg).unwrap_or_else(|error| {
            panic!("predeclared seed {seed} failed real assembly: {error}")
        });
        assert_eq!(
            exam.items.len(),
            40,
            "seed {seed} did not produce a 40-item form"
        );
        assembled_forms += 1;
        assembled_items += exam.items.len();
        let mut random = rng_from_seed(seed ^ RANDOM_CONTROL_SALT);
        for item in &exam.items {
            let classification = form_classification(item, &inventory);
            if !classification.applicable {
                continue;
            }
            applicable += 1;
            expected_random_hits += 0.25;
            if classification.lone_unmarked == Some(correct_index(&item.correct)) {
                key_hits += 1;
            }
            if classification.lone_unmarked == Some(random.gen_range(0..4)) {
                random_hits += 1;
            }
        }
    }

    let key_rate = plausibility::percentage(key_hits, applicable);
    let mean_key_hits = key_hits as f64 / assembled_forms as f64;
    let mean_expected_random = expected_random_hits / assembled_forms as f64;
    let mean_random_hits = random_hits as f64 / assembled_forms as f64;
    println!(
        "ASSEMBLER seeds={} forms={} items={} applicable={} key_is_lone_plausible={} rate={:.1}% chance_floor=25.0% delta={:+.1}pp",
        SEED_COUNT,
        assembled_forms,
        assembled_items,
        applicable,
        key_hits,
        key_rate,
        key_rate - 25.0
    );
    println!(
        "FORM_MEAN per_40_items key_is_lone_plausible={mean_key_hits:.2} random_control_expected={mean_expected_random:.2} random_control_realized={mean_random_hits:.2}"
    );
    println!(
        "LIMITATION lexical absolute/universal sub-case of F-01 only; semantic absurdity and some off-topic distractors remain outside this detector (stem-overlap covers only part of off-topicness)"
    );
}
