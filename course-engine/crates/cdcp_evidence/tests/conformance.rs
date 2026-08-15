//! Evidence conformance: approval binding, distractor links, attestation.
//!
//! Public CI is structural. It is not a factual oracle. The load-bearing
//! tokens are [`BINDING_CHECK`], [`DISTRACTOR_RULE`], [`PROHIBITED_BODY`],
//! [`ATTESTATION_RULE`], and [`NO_REEVAL`]. Deleting a check those tokens
//! sit inside makes the matching selftest non-zero.

use cdcp_evidence::{
    AiIngestion, ClaimRecord, ConformanceFault, Invalidation, ItemEvidence, LicenceKind,
    OptionBinding, RecordError, ReviewKind, ReviewRecord, SourceArtifact, SourceInput,
    ATTESTATION_RULE, BINDING_CHECK, DISTRACTOR_RULE, NO_REEVAL, PROHIBITED_BODY,
};
use std::collections::BTreeMap;

fn osha() -> SourceArtifact {
    SourceArtifact::try_new(SourceInput {
        authority: "OSHA",
        edition: "2024",
        date: "2024-01-01",
        jurisdiction: "US",
        licence_kind: LicenceKind::Public,
        licence: "17 USC 105",
        locator: "§1910.147(c)",
        ai_ingestion: AiIngestion::Permitted,
        body: Some("energy isolation / lockout-tagout for servicing"),
    })
    .expect("OSHA fixture")
}

fn ashrae() -> SourceArtifact {
    SourceArtifact::try_new(SourceInput {
        authority: "ASHRAE",
        edition: "2025",
        date: "2025",
        jurisdiction: "US",
        licence_kind: LicenceKind::PaidStandard,
        licence: "ASHRAE copyright",
        locator: "§6.5.1",
        ai_ingestion: AiIngestion::Prohibited,
        body: None,
    })
    .expect("ASHRAE fixture")
}

fn accepted_claim(
    id: &str,
    proposition: &str,
    scope: &str,
    qualifiers: &[&str],
    source: &SourceArtifact,
    kind: ReviewKind,
    licensed: bool,
) -> ClaimRecord {
    let claim = ClaimRecord::try_new(
        id,
        proposition,
        scope,
        qualifiers.iter().map(|s| (*s).to_string()).collect(),
        source,
    )
    .expect("claim");
    let review = ReviewRecord::attest(
        "licensed-reviewer-1",
        licensed,
        "2026-08-15",
        kind,
        &claim,
        source,
    )
    .expect("review");
    claim.with_review(review)
}

fn maps(
    sources: &[&SourceArtifact],
    claims: &[&ClaimRecord],
) -> (
    BTreeMap<String, ClaimRecord>,
    BTreeMap<String, SourceArtifact>,
) {
    let sources = sources
        .iter()
        .map(|s| (s.digest().to_string(), (*s).clone()))
        .collect();
    let claims = claims
        .iter()
        .map(|c| (c.id().to_string(), (*c).clone()))
        .collect();
    (claims, sources)
}

fn approved_public_item() -> (
    ItemEvidence,
    BTreeMap<String, ClaimRecord>,
    BTreeMap<String, SourceArtifact>,
) {
    let source = osha();
    let keyed_claim = accepted_claim(
        "claim-key",
        "servicing requires energy isolation",
        "electrical; OSHA 1910.147",
        &["utilization installations excluded by 1910.147 itself"],
        &source,
        ReviewKind::HumanTextual,
        false,
    );
    let dist_claim = accepted_claim(
        "claim-dist",
        "a verbal warning is not energy isolation",
        "electrical; OSHA 1910.147",
        &["distractor: procedure is not a shout"],
        &source,
        ReviewKind::HumanTextual,
        false,
    );
    let mut item = ItemEvidence::try_new(
        "m06-q001",
        "Before servicing a locked-out source, the operator must:",
        OptionBinding::linked("isolate stored energy", keyed_claim.id()).unwrap(),
        vec![
            OptionBinding::linked("shout a warning and proceed", dist_claim.id()).unwrap(),
            OptionBinding::linked("rely on the upstream breaker alone", dist_claim.id()).unwrap(),
        ],
        vec!["osha-1910-147".into()],
    )
    .unwrap();
    let (claims, sources) = maps(&[&source], &[&keyed_claim, &dist_claim]);
    item.approve("editor-1", "2026-08-15", &claims, &sources)
        .expect("structurally conformant");
    (item, claims, sources)
}

