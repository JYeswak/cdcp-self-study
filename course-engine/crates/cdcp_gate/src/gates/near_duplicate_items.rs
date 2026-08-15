//! near-duplicate-items — textual near-duplicate detection over the assembly
//! pool (bd-near-duplicate-item-gate-i5v).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises one floor: **no two assembly-eligible bank items may be
//! textually close enough to read as the same question twice.** Concretely it
//! goes RED when any pair of `status = "approved"` items clears one of the two
//! rules in [`Rule`], and it names both items, both files, and all three
//! measured similarities so the reader can adjudicate the pair without opening
//! anything.
//!
//! The floor moves from *no two items are byte-identical* to *no two items say
//! the same thing in cosmetically different words*. Exact hashing already held
//! on this bank and found nothing: measured 2026-08-14 over the live 804 items,
//! exact-stem grouping returns ZERO groups while this gate returned 25 pairs,
//! one of which (`mock40-q40` / `bank-m14-q121`) is a single item entered twice
//! with the key shuffled from B to A and two distractors reworded. All 25 were
//! read by hand and all 25 were judged genuine duplicates — a measured
//! false-positive rate of 0/25. `mock40-q40` was retired under this bead, so 24
//! remain; the other 24 are named in the RED output and are not this bead's to
//! edit. This gate therefore goes RED on the live tree today, by design and by
//! report — the backlog is real, not a calibration artefact.
//!
//! # THE METRIC, AND WHY THIS CUT
//!
//! Everything is measured on [`normalize`]d ASCII token SETS with
//! [`Sim`]: intersection over union, held as two integers and compared by
//! cross-multiplication. No floating point is involved anywhere on the decision
//! path, so the same bank yields the same findings in the same order on every
//! machine and every run.
//!
//! The discriminating field is the **correct answer text**, not the stem. Two
//! items that test different facts routinely share a stem shape — this bank is
//! full of `"X primarily aims to:"` — but they almost never share an answer.
//! Measured over the same 804 items, all 322,806 pairs:
//!
//! | cut | pairs flagged | reviewed verdict |
//! |-----|---------------|------------------|
//! | stem similarity ≥ 60% | 63 | mixed; includes plainly distinct items |
//! | key similarity ≥ 40% | 56 | first false positives appear at 47% |
//! | key similarity ≥ 50% | 35 | 1 pair judged NOT a duplicate |
//! | **key similarity ≥ 60%** | **24** | **24/24 judged genuine duplicates** |
//! | key text identical after normalisation | 11 | 11/11 genuine |
//!
//! 60% is the loosest cut with a measured false-positive rate of zero on this
//! bank, and it sits a clear 13 points above the closest reviewed false
//! positive — `m02-q067` (ISO/IEC 22237) against `m02-q203` (EN 50600), two
//! different standards whose answer texts collide at 47% on generic words
//! (`data centre facilities standards series`). A threshold parked one point
//! above a known false positive is a threshold that starts producing noise the
//! moment a new item is authored, and a finding list people learn to skim is
//! worth less than no finding list at all.
//!
//! [`Rule::ReshuffledClone`] is the second leg, and it names the exact defect
//! this bead was filed for: same answer, same wrong answers, letters moved. It
//! admits a lower key bar (40%) only when the DISTRACTOR sets also agree at
//! 50%. On this bank it recovers one further genuine pair (`mock40-q26` /
//! `m09-q124`, CRAC vs CRAH) and admits nothing else — the three reviewed false
//! positives near that key band carry 3%, 10% and 11% distractor overlap
//! respectively, nowhere near the bar.
//!
//! # WHAT THIS GATE CANNOT DECIDE
//!
//! **It cannot decide that two items test the same thing.** It measures textual
//! closeness and nothing else. A flagged pair is a pair a human should look at;
//! the verdict "these are one item twice" is the human's, never this gate's.
//! Nothing here reads meaning, and two items that share wording while testing
//! genuinely different facts will be flagged and must be dismissed by hand.
//!
//! The converse gap is larger and matters more: a **paraphrased** duplicate —
//! same proposition, freshly worded answer — scores below every threshold here
//! and passes silently. Those exist in this bank today. `m09-q111` and
//! `m09-q242` both assess "IT power becomes heat" and share 22% of their answer
//! tokens; `m09-q113` and `m09-q202` both assess the allowable-vs-recommended
//! envelope and share 5%. A green verdict from this gate therefore means "no
//! COSMETIC duplicates remain in the approved pool", never "the pool holds 804
//! distinct propositions". The headline item count is not certified by this
//! gate and must not be quoted as though it were.
//!
//! It also says nothing about item quality, difficulty, grounding, distractor
//! plausibility, or whether either member of a flagged pair is correct. It
//! reads only `approved` items, so a duplicate parked in `draft` or `retired`
//! is out of scope by construction — that is deliberate, since the harm being
//! prevented is a learner meeting the same proposition twice in one form, and
//! only approved items can be drawn (C1).
//!
//! # ANTI-VACUOUS (L4)
//!
//! A missing bank directory, zero `.toml` files, an unparseable or malformed
//! item, zero approved items, and fewer than two approved items (which is zero
//! comparisons) are each an ERROR, never a pass. A pool that was never compared
//! must not report like a pool that was compared and came back clean, so the
//! green line opens with `{SUCCESS_TOKEN}` (bd-near-dup-no-success-token-gj9y)
//! and then the scanned count, the approved count and the comparison count —
//! not a bare "ok" and not a receipt a RED run can share.
//!
//! The gate is additionally required to be shown TRIPPING, not merely passing:
//! `CDCP_NEAR_DUPLICATE_SELFTEST=1` injects a cosmetically-reworded clone of a
//! real approved item into the in-memory pool and asserts the clone is flagged
//! against its source. That run exits 0 only when the RED path was reached, and
//! exits ERROR when the detector failed to catch its own planted known-bad.

