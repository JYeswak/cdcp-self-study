//! Load bank items and compute bank_hash (OQ-03).
//!
//! Item schema floors historically matched `scripts/verify_bank.py` (see
//! docs/TESTING.md parity table); the Python oracle is now retired.
//! with TWO recorded divergences, both in the strict direction:
//!
//! 1. **C1** — `status` is loaded and enforced here (unknown value = load
//!    error, default = draft) and is NOT checked by `verify_bank.py`.
//! 2. **C2** — unknown fields are a load error here (`deny_unknown_fields`);
//!    `verify_bank.py` ignores any key it does not know about.
//! 3. **G1** — `kind` is loaded and hashed here. The 804-item bank is
//!    `single-select` (the letter-MCQ lift). A non-letter kind is a loadable
//!    row that assemble must refuse, never flatten to A–D.
//!
//! The Rust side is the stricter one in both, so the parity table is a floor,
//! not an equality — do not read a green `verify_bank.py` as evidence that item
//! statuses are well-formed or that a bank file carries no unmodelled content.
#![forbid(unsafe_code)]

pub mod answer_key_skew;
pub mod construction_faults;
pub mod grounding_wave;
pub mod key_contradiction;
pub mod leftover_honesty;
pub mod mock40_module;
pub mod near_duplicate;
pub mod orphans;
pub mod paraphrase;
pub mod quote_or_drop;
pub mod required_tests;
pub mod tick_emitter;
pub mod validate_grounding;
pub mod verify_bank;
pub mod verify_content_lock;
pub mod verify_coverage;

pub use leftover_honesty::{
    audit_bank as leftover_honesty_audit, audit_item as leftover_honesty_item,
};
pub use mock40_module::{mock40_module_audit, Mock40Audit, MOCK40_CONTENT_MODULE};

