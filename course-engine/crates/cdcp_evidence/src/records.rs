//! Evidence-conformance records: source, claim, review, item.
//!
//! A citation gate proves referential integrity. These types prove
//! **evidence conformance** — every option on an item is bound to an
//! accepted claim, and an approval is bound to the stem, keyed answer,
//! proposition, and source that were reviewed. They are not a factual
//! oracle: public CI never re-judges whether a source *supports* a
//! proposition, and it never re-evaluates text it cannot lawfully access.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;

/// Token interpolated inside [`ItemEvidence::public_ci_conformance`].
/// Deleting the binding comparison makes the selftest non-zero.
pub const BINDING_CHECK: &str =
    "changing stem, keyed answer, proposition, or source invalidates an existing approval";

/// Token interpolated inside [`ItemEvidence::public_ci_conformance`].
/// Deleting the distractor-link check makes the selftest non-zero.
pub const DISTRACTOR_RULE: &str =
    "the stem has a citation is insufficient: distractors must link to accepted claims";

/// Token interpolated inside [`SourceArtifact::try_new`] and the
/// conformance predicate. Deleting the body rejection makes the
/// selftest non-zero.
pub const PROHIBITED_BODY: &str = "ai_ingestion=PROHIBITED sources must not carry extracted text";

/// Token interpolated inside paid-standard support checks.
pub const ATTESTATION_RULE: &str =
    "paid-standard claims require licensed-reviewer attestation and a clause locator";

/// Token interpolated inside [`ItemEvidence::public_ci_conformance`].
/// Public CI is structural conformance, never textual re-evaluation.
pub const NO_REEVAL: &str = "public CI never re-evaluates text it cannot lawfully access";

/// Why a conformance record could not be constructed.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecordError {
    /// A required identity field is empty or whitespace.
    #[error("field `{field}` is blank")]
    BlankField {
        /// The rejected field name.
        field: &'static str,
    },
    /// `ai_ingestion=PROHIBITED` (or any source public CI cannot lawfully
    /// access) must not carry extracted body text.
    #[error("{PROHIBITED_BODY}")]
    ProhibitedSourceCarriesBody,
    /// ASHRAE AI policy is PROHIBITED; any other `ai_ingestion` is wrong.
    #[error("ASHRAE sources must declare ai_ingestion=PROHIBITED")]
    AshraeMustProhibitIngestion,
    /// Paid / PROHIBITED sources accept attestation only, never a textual
    /// review that would imply public CI re-read the standard.
    #[error("{NO_REEVAL}")]
    TextualReviewOfInaccessibleSource,
    /// Paid-standard attestation must come from a licensed reviewer.
    #[error("{ATTESTATION_RULE}")]
    PaidStandardRequiresLicensedReviewer,
    /// The claim does not point at the source being reviewed.
    #[error("claim source_digest does not match the supplied source")]
    SourceDigestMismatch,
    /// An item with no distractors cannot satisfy the distractor rule.
    #[error("{DISTRACTOR_RULE}")]
    NoDistractors,
    /// Approval is refused until the item is structurally conformant.
    #[error("cannot mint an approval over a nonconformant item")]
    CannotApproveNonconformant,
}

/// Whether the bytes may be entered into an AI tool.
///
/// Vocabulary matches the corpus rights policy. `Unknown` is not
/// permission: it blocks body retention the same way `Prohibited` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AiIngestion {
    /// Publisher permits AI ingestion of the artifact's text.
    Permitted,
    /// Publisher forbids AI ingestion (ASHRAE). Locator + attestation only.
    Prohibited,
    /// No published AI policy was located. Fail closed: no body.
    Unknown,
}

impl AiIngestion {
    /// Canonical vocabulary string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AiIngestion::Permitted => "permitted",
            AiIngestion::Prohibited => "PROHIBITED",
            AiIngestion::Unknown => "unknown",
        }
    }
}

/// How the source may be used to support a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LicenceKind {
    /// Lawfully accessible to public CI (e.g. 17 U.S.C. §105).
    Public,
    /// Paid SDO. Claims via licensed-reviewer attestation + clause locator.
    PaidStandard,
}

impl LicenceKind {
    fn as_str(self) -> &'static str {
        match self {
            LicenceKind::Public => "public",
            LicenceKind::PaidStandard => "paid-standard",
        }
    }
}

/// Identity fields for a [`SourceArtifact`]. `digest` is computed.
#[derive(Debug, Clone)]
pub struct SourceInput<'a> {
    /// Issuing body (`ASHRAE`, `OSHA`, `NFPA`, …).
    pub authority: &'a str,
    /// Edition year or edition label.
    pub edition: &'a str,
    /// Publication date (`YYYY` or `YYYY-MM-DD`).
    pub date: &'a str,
    /// Jurisdiction the artifact speaks for.
    pub jurisdiction: &'a str,
    /// Public vs paid — decides whether attestation is required.
    pub licence_kind: LicenceKind,
    /// Licence citation (`17 USC 105`, `ASHRAE copyright`, …).
    pub licence: &'a str,
    /// Clause locator (`§1910.147(c)`, `§6.5.1`).
    pub locator: &'a str,
    /// AI-ingestion vocabulary.
    pub ai_ingestion: AiIngestion,
    /// Extracted body. Forbidden unless public + `permitted`.
    pub body: Option<&'a str>,
}

