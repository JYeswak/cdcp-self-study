//! Learner Anki export — the `.apkg` a learner imports is the product.
//!
//! Extracted from `scripts/export_anki.py` by
//! `bd-substrate-rust-migration-jhd.13` (EXTRACT-THEN-DELETE). This is not a
//! `cdcp_gate` concern: a learner double-clicks `dist/anki/cdcp_bank.apkg`.
//!
//! # Contract (post e13a)
//!
//! * bank / seed42 export **approved-only** (retired + draft never ship)
//! * empty bank / empty approved pool / empty filter is an ERROR
//! * a card whose `correct` letter does not resolve against `choices` is
//!   an ERROR (named, with ids). The count is printed on the green path too.
//!   Every-card-unresolvable is an ERROR (anti-vacuous: cannot skip them
//!   and ship an empty-looking pass).
//! * collection timestamps and every zip entry mtime come from
//!   `SOURCE_DATE_EPOCH` or [`PINNED_EPOCH`] — never wall clock, never a temp
//!   file's mtime
//! * two successive exports of the same inputs are byte-identical
//! * `--check` plants a clock leak (bytes MUST differ) then asserts live
//!   identity (bytes MUST match) and the pinned `col.crt`
//!
//! # What this cannot decide
//!
//! Whether Anki imports the deck (Anki is not wired). Whether a card teaches
//! anything. Whether sqlite header bytes match CPython's `sqlite3` — they
//! often do not across engines. The content contract is card count, approved
//! filter, pinned clock, and two-run identity of **this** writer.
//! It therefore does not "guarantee", "prove", or "make impossible" anything
//! about the learner's experience.
//!
//! Zip uses the STORE method with a hand-rolled local/central header so the
//! archive mtime cannot leak a host clock. Compression is not the product;
//! the notes are.
#![forbid(unsafe_code)]

use serde_json::json;
use sha1::Sha1;
use sha2::Sha256;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TMP_SEQ: AtomicU64 = AtomicU64::new(1);

fn tmp_anki2(tag: &str) -> PathBuf {
    let n = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "cdcp-anki-{}-{}-{}-{}.anki2",
        tag,
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
}

pub const NAME: &str = "export-anki";
pub const SUMMARY: &str = "export approved bank items to Anki TSV/CSV/.apkg (learner product)";

/// Directory, relative to the engine root, holding one TOML per bank item.
pub const ITEMS_DIR_REL: &str = "bank/items";
/// Directory, relative to the engine root, holding the generated web packs.
pub const WEB_DATA_REL: &str = "web/data";
/// Default `--out`, relative to the engine root.
pub const DEFAULT_OUT_REL: &str = "dist/anki";
/// Default `--format` list.
pub const DEFAULT_FORMAT: &str = "tsv,apkg";
/// Default `--deck-name`.
pub const DEFAULT_DECK_NAME: &str = "CDCP Study";
/// Live-tree approved-card pin. A drift here is a bank decision, not a test fix.
pub const EXPECTED_APPROVED_LIVE: usize = 866;

/// Note type / deck ids (stable for re-import friendliness).
pub const MODEL_ID: i64 = 1_699_990_001_001;
pub const DECK_ID: i64 = 1_699_990_002_001;

/// Collection + zip clock. NEVER `time.time()`. NEVER filesystem mtime.
/// `SOURCE_DATE_EPOCH` (reproducible-builds) wins; else this pinned epoch.
/// 2023-11-14T22:13:20Z — same era as MODEL_ID / DECK_ID.
pub const PINNED_EPOCH: i64 = 1_700_000_000;
const ZIP_DOS_MIN: i64 = 315_532_800; // 1980-01-01T00:00:00Z
const ZIP_DOS_MAX: i64 = 4_354_812_799; // 2107-12-31T23:59:59Z

/// Anki field separator inside `notes.flds`.
const FSEP: char = '\x1f';

const STEM_BANK: &str = "cdcp_bank";
const STEM_SEED42: &str = "cdcp_seed42_bank";
const STEM_KEYS: &str = "cdcp_seed42_mock40";

const TSV_COMMENT_1: &str = "# CDCP Study Anki export — stem / answer / explanation / module\n";
const TSV_COMMENT_2: &str = "# Not a credential. Import as Basic (or map 4 fields).\n";
const CSV_HEADER: [&str; 4] = ["stem", "answer", "explanation", "module"];
const KNOWN_FORMATS: [&str; 3] = ["tsv", "csv", "apkg"];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AnkiError {
    #[error("{0}")]
    Msg(String),
}

impl AnkiError {
    fn msg(s: impl Into<String>) -> Self {
        Self::Msg(s.into())
    }
}

/// Everything the run decided, before anything is written.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub files: Vec<(PathBuf, Vec<u8>)>,
    pub cards: usize,
    pub scanned: usize,
}

impl Outcome {
    fn fail(msg: &str) -> Self {
        Self {
            code: 1,
            stdout: String::new(),
            stderr: format!("{msg}\n"),
            files: Vec::new(),
            cards: 0,
            scanned: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Bank,
    Seed42,
    Keys,
}

impl Source {
    fn as_str(self) -> &'static str {
        match self {
            Source::Bank => "bank",
            Source::Seed42 => "seed42",
            Source::Keys => "keys",
        }
    }

    pub fn parse(s: &str) -> Result<Self, AnkiError> {
        match s {
            "bank" => Ok(Source::Bank),
            "seed42" => Ok(Source::Seed42),
            "keys" => Ok(Source::Keys),
            other => Err(AnkiError::msg(format!(
                "--source: invalid choice {other:?} (choose from 'bank', 'seed42', 'keys')"
            ))),
        }
    }
}

/// Operator-facing request. Paths are as the caller gave them.
#[derive(Debug, Clone)]
pub struct Request {
    pub root: PathBuf,
    pub source: Source,
    pub out: PathBuf,
    pub format: String,
    pub module: Option<i64>,
    pub tag: Option<String>,
    pub limit: Option<i64>,
    pub seed: Option<i64>,
    pub deck_name: String,
    pub check: bool,
}

impl Request {
    pub fn default_for(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            source: Source::Bank,
            out: root.join(DEFAULT_OUT_REL),
            format: DEFAULT_FORMAT.into(),
            module: None,
            tag: None,
            limit: None,
            seed: None,
            deck_name: DEFAULT_DECK_NAME.into(),
            check: false,
        }
    }
}

/// A drawable card. Permissive: plants and fixtures are not `cdcp_bank` items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub stem: String,
    pub choices: Vec<String>,
    pub correct: String,
    pub explanation: String,
    pub module: String,
    pub status: String,
    pub tags: Vec<String>,
    pub topic_ids: Vec<String>,
}

