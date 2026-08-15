//! Measured paraphrase-pair ledger — honesty tripwire, not a grader-of-record.
//!
//! Extracted from `scripts/verify_paraphrase_pairs.py` by
//! `bd-substrate-rust-migration-jhd.21` (EXTRACT-THEN-DELETE). This is bank
//! product, not a `cdcp_gate` file: C3 (`near-duplicate-items`) measures
//! cosmetic answer-Jaccard and misses same-proposition paraphrases. The
//! ledger in `registries/paraphrase_pairs.toml` is the tripwire a learner
//! form can still draw twice. CHARTER forbids an LLM as grader-of-record, so
//! nothing here grades meaning and nothing here retires an item.
//!
//! # What this decides (deterministic, offline)
//!
//! * The ledger is a non-empty list of the four measured pair ids. Deleting
//!   a required row without leaving it in place as `status = "adjudicated"`
//!   plus a non-empty `adjudication_reason` is RED.
//! * Empty `[[pair]]` is ERROR. A scan of zero item files is ERROR. Fewer
//!   than two approved items is ERROR (zero comparisons is not a pass).
//! * Known-distinct `kd-hot-aisle-cold-aisle` must remain listed.
//! * A cheap stem-overlap REPORT prints candidates. It is NOT a verdict.
//!
//! # What this cannot decide
//!
//! Whether two items test the same proposition. Whether a listed pair should
//! be retired. Whether the report's untagged candidates are real debt. A
//! green run means the ledger is intact, never that the pool holds N
//! distinct propositions.

use crate::{Bank, BankError, BankItem, ItemStatus};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const NAME: &str = "verify-paraphrase-pairs";
pub const LEDGER_REL: &str = "registries/paraphrase_pairs.toml";
pub const BANK_REL: &str = "bank/items";

pub const EXIT_OK: i32 = 0;
pub const EXIT_FAIL: i32 = 2;
pub const EXIT_ERROR: i32 = 4;

pub const STATUS_OPEN: &str = "open";
pub const STATUS_ADJUDICATED: &str = "adjudicated";

/// The four pairs measured by hand (bd-e1yt). Deleting one of these ids is
/// RED even if `min_pairs` is edited down: the id is the tripwire.
pub const REQUIRED_PAIR_IDS: &[&str] = &[
    "pp-m09-it-power-heat",
    "pp-m09-allowable-vs-recommended",
    "pp-m09-dehumidification-coil",
    "pp-m09-blanking-panels",
];

/// Known-GOOD leg. Must remain listed so the report's false-positive shape
/// cannot be silently dropped.
pub const REQUIRED_DISTINCT_IDS: &[&str] = &["kd-hot-aisle-cold-aisle"];

/// Report cut: high stem overlap AND answer below C3's 60% floor.
/// Prints candidates. Never a fail condition on its own.
pub const REPORT_STEM_PCT: usize = 50;
pub const C3_ANSWER_PCT: usize = 60;
pub const REPORT_CAP: usize = 40;