fn approved_ashrae_item() -> (
    ItemEvidence,
    BTreeMap<String, ClaimRecord>,
    BTreeMap<String, SourceArtifact>,
) {
    let source = ashrae();
    let keyed_claim = accepted_claim(
        "ashrae-key",
        "recommended envelope is class-dependent",
        "cooling; ASHRAE 90.4",
        &["recommended, not code"],
        &source,
        ReviewKind::Attestation,
        true,
    );
    let dist_claim = accepted_claim(
        "ashrae-dist",
        "a single universal temperature is not the recommended envelope",
        "cooling; ASHRAE 90.4",
        &["distractor: one number for every class"],
        &source,
        ReviewKind::Attestation,
        true,
    );
    let mut item = ItemEvidence::try_new(
        "m09-q010",
        "ASHRAE recommended thermal envelope is:",
        OptionBinding::linked("class-dependent recommended band", keyed_claim.id()).unwrap(),
        vec![OptionBinding::linked("one universal temperature", dist_claim.id()).unwrap()],
        vec!["ashrae-90.4-2025".into()],
    )
    .unwrap();
    let (claims, sources) = maps(&[&source], &[&keyed_claim, &dist_claim]);
    item.approve("editor-1", "2026-08-15", &claims, &sources)
        .expect("attested ASHRAE item");
    (item, claims, sources)
}

fn has_invalidation(
    item: &ItemEvidence,
    claims: &BTreeMap<String, ClaimRecord>,
    want: Invalidation,
) {
    let empty: BTreeMap<String, SourceArtifact> = BTreeMap::new();
    let c = item.public_ci_conformance(claims, &empty);
    assert!(
        c.faults().iter().any(|f| matches!(
            f,
            ConformanceFault::ApprovalInvalidated(inv) if *inv == want
        )),
        "expected {want:?} in {c}"
    );
}

#[test]
fn unchanged_approved_item_is_conformant() {
    let (item, claims, sources) = approved_public_item();
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(c.is_conformant(), "{c}");
}

#[test]
fn unchanged_attested_ashrae_item_is_conformant() {
    let (item, claims, sources) = approved_ashrae_item();
    assert!(ashrae().public_ci_text().is_none());
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(c.is_conformant(), "{c}");
}

#[test]
fn changing_stem_invalidates_approval() {
    let (mut item, claims, sources) = approved_public_item();
    item.replace_stem("MUTATED STEM: a different question");
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(!c.is_conformant(), "stem change must be RED");
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::ApprovalInvalidated(Invalidation::StemChanged)
    )));
}

#[test]
fn changing_proposition_invalidates_approval() {
    let (item, mut claims, sources) = approved_public_item();
    claims
        .get_mut("claim-key")
        .unwrap()
        .replace_proposition("MUTATED PROPOSITION");
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(!c.is_conformant(), "proposition change must be RED");
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::ApprovalInvalidated(Invalidation::PropositionChanged)
    )));
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::ClaimNotAccepted { claim_id } if claim_id == "claim-key"
    )));
}

#[test]
fn changing_source_invalidates_approval() {
    let (item, mut claims, mut sources) = approved_public_item();
    let new_source = SourceArtifact::try_new(SourceInput {
        authority: "OSHA",
        edition: "2019",
        date: "2019-01-01",
        jurisdiction: "US",
        licence_kind: LicenceKind::Public,
        licence: "17 USC 105",
        locator: "§1910.147(c)",
        ai_ingestion: AiIngestion::Permitted,
        body: Some("older edition"),
    })
    .unwrap();
    for claim in claims.values_mut() {
        claim.retarget_source(&new_source);
    }
    sources.clear();
    sources.insert(new_source.digest().to_string(), new_source);
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(!c.is_conformant(), "source change must be RED");
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::ApprovalInvalidated(Invalidation::SourceChanged)
    )));
}