use crate::registry::{GateCtx, GateError};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const NAME: &str = "near-duplicate-items";
pub const SUMMARY: &str = "textual near-duplicates in the approved item pool (C3)";
/// Line prefix on every exit-0 path. RED report() uses `{NAME}: FAIL:` /
/// `{NAME}: ERROR:`, so a reader (or LEG D) can tell pass from fail without
/// the exit code.
pub const SUCCESS_TOKEN: &str = "near-duplicate-items: ok:";

/// The item directory, relative to the engine root.
pub const BANK_REL: &str = "bank/items";

/// The only status a `cdcp_assemble` draw can reach (C1).
pub const APPROVED: &str = "approved";

/// Leg 1 bar: answer-text similarity, in percent. See the calibration table in
/// the module header — this is the loosest cut with a measured zero
/// false-positive rate on the live bank.
pub const KEY_SIMILARITY_PCT: u32 = 60;

/// Leg 2 bar on the answer text. Lower than [`KEY_SIMILARITY_PCT`], and usable
/// only together with [`CLONE_DISTRACTOR_SIMILARITY_PCT`].
pub const CLONE_KEY_SIMILARITY_PCT: u32 = 40;

/// Leg 2 bar on the distractor set: the corroborating signal that makes the
/// lower key bar safe.
pub const CLONE_DISTRACTOR_SIMILARITY_PCT: u32 = 50;

/// Env switch for the injected-known-bad selftest.
pub const SELFTEST_ENV: &str = "CDCP_NEAR_DUPLICATE_SELFTEST";

/// Which rule admitted a pair. Both are named in the RED line so a reviewer can
/// tell "the answers are nearly the same" apart from "the whole item was
/// reshuffled".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// Answer texts alone clear [`KEY_SIMILARITY_PCT`].
    SharedKeyText,
    /// Answer texts clear the lower [`CLONE_KEY_SIMILARITY_PCT`] AND the
    /// distractor sets clear [`CLONE_DISTRACTOR_SIMILARITY_PCT`]. The archetype
    /// is one item re-entered with the choices rotated.
    ReshuffledClone,
}

impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::SharedKeyText => "shared-key-text",
            Rule::ReshuffledClone => "reshuffled-clone",
        }
    }
}

/// One item, reduced to the four fields this gate reads.
#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    /// File name only; the directory is fixed and printing it every line is noise.
    pub file: String,
    pub stem: String,
    /// The text of the correct choice.
    pub key: String,
    /// The texts of the other choices.
    pub distractors: Vec<String>,
    pub status: String,
}

impl Item {
    pub fn is_approved(&self) -> bool {
        self.status == APPROVED
    }
}