#[derive(Debug, Clone)]
pub struct Request {
    pub ledger: PathBuf,
    pub bank: PathBuf,
    /// Skip the live `check_ledger` and run only the in-process plants.
    pub selftest_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct ScanItem {
    pub id: String,
    pub stem: String,
    pub key: String,
    pub status: String,
}

impl ScanItem {
    pub fn is_approved(&self) -> bool {
        self.status == ItemStatus::Approved.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct Ledger {
    pub source: String,
    pub min_pairs: i64,
    pub pairs: Vec<toml::Value>,
    pub known_distinct: Vec<toml::Value>,
}

/// Truncated Jaccard percent over ASCII-alnum token sets. Empty union is 0,
/// never 100 — two empty stems are zero evidence of overlap.
pub fn sim_percent(a: &BTreeSet<String>, b: &BTreeSet<String>) -> usize {
    if a.is_empty() && b.is_empty() {
        return 0;
    }
    let union = a.union(b).count();
    if union == 0 {
        return 0;
    }
    a.intersection(b).count() * 100 / union
}

/// Lowercase; non-ASCII-alnum becomes a separator. Matches C3 / the retired
/// Python oracle.
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

pub fn tokens(s: &str) -> BTreeSet<String> {
    normalize(s)
        .split(' ')
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn key_of(item: &BankItem) -> String {
    let idx = match item.correct.as_str() {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        _ => return String::new(),
    };
    item.choices.get(idx).cloned().unwrap_or_default()
}

pub fn items_from_bank(bank: &Bank) -> Vec<ScanItem> {
    bank.items
        .values()
        .map(|it| ScanItem {
            id: it.id.clone(),
            stem: it.stem.clone(),
            key: key_of(it),
            status: it.status.as_str().to_string(),
        })
        .collect()
}

fn count_toml_files(dir: &Path) -> Result<usize, String> {
    let rd = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    let mut n = 0usize;
    for ent in rd {
        let ent = ent.map_err(|e| format!("read {}: {e}", dir.display()))?;
        if ent.path().extension().and_then(|x| x.to_str()) == Some("toml") {
            n += 1;
        }
    }
    Ok(n)
}

/// Load scan items via the product bank loader. Zero files / missing dir /
/// empty load is ERROR (anti-vacuous). A broken item is a load error — this
/// crate is stricter than the retired light Python scan, by design.
pub fn load_scan_items(bank_dir: &Path) -> Result<Vec<ScanItem>, String> {
    if !bank_dir.is_dir() {
        return Err(format!("bank dir missing: {}", bank_dir.display()));
    }
    let n = count_toml_files(bank_dir)?;
    if n == 0 {
        return Err(format!(
            "zero item files in {} (vacuous scan is ERROR)",
            bank_dir.display()
        ));
    }
    match Bank::load_dir(bank_dir) {
        Ok(bank) => {
            let items = items_from_bank(&bank);
            if items.is_empty() {
                return Err("zero items loaded (vacuous scan is ERROR)".into());
            }
            Ok(items)
        }
        Err(BankError::Empty) => Err("zero items loaded (vacuous scan is ERROR)".into()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn parse_ledger(raw: &toml::Value, source: &str) -> (Ledger, Vec<String>) {
    let mut errors = Vec::new();
    let schema = raw.get("schema_version").and_then(|v| v.as_integer());
    if schema != Some(1) {
        errors.push(format!("{source}: schema_version {schema:?} (expected 1)"));
    }
    let empty_reg = toml::map::Map::new();
    let reg = match raw.get("registry").and_then(|v| v.as_table()) {
        Some(t) => t,
        None => {
            errors.push(format!("{source}: [registry] missing"));
            &empty_reg
        }
    };
    let min_pairs = reg.get("min_pairs").and_then(|v| v.as_integer());
    let min_pairs = match min_pairs {
        Some(n) if n >= 1 => n,
        other => {
            errors.push(format!(
                "{source}: registry.min_pairs must be an integer >= 1 \
                 (empty/zero ledger is ERROR); got {other:?}"
            ));
            0
        }
    };

    let pairs = match raw.get("pair") {
        None => {
            errors.push(format!(
                "{source}: zero [[pair]] rows (empty ledger is ERROR)"
            ));
            Vec::new()
        }
        Some(v) => match v.as_array() {
            None => {
                errors.push(format!("{source}: [[pair]] is not a table array"));
                Vec::new()
            }
            Some(arr) if arr.is_empty() => {
                errors.push(format!(
                    "{source}: zero [[pair]] rows (empty ledger is ERROR)"
                ));
                Vec::new()
            }
            Some(arr) => arr.clone(),
        },
    };

    let known_distinct = match raw.get("known_distinct") {
        None => {
            errors.push(format!("{source}: zero [[known_distinct]] rows"));
            Vec::new()
        }
        Some(v) => match v.as_array() {
            None => {
                errors.push(format!("{source}: [[known_distinct]] is not a table array"));
                Vec::new()
            }
            Some(arr) => arr.clone(),
        },
    };

    (
        Ledger {
            source: source.to_string(),
            min_pairs,
            pairs,
            known_distinct,
        },
        errors,
    )
}

fn row_str(row: &toml::value::Table, key: &str) -> Option<String> {
    row.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn canon_ids(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

/// Decide the ledger against a loaded item list. Never grades meaning.
pub fn check_ledger(ledger: &Ledger, items: &[ScanItem]) -> Vec<String> {
    let mut errors = Vec::new();
    let by_id: BTreeMap<&str, &ScanItem> = items.iter().map(|it| (it.id.as_str(), it)).collect();
    let approved: BTreeSet<&str> = items
        .iter()
        .filter(|it| it.is_approved())
        .map(|it| it.id.as_str())
        .collect();

    if items.is_empty() {
        errors.push("zero items loaded (vacuous scan is ERROR)".into());
    } else if approved.len() < 2 {
        errors.push(format!(
            "{} approved item(s) — fewer than two means ZERO pair comparisons, which is not a pass",
            approved.len()
        ));
    }

    if ledger.pairs.is_empty() {
        errors.push("empty [[pair]] list (gate claims to track pairs)".into());
    }
    if ledger.min_pairs < REQUIRED_PAIR_IDS.len() as i64 {
        errors.push(format!(
            "registry.min_pairs={} is below the required id floor ({}); \
             lowering the floor to drop a measured pair is RED",
            ledger.min_pairs,
            REQUIRED_PAIR_IDS.len()
        ));
    }
    if (ledger.pairs.len() as i64) < ledger.min_pairs {
        errors.push(format!(
            "{} [[pair]] row(s) < registry.min_pairs {}",
            ledger.pairs.len(),
            ledger.min_pairs
        ));
    }

    let mut seen_ids: BTreeSet<String> = BTreeSet::new();
    let mut seen_ab: BTreeSet<(String, String)> = BTreeSet::new();
    let mut open_abs: BTreeSet<(String, String)> = BTreeSet::new();

    for (i, row) in ledger.pairs.iter().enumerate() {
        let Some(t) = row.as_table() else {
            errors.push(format!("pair[{i}]: not a table"));
            continue;
        };
        let pid = match row_str(t, "id") {
            Some(s) if !s.trim().is_empty() => s,
            _ => {
                errors.push(format!("pair[{i}]: missing id"));
                continue;
            }
        };
        if !seen_ids.insert(pid.clone()) {
            errors.push(format!("pair {pid}: duplicate id"));
            continue;
        }
        let a = row_str(t, "a").unwrap_or_default();
        let b = row_str(t, "b").unwrap_or_default();
        if a.is_empty() || b.is_empty() {
            errors.push(format!("pair {pid}: a and b are required item ids"));
            continue;
        }
        if a == b {
            errors.push(format!("pair {pid}: a and b are the same id ({a})"));
            continue;
        }
        let ab = canon_ids(&a, &b);
        if !seen_ab.insert(ab.clone()) {
            errors.push(format!("pair {pid}: duplicate members {} / {}", ab.0, ab.1));
            continue;
        }
        let status = row_str(t, "status").unwrap_or_default();
        if status != STATUS_OPEN && status != STATUS_ADJUDICATED {
            errors.push(format!(
                "pair {pid}: status {status:?} (want open|adjudicated)"
            ));
            continue;
        }
        if status == STATUS_ADJUDICATED {
            let reason = row_str(t, "adjudication_reason").unwrap_or_default();
            if reason.trim().is_empty() {
                errors.push(format!(
                    "pair {pid}: status=adjudicated requires a non-empty \
                     adjudication_reason (a disappeared pair without a reason is RED)"
                ));
            }
            continue;
        }
        open_abs.insert(ab);
        for mid in [&a, &b] {
            match by_id.get(mid.as_str()) {
                None => errors.push(format!(
                    "pair {pid}: item {mid} is not in the bank \
                     (listed pair disappeared without adjudication)"
                )),
                Some(it) if !it.is_approved() => errors.push(format!(
                    "pair {pid}: item {mid} is {}, not approved — retire on the \
                     ledger (status=adjudicated + reason) before leaving the drawable pool",
                    it.status
                )),
                Some(_) => {}
            }
        }
    }

    for req in REQUIRED_PAIR_IDS {
        if !seen_ids.contains(*req) {
            errors.push(format!(
                "required pair id {req} is missing from the ledger \
                 (deleting a measured pair row is RED; adjudicate in place)"
            ));
        }
    }

    let mut seen_kd: BTreeSet<String> = BTreeSet::new();
    for (i, row) in ledger.known_distinct.iter().enumerate() {
        let Some(t) = row.as_table() else {
            errors.push(format!("known_distinct[{i}]: not a table"));
            continue;
        };
        let kid = match row_str(t, "id") {
            Some(s) if !s.trim().is_empty() => s,
            _ => {
                errors.push(format!("known_distinct[{i}]: missing id"));
                continue;
            }
        };
        if !seen_kd.insert(kid.clone()) {
            errors.push(format!("known_distinct {kid}: duplicate id"));
            continue;
        }
        let a = row_str(t, "a").unwrap_or_default();
        let b = row_str(t, "b").unwrap_or_default();
        if a.is_empty() || b.is_empty() || a == b {
            errors.push(format!("known_distinct {kid}: a and b must be two ids"));
            continue;
        }
        for mid in [&a, &b] {
            match by_id.get(mid.as_str()) {
                None => errors.push(format!("known_distinct {kid}: item {mid} missing")),
                Some(it) if !it.is_approved() => errors.push(format!(
                    "known_distinct {kid}: item {mid} is {}, not approved",
                    it.status
                )),
                Some(_) => {}
            }
        }
        if open_abs.contains(&canon_ids(&a, &b)) {
            errors.push(format!(
                "known_distinct {kid}: {a}/{b} is also listed as open \
                 paraphrase debt — a known-good pair cannot be open debt"
            ));
        }
    }
    for req in REQUIRED_DISTINCT_IDS {
        if !seen_kd.contains(*req) {
            errors.push(format!(
                "required known_distinct id {req} is missing \
                 (the known-good leg cannot be dropped)"
            ));
        }
    }
    errors
}

/// Print-shaped candidate lines. Never used as a fail condition.
pub fn overlap_report(items: &[ScanItem], ledger: &Ledger) -> Vec<String> {
    let approved: Vec<&ScanItem> = items.iter().filter(|it| it.is_approved()).collect();
    let mut debt: BTreeMap<(String, String), String> = BTreeMap::new();
    for row in &ledger.pairs {
        let Some(t) = row.as_table() else {
            continue;
        };
        let (Some(a), Some(b)) = (row_str(t, "a"), row_str(t, "b")) else {
            continue;
        };
        let id = row_str(t, "id").unwrap_or_else(|| "?".into());
        debt.insert(canon_ids(&a, &b), id);
    }
    let mut distinct: BTreeMap<(String, String), String> = BTreeMap::new();
    for row in &ledger.known_distinct {
        let Some(t) = row.as_table() else {
            continue;
        };
        let (Some(a), Some(b)) = (row_str(t, "a"), row_str(t, "b")) else {
            continue;
        };
        let id = row_str(t, "id").unwrap_or_else(|| "?".into());
        distinct.insert(canon_ids(&a, &b), id);
    }

    let mut candidates: Vec<(usize, usize, String, String, String)> = Vec::new();
    let n = approved.len();
    let mut comparisons = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            comparisons += 1;
            let ia = approved[i];
            let ib = approved[j];
            let stem = sim_percent(&tokens(&ia.stem), &tokens(&ib.stem));
            let ans = sim_percent(&tokens(&ia.key), &tokens(&ib.key));
            let ab = canon_ids(&ia.id, &ib.id);
            let tagged = debt.contains_key(&ab) || distinct.contains_key(&ab);
            let paraphrase_shaped = stem >= REPORT_STEM_PCT && ans < C3_ANSWER_PCT;
            if !(tagged || paraphrase_shaped) {
                continue;
            }
            let tag = if let Some(id) = debt.get(&ab) {
                format!("known-debt {id}")
            } else if let Some(id) = distinct.get(&ab) {
                format!("known-distinct {id}")
            } else {
                "candidate (not a verdict)".into()
            };
            candidates.push((stem, ans, ia.id.clone(), ib.id.clone(), tag));
        }
    }
    candidates.sort_by(|x, y| {
        y.0.cmp(&x.0)
            .then(y.1.cmp(&x.1))
            .then(x.2.cmp(&y.2))
            .then(x.3.cmp(&y.3))
    });

    let mut lines = vec![format!(
        "REPORT (stem>={REPORT_STEM_PCT}% and answer<{C3_ANSWER_PCT}%, \
         plus ledger pairs; NOT a grader-of-record; {comparisons} comparisons)"
    )];
    let mut shown = 0usize;
    for (stem, ans, a, b, tag) in &candidates {
        let is_ledger = tag.starts_with("known-");
        if !is_ledger {
            if shown >= REPORT_CAP {
                continue;
            }
            shown += 1;
        }
        lines.push(format!(
            "  {a} <-> {b}  stem {stem}% · answer {ans}%  [{tag}]"
        ));
    }
    let extra = candidates
        .iter()
        .filter(|c| !c.4.starts_with("known-"))
        .count()
        .saturating_sub(shown);
    if extra > 0 {
        lines.push(format!(
            "  … {extra} more candidate(s) omitted (cap {REPORT_CAP})"
        ));
    }
    if !lines.iter().any(|l| l.starts_with("  ")) {
        lines.push("  (no stem-overlap candidates at this cut)".into());
    }
    lines
}

fn table_row(pairs: &[(&str, &str)]) -> toml::Value {
    let mut t = toml::map::Map::new();
    for (k, v) in pairs {
        t.insert((*k).into(), toml::Value::String((*v).into()));
    }
    toml::Value::Table(t)
}

fn plant_ledger() -> Ledger {
    Ledger {
        source: "selftest-plant".into(),
        min_pairs: 4,
        pairs: vec![
            table_row(&[
                ("id", REQUIRED_PAIR_IDS[0]),
                ("a", "m09-q111"),
                ("b", "m09-q242"),
                ("status", STATUS_OPEN),
            ]),
            table_row(&[
                ("id", REQUIRED_PAIR_IDS[1]),
                ("a", "m09-q113"),
                ("b", "m09-q202"),
                ("status", STATUS_OPEN),
            ]),
            table_row(&[
                ("id", REQUIRED_PAIR_IDS[2]),
                ("a", "m09-q122"),
                ("b", "m09-q234"),
                ("status", STATUS_OPEN),
            ]),
            table_row(&[
                ("id", REQUIRED_PAIR_IDS[3]),
                ("a", "m09-q140"),
                ("b", "m09-q209"),
                ("status", STATUS_OPEN),
            ]),
        ],
        known_distinct: vec![table_row(&[
            ("id", REQUIRED_DISTINCT_IDS[0]),
            ("a", "m09-q206"),
            ("b", "m09-q207"),
        ])],
    }
}

/// Plant known-bad ledgers in memory. Returns selftest failure strings
/// (empty = every plant went RED).
pub fn run_selftest(items: &[ScanItem]) -> Vec<String> {
    let mut fails = Vec::new();
    let base = plant_ledger();

    let mut empty = base.clone();
    empty.pairs.clear();
    let empty_errs = check_ledger(&empty, items);
    if !empty_errs
        .iter()
        .any(|e| e.contains("zero [[pair]]") || e.contains("empty [[pair]]"))
    {
        fails.push(format!(
            "selftest EMPTY LEDGER did not ERROR (got {:?})",
            empty_errs.iter().take(3).collect::<Vec<_>>()
        ));
    }

    let mut dropped = base.clone();
    dropped.pairs.retain(|row| {
        row.as_table()
            .and_then(|t| t.get("id"))
            .and_then(|v| v.as_str())
            != Some(REQUIRED_PAIR_IDS[0])
    });
    let drop_errs = check_ledger(&dropped, items);
    if !drop_errs.iter().any(|e| e.contains(REQUIRED_PAIR_IDS[0])) {
        fails.push(format!(
            "selftest MISSING PAIR {} did not RED (got {:?})",
            REQUIRED_PAIR_IDS[0],
            drop_errs.iter().take(3).collect::<Vec<_>>()
        ));
    }

    let zero_errs = check_ledger(&base, &[]);
    if !zero_errs.iter().any(|e| e.contains("zero items")) {
        fails.push(format!(
            "selftest ZERO ITEMS did not ERROR (got {:?})",
            zero_errs.iter().take(3).collect::<Vec<_>>()
        ));
    }

    if !items.is_empty() {
        let one = &items[..1];
        let one_errs = check_ledger(&base, one);
        if !one_errs
            .iter()
            .any(|e| e.contains("fewer than two") || e.contains("zero items"))
        {
            fails.push(format!(
                "selftest SINGLE ITEM did not ERROR (got {:?})",
                one_errs.iter().take(3).collect::<Vec<_>>()
            ));
        }
    }

    let mut stolen = base.clone();
    stolen.pairs.push(table_row(&[
        ("id", "pp-should-not-list-known-good"),
        ("a", "m09-q206"),
        ("b", "m09-q207"),
        ("status", STATUS_OPEN),
    ]));
    let stolen_errs = check_ledger(&stolen, items);
    if !stolen_errs
        .iter()
        .any(|e| e.contains("known-good") || e.contains("known_distinct"))
    {
        fails.push(format!(
            "selftest KNOWN-GOOD listed as open debt did not RED (got {:?})",
            stolen_errs.iter().take(3).collect::<Vec<_>>()
        ));
    }

    let mut silent = base.clone();
    if let Some(toml::Value::Table(t)) = silent.pairs.get_mut(0) {
        t.insert(
            "status".into(),
            toml::Value::String(STATUS_ADJUDICATED.into()),
        );
        t.insert(
            "adjudication_reason".into(),
            toml::Value::String(String::new()),
        );
    }
    let silent_errs = check_ledger(&silent, items);
    if !silent_errs
        .iter()
        .any(|e| e.contains("adjudication_reason"))
    {
        fails.push(format!(
            "selftest ADJUDICATED-WITHOUT-REASON did not RED (got {:?})",
            silent_errs.iter().take(3).collect::<Vec<_>>()
        ));
    }

    let mut raw = toml::map::Map::new();
    raw.insert("schema_version".into(), toml::Value::Integer(1));
    let mut reg = toml::map::Map::new();
    reg.insert("min_pairs".into(), toml::Value::Integer(0));
    raw.insert("registry".into(), toml::Value::Table(reg));
    raw.insert("pair".into(), toml::Value::Array(base.pairs.clone()));
    raw.insert(
        "known_distinct".into(),
        toml::Value::Array(base.known_distinct.clone()),
    );
    let floor_errs = parse_ledger(&toml::Value::Table(raw), "selftest-min-pairs-0").1;
    let mut floor = base;
    floor.min_pairs = 0;
    let floor_check = check_ledger(&floor, items);
    if floor_errs.is_empty() && !floor_check.iter().any(|e| e.contains("min_pairs")) {
        fails.push("selftest min_pairs=0 did not ERROR".into());
    }

    fails
}

fn fail(code: i32, msg: impl Into<String>) -> Outcome {
    Outcome {
        code,
        stdout: String::new(),
        stderr: format!("{}: FAIL: {}\n", NAME, msg.into()),
    }
}

/// Load the ledger, scan the bank, decide, print the report, plant known-bad.
pub fn run(req: &Request) -> Outcome {
    let mut errors: Vec<String> = Vec::new();

    if !req.ledger.is_file() {
        return fail(
            EXIT_ERROR,
            format!("ledger missing: {}", req.ledger.display()),
        );
    }
    let text = match fs::read_to_string(&req.ledger) {
        Ok(t) => t,
        Err(e) => {
            return fail(
                EXIT_ERROR,
                format!("ledger unreadable {}: {e}", req.ledger.display()),
            )
        }
    };
    let raw: toml::Value = match toml::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return fail(
                EXIT_ERROR,
                format!("ledger unparseable {}: {e}", req.ledger.display()),
            )
        }
    };
    let (ledger, load_errs) = parse_ledger(&raw, &req.ledger.display().to_string());
    errors.extend(load_errs);

    let items = match load_scan_items(&req.bank) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            Vec::new()
        }
    };

    if !req.selftest_only {
        errors.extend(check_ledger(&ledger, &items));
    }

    let approved_n = items.iter().filter(|it| it.is_approved()).count();
    let open_n = ledger
        .pairs
        .iter()
        .filter(|p| {
            p.as_table()
                .and_then(|t| t.get("status"))
                .and_then(|v| v.as_str())
                == Some(STATUS_OPEN)
        })
        .count();

    let mut stdout = format!(
        "{NAME}: {} scanned · {approved_n} approved · {} ledger pair(s) · \
         {} known-distinct · {open_n} open\n",
        items.len(),
        ledger.pairs.len(),
        ledger.known_distinct.len()
    );

    if !items.is_empty() && approved_n >= 2 {
        for line in overlap_report(&items, &ledger) {
            stdout.push_str(&line);
            stdout.push('\n');
        }
    }

    let selftest_fails = run_selftest(&items);
    if selftest_fails.is_empty() {
        stdout.push_str(&format!(
            "{NAME}: selftest RED on planted missing pair / empty ledger / \
             zero items / adjudicated-without-reason / known-good-as-debt\n"
        ));
    } else {
        for f in selftest_fails {
            errors.push(format!("SELFTEST: {f}"));
        }
    }

    if !errors.is_empty() {
        let mut stderr = String::new();
        for e in &errors {
            stderr.push_str(&format!("{NAME}: FAIL: {e}\n"));
        }
        stderr.push_str(&format!(
            "{NAME}: {} finding(s) — this is a ledger tripwire, \
             not a license to delete bank items\n",
            errors.len()
        ));
        let vacuous = errors.iter().any(|e| {
            e.contains("vacuous scan")
                || e.contains("bank dir missing")
                || e.contains("zero item files")
                || e.contains("zero items loaded")
        });
        let ledger_unreadable = errors.iter().any(|e| {
            e.contains("ledger missing")
                || e.contains("ledger unparseable")
                || e.contains("ledger unreadable")
        });
        let only_input_errors = errors.iter().all(|e| {
            e.contains("vacuous")
                || e.contains("bank dir")
                || e.contains("zero item")
                || e.contains("ledger missing")
                || e.contains("ledger unparseable")
                || e.contains("ledger unreadable")
        });
        let code = if (vacuous || ledger_unreadable) && only_input_errors {
            EXIT_ERROR
        } else {
            EXIT_FAIL
        };
        return Outcome {
            code,
            stdout,
            stderr,
        };
    }

    stdout.push_str(&format!(
        "{NAME}: ok — ledger intact; {}/{approved_n} is a pool size, \
         not a distinct-proposition count; report is not a verdict\n",
        items.len()
    ));
    Outcome {
        code: EXIT_OK,
        stdout,
        stderr: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, status: &str) -> ScanItem {
        ScanItem {
            id: id.into(),
            stem: format!("{id} stem tokens here"),
            key: format!("{id} key"),
            status: status.into(),
        }
    }

    fn live_shaped_items() -> Vec<ScanItem> {
        vec![
            item("m09-q111", "approved"),
            item("m09-q242", "approved"),
            item("m09-q113", "approved"),
            item("m09-q202", "approved"),
            item("m09-q122", "approved"),
            item("m09-q234", "approved"),
            item("m09-q140", "approved"),
            item("m09-q209", "approved"),
            item("m09-q206", "approved"),
            item("m09-q207", "approved"),
        ]
    }

    #[test]
    fn empty_union_is_zero_not_a_hundred() {
        let empty = BTreeSet::new();
        assert_eq!(sim_percent(&empty, &empty), 0);
        assert_eq!(sim_percent(&tokens(""), &tokens("")), 0);
    }

    #[test]
    fn normalize_matches_c3_separators() {
        assert_eq!(normalize("Hot-aisle / cold aisle"), "hot aisle cold aisle");
        assert_eq!(tokens("data-centre").len(), 2);
    }

    #[test]
    fn empty_ledger_is_error() {
        let mut led = plant_ledger();
        led.pairs.clear();
        let errs = check_ledger(&led, &live_shaped_items());
        assert!(
            errs.iter()
                .any(|e| e.contains("empty [[pair]]") || e.contains("zero [[pair]]")),
            "{errs:?}"
        );
    }

    #[test]
    fn missing_required_pair_is_red() {
        let mut led = plant_ledger();
        led.pairs.remove(0);
        let errs = check_ledger(&led, &live_shaped_items());
        assert!(
            errs.iter().any(|e| e.contains(REQUIRED_PAIR_IDS[0])),
            "{errs:?}"
        );
    }

    #[test]
    fn empty_item_list_is_error() {
        let errs = check_ledger(&plant_ledger(), &[]);
        assert!(errs.iter().any(|e| e.contains("zero items")), "{errs:?}");
    }

    #[test]
    fn fewer_than_two_approved_is_error() {
        let items = vec![item("only", "approved")];
        let errs = check_ledger(&plant_ledger(), &items);
        assert!(
            errs.iter().any(|e| e.contains("fewer than two")),
            "{errs:?}"
        );
    }

    #[test]
    fn known_distinct_must_remain() {
        let mut led = plant_ledger();
        led.known_distinct.clear();
        let errs = check_ledger(&led, &live_shaped_items());
        assert!(
            errs.iter().any(|e| e.contains(REQUIRED_DISTINCT_IDS[0])),
            "{errs:?}"
        );
    }

    #[test]
    fn adjudicated_without_reason_is_red() {
        let mut led = plant_ledger();
        if let Some(toml::Value::Table(t)) = led.pairs.get_mut(0) {
            t.insert(
                "status".into(),
                toml::Value::String(STATUS_ADJUDICATED.into()),
            );
            t.insert("adjudication_reason".into(), toml::Value::String("".into()));
        }
        let errs = check_ledger(&led, &live_shaped_items());
        assert!(
            errs.iter().any(|e| e.contains("adjudication_reason")),
            "{errs:?}"
        );
    }

    #[test]
    fn overlap_report_is_not_a_fail_condition() {
        let led = plant_ledger();
        let items = live_shaped_items();
        let report = overlap_report(&items, &led);
        assert!(report[0].contains("NOT a grader-of-record"), "{report:?}");
        let errs = check_ledger(&led, &items);
        assert!(
            errs.is_empty(),
            "report must not leak into the verdict: {errs:?}"
        );
    }

    #[test]
    fn in_process_selftest_plants_go_red() {
        let fails = run_selftest(&live_shaped_items());
        assert!(fails.is_empty(), "plants must trip: {fails:?}");
    }
}