/// A cited artifact: identity + access, never a copy of a paid standard.
///
/// `digest` is a SHA-256 of the identity fields. Body text is **not**
/// part of the digest: PROHIBITED sources have no body, and a paraphrase
/// is not the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceArtifact {
    authority: String,
    edition: String,
    date: String,
    jurisdiction: String,
    licence_kind: LicenceKind,
    licence: String,
    digest: String,
    locator: String,
    ai_ingestion: AiIngestion,
    body: Option<String>,
}

impl SourceArtifact {
    /// Construct a source. Rejects blank identity, ASHRAE without
    /// `PROHIBITED`, and extracted text on any source public CI cannot
    /// lawfully access.
    pub fn try_new(input: SourceInput<'_>) -> Result<Self, RecordError> {
        require_nonblank("authority", input.authority)?;
        require_nonblank("edition", input.edition)?;
        require_nonblank("date", input.date)?;
        require_nonblank("jurisdiction", input.jurisdiction)?;
        require_nonblank("licence", input.licence)?;
        require_nonblank("locator", input.locator)?;
        if authority_is_ashrae(input.authority)
            && !matches!(input.ai_ingestion, AiIngestion::Prohibited)
        {
            return Err(RecordError::AshraeMustProhibitIngestion);
        }
        let body = match input.body {
            Some(text) => {
                require_nonblank("body", text)?;
                Some(text.to_string())
            }
            None => None,
        };
        let digest = source_digest(
            input.authority,
            input.edition,
            input.date,
            input.jurisdiction,
            input.licence_kind,
            input.licence,
            input.locator,
            input.ai_ingestion,
        );
        let source = SourceArtifact {
            authority: input.authority.to_string(),
            edition: input.edition.to_string(),
            date: input.date.to_string(),
            jurisdiction: input.jurisdiction.to_string(),
            licence_kind: input.licence_kind,
            licence: input.licence.to_string(),
            digest,
            locator: input.locator.to_string(),
            ai_ingestion: input.ai_ingestion,
            body,
        };
        if source.carries_unlawful_extracted_text() {
            let _ = PROHIBITED_BODY;
            return Err(RecordError::ProhibitedSourceCarriesBody);
        }
        Ok(source)
    }

    /// Known-bad plant: a PROHIBITED source that illegally carries
    /// extracted text. [`try_new`] rejects this state. The plant exists
    /// so the conformance predicate can go RED.
    #[must_use]
    pub fn plant_prohibited_with_extracted_text(
        authority: &str,
        edition: &str,
        date: &str,
        jurisdiction: &str,
        licence: &str,
        locator: &str,
        extracted: &str,
    ) -> Self {
        let mut source = SourceArtifact::try_new(SourceInput {
            authority,
            edition,
            date,
            jurisdiction,
            licence_kind: LicenceKind::PaidStandard,
            licence,
            locator,
            ai_ingestion: AiIngestion::Prohibited,
            body: None,
        })
        .expect("plant identity is valid without a body");
        source.body = Some(extracted.to_string());
        source
    }

    /// Known-bad plant: a paid standard with a blank clause locator.
    /// [`try_new`] rejects this state.
    #[must_use]
    pub fn plant_paid_without_locator(
        authority: &str,
        edition: &str,
        date: &str,
        jurisdiction: &str,
        licence: &str,
    ) -> Self {
        let digest = source_digest(
            authority,
            edition,
            date,
            jurisdiction,
            LicenceKind::PaidStandard,
            licence,
            "",
            AiIngestion::Unknown,
        );
        SourceArtifact {
            authority: authority.to_string(),
            edition: edition.to_string(),
            date: date.to_string(),
            jurisdiction: jurisdiction.to_string(),
            licence_kind: LicenceKind::PaidStandard,
            licence: licence.to_string(),
            digest,
            locator: String::new(),
            ai_ingestion: AiIngestion::Unknown,
            body: None,
        }
    }

    /// Issuing body.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// Edition label.
    #[must_use]
    pub fn edition(&self) -> &str {
        &self.edition
    }

    /// Publication date as recorded.
    #[must_use]
    pub fn date(&self) -> &str {
        &self.date
    }

    /// Jurisdiction the artifact speaks for.
    #[must_use]
    pub fn jurisdiction(&self) -> &str {
        &self.jurisdiction
    }

    /// Public vs paid.
    #[must_use]
    pub fn licence_kind(&self) -> LicenceKind {
        self.licence_kind
    }

    /// Licence citation.
    #[must_use]
    pub fn licence(&self) -> &str {
        &self.licence
    }

    /// Content-address of the identity fields.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Clause locator.
    #[must_use]
    pub fn locator(&self) -> &str {
        &self.locator
    }

