//! Thin dispatcher. The lexical detector lives in cdcp_bank::plausibility.

use crate::registry::{GateCtx, GateError};
use cdcp_bank::plausibility::{self, BankAudit, MarkerInventory};
use cdcp_bank::Bank;

pub const NAME: &str = "plausibility-detector";
pub const SUMMARY: &str = "measure absolute/universal lone-plausible option cues";

const LIMITATION: &str = "LIMITATION: lexical absolute/universal sub-case of F-01 only; semantic absurdity and some off-topic distractors remain outside this detector (stem-overlap covers only part of off-topicness)";

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;
    let bank_path = ctx.root.join("bank/items");
    let bank = Bank::load_dir(&bank_path)
        .map_err(|error| GateError::error(format!("load {}: {error}", bank_path.display())))?;
    let inventory = plausibility::derive_marker_inventory(bank.items.values());
    if inventory.is_empty() {
        return Err(GateError::error(
            "plausibility: corpus-derived marker inventory is empty (vacuous scan)",
        ));
    }
    let audit = plausibility::audit_bank(&bank, &inventory).map_err(GateError::error)?;
    print_report(&inventory, &audit);

    if audit.findings.is_empty() {
        println!(
            "plausibility-detector: PASS: no key-is-lone-plausible absolute/universal branch fired"
        );
        Ok(())
    } else {
        println!(
            "plausibility-detector: RED: {} {} finding(s) require content review",
            plausibility::BRANCH,
            audit.findings.len()
        );
        Err(GateError::violation(
            audit.findings.iter().map(|finding| finding.branch_marker()),
        ))
    }
}

fn print_report(inventory: &MarkerInventory, audit: &BankAudit) {
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
        "plausibility-detector: derived-marker-inventory corpus_options={} terms={}",
        inventory.corpus_options, terms
    );
    println!(
        "bank-wide: scanned={} distribution=[0:{},1:{},2:{},3:{},4:{}] applicable_exactly_three={} non_pattern_one_or_two={} excluded_zero={} excluded_all_four={} key_is_lone_plausible={} rate={:.1}% chance_floor=25.0% delta={:+.1}pp",
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
        "exclusions: zero-marker rows have no lexical evidence; four-marker rows have no unique unmarked option; one- and two-marker rows do not have the declared three-marked/one-unmarked shape; only exactly-three rows are the predeclared applicable population"
    );
    for (module, counts) in &audit.by_module {
        println!(
            "module={module:02}: scanned={} applicable_exactly_three={} non_pattern_one_or_two={} excluded_zero={} excluded_all_four={} key_is_lone_plausible={} rate={}",
            counts.scanned,
            counts.applicable,
            counts.excluded_one_or_two(),
            counts.excluded_zero(),
            counts.excluded_all_four(),
            counts.key_hits,
            counts.rate_label()
        );
    }
    println!("{LIMITATION}");
}