/// A Jaccard ratio kept as two integers. Never a float: `0.6` is not
/// representable in binary and a threshold comparison that rounds differently
/// on two machines is a determinism defect, not a rounding detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sim {
    pub intersection: usize,
    pub union: usize,
}

impl Sim {
    /// `|A ∩ B| / |A ∪ B|` over token sets.
    pub fn of(a: &BTreeSet<String>, b: &BTreeSet<String>) -> Self {
        Sim {
            intersection: a.intersection(b).count(),
            union: a.union(b).count(),
        }
    }

    /// `self >= pct%`, by cross-multiplication.
    ///
    /// An empty union is NOT a match. Two items with no tokens between them are
    /// zero evidence of duplication, and `0/0 == 1` would turn the absence of
    /// text into the strongest possible signal.
    pub fn at_least(self, pct: u32) -> bool {
        if self.union == 0 {
            return false;
        }
        self.intersection * 100 >= (pct as usize) * self.union
    }

    /// Truncated percent, for the report line only. Never used for a decision.
    pub fn percent(self) -> usize {
        if self.union == 0 {
            return 0;
        }
        self.intersection * 100 / self.union
    }
}

/// One flagged pair, with every number that produced it.
#[derive(Debug, Clone)]
pub struct Finding {
    pub a: Item,
    pub b: Item,
    pub key: Sim,
    pub distractor: Sim,
    pub stem: Sim,
    pub rule: Rule,
}

impl Finding {
    pub fn line(&self) -> String {
        format!(
            "{} ({}) <-> {} ({}) — answer {}% · distractors {}% · stem {}% [{}] — review: same proposition twice?",
            self.a.id,
            self.a.file,
            self.b.id,
            self.b.file,
            self.key.percent(),
            self.distractor.percent(),
            self.stem.percent(),
            self.rule.as_str(),
        )
    }
}

/// Lowercase, then every character that is not an ASCII letter or digit becomes
/// a separator. `"1.75 inches (44.45 mm)"` and `"1.75 in / 44.45 mm"` reduce to
/// overlapping token sets; `"data centre"` and `"data-centre"` become the same
/// two tokens. Non-ASCII (the arrows, en dashes and curly quotes this bank
/// uses) is dropped rather than transliterated: no dependency does that, and
/// silently mapping `→` to a token would be a guess.
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_space = false;
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        } else {
            pending_space = true;
        }
    }
    out
}

/// The token SET of a normalized string. A set, not a bag: repetition of a word
/// carries no duplication signal, and set semantics make the measure symmetric.
pub fn tokens(s: &str) -> BTreeSet<String> {
    normalize(s)
        .split(' ')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// The union of every distractor's tokens.
pub fn distractor_tokens(item: &Item) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for d in &item.distractors {
        set.extend(tokens(d));
    }
    set
}

/// Decide one pair. Returns the rule that admitted it, if any.
pub fn classify(key: Sim, distractor: Sim) -> Option<Rule> {
    if key.at_least(KEY_SIMILARITY_PCT) {
        return Some(Rule::SharedKeyText);
    }
    if key.at_least(CLONE_KEY_SIMILARITY_PCT)
        && distractor.at_least(CLONE_DISTRACTOR_SIMILARITY_PCT)
    {
        return Some(Rule::ReshuffledClone);
    }
    None
}

/// Compare every pair. Returns the findings and the number of comparisons made.
///
/// `items` is expected id-sorted; the output inherits that order, so the report
/// is byte-stable for a given bank. Fewer than two items is an ERROR: zero
/// comparisons is not a clean bill of health.
pub fn find_near_duplicates(items: &[Item]) -> Result<(Vec<Finding>, usize), String> {
    if items.len() < 2 {
        return Err(format!(
            "{} approved item(s) — fewer than two means ZERO comparisons, which is not a pass",
            items.len()
        ));
    }
    let keys: Vec<BTreeSet<String>> = items.iter().map(|i| tokens(&i.key)).collect();
    let stems: Vec<BTreeSet<String>> = items.iter().map(|i| tokens(&i.stem)).collect();
    let dis: Vec<BTreeSet<String>> = items.iter().map(distractor_tokens).collect();

    let mut findings = Vec::new();
    let mut comparisons = 0usize;
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            comparisons += 1;
            let key = Sim::of(&keys[i], &keys[j]);
            let distractor = Sim::of(&dis[i], &dis[j]);
            if let Some(rule) = classify(key, distractor) {
                findings.push(Finding {
                    a: items[i].clone(),
                    b: items[j].clone(),
                    key,
                    distractor,
                    stem: Sim::of(&stems[i], &stems[j]),
                    rule,
                });
            }
        }
    }
    Ok((findings, comparisons))
}