/// Unix seconds stamped into col/notes/cards and every zip entry.
///
/// Wall clock is not an input. The deck is a function of the bank (and
/// optionally `SOURCE_DATE_EPOCH`).
pub fn deck_clock() -> Result<i64, AnkiError> {
    match std::env::var("SOURCE_DATE_EPOCH") {
        Err(_) => Ok(PINNED_EPOCH),
        Ok(raw) if raw.trim().is_empty() => Ok(PINNED_EPOCH),
        Ok(raw) => {
            let value: i64 = raw.parse().map_err(|_| {
                AnkiError::msg(format!("SOURCE_DATE_EPOCH is not an integer: {raw:?}"))
            })?;
            if !(ZIP_DOS_MIN..=ZIP_DOS_MAX).contains(&value) {
                return Err(AnkiError::msg(format!(
                    "SOURCE_DATE_EPOCH {value} is outside the zip DOS range [{ZIP_DOS_MIN}, {ZIP_DOS_MAX}]"
                )));
            }
            Ok(value)
        }
    }
}

/// UTC calendar components for a zip DOS timestamp.
/// `(year, month, day, hour, min, sec)` — TZ-independent (never localtime).
pub type ZipDateTime = (i32, u32, u32, u32, u32, u32);

/// UTC calendar tuple for a zip DOS timestamp. TZ-independent (never localtime).
pub fn zip_date_time(epoch: i64) -> ZipDateTime {
    let days = epoch.div_euclid(86_400);
    let rem = epoch.rem_euclid(86_400);
    let hour = (rem / 3_600) as u32;
    let min = ((rem % 3_600) / 60) as u32;
    let sec = (rem % 60) as u32;
    let (y, m, d) = civil_from_unix_days(days);
    (y, m, d, hour, min, sec)
}

/// Howard Hinnant's civil-from-days. `z` is days since Unix epoch (1970-01-01).
fn civil_from_unix_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

fn is_drawable(card: &Card) -> bool {
    let s = card.status.trim().to_ascii_lowercase();
    s != "retired" && s != "draft"
}

/// Resolve `correct` against `choices`. `None` means the Back would be a
/// bare letter or `'?'` — a card a learner cannot use.
///
/// Letters outside A–D, a letter past the end of `choices`, a multi-character
/// key, or an empty key are all unresolvable. This is the admissibility
/// check [`evaluate`] runs; the writer must never ship `None`.
fn resolved_answer(card: &Card) -> Option<String> {
    let correct = card.correct.trim().to_ascii_uppercase();
    if correct.len() != 1 {
        return None;
    }
    let idx = (correct.as_bytes()[0] as i32) - (b'A' as i32);
    if !(0..4).contains(&idx) {
        return None;
    }
    let i = idx as usize;
    if i >= card.choices.len() {
        return None;
    }
    Some(format!("{correct}) {}", card.choices[i]))
}

fn format_answer(card: &Card) -> String {
    resolved_answer(card).unwrap_or_else(|| {
        let correct = card.correct.trim().to_ascii_uppercase();
        if correct.is_empty() {
            "?".into()
        } else {
            correct
        }
    })
}

fn unresolvable_ids(items: &[Card]) -> Vec<String> {
    items
        .iter()
        .filter(|c| resolved_answer(c).is_none())
        .map(|c| {
            let id = c.id.trim();
            if id.is_empty() {
                "<missing-id>".into()
            } else {
                id.to_string()
            }
        })
        .collect()
}

fn fail_unresolvable(ids: &[String], n_cards: usize, scanned: usize, stderr_pre: &str) -> Outcome {
    let n = ids.len();
    let named = ids.join(", ");
    let head = if n > 0 && n == n_cards {
        format!("FAIL: {n} unresolvable answer(s) (every card unresolvable): {named}")
    } else {
        format!("FAIL: {n} unresolvable answer(s): {named}")
    };
    let mut o = Outcome::fail(&head);
    o.stderr = format!("{stderr_pre}{}", o.stderr);
    o.scanned = scanned;
    o
}

fn card_fields(card: &Card) -> [String; 4] {
    [
        card.stem.trim().to_string(),
        format_answer(card),
        card.explanation.trim().to_string(),
        card.module.clone(),
    ]
}

fn csv_row(fields: &[String], delimiter: char, lineterminator: &str) -> String {
    let mut out = String::new();
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            out.push(delimiter);
        }
        let needs_quote = f.chars().any(|c| {
            c == delimiter || c == '"' || c == '\r' || c == '\n' || lineterminator.contains(c)
        });
        if needs_quote {
            out.push('"');
            for c in f.chars() {
                if c == '"' {
                    out.push('"');
                }
                out.push(c);
            }
            out.push('"');
        } else {
            out.push_str(f);
        }
    }
    out.push_str(lineterminator);
    out
}

fn write_tsv_body(items: &[Card]) -> String {
    let mut out = String::new();
    out.push_str(TSV_COMMENT_1);
    out.push_str(TSV_COMMENT_2);
    for it in items {
        let [stem, answer, explanation, module] = card_fields(it);
        let flat = |s: String| s.replace(['\t', '\n'], " ");
        out.push_str(&csv_row(
            &[flat(stem), flat(answer), flat(explanation), module],
            '\t',
            "\n",
        ));
    }
    out
}

fn write_csv_body(items: &[Card]) -> String {
    let mut out = String::new();
    out.push_str(&csv_row(
        &CSV_HEADER
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>(),
        ',',
        "\n",
    ));
    for it in items {
        let f = card_fields(it);
        out.push_str(&csv_row(&f, ',', "\n"));
    }
    out
}

fn readme_body(cards: usize, source: &str) -> String {
    format!(
        "CDCP Study — Anki export\n\
         ========================\n\
         Study tool only. Does NOT grant EPI/EXIN certification.\n\
         Not an exam dump; original educational bank content.\n\n\
         Fields: stem | answer | explanation | module\n\
         TSV: import in Anki → File → Import → map 4 fields (or use Basic + Extra).\n\
         APKG: double-click / File → Import to load the deck.\n\
         Cards: {cards}\n\
         Source: {source}\n"
    )
}