    /// AI-ingestion vocabulary.
    #[must_use]
    pub fn ai_ingestion(&self) -> AiIngestion {
        self.ai_ingestion
    }

    /// True when public CI may lawfully look at retained body text.
    ///
    /// Paid standards, `PROHIBITED`, and `unknown` are all inaccessible.
    #[must_use]
    pub fn lawfully_accessible_to_public_ci(&self) -> bool {
        matches!(self.licence_kind, LicenceKind::Public)
            && matches!(self.ai_ingestion, AiIngestion::Permitted)
    }

    /// Body text public CI is allowed to look at. Always `None` for
    /// paid standards and `ai_ingestion=PROHIBITED` sources — even if a
    /// plant stuffed text into the record.
    #[must_use]
    pub fn public_ci_text(&self) -> Option<&str> {
        let _ = NO_REEVAL;
        if !self.lawfully_accessible_to_public_ci() {
            return None;
        }
        self.body.as_deref()
    }

    /// True when this record carries extracted text it must not.
    #[must_use]
    pub fn carries_unlawful_extracted_text(&self) -> bool {
        let _ = PROHIBITED_BODY;
        self.body.is_some() && !self.lawfully_accessible_to_public_ci()
    }

    /// Paid SDO — claims need attestation + locator, never body.
    #[must_use]
    pub fn is_paid_standard(&self) -> bool {
        matches!(self.licence_kind, LicenceKind::PaidStandard)
    }
}

/// A normalized proposition with scope and qualifiers, pointed at one source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRecord {
    id: String,
    proposition: String,
    scope: String,
    qualifiers: Vec<String>,
    source_digest: String,
    review: Option<ReviewRecord>,
}

impl ClaimRecord {
    /// Construct a claim. Qualifiers are sorted and deduplicated.
    /// Blank id / proposition / scope, or a blank qualifier, is rejected.
    pub fn try_new(
        id: impl Into<String>,
        proposition: impl Into<String>,
        scope: impl Into<String>,
        mut qualifiers: Vec<String>,
        source: &SourceArtifact,
    ) -> Result<Self, RecordError> {
        let id = id.into();
        let proposition = proposition.into();
        let scope = scope.into();
        require_nonblank("id", &id)?;
        require_nonblank("proposition", &proposition)?;
        require_nonblank("scope", &scope)?;
        if qualifiers.iter().any(|q| q.trim().is_empty()) {
            return Err(RecordError::BlankField { field: "qualifier" });
        }
        qualifiers.sort_unstable();
        qualifiers.dedup();
        Ok(ClaimRecord {
            id,
            proposition,
            scope,
            qualifiers,
            source_digest: source.digest().to_string(),
            review: None,
        })
    }

    /// Attach an independent human review of claim-to-source entailment.
    #[must_use]
    pub fn with_review(mut self, review: ReviewRecord) -> Self {
        self.review = Some(review);
        self
    }

    /// Stable claim id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Normalized proposition.
    #[must_use]
    pub fn proposition(&self) -> &str {
        &self.proposition
    }

    /// Scope the proposition is claimed under.
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// Qualifiers (sorted).
    #[must_use]
    pub fn qualifiers(&self) -> &[String] {
        &self.qualifiers
    }

    /// Digest of the source this claim is pointed at.
    #[must_use]
    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }

    /// Independent review, if any.
    #[must_use]
    pub fn review(&self) -> Option<&ReviewRecord> {
        self.review.as_ref()
    }

    /// Identity the item-level approval binds to. Changing proposition,
    /// scope, or qualifiers changes this string.
    #[must_use]
    pub fn identity(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.proposition);
        out.push('\u{1e}');
        out.push_str(&self.scope);
        out.push('\u{1e}');
        for (i, q) in self.qualifiers.iter().enumerate() {
            if i > 0 {
                out.push('\u{1f}');
            }
            out.push_str(q);
        }
        out
    }

    /// Replace the proposition without re-review. Used by the known-bad
    /// mutation tests; live editorial flow mints a new review.
    pub fn replace_proposition(&mut self, proposition: impl Into<String>) {
        self.proposition = proposition.into();
    }

    /// Point the claim at a different source without re-review.
    pub fn retarget_source(&mut self, source: &SourceArtifact) {
        self.source_digest = source.digest().to_string();
    }

    /// True when an accepted review still binds to this claim and source.
    ///
    /// This is binding equality, **not** a re-read of the source text.
    #[must_use]
    pub fn is_accepted(&self, source: &SourceArtifact) -> bool {
        let Some(review) = &self.review else {
            return false;
        };
        if !matches!(review.verdict, ReviewVerdict::Accepted) {
            return false;
        }
        if review.bound_proposition != self.proposition
            || review.bound_scope != self.scope
            || review.bound_qualifiers != self.qualifiers
            || review.bound_source_digest != self.source_digest
            || review.bound_source_digest != source.digest()
        {
            return false;
        }
        if !source.lawfully_accessible_to_public_ci() {
            return matches!(review.kind, ReviewKind::Attestation) && review.licensed_reviewer;
        }
        true
    }
}