// ── loading ────────────────────────────────────────────────────────────────

/// `sorted(dir.glob("*.toml"))`, fail-closed on a `read_dir` entry error.
fn item_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    let mut out = Vec::new();
    for entry in rd {
        let entry = entry.map_err(|e| format!("read {}: {e}", dir.display()))?;
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("toml") {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

fn want_str(t: &toml::Table, key: &str, file: &str) -> Result<String, String> {
    match t.get(key) {
        Some(toml::Value::String(s)) => Ok(s.clone()),
        Some(other) => Err(format!("{file}: {key} must be a string, got {other}")),
        None => Err(format!("{file}: missing {key}")),
    }
}

/// Parse one item file. Anything malformed is an ERROR — an item this gate
/// could not read is an item it did not compare, and skipping it would let a
/// duplicate hide behind a typo.
pub fn parse_item(text: &str, file: &str) -> Result<Item, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|e| format!("{file}: not valid TOML: {e}"))?;

    let id = want_str(&table, "id", file)?;
    let stem = want_str(&table, "stem", file)?;
    let correct = want_str(&table, "correct", file)?;
    // Absent status means draft (the cdcp_bank default): never assembled,
    // therefore out of scope here. Silence is not approval.
    let status = match table.get("status") {
        Some(toml::Value::String(s)) => s.clone(),
        Some(other) => return Err(format!("{file}: status must be a string, got {other}")),
        None => "draft".to_string(),
    };

    let Some(toml::Value::Array(raw)) = table.get("choices") else {
        return Err(format!("{file}: missing or non-array choices"));
    };
    let mut choices = Vec::with_capacity(raw.len());
    for (n, v) in raw.iter().enumerate() {
        match v {
            toml::Value::String(s) => choices.push(s.clone()),
            other => {
                return Err(format!(
                    "{file}: choices[{n}] must be a string, got {other}"
                ))
            }
        }
    }
    if choices.len() < 2 {
        return Err(format!(
            "{file}: {} choice(s) — an item with fewer than two choices cannot be compared",
            choices.len()
        ));
    }

    let idx = match correct.as_str() {
        "A" => 0usize,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        other => return Err(format!("{file}: correct must be A-D, got {other:?}")),
    };
    if idx >= choices.len() {
        return Err(format!(
            "{file}: correct = {correct:?} but there are only {} choices",
            choices.len()
        ));
    }

    let key = choices[idx].clone();
    let distractors = choices
        .iter()
        .enumerate()
        .filter(|(n, _)| *n != idx)
        .map(|(_, c)| c.clone())
        .collect();

    Ok(Item {
        id,
        file: file.to_string(),
        stem,
        key,
        distractors,
        status,
    })
}

/// Every item under `dir`, id-sorted. Returns the loaded items and the number of
/// files read, so the caller can report both counts.
pub fn load_items(dir: &Path) -> Result<(Vec<Item>, usize), String> {
    if !dir.is_dir() {
        return Err(format!("no bank directory at {}", dir.display()));
    }
    let files = item_files(dir)?;
    if files.is_empty() {
        return Err(format!(
            "zero .toml item files under {} — an unscanned bank is not a clean bank",
            dir.display()
        ));
    }
    let mut items = Vec::with_capacity(files.len());
    for path in &files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<non-utf8>")
            .to_string();
        let text = std::fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
        items.push(parse_item(&text, &name)?);
    }
    items.sort_by(|a, b| a.id.cmp(&b.id));
    Ok((items, files.len()))
}

// ── the L4 injected known-bad ──────────────────────────────────────────────