use cdcp_core::{canonical_json, sha256_hex, ChoiceLetter, BANK_HASH_DOMAIN};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The shipped bank's DELIBERATE exclusions — every item id that is knowingly
/// not `approved`, and why.
///
/// # This list is an allowlist, not a mute button
///
/// Two tests read it — `cdcp_bank::load_real_bank_items` and
/// `cdcp_assemble::real_bank_is_all_approved_and_seed42_holds` — and both assert
/// set EQUALITY in **both** directions:
///
///   * a non-approved item that is not listed here is RED (an unexplained
///     retirement, or an item that silently lost its status line);
///   * an id listed here that is missing from the bank, or that is actually
///     `approved`, is **also** RED.
///
/// The second leg is the one that matters. An allowlist checked in only one
/// direction rots into a blanket: entries accumulate, nothing ever forces one
/// out, and eventually it excuses a retirement nobody decided on. Here a stale
/// entry is a build failure, so the list can only describe the bank as it is.
///
/// Adding a row is a bank decision, not a test fix: state which copy survives
/// and why in the retired file's own header before you add it here.
///
/// # Why every row below says the same thing
///
/// Twenty-five of these are one decision, not twenty-five. `cdcp_gate
/// near-duplicate-items` flagged 25 pairs in the approved pool; all 25 were read
/// by hand and all 25 were genuine (bd-near-duplicate-item-gate-i5v). Eighteen
/// put a `mock40-*` item — the wholesale import of `practice/PRACTICE-EXAM.md` —
/// against a module-bank item, and six put the `q2xx` "bank expansion" wave
/// against the items it duplicated. Two ungated bulk writes, not 25 authoring
/// slips, so the retirement rule is mechanical: **withdraw the copy from the bulk
/// wave, unless the published seed-42 form draws it, in which case the form
/// wins.** `m12-q219` is the one row where that second clause bit.
///
/// The rule, and what stops the next import regrowing the class, is recorded in
/// `bank/IMPORT-POLICY.md`. Each retired file's own header carries the three-part
/// argument for its pair (which copy the form draws, which explanation teaches,
/// provenance); the reasons here are the index, not the argument.
pub const SANCTIONED_RETIRED: &[(&str, &str)] = &[
    (
        "mock40-q40",
        "bd-near-duplicate-item-gate-i5v (C3): duplicate of bank-m14-q121 — same \
         proposition, key moved B->A, two distractors reworded. The import-side copy \
         from practice/PRACTICE-EXAM.md is withdrawn; the module-bank copy is what \
         the published seed-42 form draws. bank/items/m14-q040.toml carries the \
         three-part argument.",
    ),
    (
        "m12-q219",
        "bd-tetz pair 1/24: duplicate of bank-m12-q060 (answer 100%). THE ONE PAIR THE \
         FORM DECIDED — the seed-42 published form draws bank-m12-q060 at position 3, so \
         the survivor is fixed by the form and the q2xx expansion copy goes. \
         bank/items/m12-q219.toml.",
    ),
    (
        "m12-q217",
        "bd-tetz pair 2/24: duplicate of bank-m12-q075 (answer 92%, distractors 93%). \
         Neither is drawn; the q2xx expansion copy is withdrawn and the module-bank copy, \
         whose key and explanation both name the misuse being excluded, survives. \
         bank/items/m12-q217.toml.",
    ),
    (
        "mock40-q36",
        "bd-tetz pair 3/24: duplicate of bank-m13-q077 (answer 64%). Import copy from \
         practice/PRACTICE-EXAM.md withdrawn; its explanation restates its own key and two \
         of its distractors are answerable by absurdity. bank/items/m13-q036.toml.",
    ),
    (
        "mock40-q39",
        "bd-tetz pair 4/24: duplicate of bank-m14-q109 (answer 100%). Import copy \
         withdrawn; the survivor explains why leak detection is worth installing rather \
         than restating the key. bank/items/m14-q039.toml.",
    ),
    (
        "mock40-q37",
        "bd-tetz pair 5/24: duplicate of bank-m15-q138 (answer 90%). Import copy \
         withdrawn. ALSO MISFILED — module = 13 with topic m13-safety-components against \
         the survivor's module 15 / m15-documentation; retiring it hides that, so the \
         topic assignment is tracked on bd-mock40-q37-cross-module-topic-76vs. \
         bank/items/m13-q037.toml.",
    ),
    (
        "m05-q200",
        "bd-tetz pair 6/24: duplicate of m05-q135 (answer 61% — the loosest pair on the \
         list, at the calibrated cut). q2xx expansion copy withdrawn; the survivor's \
         explanation gives the physical definition rather than restating the key. \
         bank/items/m05-q200.toml.",
    ),
    (
        "mock40-q14",
        "bd-tetz pair 7/24: duplicate of m06-q047 (answer 72%). Import copy withdrawn; \
         the survivor quantifies the transfer and states the precondition that makes an \
         STS applicable. bank/items/m06-q014.toml.",
    ),
    (
        "mock40-q16",
        "bd-tetz pair 8/24: duplicate of m06-q053 (answer 91%). Import copy withdrawn; \
         the survivor states 2N in the sizing language the exam uses. \
         bank/items/m06-q016.toml.",
    ),
    (
        "mock40-q18",
        "bd-tetz pair 9/24: duplicate of m06-q058 (answer 92%). Import copy withdrawn; \
         the survivor names the AC->DC->AC mechanism and its first distractor is a real \
         VFD misconception rather than an absurdity. bank/items/m06-q018.toml.",
    ),
    (
        "mock40-q19",
        "bd-tetz pair 10/24: duplicate of m06-q065 (answer 71%). Import copy withdrawn; \
         the survivor states the causal reason a UPS is paired with a genset. \
         bank/items/m06-q019.toml.",
    ),
    (
        "mock40-q20",
        "bd-tetz pair 11/24: duplicate of m06-q074 (answer 73%). Import copy withdrawn; \
         the survivor names the operational benefit (reconfiguration agility) rather than \
         restating the key. bank/items/m06-q020.toml.",
    ),
    (
        "mock40-q21",
        "bd-tetz pair 12/24: duplicate of m06-q100 — answer 100% AND distractors 100%, \
         identical wrong answers with the key moved D->B. Import copy withdrawn; the \
         survivor adds the correction that PUE is not an availability design. \
         bank/items/m06-q021.toml.",
    ),
    (
        "mock40-q23",
        "bd-tetz pair 13/24: duplicate of m07-q044 (answer 100%). Import copy withdrawn; \
         the survivor asks the learner to discriminate a set and generalises to the \
         underlying property. bank/items/m07-q023.toml.",
    ),
    (
        "mock40-q24",
        "bd-tetz pair 14/24: duplicate of m08-q042 (answer 100%). THE ONE PAIR THE \
         EXPLANATION LEG DID NOT SEPARATE — both explanations are a bare restatement of \
         1U = 1.75 in. Decided by provenance (import copy withdrawn) plus the survivor's \
         fuller stem term. bank/items/m08-q024.toml.",
    ),
    (
        "mock40-q25",
        "bd-tetz pair 15/24: duplicate of m08-q049 (answer 100%, distractors 92%). Import \
         copy withdrawn on provenance; explanations are at parity. It also carried topic \
         m09-containment, which keeps 24 other approved items — no topic is orphaned. \
         bank/items/m08-q025.toml.",
    ),
    (
        "m09-q221",
        "bd-tetz pair 16/24: duplicate of m09-q108 (answer 80%). q2xx expansion copy \
         withdrawn; the survivor names the constraint that survives economization (heat \
         still has to be rejected). bank/items/m09-q221.toml.",
    ),
    (
        "mock40-q26",
        "bd-tetz pair 17/24: duplicate of m09-q124, and the only [reshuffled-clone] leg-2 \
         catch on the list (answer 46%, distractors 63%). Import copy withdrawn; the \
         survivor expands both acronyms, which is the entire content of the CRAC/CRAH \
         distinction. bank/items/m09-q026.toml.",
    ),
    (
        "m09-q226",
        "bd-tetz pair 18/24: duplicate of m09-q156 (answer 66%) with the keys on DIFFERENT \
         letters (A vs B) — the subclass a reviewer scanning answer keys cannot see. q2xx \
         expansion copy withdrawn. bank/items/m09-q226.toml.",
    ),
    (
        "m09-q301",
        "bd-epi-ecosystem-ms4j.12: ASHRAE TC 9.9 W-class body is not in the supportable \
         source pack; the item is retired pending sourcing, as recorded by its \
         blocked_on_sourcing header. bank/items/m09-q301.toml.",
    ),
    (
        "mock40-q29",
        "bd-tetz pair 19/24: duplicate of m09-q156 (answer 80%), which is also the survivor \
         of pair 18 — one proposition entered three times. Import copy withdrawn. \
         bank/items/m09-q029.toml.",
    ),
    (
        "mock40-q31",
        "bd-tetz pair 20/24: duplicate of m10-q100 (answer 90%). Import copy withdrawn; the \
         survivor states the dependency chain from water quality to heat rejection. \
         bank/items/m10-q031.toml.",
    ),
    (
        "mock40-q22",
        "bd-tetz pair 21/24: duplicate of m10-q101 (answer 100%). Import copy withdrawn; \
         the survivor says what kind of metric WUE is. It also carried topic \
         m06-sustainability, which keeps 7 other approved items. bank/items/m10-q022.toml.",
    ),
    (
        "mock40-q32",
        "bd-tetz pair 22/24: duplicate of m11-q101 (answer 100%). Import copy withdrawn; \
         the survivor states the contrast its distractors probe. bank/items/m11-q032.toml.",
    ),
    (
        "mock40-q33",
        "bd-tetz pair 23/24: duplicate of m11-q102 (answer 100%). Import copy withdrawn; \
         the survivor names the failure mode of not planning pathways. \
         bank/items/m11-q033.toml.",
    ),
    (
        "m11-q226",
        "bd-tetz pair 24/24: duplicate of m11-q139 (answer 100%, distractors 92%) at 16% \
         STEM similarity — the pair a stem-based detector could never have found. q2xx \
         expansion copy withdrawn; the survivor keeps both halves of the practice and is \
         bloom = evaluate. bank/items/m11-q226.toml.",
    ),
];

/// Adjudicate a loaded bank against [`SANCTIONED_RETIRED`], both directions.
///
/// `Ok(())` means: every non-approved item is a listed, deliberate retirement,
/// AND every listed id is present and genuinely non-approved. Returns the
/// discrepancy as a message otherwise.
///
/// Both anchoring tests call THIS, rather than each re-implementing the check.
/// Two lookalike implementations of one predicate is the bd-n1aj defect; the
/// point of the allowlist is undone if one caller checks a direction the other
/// does not.
pub fn sanctioned_retirement_report(bank: &Bank) -> Result<(), String> {
    let listed: BTreeMap<&str, &str> = SANCTIONED_RETIRED.iter().copied().collect();
    if listed.len() != SANCTIONED_RETIRED.len() {
        return Err("SANCTIONED_RETIRED contains a duplicate id".to_string());
    }

    let mut unexplained: Vec<&str> = bank
        .items
        .values()
        .filter(|i| !i.is_approved() && !listed.contains_key(i.id.as_str()))
        .map(|i| i.id.as_str())
        .collect();
    unexplained.sort_unstable();

    let mut stale: Vec<String> = Vec::new();
    for id in listed.keys() {
        match bank.get(id) {
            None => stale.push(format!("{id} (listed, but no such item in the bank)")),
            Some(item) if item.is_approved() => {
                stale.push(format!("{id} (listed, but it is approved)"))
            }
            Some(_) => {}
        }
    }

    if unexplained.is_empty() && stale.is_empty() {
        return Ok(());
    }
    Err(format!(
        "bank retirements disagree with cdcp_bank::SANCTIONED_RETIRED\n  \
         not approved and NOT listed (decide it, then list it): {unexplained:?}\n  \
         listed but no longer true (remove the row): {stale:?}"
    ))
}