/// Known-bad: mutate a keyed answer without re-review → approval invalid, RED.
#[test]
fn known_bad_mutate_keyed_answer_without_rereview_is_red() {
    let (mut item, claims, sources) = approved_public_item();
    assert!(
        item.public_ci_conformance(&claims, &sources)
            .is_conformant(),
        "precondition: live approved item is conformant"
    );
    item.replace_keyed_answer("shout a warning and proceed");
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(
        !c.is_conformant(),
        "mutated keyed answer without re-review must be RED, got {c}"
    );
    assert!(
        c.faults().iter().any(|f| matches!(
            f,
            ConformanceFault::ApprovalInvalidated(Invalidation::KeyedAnswerChanged)
        )),
        "known-bad must trip KeyedAnswerChanged: {c}"
    );
}

#[test]
fn stem_citation_alone_is_insufficient() {
    let source = osha();
    let keyed_claim = accepted_claim(
        "claim-key",
        "servicing requires energy isolation",
        "electrical; OSHA 1910.147",
        &[],
        &source,
        ReviewKind::HumanTextual,
        false,
    );
    let mut item = ItemEvidence::try_new(
        "m06-q002",
        "Before servicing a locked-out source, the operator must:",
        OptionBinding::linked("isolate stored energy", keyed_claim.id()).unwrap(),
        vec![
            OptionBinding::unlinked("shout a warning and proceed").unwrap(),
            OptionBinding::unlinked("rely on the upstream breaker alone").unwrap(),
        ],
        vec!["osha-1910-147".into()],
    )
    .unwrap();
    let (claims, sources) = maps(&[&source], &[&keyed_claim]);
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(
        !c.is_conformant(),
        "stem citation without distractor claims must be RED"
    );
    assert!(c
        .faults()
        .iter()
        .any(|f| matches!(f, ConformanceFault::StemCitationInsufficient)));
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::DistractorUnlinked { text } if text.contains("shout")
    )));
    assert!(item
        .approve("editor-1", "2026-08-15", &claims, &sources)
        .is_err());
}

#[test]
fn distractors_must_link_to_accepted_claims() {
    let source = osha();
    let keyed_claim = accepted_claim(
        "claim-key",
        "servicing requires energy isolation",
        "electrical; OSHA 1910.147",
        &[],
        &source,
        ReviewKind::HumanTextual,
        false,
    );
    let unreviewed = ClaimRecord::try_new(
        "claim-dist-draft",
        "a verbal warning is not energy isolation",
        "electrical; OSHA 1910.147",
        vec![],
        &source,
    )
    .unwrap();
    let item = ItemEvidence::try_new(
        "m06-q003",
        "Before servicing:",
        OptionBinding::linked("isolate stored energy", keyed_claim.id()).unwrap(),
        vec![OptionBinding::linked("shout a warning", unreviewed.id()).unwrap()],
        vec![],
    )
    .unwrap();
    let (claims, sources) = maps(&[&source], &[&keyed_claim, &unreviewed]);
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(!c.is_conformant(), "{c}");
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::ClaimNotAccepted { claim_id } if claim_id == "claim-dist-draft"
    )));
}

#[test]
fn paid_standard_requires_attestation_and_locator() {
    let source = ashrae();
    let draft = ClaimRecord::try_new(
        "ashrae-draft",
        "recommended envelope is class-dependent",
        "cooling; ASHRAE 90.4",
        vec![],
        &source,
    )
    .unwrap();
    let item = ItemEvidence::try_new(
        "m09-q011",
        "ASHRAE recommended thermal envelope is:",
        OptionBinding::linked("class-dependent recommended band", draft.id()).unwrap(),
        vec![OptionBinding::linked("one universal temperature", draft.id()).unwrap()],
        vec!["ashrae-90.4-2025".into()],
    )
    .unwrap();
    let (claims, sources) = maps(&[&source], &[&draft]);
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(!c.is_conformant(), "{c}");
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::PaidStandardMissingAttestation { authority } if authority == "ASHRAE"
    )));
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::ClaimNotAccepted { claim_id } if claim_id == "ashrae-draft"
    )));

    let planted =
        SourceArtifact::plant_paid_without_locator("TIA", "942-C", "2024", "US", "TIA copyright");
    assert!(planted.locator().is_empty());
    let locator_claim = ClaimRecord::try_new(
        "tia-draft",
        "Rated-4 is not an Uptime Tier",
        "standards map; TIA-942",
        vec![],
        &planted,
    )
    .unwrap();
    let item = ItemEvidence::try_new(
        "m02-q001",
        "TIA-942 Rated-4 is:",
        OptionBinding::linked("a TIA rating, not an Uptime Tier", locator_claim.id()).unwrap(),
        vec![OptionBinding::linked("identical to Uptime Tier IV", locator_claim.id()).unwrap()],
        vec![],
    )
    .unwrap();
    let (claims, sources) = maps(&[&planted], &[&locator_claim]);
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::PaidStandardMissingLocator { authority } if authority == "TIA"
    )));
}