/// How the human reviewed claim-to-source entailment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewKind {
    /// Licensed-reviewer attestation + clause locator. The only kind a
    /// paid or PROHIBITED source may carry.
    Attestation,
    /// Human read lawfully-accessible text. Public CI still does not
    /// re-judge entailment from that text.
    HumanTextual,
}

/// The reviewer's verdict. `Rejected` is recorded; it does not accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewVerdict {
    /// Claim-to-source entailment accepted by the reviewer.
    Accepted,
    /// Reviewer rejected the entailment.
    Rejected,
}

/// Independent human review of claim-to-source entailment.
///
/// The review stores the proposition and source digest it was minted
/// against. Changing either invalidates acceptance without anyone
/// having to re-open the standard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRecord {
    reviewer_id: String,
    licensed_reviewer: bool,
    reviewed_at: String,
    kind: ReviewKind,
    verdict: ReviewVerdict,
    bound_proposition: String,
    bound_scope: String,
    bound_qualifiers: Vec<String>,
    bound_source_digest: String,
}

impl ReviewRecord {
    /// Mint an accepted review bound to the current claim and source.
    ///
    /// Paid / PROHIBITED sources accept [`ReviewKind::Attestation`] from
    /// a licensed reviewer only. Public CI will not re-read the source.
    pub fn attest(
        reviewer_id: impl Into<String>,
        licensed_reviewer: bool,
        reviewed_at: impl Into<String>,
        kind: ReviewKind,
        claim: &ClaimRecord,
        source: &SourceArtifact,
    ) -> Result<Self, RecordError> {
        let reviewer_id = reviewer_id.into();
        let reviewed_at = reviewed_at.into();
        require_nonblank("reviewer_id", &reviewer_id)?;
        require_nonblank("reviewed_at", &reviewed_at)?;
        if claim.source_digest != source.digest() {
            return Err(RecordError::SourceDigestMismatch);
        }
        if !source.lawfully_accessible_to_public_ci() {
            if !matches!(kind, ReviewKind::Attestation) {
                return Err(RecordError::TextualReviewOfInaccessibleSource);
            }
            if source.is_paid_standard() && !licensed_reviewer {
                return Err(RecordError::PaidStandardRequiresLicensedReviewer);
            }
            // PROHIBITED-but-public-licence (e.g. a free ASHRAE white
            // paper we still must not ingest) also needs a licensed
            // attestation so public CI never invents a textual path.
            if matches!(source.ai_ingestion, AiIngestion::Prohibited) && !licensed_reviewer {
                return Err(RecordError::PaidStandardRequiresLicensedReviewer);
            }
        }
        Ok(ReviewRecord {
            reviewer_id,
            licensed_reviewer,
            reviewed_at,
            kind,
            verdict: ReviewVerdict::Accepted,
            bound_proposition: claim.proposition.clone(),
            bound_scope: claim.scope.clone(),
            bound_qualifiers: claim.qualifiers.clone(),
            bound_source_digest: source.digest().to_string(),
        })
    }

    /// Reviewer identifier.
    #[must_use]
    pub fn reviewer_id(&self) -> &str {
        &self.reviewer_id
    }

    /// Whether the reviewer is licensed to the paid standard.
    #[must_use]
    pub fn licensed_reviewer(&self) -> bool {
        self.licensed_reviewer
    }

    /// When the review was recorded.
    #[must_use]
    pub fn reviewed_at(&self) -> &str {
        &self.reviewed_at
    }

    /// Attestation vs human-textual.
    #[must_use]
    pub fn kind(&self) -> ReviewKind {
        self.kind
    }

    /// Accepted or rejected.
    #[must_use]
    pub fn verdict(&self) -> ReviewVerdict {
        self.verdict
    }
}

/// One option on an item, optionally linked to a claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionBinding {
    text: String,
    claim_id: Option<String>,
}

impl OptionBinding {
    /// An option already linked to an accepted-claim candidate.
    pub fn linked(
        text: impl Into<String>,
        claim_id: impl Into<String>,
    ) -> Result<Self, RecordError> {
        let text = text.into();
        let claim_id = claim_id.into();
        require_nonblank("option", &text)?;
        require_nonblank("claim_id", &claim_id)?;
        Ok(OptionBinding {
            text,
            claim_id: Some(claim_id),
        })
    }

    /// An option with no claim. Representable so conformance can go RED
    /// ("the stem has a citation" is not enough).
    pub fn unlinked(text: impl Into<String>) -> Result<Self, RecordError> {
        let text = text.into();
        require_nonblank("option", &text)?;
        Ok(OptionBinding {
            text,
            claim_id: None,
        })
    }

    /// Option text (the keyed answer or a distractor).
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Linked claim id, if any.
    #[must_use]
    pub fn claim_id(&self) -> Option<&str> {
        self.claim_id.as_deref()
    }
}

/// Item-level approval snapshot. Separate from [`ReviewRecord`]: that
/// type is claim-to-source entailment; this binds the item's wording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemApproval {
    reviewer_id: String,
    reviewed_at: String,
    bound_stem: String,
    bound_keyed_answer: String,
    bound_propositions: BTreeMap<String, String>,
    bound_source_digests: BTreeMap<String, String>,
}