/// Allowed `correct` letters — same as verify_bank.py `ALLOWED_CORRECT` (uppercase only).
const ALLOWED_CORRECT: &[&str] = &["A", "B", "C", "D"];

/// Bloom taxonomy — same as verify_bank.py `ALLOWED_BLOOM`.
const ALLOWED_BLOOM: &[&str] = &[
    "remember",
    "understand",
    "apply",
    "analyze",
    "evaluate",
    "create",
];

/// Default quantity_evidence set — same as verify_bank.py / fact_policy.toml.
const ALLOWED_QUANTITY_EVIDENCE: &[&str] = &[
    "free_url",
    "licensed_note",
    "qualitative_only",
    "exam_form_public",
];

/// Minimum explanation length — same as verify_bank.py.
const MIN_EXPLANATION_LEN: usize = 12;

#[derive(Debug, Error)]
pub enum BankError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml: {0}")]
    Toml(String),
    #[error("json: {0}")]
    Json(String),
    #[error("empty bank")]
    Empty,
    #[error("item {0}: {1}")]
    Item(String, String),
    #[error("core: {0}")]
    Core(#[from] cdcp_core::CoreError),
}

/// Editorial lifecycle of a bank item (C1).
///
/// # Why the default is `Draft`
///
/// The default is the status that CANNOT reach a learner. Before C1 every
/// loaded item was eligible for assembly, so anything authored — a stub, a
/// near-duplicate, a retired item kept for reference — landed in the same
/// undifferentiated pool a mock exam draws from. Defaulting to `Approved`
/// would preserve exactly that defect behind a new field name: forgetting the
/// field would silently publish the item. Approval is therefore a positive,
/// recorded act in the item file; silence means draft.
///
/// An unrecognised value is a **load error**, never a coerced default —
/// serde rejects unknown variants, so `status = "published"` fails the load
/// rather than quietly becoming something else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    /// Authored but not cleared for assessment. Never assembled.
    #[default]
    Draft,
    /// Cleared for assessment. The only status `cdcp_assemble` may draw.
    Approved,
    /// Withdrawn from assessment; kept for history/regeneration. Never assembled.
    Retired,
}

impl ItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemStatus::Draft => "draft",
            ItemStatus::Approved => "approved",
            ItemStatus::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Assessment kind on a bank row (G1 / `bd-hardening-g-assess-64t.1`).
///
/// The shipped 804 items are [`ItemKind::SingleSelect`] — the letter-MCQ
/// lift. This tag is hashed: a kind flip changes what assemble will admit
/// (letter kinds only) without changing stem or key, so it belongs in the
/// content address. Serde default is `single-select` so a file that has
/// not yet been migrated still loads as the historical kind rather than
/// becoming an unrecognised load error. The migrate writes the field
/// explicitly on every file.
///
/// An unrecognised value is a **load error**. New kinds are added here
/// before they may appear in a bank file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemKind {
    /// Four-option single-select (letter-MCQ lift). The only kind assemble
    /// will present as A–D.
    #[default]
    SingleSelect,
    MultiSelect,
    Ordering,
    NumericRange,
    TopologySelection,
    ProceduralSequence,
}

impl ItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemKind::SingleSelect => "single-select",
            ItemKind::MultiSelect => "multi-select",
            ItemKind::Ordering => "ordering",
            ItemKind::NumericRange => "numeric-range",
            ItemKind::TopologySelection => "topology-selection",
            ItemKind::ProceduralSequence => "procedural-sequence",
        }
    }

    /// True iff assemble may present this row as a letter form.
    pub fn is_letter_form(&self) -> bool {
        matches!(self, ItemKind::SingleSelect)
    }
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A bank item.
///
/// # Unknown-field policy: REJECT (C2)
///
/// `deny_unknown_fields` is deliberate and load-bearing. `bank_hash` is a
/// content address over `hash_payload()`; a field serde silently dropped would
/// be file content that no hash covers, which is exactly the C2 defect at a
/// smaller scale. Measured 2026-08-14 before this bead: all 804 items carried
/// `objective_ids` and six carried `tags`, and **neither field existed on this
/// struct** — serde discarded both on load, so no hash, no gate, and no test
/// could ever see them. Under this policy a new field in a bank file is a load
/// error naming the field until someone models it here and decides, explicitly,
/// whether it belongs in the content address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankItem {
    pub id: String,
    pub module: u32,
    pub stem: String,
    pub choices: Vec<String>,
    pub correct: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub topic_ids: Vec<String>,
    /// Learning-objective ids this item assesses (C2). Hashed.
    #[serde(default)]
    pub objective_ids: Vec<String>,
    /// Evidence backing this item — ids into the citation registry (C2). Hashed.
    #[serde(default)]
    pub citation_ids: Vec<String>,
    /// Free-form editorial labels (C2). Hashed: they are file content.
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub bloom: String,
    #[serde(default)]
    pub source_class: String,
    #[serde(default)]
    pub quantity_evidence: String,
    /// Editorial lifecycle (C1). Defaults to [`ItemStatus::Draft`]; only
    /// `approved` items are eligible for assembly.
    #[serde(default)]
    pub status: ItemStatus,
    /// Assessment kind (G1). Default [`ItemKind::SingleSelect`]. Hashed.
    #[serde(default)]
    pub kind: ItemKind,
}

impl BankItem {
    /// True iff this item may be drawn into an assessment (C1).
    pub fn is_approved(&self) -> bool {
        self.status == ItemStatus::Approved
    }

    pub fn correct_letter(&self) -> Result<ChoiceLetter, BankError> {
        ChoiceLetter::parse(&self.correct)
            .map_err(|e| BankError::Item(self.id.clone(), e.to_string()))
    }