/// A cosmetically-reworded clone of `src`: the stem gains a filler word, the
/// answer gains a trailing qualifier, one distractor is reworded, and the
/// choices rotate so the correct LETTER differs. That last part is the
/// dangerous subclass — a reviewer scanning answer keys sees `B` and `A` and
/// reads them as different questions.
pub fn cosmetic_clone(src: &Item) -> Item {
    let mut distractors: Vec<String> = src.distractors.clone();
    if let Some(first) = distractors.first_mut() {
        *first = format!("{first} in isolation");
    }
    distractors.rotate_left(1);
    Item {
        id: format!("{}-selftest-clone", src.id),
        file: format!("{}#selftest-clone", src.file),
        stem: format!("{} (restated)", src.stem),
        key: format!("{} under planned conditions", src.key),
        distractors,
        status: APPROVED.to_string(),
    }
}

/// Plant a known-bad and require the detector to catch it.
///
/// Exits Ok only when the injected clone was flagged against its source. A
/// detector that no longer trips on its own planted duplicate is an ERROR — the
/// whole point of this path is that green here means the gate demonstrably
/// goes RED on a real known-bad, not that it silently found nothing.
pub fn selftest(approved: &[Item]) -> Result<String, String> {
    let Some(src) = approved.first() else {
        return Err("selftest needs at least one approved item to clone".to_string());
    };
    let clone = cosmetic_clone(src);
    let clone_id = clone.id.clone();
    let mut pool: Vec<Item> = vec![src.clone(), clone];
    pool.sort_by(|a, b| a.id.cmp(&b.id));

    let (findings, comparisons) = find_near_duplicates(&pool)?;
    if comparisons == 0 {
        return Err("selftest made zero comparisons".to_string());
    }
    let caught = findings.iter().any(|f| {
        (f.a.id == src.id && f.b.id == clone_id) || (f.b.id == src.id && f.a.id == clone_id)
    });
    if !caught {
        return Err(format!(
            "planted a cosmetic clone of {} and the detector did NOT flag it — the known-bad escaped",
            src.id
        ));
    }
    Ok(format!(
        "{SUCCESS_TOKEN} selftest reached RED: planted clone {clone_id} flagged against {} ({} comparison(s))",
        src.id, comparisons
    ))
}