impl ItemApproval {
    /// Who approved the item binding.
    #[must_use]
    pub fn reviewer_id(&self) -> &str {
        &self.reviewer_id
    }

    /// When the approval was recorded.
    #[must_use]
    pub fn reviewed_at(&self) -> &str {
        &self.reviewed_at
    }
}

/// The correct option AND every plausible distractor, each linked to a
/// claim. Stem-level citations alone do not make this conformant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEvidence {
    item_id: String,
    stem: String,
    keyed: OptionBinding,
    distractors: Vec<OptionBinding>,
    stem_citations: Vec<String>,
    approval: Option<ItemApproval>,
}

impl ItemEvidence {
    /// Construct an item-evidence record. At least one distractor is
    /// required — otherwise "every plausible distractor is linked" is
    /// vacuously true.
    pub fn try_new(
        item_id: impl Into<String>,
        stem: impl Into<String>,
        keyed: OptionBinding,
        distractors: Vec<OptionBinding>,
        stem_citations: Vec<String>,
    ) -> Result<Self, RecordError> {
        let item_id = item_id.into();
        let stem = stem.into();
        require_nonblank("item_id", &item_id)?;
        require_nonblank("stem", &stem)?;
        if distractors.is_empty() {
            return Err(RecordError::NoDistractors);
        }
        Ok(ItemEvidence {
            item_id,
            stem,
            keyed,
            distractors,
            stem_citations,
            approval: None,
        })
    }

    /// Item id.
    #[must_use]
    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    /// Stem text the approval is bound to.
    #[must_use]
    pub fn stem(&self) -> &str {
        &self.stem
    }

    /// The keyed (correct) option.
    #[must_use]
    pub fn keyed(&self) -> &OptionBinding {
        &self.keyed
    }

    /// Distractors. Every one must link to an accepted claim.
    #[must_use]
    pub fn distractors(&self) -> &[OptionBinding] {
        &self.distractors
    }

    /// Stem-level citation ids. Insufficient on their own.
    #[must_use]
    pub fn stem_citations(&self) -> &[String] {
        &self.stem_citations
    }

    /// Item-level approval, if minted.
    #[must_use]
    pub fn approval(&self) -> Option<&ItemApproval> {
        self.approval.as_ref()
    }

    /// Replace the stem without re-review.
    pub fn replace_stem(&mut self, stem: impl Into<String>) {
        self.stem = stem.into();
    }

    /// Replace the keyed answer without re-review.
    pub fn replace_keyed_answer(&mut self, text: impl Into<String>) {
        self.keyed.text = text.into();
    }

    /// Drop a distractor's claim link without re-review.
    pub fn unlink_distractor(&mut self, index: usize) {
        if let Some(d) = self.distractors.get_mut(index) {
            d.claim_id = None;
        }
    }

    /// Mint an item-level approval over the current wording and the
    /// current accepted claims. Refuses a structurally nonconformant item.
    pub fn approve(
        &mut self,
        reviewer_id: impl Into<String>,
        reviewed_at: impl Into<String>,
        claims: &BTreeMap<String, ClaimRecord>,
        sources: &BTreeMap<String, SourceArtifact>,
    ) -> Result<(), RecordError> {
        let reviewer_id = reviewer_id.into();
        let reviewed_at = reviewed_at.into();
        require_nonblank("reviewer_id", &reviewer_id)?;
        require_nonblank("reviewed_at", &reviewed_at)?;
        let structural = self.structural_faults(claims, sources);
        if !structural.is_empty() {
            return Err(RecordError::CannotApproveNonconformant);
        }
        let mut bound_propositions = BTreeMap::new();
        let mut bound_source_digests = BTreeMap::new();
        for id in self.linked_claim_ids() {
            if let Some(claim) = claims.get(&id) {
                bound_propositions.insert(id.clone(), claim.identity());
                bound_source_digests.insert(id, claim.source_digest.clone());
            }
        }
        self.approval = Some(ItemApproval {
            reviewer_id,
            reviewed_at,
            bound_stem: self.stem.clone(),
            bound_keyed_answer: self.keyed.text.clone(),
            bound_propositions,
            bound_source_digests,
        });
        Ok(())
    }

    /// Public-CI evidence conformance. Never a factual oracle.
    ///
    /// Checks binding, option-to-claim links, attestation + locator on
    /// paid standards, and the PROHIBITED-body rule. Does **not** read
    /// [`SourceArtifact`] body text and does **not** judge whether a
    /// source supports a proposition.
    pub fn public_ci_conformance(
        &self,
        claims: &BTreeMap<String, ClaimRecord>,
        sources: &BTreeMap<String, SourceArtifact>,
    ) -> Conformance {
        let _ = NO_REEVAL;
        let mut faults = self.structural_faults(claims, sources);
        match &self.approval {
            None => faults.push(ConformanceFault::Unapproved),
            Some(approval) => {
                let _ = BINDING_CHECK;
                for inv in self.invalidations(approval, claims) {
                    faults.push(ConformanceFault::ApprovalInvalidated(inv));
                }
            }
        }
        if faults.is_empty() {
            Conformance::Conformant
        } else {
            Conformance::Nonconformant(faults)
        }
    }