    /// Per-item schema floors aligned with the former Python bank oracle.
    pub fn validate(&self) -> Result<(), BankError> {
        let label = if self.id.is_empty() {
            "<missing-id>".to_string()
        } else {
            self.id.clone()
        };

        if self.id.trim().is_empty() {
            return Err(BankError::Item(label, "missing id".into()));
        }

        if self.stem.trim().is_empty() {
            return Err(BankError::Item(self.id.clone(), "empty stem".into()));
        }

        if self.choices.len() != 4 {
            return Err(BankError::Item(
                self.id.clone(),
                "choices must be length 4".into(),
            ));
        }
        if self.choices.iter().any(|c| c.trim().is_empty()) {
            return Err(BankError::Item(self.id.clone(), "empty choice text".into()));
        }

        // Uppercase A–D only (verify_bank ALLOWED_CORRECT); reject lowercase.
        if !ALLOWED_CORRECT.contains(&self.correct.as_str()) {
            return Err(BankError::Item(
                self.id.clone(),
                format!("correct must be A-D, got {:?}", self.correct),
            ));
        }
        // Keep ChoiceLetter parse as a second check (should always pass for A–D).
        let _ = self.correct_letter()?;

        if self.explanation.trim().len() < MIN_EXPLANATION_LEN {
            return Err(BankError::Item(
                self.id.clone(),
                "explanation too short".into(),
            ));
        }

        if self.topic_ids.is_empty() {
            return Err(BankError::Item(
                self.id.clone(),
                "topic_ids required".into(),
            ));
        }

        if self.source_class != "original" {
            return Err(BankError::Item(
                self.id.clone(),
                format!("source_class must be original, got {:?}", self.source_class),
            ));
        }

        if !ALLOWED_QUANTITY_EVIDENCE.contains(&self.quantity_evidence.as_str()) {
            return Err(BankError::Item(
                self.id.clone(),
                format!("bad quantity_evidence {:?}", self.quantity_evidence),
            ));
        }

        if !ALLOWED_BLOOM.contains(&self.bloom.as_str()) {
            return Err(BankError::Item(
                self.id.clone(),
                format!("bad bloom {:?}", self.bloom),
            ));
        }

        Ok(())
    }

    /// Canonical fields for hashing — **every** modelled field (C2).
    ///
    /// # What changed and why (C2, bd-hardening-c-status-hzs.2)
    ///
    /// This payload used to omit `objective_ids`, evidence/citation ids and
    /// `status`, so `bank_hash` was a content address that could not see the
    /// evidence behind an item nor whether that item was allowed to reach a
    /// learner. C1 made the second one load-bearing: assembly draws
    /// `approved` items only, so flipping one item `approved` → `draft`
    /// changes what a learner is assessed on. Measured 2026-08-14 on
    /// `m04-q129`: that flip left `bank_hash` **byte-identical** while moving
    /// 38 of 40 positions in the seed-42 exam. A hash that misses that is not
    /// a content address; it is a decoration.
    ///
    /// # Invariants
    ///
    /// * **Total over modelled fields.** Every field of [`BankItem`] appears
    ///   here. Combined with `deny_unknown_fields` on the struct, no content
    ///   in a bank file can be outside the hash.
    /// * **Set-valued lists are sorted** (`topic_ids`, `objective_ids`,
    ///   `citation_ids`, `tags`) — reordering them is a cosmetic edit and must
    ///   not move the hash. `choices` is **not** sorted: its order is the
    ///   presentation order that `correct` indexes into, so permuting it is a
    ///   semantic change.
    /// * **Deterministic.** `BTreeMap` keys plus `canonical_json` — no
    ///   iteration over an unordered container anywhere on this path.
    pub fn hash_payload(&self) -> BTreeMap<String, serde_json::Value> {
        /// Set-valued list → sorted, so reordering is not a content change.
        fn sorted(v: &[String]) -> Vec<String> {
            let mut out = v.to_vec();
            out.sort();
            out
        }

        let mut m = BTreeMap::new();
        m.insert("id".into(), serde_json::json!(self.id));
        m.insert("kind".into(), serde_json::json!(self.kind.as_str()));
        m.insert("module".into(), serde_json::json!(self.module));
        m.insert("stem".into(), serde_json::json!(self.stem));
        m.insert("choices".into(), serde_json::json!(self.choices));
        m.insert("correct".into(), serde_json::json!(self.correct));
        m.insert("explanation".into(), serde_json::json!(self.explanation));
        m.insert(
            "topic_ids".into(),
            serde_json::json!(sorted(&self.topic_ids)),
        );
        m.insert(
            "objective_ids".into(),
            serde_json::json!(sorted(&self.objective_ids)),
        );
        m.insert(
            "citation_ids".into(),
            serde_json::json!(sorted(&self.citation_ids)),
        );
        m.insert("tags".into(), serde_json::json!(sorted(&self.tags)));
        m.insert("bloom".into(), serde_json::json!(self.bloom));
        m.insert("source_class".into(), serde_json::json!(self.source_class));
        m.insert(
            "quantity_evidence".into(),
            serde_json::json!(self.quantity_evidence),
        );
        m.insert("status".into(), serde_json::json!(self.status.as_str()));
        m
    }
}

#[derive(Debug, Clone)]
pub struct Bank {
    pub items: BTreeMap<String, BankItem>,
    pub bank_hash: String,
}

impl Bank {
    pub fn load_dir(dir: &Path) -> Result<Self, BankError> {
        let mut items: BTreeMap<String, BankItem> = BTreeMap::new();
        if !dir.is_dir() {
            return Err(BankError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("not a directory: {}", dir.display()),
            )));
        }
        // Fail closed on read_dir entry errors (do not filter_map e.ok()).
        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) == Some("toml") {
                paths.push(path);
            }
        }
        paths.sort();
        for path in paths {
            let text = fs::read_to_string(&path)?;
            let item: BankItem = toml::from_str(&text)
                .map_err(|e| BankError::Toml(format!("{}: {e}", path.display())))?;
            item.validate()?;
            if items.insert(item.id.clone(), item.clone()).is_some() {
                return Err(BankError::Item(item.id, "duplicate id".into()));
            }
        }
        if items.is_empty() {
            return Err(BankError::Empty);
        }
        let bank_hash = compute_bank_hash(&items)?;
        Ok(Self { items, bank_hash })
    }

    pub fn get(&self, id: &str) -> Option<&BankItem> {
        self.items.get(id)
    }

    /// Build a bank from an in-memory item list (WASM / JSON dual-path).
    /// Validates each item, rejects duplicates and empty sets, recomputes `bank_hash`.
    pub fn from_items(items: impl IntoIterator<Item = BankItem>) -> Result<Self, BankError> {
        let mut map: BTreeMap<String, BankItem> = BTreeMap::new();
        for item in items {
            item.validate()?;
            if map.insert(item.id.clone(), item.clone()).is_some() {
                return Err(BankError::Item(item.id, "duplicate id".into()));
            }
        }
        if map.is_empty() {
            return Err(BankError::Empty);
        }
        let bank_hash = compute_bank_hash(&map)?;
        Ok(Self {
            items: map,
            bank_hash,
        })
    }

    /// Deserialize bank items from JSON (array of items, or `{"items":[...]}`).
    pub fn from_json_str(json: &str) -> Result<Self, BankError> {
        #[derive(Deserialize)]
        struct Wrapper {
            items: Vec<BankItem>,
        }
        let items: Vec<BankItem> = match serde_json::from_str::<Wrapper>(json) {
            Ok(w) => w.items,
            Err(_) => serde_json::from_str(json)
                .map_err(|e| BankError::Json(format!("bank json: {e}")))?,
        };
        Self::from_items(items)
    }

    /// Export items as a JSON array (stable BTreeMap key order).
    pub fn to_json_items(&self) -> Result<String, BankError> {
        let list: Vec<&BankItem> = self.items.values().collect();
        serde_json::to_string(&list).map_err(|e| BankError::Json(format!("bank json encode: {e}")))
    }
}