// ── the gate ───────────────────────────────────────────────────────────────

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(&[])?;

    let dir = ctx.root.join(BANK_REL);
    let (items, files) = load_items(&dir).map_err(GateError::error)?;
    if items.is_empty() {
        return Err(GateError::error(format!(
            "zero items parsed from {} files under {}",
            files,
            dir.display()
        )));
    }

    let approved: Vec<Item> = items.iter().filter(|i| i.is_approved()).cloned().collect();
    if approved.is_empty() {
        return Err(GateError::error(format!(
            "{files} item file(s) scanned and ZERO are status = {APPROVED:?} — nothing is assembly-eligible, so nothing was compared"
        )));
    }

    if std::env::var(SELFTEST_ENV).unwrap_or_default() == "1" {
        let msg = selftest(&approved).map_err(GateError::error)?;
        println!("{msg}");
        return Ok(());
    }

    let (findings, comparisons) = find_near_duplicates(&approved).map_err(GateError::error)?;
    if comparisons == 0 {
        return Err(GateError::error(
            "zero pair comparisons made — a pool that was never compared is not a clean pool",
        ));
    }

    if !findings.is_empty() {
        return Err(GateError::violation(findings.iter().map(|f| f.line())));
    }

    println!(
        "{SUCCESS_TOKEN} {files} item file(s) · {} approved · {comparisons} pair comparison(s) · 0 near-duplicate pair(s) at answer≥{KEY_SIMILARITY_PCT}% or answer≥{CLONE_KEY_SIMILARITY_PCT}%+distractors≥{CLONE_DISTRACTOR_SIMILARITY_PCT}%",
        approved.len()
    );
    println!(
        "{NAME}: this does NOT mean the pool holds {} distinct propositions — paraphrased duplicates score below every threshold here and are invisible to this gate",
        approved.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, stem: &str, key: &str, distractors: &[&str]) -> Item {
        Item {
            id: id.to_string(),
            file: format!("{id}.toml"),
            stem: stem.to_string(),
            key: key.to_string(),
            distractors: distractors.iter().map(|s| s.to_string()).collect(),
            status: APPROVED.to_string(),
        }
    }

    #[test]
    fn success_token_is_the_name_plus_ok() {
        assert_eq!(SUCCESS_TOKEN, format!("{NAME}: ok:"));
        assert!(
            SUCCESS_TOKEN.starts_with(NAME),
            "LEG D keys on a line prefix; the token must open with the gate name"
        );
    }

    #[test]
    fn normalize_folds_punctuation_case_and_non_ascii() {
        assert_eq!(normalize("1.75 inches (44.45 mm)"), "1 75 inches 44 45 mm");
        assert_eq!(normalize("Data-Centre"), "data centre");
        assert_eq!(normalize("A → B"), "a b");
        assert_eq!(normalize("   "), "");
    }

    #[test]
    fn empty_union_is_never_a_match() {
        let s = Sim {
            intersection: 0,
            union: 0,
        };
        assert!(!s.at_least(0), "0/0 must not read as a perfect match");
        assert!(!s.at_least(60));
        assert_eq!(s.percent(), 0);
    }

    #[test]
    fn threshold_is_integer_exact_at_the_boundary() {
        // 3/5 = 60% exactly: the bar is inclusive and must not depend on a float.
        let s = Sim {
            intersection: 3,
            union: 5,
        };
        assert!(s.at_least(60));
        assert!(!s.at_least(61));
    }

    #[test]
    fn the_calibration_pair_shape_is_flagged() {
        // The real m14-q040 / m14-q121 text, verbatim.
        let a = item(
            "mock40-q40",
            "Integrated Systems Testing (IST) is valuable because it:",
            "Validates power, cooling, and controls failovers as a combined system under planned scenarios",
            &[
                "Only tests a single breaker nameplate",
                "Replaces daily backups of VMs",
                "Is optional marketing text with no technical meaning",
            ],
        );
        let b = item(
            "bank-m14-q121",
            "Integrated Systems Testing (IST) is valuable primarily because it:",
            "Validates power, cooling, and controls failovers as a combined system under planned scenarios",
            &[
                "Only tests a single breaker nameplate in isolation forever",
                "Replaces daily VM backups",
                "Is optional marketing text with no technical meaning",
            ],
        );
        let (findings, comparisons) = find_near_duplicates(&[a, b]).unwrap();
        assert_eq!(comparisons, 1);
        assert_eq!(findings.len(), 1, "the known-true pair must be flagged");
        assert_eq!(findings[0].key.percent(), 100);
        assert_eq!(findings[0].rule, Rule::SharedKeyText);
    }

    #[test]
    fn a_shared_stem_shape_with_a_different_answer_is_not_flagged() {
        // m09-q206 / m09-q207 verbatim: hot-aisle vs cold-aisle containment.
        // Same stem shape, genuinely different facts.
        let a = item(
            "m09-q206",
            "Hot-aisle containment primarily aims to:",
            "Capture and isolate hot exhaust so return temperatures rise cleanly and cold aisles stay cooler",
            &[
                "Raise humidity setpoints",
                "Remove the need for blanking panels",
                "Increase bypass airflow",
            ],
        );
        let b = item(
            "m09-q207",
            "Cold-aisle containment primarily aims to:",
            "Keep supply air focused on IT intakes and reduce mixing with room air",
            &[
                "Raise chilled water temperature only",
                "Eliminate the need for a return path",
                "Guarantee lower PUE in every design",
            ],
        );
        let (findings, comparisons) = find_near_duplicates(&[a, b]).unwrap();
        assert_eq!(comparisons, 1);
        assert!(
            findings.is_empty(),
            "known-GOOD pair was flagged: {:?}",
            findings.iter().map(|f| f.line()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_reviewed_false_positive_stays_below_the_bar() {
        // m02-q067 / m02-q203 verbatim: ISO/IEC 22237 vs EN 50600. Different
        // standards; their answers collide on generic vocabulary at 47%.
        let a = item(
            "m02-q067",
            "ISO/IEC 22237 is best described for CDCP awareness as:",
            "A series addressing data centre facilities and infrastructures at an international standards level",
            &["A product warranty", "A fire code", "A cabling connector spec"],
        );
        let b = item(
            "m02-q203",
            "EN 50600 series is best described at awareness level as:",
            "A European data centre facilities and infrastructure standards series",
            &[
                "A payment card rule",
                "A humidity setpoint",
                "A rack unit definition",
            ],
        );
        let (findings, _) = find_near_duplicates(&[a, b]).unwrap();
        assert!(
            findings.is_empty(),
            "the calibrating false positive crossed the bar: {:?}",
            findings.iter().map(|f| f.line()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reshuffled_clone_leg_needs_the_distractors_too() {
        let key_a = "alpha beta gamma delta epsilon";
        // 3/7 = 42% key overlap on its own: over CLONE_KEY_SIMILARITY_PCT,
        // under KEY_SIMILARITY_PCT, so leg 1 alone cannot admit it.
        let key_b = "alpha beta gamma zeta eta";
        let shared = ["one two three", "four five six", "seven eight nine"];
        let a = item("x-1", "stem one", key_a, &shared);
        let b = item("x-2", "stem two", key_b, &shared);
        let (findings, _) = find_near_duplicates(&[a.clone(), b.clone()]).unwrap();
        assert_eq!(findings.len(), 1, "shared distractors must admit the pair");
        assert_eq!(findings[0].rule, Rule::ReshuffledClone);

        let b_alone = item("x-2", "stem two", key_b, &["nothing", "at", "all"]);
        let (none, _) = find_near_duplicates(&[a, b_alone]).unwrap();
        assert!(
            none.is_empty(),
            "without distractor overlap the pair must not be admitted"
        );
    }

    #[test]
    fn fewer_than_two_items_is_an_error_not_an_empty_pass() {
        let one = vec![item("solo", "s", "k", &["a", "b", "c"])];
        assert!(find_near_duplicates(&one).is_err());
        assert!(find_near_duplicates(&[]).is_err());
    }

    #[test]
    fn selftest_catches_its_own_planted_clone() {
        let pool = vec![item(
            "seed-1",
            "Integrated Systems Testing is valuable because it:",
            "Validates power, cooling, and controls failovers as a combined system",
            &[
                "Only tests one breaker",
                "Replaces VM backups",
                "Is marketing text",
            ],
        )];
        let msg = selftest(&pool).expect("planted clone must be caught");
        assert!(msg.contains("seed-1-selftest-clone"), "{msg}");
    }

    #[test]
    fn the_planted_clone_moves_the_correct_letter() {
        let src = item(
            "seed-1",
            "stem",
            "the key text",
            &["first distractor", "second distractor", "third distractor"],
        );
        let clone = cosmetic_clone(&src);
        assert_ne!(clone.distractors[0], src.distractors[0]);
        assert_ne!(clone.key, src.key);
        assert!(clone.key.starts_with(&src.key));
    }

    #[test]
    fn findings_are_deterministic_in_content_and_order() {
        let a = item("a-1", "s", "shared answer text here", &["p", "q", "r"]);
        let b = item("b-2", "s", "shared answer text here", &["p", "q", "r"]);
        let c = item("c-3", "s", "shared answer text here", &["p", "q", "r"]);
        let pool = vec![a, b, c];
        let (f1, n1) = find_near_duplicates(&pool).unwrap();
        let (f2, n2) = find_near_duplicates(&pool).unwrap();
        assert_eq!(n1, 3);
        assert_eq!(n1, n2);
        let l1: Vec<String> = f1.iter().map(|f| f.line()).collect();
        let l2: Vec<String> = f2.iter().map(|f| f.line()).collect();
        assert_eq!(l1, l2);
        assert_eq!(l1.len(), 3);
        assert!(l1[0].starts_with("a-1"), "{:?}", l1);
    }

    #[test]
    fn parse_rejects_a_malformed_item_rather_than_skipping_it() {
        assert!(parse_item("id = \"x\"\n", "x.toml").is_err());
        assert!(parse_item(
            "id = \"x\"\nstem = \"s\"\ncorrect = \"E\"\nchoices = [\"a\",\"b\",\"c\",\"d\"]\n",
            "x.toml"
        )
        .is_err());
        assert!(parse_item(
            "id = \"x\"\nstem = \"s\"\ncorrect = \"D\"\nchoices = [\"a\",\"b\"]\n",
            "x.toml"
        )
        .is_err());
    }

    #[test]
    fn parse_defaults_absent_status_to_draft_not_approved() {
        let it = parse_item(
            "id = \"x\"\nstem = \"s\"\ncorrect = \"B\"\nchoices = [\"a\",\"b\",\"c\",\"d\"]\n",
            "x.toml",
        )
        .unwrap();
        assert_eq!(it.status, "draft");
        assert!(!it.is_approved());
        assert_eq!(it.key, "b");
        assert_eq!(it.distractors, vec!["a", "c", "d"]);
    }
}