    fn structural_faults(
        &self,
        claims: &BTreeMap<String, ClaimRecord>,
        sources: &BTreeMap<String, SourceArtifact>,
    ) -> Vec<ConformanceFault> {
        let mut faults = Vec::new();
        let _ = DISTRACTOR_RULE;
        let stem_cited = !self.stem_citations.is_empty();
        if self.keyed.claim_id.is_none() {
            faults.push(ConformanceFault::KeyedAnswerUnlinked);
            if stem_cited {
                faults.push(ConformanceFault::StemCitationInsufficient);
            }
        }
        let mut any_distractor_unlinked = false;
        for d in &self.distractors {
            if d.claim_id.is_none() {
                any_distractor_unlinked = true;
                faults.push(ConformanceFault::DistractorUnlinked {
                    text: d.text.clone(),
                });
            }
        }
        if any_distractor_unlinked
            && stem_cited
            && !faults
                .iter()
                .any(|f| matches!(f, ConformanceFault::StemCitationInsufficient))
        {
            faults.push(ConformanceFault::StemCitationInsufficient);
        }
        let mut seen = BTreeSet::new();
        for id in self.linked_claim_ids() {
            if !seen.insert(id.clone()) {
                continue;
            }
            self.claim_faults(&id, claims, sources, &mut faults);
        }
        faults
    }

    fn claim_faults(
        &self,
        claim_id: &str,
        claims: &BTreeMap<String, ClaimRecord>,
        sources: &BTreeMap<String, SourceArtifact>,
        faults: &mut Vec<ConformanceFault>,
    ) {
        let Some(claim) = claims.get(claim_id) else {
            faults.push(ConformanceFault::ClaimMissing {
                claim_id: claim_id.to_string(),
            });
            return;
        };
        let Some(source) = sources.get(&claim.source_digest) else {
            faults.push(ConformanceFault::SourceMissing {
                digest: claim.source_digest.clone(),
            });
            return;
        };
        if source.carries_unlawful_extracted_text() {
            let _ = PROHIBITED_BODY;
            faults.push(ConformanceFault::ProhibitedSourceCarriesBody {
                authority: source.authority.clone(),
            });
        }
        if source.is_paid_standard() || matches!(source.ai_ingestion, AiIngestion::Prohibited) {
            let _ = ATTESTATION_RULE;
            if source.locator.trim().is_empty() {
                faults.push(ConformanceFault::PaidStandardMissingLocator {
                    authority: source.authority.clone(),
                });
            }
            let attestation_ok = claim.review.as_ref().is_some_and(|r| {
                matches!(r.kind, ReviewKind::Attestation)
                    && r.licensed_reviewer
                    && matches!(r.verdict, ReviewVerdict::Accepted)
            });
            if !attestation_ok {
                faults.push(ConformanceFault::PaidStandardMissingAttestation {
                    authority: source.authority.clone(),
                });
            }
        }
        if !claim.is_accepted(source) {
            faults.push(ConformanceFault::ClaimNotAccepted {
                claim_id: claim_id.to_string(),
            });
        }
    }

    fn invalidations(
        &self,
        approval: &ItemApproval,
        claims: &BTreeMap<String, ClaimRecord>,
    ) -> Vec<Invalidation> {
        let mut out = Vec::new();
        if approval.bound_stem != self.stem {
            out.push(Invalidation::StemChanged);
        }
        if approval.bound_keyed_answer != self.keyed.text {
            out.push(Invalidation::KeyedAnswerChanged);
        }
        let current_ids: BTreeSet<String> = self.linked_claim_ids().into_iter().collect();
        let bound_ids: BTreeSet<String> = approval
            .bound_propositions
            .keys()
            .cloned()
            .chain(approval.bound_source_digests.keys().cloned())
            .collect();
        for id in current_ids.union(&bound_ids) {
            let current_prop = claims.get(id).map(|c| c.identity());
            match (current_prop.as_ref(), approval.bound_propositions.get(id)) {
                (Some(cur), Some(bound)) if cur != bound => {
                    out.push(Invalidation::PropositionChanged);
                }
                (None, Some(_)) | (Some(_), None) => {
                    out.push(Invalidation::PropositionChanged);
                }
                _ => {}
            }
            let current_src = claims.get(id).map(|c| c.source_digest.as_str());
            match (current_src, approval.bound_source_digests.get(id)) {
                (Some(cur), Some(bound)) if cur != bound => {
                    out.push(Invalidation::SourceChanged);
                }
                (None, Some(_)) | (Some(_), None) => {
                    out.push(Invalidation::SourceChanged);
                }
                _ => {}
            }
        }
        out.sort();
        out.dedup();
        out
    }