#[test]
fn public_ci_does_not_reevaluate_inaccessible_text() {
    let (item, claims, sources) = approved_ashrae_item();
    let source = sources.values().next().unwrap();
    assert!(
        source.public_ci_text().is_none(),
        "ASHRAE must not expose body to public CI"
    );
    // Conformance is GREEN on attestation + locator even though no body
    // exists to "check" the proposition against. That is the point.
    assert!(item
        .public_ci_conformance(&claims, &sources)
        .is_conformant());
}

#[test]
fn prohibited_source_accepts_locator_plus_attestation_without_body() {
    let source = ashrae();
    assert!(source.public_ci_text().is_none());
    assert!(!source.carries_unlawful_extracted_text());
    assert_eq!(source.locator(), "§6.5.1");
    let (item, claims, sources) = approved_ashrae_item();
    assert!(item
        .public_ci_conformance(&claims, &sources)
        .is_conformant());
}

#[test]
fn prohibited_plant_with_extracted_text_is_red() {
    let planted = SourceArtifact::plant_prohibited_with_extracted_text(
        "ASHRAE",
        "2025",
        "2025",
        "US",
        "ASHRAE copyright",
        "§6.5.1",
        "the recommended temperature range shall be",
    );
    assert!(planted.carries_unlawful_extracted_text());
    assert!(planted.public_ci_text().is_none());
    let claim = accepted_claim(
        "ashrae-key",
        "recommended envelope is class-dependent",
        "cooling; ASHRAE 90.4",
        &[],
        &planted,
        ReviewKind::Attestation,
        true,
    );
    let item = ItemEvidence::try_new(
        "m09-q012",
        "ASHRAE recommended thermal envelope is:",
        OptionBinding::linked("class-dependent recommended band", claim.id()).unwrap(),
        vec![OptionBinding::linked("one universal temperature", claim.id()).unwrap()],
        vec![],
    )
    .unwrap();
    let (claims, sources) = maps(&[&planted], &[&claim]);
    let c = item.public_ci_conformance(&claims, &sources);
    assert!(!c.is_conformant(), "{c}");
    assert!(c.faults().iter().any(|f| matches!(
        f,
        ConformanceFault::ProhibitedSourceCarriesBody { authority } if authority == "ASHRAE"
    )));
}

#[test]
fn unlicensed_attestation_of_paid_standard_is_rejected() {
    let source = ashrae();
    let claim = ClaimRecord::try_new(
        "c",
        "recommended envelope is class-dependent",
        "cooling",
        vec![],
        &source,
    )
    .unwrap();
    let err = ReviewRecord::attest(
        "intern",
        false,
        "2026-08-15",
        ReviewKind::Attestation,
        &claim,
        &source,
    )
    .expect_err("unlicensed");
    assert_eq!(err, RecordError::PaidStandardRequiresLicensedReviewer);
}