/// Content address of a bank.
///
/// # Anti-vacuous
///
/// An empty item set is an **ERROR**, not a hash. It used to return
/// `sha256(BANK_HASH_DOMAIN)` — a well-formed 64-hex string that any caller
/// would pin, compare, and report green, certifying a bank that contains
/// nothing. `Bank::load_dir` and `Bank::from_items` already refused empty
/// inputs, so the hole was only reachable through this function directly; it is
/// closed here so the guarantee belongs to the hash, not to its two callers.
pub fn compute_bank_hash(items: &BTreeMap<String, BankItem>) -> Result<String, BankError> {
    if items.is_empty() {
        return Err(BankError::Empty);
    }
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(BANK_HASH_DOMAIN);
    // BTreeMap values() iteration is sorted by key (item id)
    for item in items.values() {
        let payload = item.hash_payload();
        let bytes = canonical_json(&payload)?;
        buf.extend_from_slice(&bytes);
        buf.push(0);
    }
    Ok(sha256_hex(&buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn valid_item_toml(id: &str, correct: &str) -> String {
        format!(
            r#"
id = "{id}"
module = 1
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "{correct}"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "approved"
"#
        )
    }

    fn write_item(dir: &Path, name: &str, body: &str) {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{body}").unwrap();
    }

    #[test]
    fn empty_bank_errors() {
        let dir = std::env::temp_dir().join(format!("cdcp-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Empty));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hash_changes_when_correct_flips() {
        let dir = std::env::temp_dir().join(format!("cdcp-hash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(&dir, "t.toml", &valid_item_toml("t1", "A"));
        let b1 = Bank::load_dir(&dir).unwrap();
        write_item(&dir, "t.toml", &valid_item_toml("t1", "B"));
        let b2 = Bank::load_dir(&dir).unwrap();
        assert_ne!(b1.bank_hash, b2.bank_hash);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_empty_stem() {
        let dir = std::env::temp_dir().join(format!("cdcp-stem-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "bad-stem"
module = 1
stem = "   "
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("empty stem")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_empty_choice() {
        let dir = std::env::temp_dir().join(format!("cdcp-choice-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "bad-choice"
module = 1
stem = "stem"
choices = ["a","","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("empty choice")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_lowercase_correct() {
        // verify_bank.py ALLOWED_CORRECT is uppercase-only
        let dir = std::env::temp_dir().join(format!("cdcp-lc-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(&dir, "t.toml", &valid_item_toml("t1", "a"));
        // valid_item_toml uses correct as-is; force lowercase
        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "a"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("correct must be A-D")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_bad_bloom_and_quantity_evidence() {
        let dir = std::env::temp_dir().join(format!("cdcp-bloom-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "stem text"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "not-a-level"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("bad bloom")));

        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "stem text"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "made_up"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("bad quantity_evidence")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_empty_topic_ids() {
        let dir = std::env::temp_dir().join(format!("cdcp-topics-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "t1"
module = 1
stem = "stem text"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = []
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        assert!(matches!(err, BankError::Item(_, msg) if msg.contains("topic_ids")));
        let _ = fs::remove_dir_all(&dir);
    }

    /// Integration: load the real repo bank/items (path via CARGO_MANIFEST_DIR).
    #[test]
    fn load_real_bank_items() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bank/items");
        assert!(
            dir.is_dir(),
            "expected real bank at {} (run tests from repo checkout)",
            dir.display()
        );
        let bank = Bank::load_dir(&dir).expect("real bank should load under unified validation");
        assert!(
            bank.items.len() >= 40,
            "expected a real pool, got {}",
            bank.items.len()
        );
        assert_eq!(bank.bank_hash.len(), 64);
        // Spot-check fields that verify_bank also requires
        for item in bank.items.values() {
            assert!(!item.stem.trim().is_empty());
            assert_eq!(item.choices.len(), 4);
            assert!(ALLOWED_CORRECT.contains(&item.correct.as_str()));
            assert!(ALLOWED_BLOOM.contains(&item.bloom.as_str()));
            assert!(ALLOWED_QUANTITY_EVIDENCE.contains(&item.quantity_evidence.as_str()));
            assert!(!item.topic_ids.is_empty());
            assert_eq!(item.source_class, "original");
        }
        // C1 migration: every shipped item carries an EXPLICIT status line.
        // This is the anchoring assertion — it reads the real 804-item corpus,
        // not a fixture we wrote for ourselves.
        let text_dir = &dir;
        let mut missing_explicit: Vec<String> = Vec::new();
        for entry in fs::read_dir(text_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|x| x.to_str()) != Some("toml") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            if !text.lines().any(|l| l.trim_start().starts_with("status")) {
                missing_explicit.push(path.display().to_string());
            }
        }
        assert!(
            missing_explicit.is_empty(),
            "items relying on the implicit draft default (explicit status required): {:?}",
            &missing_explicit[..missing_explicit.len().min(5)]
        );
        // G1 migrate: every shipped item carries an EXPLICIT kind line and
        // loads as single-select. A file relying on the serde default is
        // not "migrated". A non-letter kind in this corpus is flatten risk.
        let mut missing_kind: Vec<String> = Vec::new();
        let mut not_single: Vec<String> = Vec::new();
        for entry in fs::read_dir(text_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|x| x.to_str()) != Some("toml") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            if !text.lines().any(|l| l.trim_start().starts_with("kind")) {
                missing_kind.push(path.display().to_string());
            }
        }
        assert!(
            missing_kind.is_empty(),
            "items missing explicit kind= (G1 migrate incomplete): {:?}",
            &missing_kind[..missing_kind.len().min(5)]
        );
        for item in bank.items.values() {
            if item.kind != ItemKind::SingleSelect {
                not_single.push(format!("{}={}", item.id, item.kind));
            }
        }
        assert!(
            not_single.is_empty(),
            "shipped bank must be single-select after G1 migrate: {:?}",
            &not_single[..not_single.len().min(5)]
        );
        // The shipped bank is approved-only EXCEPT for the deliberate
        // retirements named in SANCTIONED_RETIRED. Checked both directions, so
        // neither an unexplained retirement nor a stale allowlist row passes.
        if let Err(msg) = sanctioned_retirement_report(&bank) {
            panic!("{msg}");
        }
        // bd-mock40-q37-cross-module-topic-76vs: every mock40-* stem was
        // re-read against `module`. Zero live misfiles. The scan is this
        // function — an empty mock40 set or an unreviewed new id is RED.
        let mock40 =
            crate::mock40_module_audit(bank.items.values()).expect("mock40 module-vs-stem audit");
        assert_eq!(
            mock40.checked,
            crate::MOCK40_CONTENT_MODULE.len(),
            "scan must name every mock40-* item"
        );
        assert_eq!(mock40.live_misfiles, 0, "approved mock40 misfile leaked");
        // bd-curriculum-truth-ebrr.30 / .32: TEMP restore of the old q154
        // singular-root-cause stem or the old mock40-q04 peer-bucket
        // explanation is RED. m01-q210 stays approved.
        if let Err(msg) = crate::leftover_honesty::audit_bank(bank.items.values()) {
            panic!("{msg}");
        }
    }

    // --- C1: item status ---------------------------------------------------

    #[test]
    fn status_defaults_to_draft_when_absent() {
        // Fail-safe direction: an item file that forgets the field must NOT be
        // assessable. Silence means draft, never approved.
        let dir = std::env::temp_dir().join(format!("cdcp-status-default-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "no-status"
module = 1
stem = "stem text here"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
"#,
        );
        let bank = Bank::load_dir(&dir).unwrap();
        let item = bank.get("no-status").unwrap();
        assert_eq!(item.status, ItemStatus::Draft);
        assert!(!item.is_approved());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_status_is_a_load_error_not_a_silent_default() {
        let dir = std::env::temp_dir().join(format!("cdcp-status-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(
            &dir,
            "t.toml",
            r#"
id = "bad-status"
module = 1
stem = "stem text here"
choices = ["a","b","c","d"]
correct = "A"
explanation = "because reasons here"
topic_ids = ["m01-importance"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "published"
"#,
        );
        let err = Bank::load_dir(&dir).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unknown variant") && msg.contains("published"),
            "expected an unknown-variant load error naming the bad value, got: {msg}"
        );
        // And the same rejection on the JSON dual path (WASM bank packs).
        let jerr = Bank::from_json_str(
            r#"[{"id":"x","module":1,"stem":"s","choices":["a","b","c","d"],
                 "correct":"A","explanation":"because reasons here",
                 "topic_ids":["t"],"bloom":"understand","source_class":"original",
                 "quantity_evidence":"qualitative_only","status":"published"}]"#,
        )
        .unwrap_err();
        assert!(
            jerr.to_string().contains("unknown variant"),
            "json path must reject unknown status too, got: {jerr}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn status_roundtrips_through_json_lowercase() {
        for (s, txt) in [
            (ItemStatus::Draft, "\"draft\""),
            (ItemStatus::Approved, "\"approved\""),
            (ItemStatus::Retired, "\"retired\""),
        ] {
            assert_eq!(serde_json::to_string(&s).unwrap(), txt);
            assert_eq!(serde_json::from_str::<ItemStatus>(txt).unwrap(), s);
            assert_eq!(s.as_str(), txt.trim_matches('"'));
        }
    }

    // --- C2: bank_hash covers every load-bearing field ---------------------
    //
    // bd-hardening-c-status-hzs.2. Before this bead `hash_payload` omitted
    // `objective_ids`, evidence/citation ids and `status`, so two banks that
    // differ in what a learner is assessed on — and in what backs it — hashed
    // identically. The known-bad legs below are the assertions that defect
    // could not survive; the known-GOOD legs are what stops the cure from
    // becoming a hash that moves on cosmetic edits, which is how a pin decays
    // into something people regenerate reflexively.

    /// One item file, every field explicit, so a leg can vary exactly one thing.
    #[allow(clippy::too_many_arguments)]
    fn item_toml(
        id: &str,
        topic_ids: &str,
        objective_ids: &str,
        citation_ids: &str,
        tags: &str,
        status: &str,
    ) -> String {
        format!(
            r#"
id = "{id}"
kind = "single-select"
module = 3
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "B"
explanation = "because reasons here"
topic_ids = {topic_ids}
objective_ids = {objective_ids}
citation_ids = {citation_ids}
tags = {tags}
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "{status}"
"#
        )
    }

    /// Load a one-item bank from a fresh temp dir and return its `bank_hash`.
    fn hash_of(tag: &str, body: &str) -> String {
        let dir = std::env::temp_dir().join(format!("cdcp-c2-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_item(&dir, "t.toml", body);
        let h = Bank::load_dir(&dir).expect("bank should load").bank_hash;
        let _ = fs::remove_dir_all(&dir);
        h
    }

    /// **THE C2 ASSERTION.** Flipping one item `approved` → `draft` MUST move
    /// `bank_hash`.
    ///
    /// C1 restricted assembly to `approved` items, so this flip changes what
    /// can reach a learner. Measured 2026-08-14 on the live bank, before this
    /// bead: the flip produced a byte-identical `bank_hash` while changing the
    /// seed-42 selection in **38 of 40 positions**. Remove `status` from
    /// `hash_payload` and this test goes RED — that pair is the meta-test.
    /// Named CHARTER pair: `tests/c2_charter_pair.rs`, driven by
    /// `scripts/selftest_reconstructed.sh` via `cdcp_restore_safe`. Re-run
    /// 2026-08-15 (`bd-metatest-rerun-blocked-yhg6`): mutate 101, restore 0,
    /// artifact mtime moved.
    #[test]
    fn status_flip_moves_bank_hash() {
        let approved = hash_of(
            "st-approved",
            &item_toml("s1", "[\"t1\"]", "[]", "[]", "[]", "approved"),
        );
        let draft = hash_of(
            "st-draft",
            &item_toml("s1", "[\"t1\"]", "[]", "[]", "[]", "draft"),
        );
        let retired = hash_of(
            "st-retired",
            &item_toml("s1", "[\"t1\"]", "[]", "[]", "[]", "retired"),
        );

        assert_ne!(
            approved, draft,
            "approved -> draft MUST move bank_hash: assembly draws approved items only (C1), \
             so this flip changes what a learner can be assessed on. A content address that \
             cannot see it is not a content address."
        );
        assert_ne!(
            approved, retired,
            "approved -> retired MUST move bank_hash (withdrawn from assessment)"
        );
        assert_ne!(
            draft, retired,
            "draft and retired are distinct editorial states and must hash distinctly"
        );
    }

    /// G1: flipping kind must move bank_hash. Assemble admits letter
    /// kinds only; a kind flip changes what can reach a learner even
    /// when stem and key stay put.
    #[test]
    fn kind_change_moves_bank_hash() {
        let single = hash_of(
            "kind-ss",
            &item_toml("k1", "[\"t1\"]", "[]", "[]", "[]", "approved"),
        );
        let multi = hash_of(
            "kind-ms",
            r#"
id = "k1"
kind = "multi-select"
module = 3
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "B"
explanation = "because reasons here"
topic_ids = ["t1"]
objective_ids = []
citation_ids = []
tags = []
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "approved"
"#,
        );
        assert_ne!(
            single, multi,
            "single-select -> multi-select MUST move bank_hash: assemble refuses \
             the second kind, so this flip changes what a learner can be assessed on"
        );
    }

    /// Evidence changes must reach the content address: objective ids.
    #[test]
    fn objective_ids_change_moves_bank_hash() {
        let none = hash_of(
            "obj-none",
            &item_toml("o1", "[\"t1\"]", "[]", "[]", "[]", "approved"),
        );
        let one = hash_of(
            "obj-one",
            &item_toml("o1", "[\"t1\"]", "[\"lo-01\"]", "[]", "[]", "approved"),
        );
        let other = hash_of(
            "obj-other",
            &item_toml("o1", "[\"t1\"]", "[\"lo-02\"]", "[]", "[]", "approved"),
        );
        assert_ne!(none, one, "adding an objective_id must move bank_hash");
        assert_ne!(one, other, "changing an objective_id must move bank_hash");
    }

    /// Evidence changes must reach the content address: citation ids.
    #[test]
    fn citation_ids_change_moves_bank_hash() {
        let none = hash_of(
            "cit-none",
            &item_toml("c1", "[\"t1\"]", "[]", "[]", "[]", "approved"),
        );
        let one = hash_of(
            "cit-one",
            &item_toml(
                "c1",
                "[\"t1\"]",
                "[]",
                "[\"uptime-tier-topology-2026\"]",
                "[]",
                "approved",
            ),
        );
        let other = hash_of(
            "cit-other",
            &item_toml(
                "c1",
                "[\"t1\"]",
                "[]",
                "[\"some-other-citation\"]",
                "[]",
                "approved",
            ),
        );
        assert_ne!(none, one, "attaching evidence must move bank_hash");
        assert_ne!(
            one, other,
            "swapping the evidence behind an item must move bank_hash"
        );
    }

    /// `tags` is file content too — six shipped items carry it, and before C2
    /// the struct did not even model it.
    #[test]
    fn tags_change_moves_bank_hash() {
        let none = hash_of(
            "tag-none",
            &item_toml("g1", "[\"t1\"]", "[]", "[]", "[]", "approved"),
        );
        let tagged = hash_of(
            "tag-some",
            &item_toml(
                "g1",
                "[\"t1\"]",
                "[]",
                "[]",
                "[\"runbook\",\"vignette\"]",
                "approved",
            ),
        );
        assert_ne!(none, tagged, "adding tags must move bank_hash");
    }

    /// Unknown-field policy is **REJECT**, on both load paths.
    ///
    /// An ignored field is file content outside the hash — the C2 defect in
    /// miniature. Removing `deny_unknown_fields` turns this RED.
    #[test]
    fn unknown_field_is_a_load_error() {
        let dir = std::env::temp_dir().join(format!("cdcp-c2-unknown-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut body = item_toml("u1", "[\"t1\"]", "[]", "[]", "[]", "approved");
        body.push_str("evidence_url = \"https://example.invalid/spec\"\n");
        write_item(&dir, "t.toml", &body);
        let err = Bank::load_dir(&dir).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unknown field") && msg.contains("evidence_url"),
            "an unmodelled field must be a load error naming the field \
             (silently ignoring it puts file content outside bank_hash), got: {msg}"
        );
        let _ = fs::remove_dir_all(&dir);

        let jerr = Bank::from_json_str(
            r#"[{"id":"x","module":1,"stem":"stem text","choices":["a","b","c","d"],
                 "correct":"A","explanation":"because reasons here","topic_ids":["t"],
                 "bloom":"understand","source_class":"original",
                 "quantity_evidence":"qualitative_only","status":"approved",
                 "evidence_url":"https://example.invalid/spec"}]"#,
        )
        .unwrap_err();
        assert!(
            jerr.to_string().contains("unknown field"),
            "the JSON dual path (WASM bank packs) must reject unknown fields too, got: {jerr}"
        );
    }

    /// **Structural**: the hash payload covers every modelled field.
    ///
    /// Compares `hash_payload`'s key set against the item's own serde field
    /// set, so adding a field to [`BankItem`] without adding it to
    /// `hash_payload` is a RED test rather than a silent hole. This is the
    /// assertion that makes "C2 is fixed" durable instead of a one-time edit.
    #[test]
    fn hash_payload_covers_every_modelled_field() {
        let item = sample_item("x1", 1, "A");
        let serialized = serde_json::to_value(&item).unwrap();
        let mut struct_fields: Vec<String> = serialized
            .as_object()
            .expect("BankItem serializes to an object")
            .keys()
            .cloned()
            .collect();
        struct_fields.sort();
        let mut payload_fields: Vec<String> = item.hash_payload().keys().cloned().collect();
        payload_fields.sort();

        assert!(
            !struct_fields.is_empty(),
            "anti-vacuous: zero fields scanned is an ERROR, not a pass"
        );
        assert_eq!(
            payload_fields, struct_fields,
            "every modelled BankItem field must be in hash_payload — a field outside the \
             content address is content the bank_hash pin cannot see (C2)"
        );
    }

    /// Known-GOOD: cosmetic file edits must NOT move `bank_hash`.
    ///
    /// Key order, whitespace, array formatting, and comments are not content.
    /// A hash that moves on these is a hash people learn to regenerate
    /// reflexively, and a reflexively regenerated pin means nothing.
    #[test]
    fn cosmetic_edits_do_not_move_bank_hash() {
        let plain = hash_of(
            "cos-plain",
            &item_toml("k1", "[\"t1\",\"t2\"]", "[]", "[]", "[]", "approved"),
        );
        let reformatted = hash_of(
            "cos-reformatted",
            r#"
# A comment that is not content.
status   =    "approved"
quantity_evidence = "qualitative_only"
source_class = "original"
bloom = "understand"

tags = []
citation_ids = []
objective_ids = []
topic_ids = [
  "t1",
  "t2",
]
explanation = "because reasons here"
correct = "B"
choices = [
  "a",
  "b",
  "c",
  "d",
]
stem  =  "A valid stem for testing"
module = 3
id = "k1"
"#,
        );
        assert_eq!(
            plain, reformatted,
            "reordering keys, re-wrapping arrays, adding comments and whitespace are cosmetic; \
             bank_hash must not move"
        );
    }

    /// Known-GOOD: set-valued lists are sets — permuting them is cosmetic.
    #[test]
    fn reordering_set_valued_lists_does_not_move_bank_hash() {
        let a = hash_of(
            "perm-a",
            &item_toml(
                "p1",
                "[\"t1\",\"t2\"]",
                "[\"lo-01\",\"lo-02\"]",
                "[\"c-a\",\"c-b\"]",
                "[\"runbook\",\"vignette\"]",
                "approved",
            ),
        );
        let b = hash_of(
            "perm-b",
            &item_toml(
                "p1",
                "[\"t2\",\"t1\"]",
                "[\"lo-02\",\"lo-01\"]",
                "[\"c-b\",\"c-a\"]",
                "[\"vignette\",\"runbook\"]",
                "approved",
            ),
        );
        assert_eq!(
            a, b,
            "topic_ids / objective_ids / citation_ids / tags are sets; permuting them must not \
             move bank_hash"
        );
    }

    /// Known-GOOD: item order on disk is not content.
    ///
    /// `load_dir` reads files in sorted path order but keys them by item id, so
    /// renaming the files that carry the items must not move the hash. This is
    /// the on-disk twin of `bank_hash_independent_of_insert_order`.
    #[test]
    fn item_file_order_does_not_move_bank_hash() {
        let one = item_toml("z-second", "[\"t1\"]", "[]", "[]", "[]", "approved");
        let two = item_toml("a-first", "[\"t2\"]", "[]", "[]", "[]", "approved");

        let mk = |tag: &str, first: (&str, &str), second: (&str, &str)| {
            let dir = std::env::temp_dir().join(format!("cdcp-c2-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            write_item(&dir, first.0, first.1);
            write_item(&dir, second.0, second.1);
            let h = Bank::load_dir(&dir).unwrap().bank_hash;
            let _ = fs::remove_dir_all(&dir);
            h
        };

        let forward = mk("ord-fwd", ("01.toml", &one), ("02.toml", &two));
        let reverse = mk("ord-rev", ("01.toml", &two), ("02.toml", &one));
        assert_eq!(
            forward, reverse,
            "which file an item lives in is not content; bank_hash must not move"
        );
    }

    /// `choices` order IS content — it must NOT be sorted away.
    ///
    /// `correct` indexes into the presentation order, so permuting `choices`
    /// changes which answer is right. Guards against "sort everything".
    #[test]
    fn choices_order_moves_bank_hash() {
        let a = hash_of(
            "ch-a",
            r#"
id = "h1"
module = 3
stem = "A valid stem for testing"
choices = ["a","b","c","d"]
correct = "B"
explanation = "because reasons here"
topic_ids = ["t1"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "approved"
"#,
        );
        let b = hash_of(
            "ch-b",
            r#"
id = "h1"
module = 3
stem = "A valid stem for testing"
choices = ["b","a","c","d"]
correct = "B"
explanation = "because reasons here"
topic_ids = ["t1"]
bloom = "understand"
source_class = "original"
quantity_evidence = "qualitative_only"
status = "approved"
"#,
        );
        assert_ne!(
            a, b,
            "choices order is the presentation order `correct` indexes into — permuting it \
             changes the right answer and must move bank_hash"
        );
    }

    /// Anti-vacuous: hashing an empty bank is an ERROR, not a hash.
    #[test]
    fn empty_bank_is_an_error_not_a_hash() {
        let empty: BTreeMap<String, BankItem> = BTreeMap::new();
        let err = compute_bank_hash(&empty).unwrap_err();
        assert!(
            matches!(err, BankError::Empty),
            "an empty item set must not produce a well-formed 64-hex digest that a caller \
             would pin and report green, got: {err}"
        );
        assert!(matches!(
            Bank::from_items(Vec::new()).unwrap_err(),
            BankError::Empty
        ));
    }

    // --- proptest (bd-334): bank_hash independent of BTreeMap insert order ---

    use proptest::prelude::*;

    fn sample_item(id: &str, module: u32, correct: &str) -> BankItem {
        BankItem {
            id: id.to_string(),
            module,
            stem: format!("stem-{id}"),
            choices: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            correct: correct.to_string(),
            explanation: "because reasons here".into(),
            topic_ids: vec![format!("t-{id}")],
            objective_ids: Vec::new(),
            citation_ids: Vec::new(),
            tags: Vec::new(),
            bloom: "understand".into(),
            source_class: "original".into(),
            quantity_evidence: "qualitative_only".into(),
            status: ItemStatus::Approved,
            kind: ItemKind::SingleSelect,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// compute_bank_hash depends only on sorted item ids/payloads, not insert order.
        #[test]
        fn bank_hash_independent_of_insert_order(
            mut ids in proptest::collection::vec("[a-z][a-z0-9]{0,7}", 1..12),
            seed in any::<u64>(),
        ) {
            ids.sort();
            ids.dedup();
            prop_assume!(!ids.is_empty());

            let letters = ["A", "B", "C", "D"];
            let items: Vec<BankItem> = ids
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    let correct = letters[(seed as usize + i) % 4];
                    sample_item(id, ((i as u32) % 14) + 1, correct)
                })
                .collect();

            let mut forward = BTreeMap::new();
            for it in &items {
                forward.insert(it.id.clone(), it.clone());
            }
            let mut reverse = BTreeMap::new();
            for it in items.iter().rev() {
                reverse.insert(it.id.clone(), it.clone());
            }
            // Third order: even then odd indices
            let mut interleaved = BTreeMap::new();
            for it in items.iter().step_by(2).chain(items.iter().skip(1).step_by(2)) {
                interleaved.insert(it.id.clone(), it.clone());
            }

            let h1 = compute_bank_hash(&forward).unwrap();
            let h2 = compute_bank_hash(&reverse).unwrap();
            let h3 = compute_bank_hash(&interleaved).unwrap();
            assert_eq!(h1, h2);
            assert_eq!(h1, h3);
            assert_eq!(h1.len(), 64);
        }
    }
}