    fn linked_claim_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        if let Some(id) = &self.keyed.claim_id {
            ids.push(id.clone());
        }
        for d in &self.distractors {
            if let Some(id) = &d.claim_id {
                ids.push(id.clone());
            }
        }
        ids
    }
}

/// Why an existing approval is no longer valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Invalidation {
    /// Stem text no longer matches the approved snapshot.
    StemChanged,
    /// Keyed-answer text no longer matches the approved snapshot.
    KeyedAnswerChanged,
    /// A linked proposition (or its scope/qualifiers) changed.
    PropositionChanged,
    /// A linked source digest changed.
    SourceChanged,
}

/// One reason public CI marked the item nonconformant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConformanceFault {
    /// No item-level approval has been minted.
    Unapproved,
    /// An approval exists but no longer binds to the current wording.
    ApprovalInvalidated(Invalidation),
    /// Stem-level citations do not cover unlinked options.
    StemCitationInsufficient,
    /// The keyed answer has no claim.
    KeyedAnswerUnlinked,
    /// A distractor has no claim.
    DistractorUnlinked {
        /// Distractor text.
        text: String,
    },
    /// Linked claim id is not in the supplied set.
    ClaimMissing {
        /// Missing claim id.
        claim_id: String,
    },
    /// Claim points at a digest that is not in the supplied set.
    SourceMissing {
        /// Missing source digest.
        digest: String,
    },
    /// Linked claim is not accepted under its source.
    ClaimNotAccepted {
        /// Claim that failed acceptance.
        claim_id: String,
    },
    /// A PROHIBITED / inaccessible source carries extracted text.
    ProhibitedSourceCarriesBody {
        /// Authority on the offending source.
        authority: String,
    },
    /// Paid / PROHIBITED claim lacks licensed-reviewer attestation.
    PaidStandardMissingAttestation {
        /// Authority that required attestation.
        authority: String,
    },
    /// Paid / PROHIBITED claim lacks a clause locator.
    PaidStandardMissingLocator {
        /// Authority that required a locator.
        authority: String,
    },
}

/// Public-CI evidence-conformance verdict. Not a factual-oracle grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conformance {
    /// Structurally complete, approval still bound.
    Conformant,
    /// One or more faults. The item is not evidence-conformant.
    Nonconformant(Vec<ConformanceFault>),
}

impl Conformance {
    /// True when there are no faults.
    #[must_use]
    pub fn is_conformant(&self) -> bool {
        matches!(self, Conformance::Conformant)
    }

    /// Fault list (empty when conformant).
    #[must_use]
    pub fn faults(&self) -> &[ConformanceFault] {
        match self {
            Conformance::Conformant => &[],
            Conformance::Nonconformant(f) => f,
        }
    }
}

impl fmt::Display for Invalidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let which = match self {
            Invalidation::StemChanged => "stem",
            Invalidation::KeyedAnswerChanged => "keyed answer",
            Invalidation::PropositionChanged => "proposition",
            Invalidation::SourceChanged => "source",
        };
        write!(f, "{BINDING_CHECK} ({which})")
    }
}

impl fmt::Display for ConformanceFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConformanceFault::Unapproved => write!(f, "unapproved"),
            ConformanceFault::ApprovalInvalidated(inv) => write!(f, "{inv}"),
            ConformanceFault::StemCitationInsufficient => write!(f, "{DISTRACTOR_RULE}"),
            ConformanceFault::KeyedAnswerUnlinked => write!(f, "keyed answer is unlinked"),
            ConformanceFault::DistractorUnlinked { text } => {
                write!(f, "distractor unlinked: {text}")
            }
            ConformanceFault::ClaimMissing { claim_id } => {
                write!(f, "claim missing: {claim_id}")
            }
            ConformanceFault::SourceMissing { digest } => {
                write!(f, "source missing: {digest}")
            }
            ConformanceFault::ClaimNotAccepted { claim_id } => {
                write!(f, "claim not accepted: {claim_id}")
            }
            ConformanceFault::ProhibitedSourceCarriesBody { authority } => {
                write!(f, "{PROHIBITED_BODY} ({authority})")
            }
            ConformanceFault::PaidStandardMissingAttestation { authority } => {
                write!(f, "{ATTESTATION_RULE} (attestation, {authority})")
            }
            ConformanceFault::PaidStandardMissingLocator { authority } => {
                write!(f, "{ATTESTATION_RULE} (locator, {authority})")
            }
        }
    }
}

impl fmt::Display for Conformance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Conformance::Conformant => write!(f, "conformant"),
            Conformance::Nonconformant(faults) => {
                write!(f, "nonconformant")?;
                for fault in faults {
                    write!(f, "; {fault}")?;
                }
                Ok(())
            }
        }
    }
}

fn require_nonblank(field: &'static str, value: &str) -> Result<(), RecordError> {
    if value.trim().is_empty() {
        Err(RecordError::BlankField { field })
    } else {
        Ok(())
    }
}

fn authority_is_ashrae(authority: &str) -> bool {
    authority
        .split(|c: char| !c.is_ascii_alphabetic())
        .any(|tok| tok.eq_ignore_ascii_case("ashrae"))
}