/// Meta-test: delete the binding comparison → this selftest is non-zero.
/// Keys on the conformance *body*, not a comment.
#[test]
fn selftest_delete_binding_check_is_nonzero() {
    let src = include_str!("../src/records.rs");
    let mut parts = src.split("fn public_ci_conformance");
    let _before = parts.next().expect("split");
    let body = parts
        .next()
        .expect("ItemEvidence::public_ci_conformance is missing");
    let body = body.split("fn structural_faults").next().unwrap_or(body);
    assert!(
        body.contains("BINDING_CHECK"),
        "delete the binding check → selftest non-zero"
    );
    assert!(
        body.contains("invalidations"),
        "delete the binding comparison → selftest non-zero"
    );

    let mut parts = src.split("fn invalidations");
    let _before = parts.next().expect("split");
    let inv = parts.next().expect("invalidations is missing");
    let inv = inv.split("fn linked_claim_ids").next().unwrap_or(inv);
    assert!(
        inv.contains("Invalidation::StemChanged")
            && inv.contains("Invalidation::KeyedAnswerChanged")
            && inv.contains("Invalidation::PropositionChanged")
            && inv.contains("Invalidation::SourceChanged"),
        "delete an invalidation reason → selftest non-zero"
    );
}

/// Meta-test: delete the distractor rule → this selftest is non-zero.
#[test]
fn selftest_delete_distractor_rule_is_nonzero() {
    let src = include_str!("../src/records.rs");
    let mut parts = src.split("fn structural_faults");
    let _before = parts.next().expect("split");
    let body = parts.next().expect("structural_faults is missing");
    let body = body.split("fn claim_faults").next().unwrap_or(body);
    assert!(
        body.contains("DISTRACTOR_RULE"),
        "delete the distractor rule → selftest non-zero"
    );
    assert!(
        body.contains("StemCitationInsufficient") && body.contains("DistractorUnlinked"),
        "delete the distractor-link check → selftest non-zero"
    );
}

/// Meta-test: public CI must not re-evaluate inaccessible body text.
#[test]
fn selftest_public_ci_does_not_read_body() {
    let src = include_str!("../src/records.rs");
    let mut parts = src.split("fn public_ci_conformance");
    let _before = parts.next().expect("split");
    let body = parts
        .next()
        .expect("ItemEvidence::public_ci_conformance is missing");
    let body = body.split("fn structural_faults").next().unwrap_or(body);
    assert!(
        body.contains("NO_REEVAL"),
        "delete the no-reeval token → selftest non-zero"
    );
    assert!(
        !body.contains(".body") && !body.contains("public_ci_text"),
        "public CI must not read SourceArtifact body text"
    );
}

/// Meta-test: delete the PROHIBITED-body rejection → this selftest is non-zero.
#[test]
fn selftest_delete_prohibited_body_rejection_is_nonzero() {
    let src = include_str!("../src/records.rs");
    let mut parts = src.split("impl SourceArtifact");
    let _before = parts.next().expect("split");
    let impl_body = parts.next().expect("impl SourceArtifact is missing");
    let ctor = impl_body
        .split("fn plant_prohibited_with_extracted_text")
        .next()
        .unwrap_or(impl_body);
    assert!(
        ctor.contains("PROHIBITED_BODY"),
        "delete the PROHIBITED-body rejection → selftest non-zero"
    );
    assert!(
        ctor.contains("ProhibitedSourceCarriesBody")
            || ctor.contains("carries_unlawful_extracted_text"),
        "delete the PROHIBITED-body rejection → selftest non-zero"
    );

    let mut parts = src.split("fn claim_faults");
    let _before = parts.next().expect("split");
    let faults = parts.next().expect("claim_faults is missing");
    let faults = faults.split("fn invalidations").next().unwrap_or(faults);
    assert!(
        faults.contains("ATTESTATION_RULE") && faults.contains("PaidStandardMissingAttestation"),
        "delete the attestation check → selftest non-zero"
    );
}

#[test]
fn tokens_are_the_acceptance_sentences() {
    assert!(BINDING_CHECK.contains("invalidates"));
    assert!(DISTRACTOR_RULE.contains("stem has a citation"));
    assert!(PROHIBITED_BODY.contains("PROHIBITED"));
    assert!(ATTESTATION_RULE.contains("attestation"));
    assert!(NO_REEVAL.contains("never re-evaluates"));
}

// Silence a dead helper: invalidation-only path is covered above with sources.
#[test]
fn invalidation_helper_names_stem() {
    let (mut item, claims, _) = approved_public_item();
    item.replace_stem("x");
    has_invalidation(&item, &claims, Invalidation::StemChanged);
}