fn filter_items(
    items: &[Card],
    module: Option<i64>,
    tag: Option<&str>,
    limit: Option<i64>,
    seed: Option<i64>,
) -> Vec<Card> {
    let mut out: Vec<Card> = items.to_vec();
    if let Some(want) = module {
        out.retain(|it| module_as_i64(&it.module) == Some(want));
    }
    if let Some(tag) = tag {
        let want = tag.to_ascii_lowercase();
        out.retain(|it| {
            let tag_blob = it.tags.join(" ").to_ascii_lowercase();
            let topic_blob = it.topic_ids.join(" ").to_ascii_lowercase();
            tag_blob.contains(&want) || topic_blob.contains(&want)
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(limit) = limit {
        if limit > 0 && out.len() > limit as usize {
            if let Some(seed) = seed {
                shuffle_with_seed(&mut out, seed);
                out.truncate(limit as usize);
                out.sort_by(|a, b| a.id.cmp(&b.id));
            } else {
                out.truncate(limit as usize);
            }
        }
    }
    out
}

fn module_as_i64(s: &str) -> Option<i64> {
    s.trim().parse().ok()
}

/// Deterministic Fisher–Yates. Not CPython MT19937 — `--limit --seed` is an
/// operator convenience, not a published exam form.
fn shuffle_with_seed(items: &mut [Card], seed: i64) {
    let mut state = (seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xA24B_AED4_96E6_61D4;
    if items.len() < 2 {
        return;
    }
    for i in (1..items.len()).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (state as usize) % (i + 1);
        items.swap(i, j);
    }
}

// ── loaders ────────────────────────────────────────────────────────────────

fn load_bank_items(root: &Path) -> Result<Vec<Card>, AnkiError> {
    let dir = root.join(ITEMS_DIR_REL);
    let mut paths: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
            .collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    let mut out = Vec::new();
    for p in paths {
        let text = fs::read_to_string(&p)
            .map_err(|e| AnkiError::msg(format!("read {}: {e}", p.display())))?;
        let v: toml::Value = text
            .parse()
            .map_err(|e| AnkiError::msg(format!("parse {}: {e}", p.display())))?;
        if let Some(card) = card_from_toml(&v) {
            out.push(card);
        }
    }
    Ok(out)
}

fn card_from_toml(v: &toml::Value) -> Option<Card> {
    let id = v.get("id")?.as_str()?.to_string();
    Some(Card {
        id,
        stem: toml_str(v, "stem"),
        choices: toml_str_list(v.get("choices")),
        correct: toml_str(v, "correct"),
        explanation: toml_str(v, "explanation"),
        module: toml_module(v.get("module")),
        status: toml_str(v, "status"),
        tags: toml_str_list(v.get("tags")),
        topic_ids: toml_str_list(v.get("topic_ids")),
    })
}

fn toml_str(v: &toml::Value, key: &str) -> String {
    match v.get(key) {
        Some(toml::Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn toml_module(v: Option<&toml::Value>) -> String {
    match v {
        None => String::new(),
        Some(toml::Value::String(s)) if s.is_empty() => String::new(),
        Some(toml::Value::Integer(i)) => i.to_string(),
        Some(toml::Value::Float(f)) => {
            if f.fract() == 0.0 {
                (*f as i64).to_string()
            } else {
                f.to_string()
            }
        }
        Some(toml::Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

fn toml_str_list(v: Option<&toml::Value>) -> Vec<String> {
    match v {
        Some(toml::Value::Array(a)) => a
            .iter()
            .map(|x| match x {
                toml::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect(),
        Some(toml::Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn load_json_value(path: &Path) -> Result<serde_json::Value, AnkiError> {
    let text = fs::read_to_string(path)
        .map_err(|e| AnkiError::msg(format!("read {}: {e}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|e| AnkiError::msg(format!("parse {}: {e}", path.display())))
}

fn card_from_json(v: &serde_json::Value) -> Option<Card> {
    let id = v.get("id")?.as_str()?.to_string();
    Some(Card {
        id,
        stem: json_str(v, "stem"),
        choices: json_str_list(v.get("choices")),
        correct: json_str(v, "correct"),
        explanation: json_str(v, "explanation"),
        module: json_module(v.get("module")),
        status: json_str(v, "status"),
        tags: json_str_list(v.get("tags")),
        topic_ids: json_str_list(v.get("topic_ids")),
    })
}

fn json_str(v: &serde_json::Value, key: &str) -> String {
    match v.get(key) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::Bool(b)) => b.to_string(),
        Some(serde_json::Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn json_module(v: Option<&serde_json::Value>) -> String {
    match v {
        None | Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

fn json_str_list(v: Option<&serde_json::Value>) -> Vec<String> {
    match v {
        Some(serde_json::Value::Array(a)) => a
            .iter()
            .map(|x| match x {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect(),
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn load_seed42_bank_items(root: &Path) -> Result<Option<Vec<Card>>, AnkiError> {
    let path = root.join(WEB_DATA_REL).join("bank_items_seed42.json");
    if !path.is_file() {
        return Ok(None);
    }
    let data = load_json_value(&path)?;
    let rows = match &data {
        serde_json::Value::Array(v) => v.clone(),
        serde_json::Value::Object(m) => match m.get("items") {
            Some(serde_json::Value::Array(v)) => v.clone(),
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };
    Ok(Some(rows.iter().filter_map(card_from_json).collect()))
}

fn load_keys_seed42_pack(root: &Path) -> Result<Option<Vec<Card>>, AnkiError> {
    let mock_path = root.join(WEB_DATA_REL).join("mock40_seed42.json");
    let keys_path = root.join(WEB_DATA_REL).join("keys_seed42.json");
    if mock_path.is_file() && keys_path.is_file() {
        let mock = load_json_value(&mock_path)?;
        let keys = load_json_value(&keys_path)?;
        let key_rows = keys
            .get("keys")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mock_items = mock
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut rows = Vec::new();
        for it in &mock_items {
            let id = match it.get("id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => return Err(AnkiError::msg("mock40 item has no 'id'")),
            };
            let k = key_rows
                .iter()
                .rev()
                .find(|k| k.get("item_id").and_then(|v| v.as_str()) == Some(id.as_str()));
            rows.push(Card {
                id: id.clone(),
                stem: json_str(it, "stem"),
                choices: json_str_list(it.get("choices")),
                correct: k.map(|k| json_str(k, "correct")).unwrap_or_default(),
                explanation: k.map(|k| json_str(k, "explanation")).unwrap_or_default(),
                module: json_module(it.get("module")),
                status: String::new(),
                tags: Vec::new(),
                topic_ids: Vec::new(),
            });
        }
        if rows.iter().any(|r| r.module.is_empty()) {
            let bank = load_bank_items(root)?;
            for r in rows.iter_mut() {
                if r.module.is_empty() {
                    if let Some(b) = bank.iter().rev().find(|b| b.id == r.id) {
                        r.module = b.module.clone();
                    }
                }
            }
        }
        return Ok(Some(rows));
    }

    let fix = root.join("goldens/fixtures/mock40_seed42.json");
    if !fix.is_file() {
        return Ok(None);
    }
    let data = load_json_value(&fix)?;
    let items = data
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if items.is_empty() {
        return Ok(None);
    }
    let mut rows = Vec::new();
    for it in &items {
        let id = it
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AnkiError::msg("goldens fixture item has no 'id'"))?
            .to_string();
        let stem = it
            .get("stem")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AnkiError::msg("goldens fixture item has no 'stem'"))?
            .to_string();
        let correct = it
            .get("correct")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AnkiError::msg("goldens fixture item has no 'correct'"))?
            .to_string();
        rows.push(Card {
            id,
            stem,
            choices: json_str_list(it.get("choices")),
            correct,
            explanation: json_str(it, "explanation"),
            module: json_module(it.get("module")),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        });
    }
    let bank = load_bank_items(root)?;
    for r in rows.iter_mut() {
        if r.explanation.is_empty() {
            if let Some(b) = bank.iter().rev().find(|b| b.id == r.id) {
                r.explanation = b.explanation.clone();
                if r.module.is_empty() {
                    r.module = b.module.clone();
                }
            }
        }
    }
    Ok(Some(rows))
}

// ── .apkg ──────────────────────────────────────────────────────────────────

fn sha1_hex(bytes: &[u8]) -> String {
    use sha1::Digest;
    hex::encode(Sha1::digest(bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    hex::encode(Sha256::digest(bytes))
}

fn guid_for(item_id: &str) -> String {
    sha1_hex(format!("cdcp-anki:{item_id}").as_bytes())
        .chars()
        .take(10)
        .collect()
}

fn csum(sfld: &str) -> i64 {
    let h = sha1_hex(sfld.as_bytes());
    i64::from_str_radix(&h[..8], 16).unwrap_or(0)
}

fn stable_id(prefix: &str, iid: &str, i: usize) -> i64 {
    let h = sha1_hex(format!("{prefix}:{iid}").as_bytes());
    let n = i64::from_str_radix(&h[..12], 16).unwrap_or(0);
    n.rem_euclid(10_i64.pow(13)) + i as i64
}

/// Build `collection.anki2` bytes. `now` is the collection clock.
pub fn write_collection_anki2(
    items: &[Card],
    deck_name: &str,
    now: i64,
) -> Result<Vec<u8>, AnkiError> {
    let model = json!({
        MODEL_ID.to_string(): {
            "id": MODEL_ID,
            "name": "CDCP Study Basic",
            "type": 0,
            "mod": now,
            "usn": -1,
            "sortf": 0,
            "did": DECK_ID,
            "tmpls": [{
                "name": "Card 1",
                "ord": 0,
                "qfmt": "{{Stem}}<br><br><i>Module {{Module}}</i>",
                "afmt": "{{FrontSide}}<hr id=answer>{{Answer}}<br><br>{{Explanation}}",
                "bqfmt": "",
                "bafmt": "",
                "did": serde_json::Value::Null,
                "bfont": "",
                "bsize": 0
            }],
            "flds": [
                field("Stem", 0, 20),
                field("Answer", 1, 20),
                field("Explanation", 2, 16),
                field("Module", 3, 14)
            ],
            "css": ".card { font-family: arial; font-size: 18px; text-align: left; color: black; background-color: white; }",
            "latexPre": "",
            "latexPost": "",
            "latexsvg": false,
            "req": [[0, "all", [0]]],
            "tags": [],
            "vers": []
        }
    });
    let decks = json!({
        "1": {
            "id": 1,
            "name": "Default",
            "mod": now,
            "usn": -1,
            "collapsed": false,
            "browserCollapsed": false,
            "desc": "",
            "dyn": 0,
            "conf": 1,
            "extendNew": 0,
            "extendRev": 0
        },
        DECK_ID.to_string(): {
            "id": DECK_ID,
            "name": deck_name,
            "mod": now,
            "usn": -1,
            "collapsed": false,
            "browserCollapsed": false,
            "desc": "CDCP self-study cards. Study signal only — not EPI/EXIN certification.",
            "dyn": 0,
            "conf": 1,
            "extendNew": 0,
            "extendRev": 0
        }
    });
    let conf = json!({
        "nextPos": 1,
        "estTimes": true,
        "activeDecks": [DECK_ID],
        "sortType": "noteFld",
        "timeLim": 0,
        "sortBackwards": false,
        "addToCur": true,
        "curDeck": DECK_ID,
        "newBury": true,
        "newSpread": 0,
        "dueCounts": true,
        "curModel": MODEL_ID,
        "collapseTime": 1200
    });
    let dconf = json!({
        "1": {
            "id": 1,
            "name": "Default",
            "mod": 0,
            "usn": 0,
            "maxTaken": 60,
            "autoplay": true,
            "timer": 0,
            "replayq": true,
            "new": {
                "bury": true,
                "delays": [1, 10],
                "initialFactor": 2500,
                "ints": [1, 4, 0],
                "order": 1,
                "perDay": 20
            },
            "rev": {
                "bury": true,
                "ease4": 1.3,
                "ivlFct": 1,
                "maxIvl": 36500,
                "perDay": 200,
                "hardFactor": 1.2
            },
            "lapse": {
                "delays": [10],
                "leechAction": 0,
                "leechFails": 8,
                "minInt": 1,
                "mult": 0
            }
        }
    });

    let tmp = tmp_anki2("col");
    // A leftover from a killed run must not become this run's input.
    let _ = fs::remove_file(&tmp);
    let bytes = (|| -> Result<Vec<u8>, AnkiError> {
        {
            let conn = rusqlite::Connection::open(&tmp)
                .map_err(|e| AnkiError::msg(format!("sqlite open: {e}")))?;
            conn.execute_batch(
                r#"
                PRAGMA journal_mode = OFF;
                PRAGMA synchronous = OFF;
                CREATE TABLE col (
                  id integer primary key,
                  crt integer not null,
                  mod integer not null,
                  scm integer not null,
                  ver integer not null,
                  dty integer not null,
                  usn integer not null,
                  ls integer not null,
                  conf text not null,
                  models text not null,
                  decks text not null,
                  dconf text not null,
                  tags text not null
                );
                CREATE TABLE notes (
                  id integer primary key,
                  guid text not null,
                  mid integer not null,
                  mod integer not null,
                  usn integer not null,
                  tags text not null,
                  flds text not null,
                  sfld text not null,
                  csum integer not null,
                  flags integer not null,
                  data text not null
                );
                CREATE TABLE cards (
                  id integer primary key,
                  nid integer not null,
                  did integer not null,
                  ord integer not null,
                  mod integer not null,
                  usn integer not null,
                  type integer not null,
                  queue integer not null,
                  due integer not null,
                  ivl integer not null,
                  factor integer not null,
                  reps integer not null,
                  lapses integer not null,
                  left integer not null,
                  odue integer not null,
                  odid integer not null,
                  flags integer not null,
                  data text not null
                );
                CREATE TABLE revlog (
                  id integer primary key,
                  cid integer not null,
                  usn integer not null,
                  ease integer not null,
                  ivl integer not null,
                  lastIvl integer not null,
                  factor integer not null,
                  time integer not null,
                  type integer not null
                );
                CREATE TABLE graves (
                  usn integer not null,
                  oid integer not null,
                  type integer not null
                );
                CREATE INDEX ix_notes_usn on notes (usn);
                CREATE INDEX ix_cards_usn on cards (usn);
                CREATE INDEX ix_revlog_usn on revlog (usn);
                CREATE INDEX ix_cards_nid on cards (nid);
                CREATE INDEX ix_cards_sched on cards (did, queue, due);
                CREATE INDEX ix_revlog_cid on revlog (cid);
                CREATE INDEX ix_notes_csum on notes (csum);
                "#,
            )
            .map_err(|e| AnkiError::msg(format!("sqlite schema: {e}")))?;

            let models = serde_json::to_string(&model)
                .map_err(|e| AnkiError::msg(format!("json models: {e}")))?;
            let decks_s = serde_json::to_string(&decks)
                .map_err(|e| AnkiError::msg(format!("json decks: {e}")))?;
            let conf_s = serde_json::to_string(&conf)
                .map_err(|e| AnkiError::msg(format!("json conf: {e}")))?;
            let dconf_s = serde_json::to_string(&dconf)
                .map_err(|e| AnkiError::msg(format!("json dconf: {e}")))?;
            let tags_s = "{}".to_string();

            conn.execute(
                "INSERT INTO col (id,crt,mod,scm,ver,dty,usn,ls,conf,models,decks,dconf,tags) \
                 VALUES (1,?,?,?,11,0,0,0,?,?,?,?,?)",
                rusqlite::params![
                    now,
                    now * 1000,
                    now * 1000,
                    conf_s,
                    models,
                    decks_s,
                    dconf_s,
                    tags_s
                ],
            )
            .map_err(|e| AnkiError::msg(format!("sqlite col: {e}")))?;

            for (i, it) in items.iter().enumerate() {
                let [stem, answer, explanation, module] = card_fields(it);
                let iid = if it.id.is_empty() {
                    format!("row-{i}")
                } else {
                    it.id.clone()
                };
                let note_id = stable_id("note", &iid, i);
                let card_id = stable_id("card", &iid, i);
                let flds = [
                    stem.as_str(),
                    answer.as_str(),
                    explanation.as_str(),
                    module.as_str(),
                ]
                .join(&FSEP.to_string());
                let tags = if module.is_empty() {
                    " cdcp-study ".to_string()
                } else {
                    format!(" cdcp-study module{module} ")
                };
                conn.execute(
                    "INSERT INTO notes (id,guid,mid,mod,usn,tags,flds,sfld,csum,flags,data) \
                     VALUES (?,?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        note_id,
                        guid_for(&iid),
                        MODEL_ID,
                        now,
                        -1i64,
                        tags,
                        flds,
                        stem,
                        csum(&stem),
                        0i64,
                        ""
                    ],
                )
                .map_err(|e| AnkiError::msg(format!("sqlite note {iid}: {e}")))?;
                conn.execute(
                    "INSERT INTO cards (id,nid,did,ord,mod,usn,type,queue,due,ivl,factor,\
                     reps,lapses,left,odue,odid,flags,data) \
                     VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        card_id,
                        note_id,
                        DECK_ID,
                        0i64,
                        now,
                        -1i64,
                        0i64,
                        0i64,
                        (i as i64) + 1,
                        0i64,
                        0i64,
                        0i64,
                        0i64,
                        0i64,
                        0i64,
                        0i64,
                        0i64,
                        ""
                    ],
                )
                .map_err(|e| AnkiError::msg(format!("sqlite card {iid}: {e}")))?;
            }
            conn.execute_batch("PRAGMA optimize;")
                .map_err(|e| AnkiError::msg(format!("sqlite close-pragma: {e}")))?;
        }
        fs::read(&tmp).map_err(|e| AnkiError::msg(format!("read anki2: {e}")))
    })();
    let _ = fs::remove_file(&tmp);
    bytes
}

fn field(name: &str, ord: i64, size: i64) -> serde_json::Value {
    json!({
        "name": name,
        "ord": ord,
        "sticky": false,
        "rtl": false,
        "font": "Arial",
        "size": size,
        "media": []
    })
}

/// ZIP_STORED archive whose entry mtimes are `epoch`, not file mtimes.
///
/// `ZipFile.write()` of a temp file would copy the host mtime into the local
/// header. We emit ZipInfo-equivalent headers (Unix create_system=3,
/// mode 0644, empty extra/comment) so the host cannot leak into the bytes.
pub fn write_deterministic_zip(entries: &[(String, Vec<u8>)], epoch: i64) -> Vec<u8> {
    let (year, month, day, hour, min, sec) = zip_date_time(epoch);
    let dos_time = ((hour as u16) << 11) | ((min as u16) << 5) | ((sec as u16) / 2);
    let dos_date = (((year - 1980) as u16) << 9) | ((month as u16) << 5) | (day as u16);

    let mut buf = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let name_b = name.as_bytes();
        let crc = crc32(data);
        let size = data.len() as u32;
        let local_off = buf.len() as u32;

        buf.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        buf.extend_from_slice(&20u16.to_le_bytes()); // version needed
        buf.extend_from_slice(&0u16.to_le_bytes()); // flags
        buf.extend_from_slice(&0u16.to_le_bytes()); // STORE
        buf.extend_from_slice(&dos_time.to_le_bytes());
        buf.extend_from_slice(&dos_date.to_le_bytes());
        buf.extend_from_slice(&crc.to_le_bytes());
        buf.extend_from_slice(&size.to_le_bytes());
        buf.extend_from_slice(&size.to_le_bytes());
        buf.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // extra
        buf.extend_from_slice(name_b);
        buf.extend_from_slice(data);

        // version made by: 20 | (3 << 8) = Unix + 2.0
        let made_by: u16 = 20 | (3 << 8);
        let ext_attr: u32 = 0o644 << 16;
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&made_by.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&dos_time.to_le_bytes());
        central.extend_from_slice(&dos_date.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra
        central.extend_from_slice(&0u16.to_le_bytes()); // comment
        central.extend_from_slice(&0u16.to_le_bytes()); // disk
        central.extend_from_slice(&0u16.to_le_bytes()); // int attr
        central.extend_from_slice(&ext_attr.to_le_bytes());
        central.extend_from_slice(&local_off.to_le_bytes());
        central.extend_from_slice(name_b);
    }
    let cd_off = buf.len() as u32;
    let cd_size = central.len() as u32;
    let n = entries.len() as u16;
    buf.extend_from_slice(&central);
    buf.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes());
    buf.extend_from_slice(&n.to_le_bytes());
    buf.extend_from_slice(&n.to_le_bytes());
    buf.extend_from_slice(&cd_size.to_le_bytes());
    buf.extend_from_slice(&cd_off.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes());
    buf
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Pack `collection.anki2` + `media` into a deterministic `.apkg`.
pub fn write_apkg(items: &[Card], deck_name: &str, now: i64) -> Result<Vec<u8>, AnkiError> {
    let db = write_collection_anki2(items, deck_name, now)?;
    Ok(write_deterministic_zip(
        &[
            ("collection.anki2".into(), db),
            ("media".into(), b"{}".to_vec()),
        ],
        now,
    ))
}

/// `(col.crt, col.mod, col.scm, zip date_time of collection.anki2)`.
pub fn peek_apkg(bytes: &[u8]) -> Result<(i64, i64, i64, ZipDateTime), AnkiError> {
    let (db, date_time) = unzip_collection(bytes)?;
    let tmp = tmp_anki2("peek");
    let _ = fs::remove_file(&tmp);
    fs::write(&tmp, &db).map_err(|e| AnkiError::msg(format!("peek write: {e}")))?;
    let result = (|| -> Result<(i64, i64, i64), AnkiError> {
        let conn = rusqlite::Connection::open(&tmp)
            .map_err(|e| AnkiError::msg(format!("peek open: {e}")))?;
        conn.query_row("SELECT crt, mod, scm FROM col WHERE id = 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| AnkiError::msg(format!("peek select: {e}")))
    })();
    let _ = fs::remove_file(&tmp);
    let (crt, mod_, scm) = result?;
    Ok((crt, mod_, scm, date_time))
}

fn unzip_collection(bytes: &[u8]) -> Result<(Vec<u8>, ZipDateTime), AnkiError> {
    if bytes.len() < 30 {
        return Err(AnkiError::msg("apkg too short"));
    }
    let mut i = 0usize;
    while i + 30 <= bytes.len() {
        let sig = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
        if sig != 0x0403_4b50 {
            break;
        }
        let flags = u16::from_le_bytes(bytes[i + 6..i + 8].try_into().unwrap());
        let method = u16::from_le_bytes(bytes[i + 8..i + 10].try_into().unwrap());
        let dos_time = u16::from_le_bytes(bytes[i + 10..i + 12].try_into().unwrap());
        let dos_date = u16::from_le_bytes(bytes[i + 12..i + 14].try_into().unwrap());
        let csize = u32::from_le_bytes(bytes[i + 18..i + 22].try_into().unwrap()) as usize;
        let name_len = u16::from_le_bytes(bytes[i + 26..i + 28].try_into().unwrap()) as usize;
        let extra_len = u16::from_le_bytes(bytes[i + 28..i + 30].try_into().unwrap()) as usize;
        let name_at = i + 30;
        let data_at = name_at + name_len + extra_len;
        if data_at + csize > bytes.len() {
            return Err(AnkiError::msg("apkg truncated"));
        }
        let name = std::str::from_utf8(&bytes[name_at..name_at + name_len])
            .map_err(|_| AnkiError::msg("apkg name not utf-8"))?;
        if name == "collection.anki2" {
            if method != 0 {
                return Err(AnkiError::msg(format!(
                    "collection.anki2 compress method {method} (expected STORE=0)"
                )));
            }
            if flags & 0x8 != 0 {
                return Err(AnkiError::msg("data descriptor bit not supported"));
            }
            let data = bytes[data_at..data_at + csize].to_vec();
            let hour = (dos_time >> 11) as u32;
            let min = ((dos_time >> 5) & 0x3f) as u32;
            let sec = ((dos_time & 0x1f) * 2) as u32;
            let year = 1980 + (dos_date >> 9) as i32;
            let month = ((dos_date >> 5) & 0xf) as u32;
            let day = (dos_date & 0x1f) as u32;
            return Ok((data, (year, month, day, hour, min, sec)));
        }
        i = data_at + csize;
    }
    Err(AnkiError::msg("collection.anki2 missing from apkg"))
}

fn count_notes(db: &[u8]) -> Result<usize, AnkiError> {
    let tmp = tmp_anki2("count");
    let _ = fs::remove_file(&tmp);
    fs::write(&tmp, db).map_err(|e| AnkiError::msg(format!("count write: {e}")))?;
    let n = (|| -> Result<i64, AnkiError> {
        let conn = rusqlite::Connection::open(&tmp)
            .map_err(|e| AnkiError::msg(format!("count open: {e}")))?;
        conn.query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .map_err(|e| AnkiError::msg(format!("count select: {e}")))
    })();
    let _ = fs::remove_file(&tmp);
    Ok(n? as usize)
}

/// Retired/draft ids that leaked into an `.apkg`. Empty is the contract.
pub fn retired_ids_in_apkg(bytes: &[u8], bank: &[Card]) -> Result<Vec<String>, AnkiError> {
    let (db, _) = unzip_collection(bytes)?;
    let tmp = tmp_anki2("scan");
    let _ = fs::remove_file(&tmp);
    fs::write(&tmp, &db).map_err(|e| AnkiError::msg(format!("scan write: {e}")))?;
    let guids: Result<Vec<String>, AnkiError> = (|| {
        let conn = rusqlite::Connection::open(&tmp)
            .map_err(|e| AnkiError::msg(format!("scan open: {e}")))?;
        let mut stmt = conn
            .prepare("SELECT guid FROM notes")
            .map_err(|e| AnkiError::msg(format!("scan prep: {e}")))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| AnkiError::msg(format!("scan query: {e}")))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| AnkiError::msg(format!("scan row: {e}")))?);
        }
        Ok(out)
    })();
    let _ = fs::remove_file(&tmp);
    let guids = guids?;
    let mut leaked = Vec::new();
    for it in bank {
        if !is_drawable(it) {
            let g = guid_for(&it.id);
            if guids.iter().any(|x| x == &g) {
                leaked.push(it.id.clone());
            }
        }
    }
    leaked.sort();
    Ok(leaked)
}

fn plant_items() -> Vec<Card> {
    vec![
        Card {
            id: "plant-a".into(),
            stem: "s".into(),
            choices: vec!["x".into()],
            correct: "A".into(),
            explanation: "e".into(),
            module: "1".into(),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        },
        Card {
            id: "plant-b".into(),
            stem: "t".into(),
            choices: vec!["x".into()],
            correct: "A".into(),
            explanation: "e".into(),
            module: "1".into(),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        },
    ]
}

/// L4: two different clocks MUST change the `.apkg`, or `--check` is vacuous.
pub fn planted_clock_leak_trips() -> Result<(), AnkiError> {
    let items = plant_items();
    let a = write_apkg(&items, DEFAULT_DECK_NAME, PINNED_EPOCH)?;
    let b = write_apkg(&items, DEFAULT_DECK_NAME, PINNED_EPOCH + 1)?;
    if a == b {
        return Err(AnkiError::msg(
            "planted clock leak did not change .apkg bytes — --check is vacuous",
        ));
    }
    Ok(())
}

fn run_repro_check(items: &[Card], deck_name: &str) -> Outcome {
    match planted_clock_leak_trips() {
        Ok(()) => {}
        Err(e) => {
            return Outcome {
                code: 1,
                stdout: String::new(),
                stderr: format!("FAIL: {e}\n"),
                files: Vec::new(),
                cards: items.len(),
                scanned: items.len(),
            };
        }
    }
    let mut stdout = String::from("export_anki check: planted clock leak trips (bytes differ)\n");

    let clock = match deck_clock() {
        Ok(c) => c,
        Err(e) => {
            return Outcome {
                code: 1,
                stdout,
                stderr: format!("FAIL: {e}\n"),
                files: Vec::new(),
                cards: items.len(),
                scanned: items.len(),
            };
        }
    };

    let a_apkg = match write_apkg(items, deck_name, clock) {
        Ok(b) => b,
        Err(e) => {
            return Outcome {
                code: 1,
                stdout,
                stderr: format!("FAIL: {e}\n"),
                files: Vec::new(),
                cards: items.len(),
                scanned: items.len(),
            };
        }
    };
    let a_tsv = write_tsv_body(items);
    // Cross a whole-second boundary so a leftover wall-clock stamp would flip.
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let b_apkg = match write_apkg(items, deck_name, clock) {
        Ok(b) => b,
        Err(e) => {
            return Outcome {
                code: 1,
                stdout,
                stderr: format!("FAIL: {e}\n"),
                files: Vec::new(),
                cards: items.len(),
                scanned: items.len(),
            };
        }
    };
    let b_tsv = write_tsv_body(items);
    if a_tsv != b_tsv {
        return Outcome {
            code: 1,
            stdout,
            stderr: "FAIL: TSV differed across two successive exports\n".into(),
            files: Vec::new(),
            cards: items.len(),
            scanned: items.len(),
        };
    }
    if a_apkg != b_apkg {
        let differ = a_apkg
            .iter()
            .zip(b_apkg.iter())
            .filter(|(x, y)| x != y)
            .count()
            + a_apkg.len().abs_diff(b_apkg.len());
        return Outcome {
            code: 1,
            stdout,
            stderr: format!(
                "FAIL: .apkg differed across two successive exports ({differ} of {} bytes)\n",
                a_apkg.len().max(b_apkg.len())
            ),
            files: Vec::new(),
            cards: items.len(),
            scanned: items.len(),
        };
    }

    let (crt, mod_, scm, date_time) = match peek_apkg(&a_apkg) {
        Ok(v) => v,
        Err(e) => {
            return Outcome {
                code: 1,
                stdout,
                stderr: format!("FAIL: {e}\n"),
                files: Vec::new(),
                cards: items.len(),
                scanned: items.len(),
            };
        }
    };
    let expect_dt = zip_date_time(clock);
    // DOS seconds are 2-second resolution; compare the DOS-rounded form.
    let expect_dos = (
        expect_dt.0,
        expect_dt.1,
        expect_dt.2,
        expect_dt.3,
        expect_dt.4,
        expect_dt.5 - (expect_dt.5 % 2),
    );
    if crt != clock || date_time != expect_dos {
        return Outcome {
            code: 1,
            stdout,
            stderr: format!(
                "FAIL: clock leak in artifact: col.crt={crt} zip_date_time={date_time:?} \
                 expected crt={clock} date_time={expect_dos:?}\n"
            ),
            files: Vec::new(),
            cards: items.len(),
            scanned: items.len(),
        };
    }
    if mod_ != clock * 1000 || scm != clock * 1000 {
        return Outcome {
            code: 1,
            stdout,
            stderr: format!(
                "FAIL: col.mod/scm not derived from the pinned clock: \
                 mod={mod_} scm={scm} expected {}\n",
                clock * 1000
            ),
            files: Vec::new(),
            cards: items.len(),
            scanned: items.len(),
        };
    }

    let digest = sha256_hex(&a_apkg);
    stdout.push_str(&format!(
        "export_anki check: two runs identical sha256={digest}\n"
    ));
    stdout.push_str(&format!(
        "export_anki check: col.crt={crt} col.mod={mod_} zip_date_time={date_time:?}\n"
    ));
    stdout.push_str(&format!(
        "export_anki check: ok cards={} unresolvable=0\n",
        items.len()
    ));
    Outcome {
        code: 0,
        stdout,
        stderr: String::new(),
        files: Vec::new(),
        cards: items.len(),
        scanned: items.len(),
    }
}

// ── evaluate / write / cli ─────────────────────────────────────────────────

fn parse_formats(raw: &str) -> Result<Vec<String>, String> {
    let mut formats: Vec<String> = Vec::new();
    for f in raw.split(',') {
        let f = f.trim();
        if f.is_empty() {
            continue;
        }
        let f = f.to_ascii_lowercase();
        if !formats.contains(&f) {
            formats.push(f);
        }
    }
    let mut unknown: Vec<String> = formats
        .iter()
        .filter(|f| !KNOWN_FORMATS.contains(&f.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        unknown.sort();
        return Err(format!("FAIL: unknown format(s): {unknown:?}"));
    }
    if formats.is_empty() {
        return Err("FAIL: no formats requested".into());
    }
    Ok(formats)
}

/// Decide the export. Writes nothing. A RED outcome carries no files.
pub fn evaluate(req: &Request) -> Outcome {
    let root = req.root.canonicalize().unwrap_or_else(|_| req.root.clone());

    let (mut items, stem, stderr_pre) = match req.source {
        Source::Bank => match load_bank_items(&root) {
            Ok(v) => (v, STEM_BANK.to_string(), String::new()),
            Err(e) => return Outcome::fail(&format!("FAIL: {e}")),
        },
        Source::Seed42 => match load_seed42_bank_items(&root) {
            Ok(Some(v)) => (v, STEM_SEED42.to_string(), String::new()),
            Ok(None) => match load_bank_items(&root) {
                Ok(v) => (
                    v,
                    STEM_BANK.to_string(),
                    "WARN: bank_items_seed42.json missing — falling back to bank\n".into(),
                ),
                Err(e) => return Outcome::fail(&format!("FAIL: {e}")),
            },
            Err(e) => return Outcome::fail(&format!("FAIL: {e}")),
        },
        Source::Keys => match load_keys_seed42_pack(&root) {
            Ok(Some(v)) => (v, STEM_KEYS.to_string(), String::new()),
            Ok(None) => return Outcome::fail("FAIL: keys/seed42 packs not found"),
            Err(e) => return Outcome::fail(&format!("FAIL: {e}")),
        },
    };

    let scanned = items.len();
    if scanned == 0 {
        let mut o = Outcome::fail("FAIL: zero items to export");
        o.stderr = format!("{stderr_pre}{}", o.stderr);
        o.scanned = scanned;
        return o;
    }

    if matches!(req.source, Source::Bank | Source::Seed42) {
        items.retain(is_drawable);
        if items.is_empty() {
            let mut o = Outcome::fail("FAIL: zero approved items to export");
            o.stderr = format!("{stderr_pre}{}", o.stderr);
            o.scanned = scanned;
            return o;
        }
    }

    items = filter_items(&items, req.module, req.tag.as_deref(), req.limit, req.seed);
    if items.is_empty() {
        let mut o = Outcome::fail("FAIL: filter removed all items");
        o.stderr = format!("{stderr_pre}{}", o.stderr);
        o.scanned = scanned;
        return o;
    }

    let bad = unresolvable_ids(&items);
    if !bad.is_empty() {
        return fail_unresolvable(&bad, items.len(), scanned, &stderr_pre);
    }

    if req.check {
        let mut o = run_repro_check(&items, &req.deck_name);
        o.scanned = scanned;
        if !stderr_pre.is_empty() {
            o.stderr = format!("{stderr_pre}{}", o.stderr);
        }
        return o;
    }

    let formats = match parse_formats(&req.format) {
        Ok(f) => f,
        Err(msg) => {
            let mut o = Outcome::fail(&msg);
            o.stderr = format!("{stderr_pre}{}", o.stderr);
            o.scanned = scanned;
            return o;
        }
    };

    let clock = match deck_clock() {
        Ok(c) => c,
        Err(e) => {
            let mut o = Outcome::fail(&format!("FAIL: {e}"));
            o.stderr = format!("{stderr_pre}{}", o.stderr);
            o.scanned = scanned;
            return o;
        }
    };

    let out_dir = req.out.clone();
    let mut files: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let mut written: Vec<String> = Vec::new();
    let shown = |p: &Path| p.to_string_lossy().into_owned();

    if formats.iter().any(|f| f == "tsv") {
        let p = out_dir.join(format!("{stem}.tsv"));
        files.push((p.clone(), write_tsv_body(&items).into_bytes()));
        written.push(shown(&p));
    }
    if formats.iter().any(|f| f == "csv") {
        let p = out_dir.join(format!("{stem}.csv"));
        files.push((p.clone(), write_csv_body(&items).into_bytes()));
        written.push(shown(&p));
    }
    if formats.iter().any(|f| f == "apkg") {
        let p = out_dir.join(format!("{stem}.apkg"));
        match write_apkg(&items, &req.deck_name, clock) {
            Ok(b) => {
                written.push(format!("{} (pure-sqlite)", shown(&p)));
                files.push((p, b));
            }
            Err(e) => {
                let mut o = Outcome::fail(&format!("FAIL: {e}"));
                o.stderr = format!("{stderr_pre}{}", o.stderr);
                o.scanned = scanned;
                return o;
            }
        }
    }
    let note = out_dir.join("README.txt");
    files.push((
        note.clone(),
        readme_body(items.len(), req.source.as_str()).into_bytes(),
    ));
    written.push(shown(&note));

    let mut stdout = String::new();
    stdout.push_str("export_anki ok\n");
    stdout.push_str(&format!("  cards={}\n", items.len()));
    stdout.push_str("  unresolvable=0\n");
    stdout.push_str(&format!("  {scanned} scanned, {} exported\n", items.len()));
    stdout.push_str(&format!("  source={}\n", req.source.as_str()));
    for w in &written {
        stdout.push_str(&format!("  wrote {w}\n"));
    }

    Outcome {
        code: 0,
        stdout,
        stderr: stderr_pre,
        files,
        cards: items.len(),
        scanned,
    }
}

/// Write a GREEN outcome's files. A RED outcome is a no-op (and is asserted).
pub fn commit(outcome: &Outcome) -> Result<(), AnkiError> {
    if outcome.code != 0 {
        if !outcome.files.is_empty() {
            return Err(AnkiError::msg("a failing export must not carry files"));
        }
        return Ok(());
    }
    for (path, body) in &outcome.files {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AnkiError::msg(format!("mkdir {}: {e}", parent.display())))?;
        }
        let mut f = fs::File::create(path)
            .map_err(|e| AnkiError::msg(format!("write {}: {e}", path.display())))?;
        f.write_all(body)
            .map_err(|e| AnkiError::msg(format!("write {}: {e}", path.display())))?;
    }
    Ok(())
}

/// Evaluate, write on GREEN, return the outcome.
pub fn run(req: &Request) -> Outcome {
    let outcome = evaluate(req);
    if let Err(e) = commit(&outcome) {
        return Outcome::fail(&format!("FAIL: {e}"));
    }
    outcome
}

/// Walk up from `start` to the course-engine root (`registries/claims.toml`).
///
/// No compile-time crate-directory fallback.
pub fn resolve_engine_root(start: &Path) -> Result<PathBuf, AnkiError> {
    cdcp_root::walk_engine_root(start).map_err(|e| AnkiError::msg(e.to_string()))
}

/// Load the live bank's drawable + withdrawn items (permissive TOML).
pub fn load_live_bank(root: &Path) -> Result<Vec<Card>, AnkiError> {
    load_bank_items(root)
}

/// Approved-only view of a card list.
pub fn approved_only(items: &[Card]) -> Vec<Card> {
    items.iter().filter(|c| is_drawable(c)).cloned().collect()
}

pub fn note_count_in_apkg(bytes: &[u8]) -> Result<usize, AnkiError> {
    let (db, _) = unzip_collection(bytes)?;
    count_notes(&db)
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn pinned_epoch_is_2023_11_14_22_13_20_utc() {
        assert_eq!(zip_date_time(PINNED_EPOCH), (2023, 11, 14, 22, 13, 20));
    }

    #[test]
    fn empty_bank_is_never_green() {
        let td = tempfile::tempdir().unwrap();
        fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
        let mut req = Request::default_for(td.path());
        req.format = "tsv".into();
        let o = evaluate(&req);
        assert_eq!(o.code, 1);
        assert_eq!(o.stderr, "FAIL: zero items to export\n");
        assert!(o.files.is_empty());
        assert!(!o.stdout.contains("ok"));
    }

    #[test]
    fn missing_bank_dir_is_never_green() {
        let td = tempfile::tempdir().unwrap();
        let mut req = Request::default_for(td.path());
        req.format = "tsv".into();
        let o = evaluate(&req);
        assert_eq!(o.code, 1);
        assert_eq!(o.stderr, "FAIL: zero items to export\n");
        assert!(o.files.is_empty());
    }

    #[test]
    fn all_retired_is_error_and_writes_nothing() {
        let td = tempfile::tempdir().unwrap();
        fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
        fs::write(
            td.path().join(ITEMS_DIR_REL).join("r.toml"),
            "id = \"r\"\nstatus = \"retired\"\nstem = \"gone\"\ncorrect = \"A\"\nchoices = [\"x\"]\n",
        )
        .unwrap();
        let mut req = Request::default_for(td.path());
        req.format = "tsv".into();
        let o = evaluate(&req);
        assert_eq!(o.code, 1);
        assert_eq!(o.stderr, "FAIL: zero approved items to export\n");
        assert!(o.files.is_empty());
    }

    #[test]
    fn planted_clock_leak_is_not_vacuous() {
        planted_clock_leak_trips().unwrap();
    }

    #[test]
    fn format_answer_maps_letter_to_choice() {
        let c = Card {
            id: "x".into(),
            stem: "s".into(),
            choices: vec!["one".into(), "two".into(), "three".into(), "four".into()],
            correct: "b".into(),
            explanation: String::new(),
            module: "1".into(),
            status: "approved".into(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        };
        assert_eq!(resolved_answer(&c).as_deref(), Some("B) two"));
        assert_eq!(format_answer(&c), "B) two");
    }

    #[test]
    fn out_of_range_letter_is_unresolvable() {
        // choices=['only'] correct='D' — the planted known-bad for urz3.
        let c = Card {
            id: "q01".into(),
            stem: "s".into(),
            choices: vec!["only".into()],
            correct: "D".into(),
            explanation: "e".into(),
            module: "1".into(),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        };
        assert_eq!(resolved_answer(&c), None);
        let e = Card {
            id: "q02".into(),
            stem: "s".into(),
            choices: vec!["only".into()],
            correct: "E".into(),
            explanation: "e".into(),
            module: "1".into(),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        };
        assert_eq!(resolved_answer(&e), None);
    }

    #[test]
    fn empty_and_multichar_correct_are_unresolvable() {
        // The retired Python `correct in "ABCD"` accepted both of these
        // (substring containment) and then crashed in `ord()`. Membership
        // is a single letter in {A,B,C,D}; empty and "AB" are unresolvable.
        let empty = Card {
            id: "empty".into(),
            stem: "s".into(),
            choices: vec![
                "alpha".into(),
                "beta".into(),
                "gamma".into(),
                "delta".into(),
            ],
            correct: "".into(),
            explanation: "e".into(),
            module: "1".into(),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        };
        assert_eq!(resolved_answer(&empty), None);
        let ab = Card {
            id: "ab".into(),
            stem: "s".into(),
            choices: vec![
                "alpha".into(),
                "beta".into(),
                "gamma".into(),
                "delta".into(),
            ],
            correct: "AB".into(),
            explanation: "e".into(),
            module: "1".into(),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        };
        assert_eq!(resolved_answer(&ab), None);
        let abcd = Card {
            id: "abcd".into(),
            stem: "s".into(),
            choices: vec![
                "alpha".into(),
                "beta".into(),
                "gamma".into(),
                "delta".into(),
            ],
            correct: "ABCD".into(),
            explanation: "e".into(),
            module: "1".into(),
            status: String::new(),
            tags: Vec::new(),
            topic_ids: Vec::new(),
        };
        assert_eq!(resolved_answer(&abcd), None);
    }
}