fn source_digest(
    authority: &str,
    edition: &str,
    date: &str,
    jurisdiction: &str,
    licence_kind: LicenceKind,
    licence: &str,
    locator: &str,
    ai_ingestion: AiIngestion,
) -> String {
    let mut h = Sha256::new();
    for part in [
        authority,
        edition,
        date,
        jurisdiction,
        licence_kind.as_str(),
        licence,
        locator,
        ai_ingestion.as_str(),
    ] {
        h.update(part.as_bytes());
        h.update([0xff]);
    }
    hex::encode(h.finalize())
}

#[cfg(test)]
mod unit {
    use super::*;

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
            body: Some("lockout/tagout procedures for energy isolation"),
        })
        .unwrap()
    }

    #[test]
    fn digest_is_stable_and_ignores_body() {
        let a = osha();
        let b = SourceArtifact::try_new(SourceInput {
            authority: "OSHA",
            edition: "2024",
            date: "2024-01-01",
            jurisdiction: "US",
            licence_kind: LicenceKind::Public,
            licence: "17 USC 105",
            locator: "§1910.147(c)",
            ai_ingestion: AiIngestion::Permitted,
            body: Some("a different paraphrase of the same clause"),
        })
        .unwrap();
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.digest().len(), 64);
    }

    #[test]
    fn digest_moves_when_edition_or_locator_changes() {
        let a = osha();
        let edition = SourceArtifact::try_new(SourceInput {
            authority: "OSHA",
            edition: "2023",
            date: "2024-01-01",
            jurisdiction: "US",
            licence_kind: LicenceKind::Public,
            licence: "17 USC 105",
            locator: "§1910.147(c)",
            ai_ingestion: AiIngestion::Permitted,
            body: None,
        })
        .unwrap();
        let locator = SourceArtifact::try_new(SourceInput {
            authority: "OSHA",
            edition: "2024",
            date: "2024-01-01",
            jurisdiction: "US",
            licence_kind: LicenceKind::Public,
            licence: "17 USC 105",
            locator: "§1910.147(d)",
            ai_ingestion: AiIngestion::Permitted,
            body: None,
        })
        .unwrap();
        assert_ne!(a.digest(), edition.digest());
        assert_ne!(a.digest(), locator.digest());
    }

    #[test]
    fn ashrae_without_prohibited_is_rejected() {
        let err = SourceArtifact::try_new(SourceInput {
            authority: "ASHRAE 90.4",
            edition: "2025",
            date: "2025",
            jurisdiction: "US",
            licence_kind: LicenceKind::PaidStandard,
            licence: "ASHRAE copyright",
            locator: "§6.5.1",
            ai_ingestion: AiIngestion::Permitted,
            body: None,
        })
        .expect_err("ASHRAE must be PROHIBITED");
        assert_eq!(err, RecordError::AshraeMustProhibitIngestion);
    }

    #[test]
    fn prohibited_constructor_rejects_extracted_text() {
        let err = SourceArtifact::try_new(SourceInput {
            authority: "ASHRAE",
            edition: "2025",
            date: "2025",
            jurisdiction: "US",
            licence_kind: LicenceKind::PaidStandard,
            licence: "ASHRAE copyright",
            locator: "§6.5.1",
            ai_ingestion: AiIngestion::Prohibited,
            body: Some("the recommended temperature range is"),
        })
        .expect_err("PROHIBITED must not carry a body");
        assert_eq!(err, RecordError::ProhibitedSourceCarriesBody);
    }

    #[test]
    fn public_ci_text_is_none_on_planted_prohibited_body() {
        let planted = SourceArtifact::plant_prohibited_with_extracted_text(
            "ASHRAE",
            "2025",
            "2025",
            "US",
            "ASHRAE copyright",
            "§6.5.1",
            "extracted standard text that must not leak into CI",
        );
        assert!(planted.carries_unlawful_extracted_text());
        assert!(
            planted.public_ci_text().is_none(),
            "public CI must not be able to read the planted body"
        );
    }

    #[test]
    fn textual_review_of_ashrae_is_rejected() {
        let source = SourceArtifact::try_new(SourceInput {
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
        .unwrap();
        let claim = ClaimRecord::try_new(
            "c1",
            "recommended envelope is class-dependent",
            "cooling; ASHRAE 90.4",
            vec!["recommended, not code".into()],
            &source,
        )
        .unwrap();
        let err = ReviewRecord::attest(
            "reviewer-1",
            true,
            "2026-08-15",
            ReviewKind::HumanTextual,
            &claim,
            &source,
        )
        .expect_err("textual review of ASHRAE");
        assert_eq!(err, RecordError::TextualReviewOfInaccessibleSource);
    }

    #[test]
    fn item_without_distractors_is_rejected() {
        let keyed = OptionBinding::unlinked("correct").unwrap();
        let err = ItemEvidence::try_new("i1", "stem?", keyed, vec![], vec!["cite-1".into()])
            .expect_err("no distractors");
        assert_eq!(err, RecordError::NoDistractors);
    }
}
