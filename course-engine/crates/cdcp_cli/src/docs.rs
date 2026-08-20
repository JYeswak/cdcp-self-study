//! `cdcp docs sync` — advertised content counts come from the ledger, not hands.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! Public prose that names a bank / learn / WASM count is compared to
//! `web/data/units_index.json` plus one named tree fact (committed WASM KiB).
//! Drift is RED and names file:line + advertised + actual. `--write` rewrites
//! only those tokens and REFUSES when the ledger is unsound (missing key or
//! zero), so a bogus source cannot launder a number into README.
//!
//! Receipt-enforced numbers (check.sh steps, known-bad injections, suite
//! table) are out of scope — they already have regenerators. Decoration
//! counts that name no reader decision ("N scripts", "N Rust crates",
//! "~Nk lines") are not mechanized: `--check` goes RED if they reappear.
//!
//! # WHAT THIS CANNOT DECIDE
//!
//! * Whether `units_index.json` is *right* — only that advertised prose
//!   matches it. A regenerated index with a wrong total makes every site
//!   "correct" together.
//! * Whether a number that is not one of the catalogued content patterns
//!   is an advertisement (dates, "Module 15", "Learn-15", "14 EPI domains").
//! * Whether 90 / 72 / the per-suite table are honest — those are
//!   `verify-step-count` / `verify-injection-count`.
//!
//! # TESTING
//!
//! Oracle problem for arbitrary markdown: we cannot name the expected hit
//! set without the scanner. Metamorphic relations below (score ≥ 2.0) plus
//! a fuzz target on `scan_document(&[u8])` are the correctness floor.
//! Known-bad TEMP fixtures prove `--check` bites and `--write` refuses.

#![forbid(unsafe_code)]

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// KiB conversion for the committed WASM blob. 524_997 B rounds to 513.
pub const WASM_REL: &str = "web/assets/wasm/cdcp_wasm.wasm";
pub const UNITS_INDEX_REL: &str = "web/data/units_index.json";

/// Anti-vacuous: a scan that found no advertisement sites is an ERROR.
pub const MIN_ADVERTISEMENT_SITES: usize = 20;

/// Public files scanned, relative to the engine root (`registries/` lives here).
pub const SCANNED_FILES: &[&str] = &["../README.md", "../CHARTER.md", "README.md"];

/// Monotonic coverage floor: (rel path, ledger key, min hits).
/// A site dropping out is RED. Adding a site is free.
pub const RATCHET: &[(&str, LedgerKey, usize)] = &[
    ("../README.md", LedgerKey::BankItemCount, 8),
    ("../README.md", LedgerKey::ApprovedItemCount, 4),
    ("../README.md", LedgerKey::RetiredItemCount, 1),
    ("../README.md", LedgerKey::ModuleCount, 4),
    ("../README.md", LedgerKey::UnitCount, 1),
    ("../README.md", LedgerKey::WasmKib, 1),
    ("../CHARTER.md", LedgerKey::BankItemCount, 1),
    ("../CHARTER.md", LedgerKey::ApprovedItemCount, 1),
    ("../CHARTER.md", LedgerKey::ModuleCount, 4),
    ("README.md", LedgerKey::BankItemCount, 1),
    ("README.md", LedgerKey::ApprovedItemCount, 1),
    ("README.md", LedgerKey::RetiredItemCount, 1),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LedgerKey {
    BankItemCount,
    ApprovedItemCount,
    RetiredItemCount,
    ModuleCount,
    UnitCount,
    WasmKib,
}

impl LedgerKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BankItemCount => "bank_item_count",
            Self::ApprovedItemCount => "approved_item_count",
            Self::RetiredItemCount => "retired_item_count",
            Self::ModuleCount => "module_count",
            Self::UnitCount => "unit_count",
            Self::WasmKib => "wasm_kib",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub key: LedgerKey,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecorationHit {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    pub bank_item_count: u64,
    pub approved_item_count: u64,
    pub retired_item_count: u64,
    pub module_count: u64,
    pub unit_count: u64,
    pub wasm_kib: u64,
}

impl Ledger {
    pub fn get(&self, key: LedgerKey) -> u64 {
        match key {
            LedgerKey::BankItemCount => self.bank_item_count,
            LedgerKey::ApprovedItemCount => self.approved_item_count,
            LedgerKey::RetiredItemCount => self.retired_item_count,
            LedgerKey::ModuleCount => self.module_count,
            LedgerKey::UnitCount => self.unit_count,
            LedgerKey::WasmKib => self.wasm_kib,
        }
    }
}

#[derive(Debug, Deserialize)]
struct UnitsIndex {
    bank_item_count: Option<u64>,
    approved_item_count: Option<u64>,
    module_count: Option<u64>,
    unit_count: Option<u64>,
}

#[derive(Debug)]
pub enum SyncError {
    Usage(String),
    Unsound(String),
    Io(String),
    Drift(Vec<String>),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(s) | Self::Unsound(s) | Self::Io(s) => write!(f, "{s}"),
            Self::Drift(v) => write!(f, "{}", v.join("\n")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Write,
}

/// Parse argv after `docs sync`. Unknown flags are usage, never silence.
pub fn parse_mode(args: &[String]) -> Result<Mode, SyncError> {
    let mut check = false;
    let mut write = false;
    for a in args {
        match a.as_str() {
            "--check" => check = true,
            "--write" => write = true,
            other if other.starts_with('-') => {
                return Err(SyncError::Usage(format!(
                    "unknown argument {other:?}; known: --check --write --root <dir>"
                )));
            }
            _ => {}
        }
    }
    match (check, write) {
        (false, false) => Ok(Mode::Check),
        (true, false) => Ok(Mode::Check),
        (false, true) => Ok(Mode::Write),
        (true, true) => Err(SyncError::Usage(
            "--check and --write are mutually exclusive".into(),
        )),
    }
}

pub fn load_ledger(engine_root: &Path) -> Result<Ledger, SyncError> {
    let index_path = engine_root.join(UNITS_INDEX_REL);
    let raw = fs::read_to_string(&index_path).map_err(|e| {
        SyncError::Unsound(format!(
            "ledger unsound: cannot read {}: {e}",
            index_path.display()
        ))
    })?;
    let idx: UnitsIndex = serde_json::from_str(&raw).map_err(|e| {
        SyncError::Unsound(format!(
            "ledger unsound: {} is not a units_index object: {e}",
            index_path.display()
        ))
    })?;
    let bank = require_positive(&idx.bank_item_count, "bank_item_count")?;
    let approved = require_positive(&idx.approved_item_count, "approved_item_count")?;
    let modules = require_positive(&idx.module_count, "module_count")?;
    let units = require_positive(&idx.unit_count, "unit_count")?;
    if approved > bank {
        return Err(SyncError::Unsound(format!(
            "ledger unsound: approved_item_count={approved} exceeds bank_item_count={bank}"
        )));
    }
    let retired = bank - approved;
    if retired == 0 {
        return Err(SyncError::Unsound(
            "ledger unsound: retired_item_count is 0 (bank == approved) — refuse to advertise a zero retired pool".into(),
        ));
    }
    let wasm_path = engine_root.join(WASM_REL);
    let wasm_bytes = fs::metadata(&wasm_path)
        .map_err(|e| {
            SyncError::Unsound(format!(
                "ledger unsound: cannot stat {}: {e}",
                wasm_path.display()
            ))
        })?
        .len();
    if wasm_bytes == 0 {
        return Err(SyncError::Unsound(format!(
            "ledger unsound: {} is 0 bytes",
            wasm_path.display()
        )));
    }
    let wasm_kib = (wasm_bytes + 512) / 1024;
    if wasm_kib == 0 {
        return Err(SyncError::Unsound(
            "ledger unsound: wasm_kib rounded to 0".into(),
        ));
    }
    Ok(Ledger {
        bank_item_count: bank,
        approved_item_count: approved,
        retired_item_count: retired,
        module_count: modules,
        unit_count: units,
        wasm_kib,
    })
}

fn require_positive(v: &Option<u64>, key: &str) -> Result<u64, SyncError> {
    match *v {
        None => Err(SyncError::Unsound(format!(
            "ledger unsound: missing key {key}"
        ))),
        Some(0) => Err(SyncError::Unsound(format!(
            "ledger unsound: {key} is 0 — refuse to write"
        ))),
        Some(n) => Ok(n),
    }
}

/// Fuzz entry: never panics. Guards size. Returns every catalog hit.
pub fn scan_document(data: &[u8]) -> Vec<Hit> {
    if data.len() > 1_000_000 {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(data);
    scan_text(&text)
}

pub fn scan_text(text: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    let mut line_start = 0usize;
    for (line_no, line) in (1usize..).zip(text.split_inclusive('\n')) {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let body = body.strip_suffix('\r').unwrap_or(body);
        scan_line(body, line_start, line_no, &mut hits);
        line_start += line.len();
    }
    hits
}

pub fn scan_decoration(text: &str) -> Vec<DecorationHit> {
    let mut out = Vec::new();
    let mut line_start = 0usize;
    for (line_no, line) in (1usize..).zip(text.split_inclusive('\n')) {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let body = body.strip_suffix('\r').unwrap_or(body);
        scan_decoration_line(body, line_start, line_no, &mut out);
        line_start += line.len();
    }
    out
}

fn scan_line(line: &str, line_start: usize, line_no: usize, out: &mut Vec<Hit>) {
    let mut p = 0usize;
    while p < line.len() {
        if !line.is_char_boundary(p) {
            p += 1;
            continue;
        }
        if let Some((end, hit)) = try_content_at(line, p, line_start, line_no) {
            out.push(hit);
            p = end;
            continue;
        }
        p += line[p..].chars().next().map_or(1, char::len_utf8);
    }
}

fn scan_decoration_line(
    line: &str,
    line_start: usize,
    line_no: usize,
    out: &mut Vec<DecorationHit>,
) {
    let mut p = 0usize;
    while p < line.len() {
        if !line.is_char_boundary(p) {
            p += 1;
            continue;
        }
        if let Some((end, kind)) = try_decoration_at(line, p) {
            out.push(DecorationHit {
                start: line_start + p,
                end: line_start + end,
                line: line_no,
                kind,
            });
            p = end;
            continue;
        }
        p += line[p..].chars().next().map_or(1, char::len_utf8);
    }
}

fn try_content_at(line: &str, p: usize, line_start: usize, line_no: usize) -> Option<(usize, Hit)> {
    if let Some((full_end, hit)) = match_fifteen_modules(line, p, line_start, line_no) {
        return Some((full_end, hit));
    }
    if skip_non_count(line, p) {
        return None;
    }
    let (num_end, value) = parse_count_token(line, p)?;
    if let Some(key) = slash_pair_hit(line, p, num_end) {
        return Some((
            num_end,
            Hit {
                key,
                start: line_start + p,
                end: line_start + num_end,
                line: line_no,
                value,
            },
        ));
    }
    if let Some(full_end) = bank_tail(line, num_end) {
        return Some((
            full_end,
            Hit {
                key: LedgerKey::BankItemCount,
                start: line_start + p,
                end: line_start + num_end,
                line: line_no,
                value,
            },
        ));
    }
    if let Some(full_end) = approved_tail(line, num_end) {
        return Some((
            full_end,
            Hit {
                key: LedgerKey::ApprovedItemCount,
                start: line_start + p,
                end: line_start + num_end,
                line: line_no,
                value,
            },
        ));
    }
    if let Some(full_end) = retired_tail(line, num_end) {
        return Some((
            full_end,
            Hit {
                key: LedgerKey::RetiredItemCount,
                start: line_start + p,
                end: line_start + num_end,
                line: line_no,
                value,
            },
        ));
    }
    if let Some(full_end) = unit_tail(line, num_end) {
        return Some((
            full_end,
            Hit {
                key: LedgerKey::UnitCount,
                start: line_start + p,
                end: line_start + num_end,
                line: line_no,
                value,
            },
        ));
    }
    if let Some(full_end) = wasm_tail(line, num_end) {
        return Some((
            full_end,
            Hit {
                key: LedgerKey::WasmKib,
                start: line_start + p,
                end: line_start + num_end,
                line: line_no,
                value,
            },
        ));
    }
    if let Some(full_end) = module_tail(line, p, num_end) {
        return Some((
            full_end,
            Hit {
                key: LedgerKey::ModuleCount,
                start: line_start + p,
                end: line_start + num_end,
                line: line_no,
                value,
            },
        ));
    }
    None
}

fn match_fifteen_modules(
    line: &str,
    p: usize,
    line_start: usize,
    line_no: usize,
) -> Option<(usize, Hit)> {
    let end = lit_ci(line, p, "fifteen")?;
    let ws = run_ws(line, end);
    if ws == end {
        return None;
    }
    let full = lit_ci(line, ws, "modules")?;
    if !at_word_boundary(line, full) {
        return None;
    }
    Some((
        full,
        Hit {
            key: LedgerKey::ModuleCount,
            start: line_start + p,
            end: line_start + end,
            line: line_no,
            value: 15,
        },
    ))
}

/// Dates, product names, and module *indexes* are not counts.
fn skip_non_count(line: &str, p: usize) -> bool {
    // ISO date …-15 or 2026-08-15
    if p >= 8 {
        let before = &line[..p];
        if before.ends_with("2026-08-") || before.ends_with('-') && looks_like_date_prefix(before) {
            return true;
        }
    }
    // Learn-15
    if p >= 6 && line[..p].ends_with("Learn-") {
        return true;
    }
    // Module 15 / modules/15-
    if p >= 7 && (line[..p].ends_with("Module ") || line[..p].ends_with("modules/")) {
        return true;
    }
    false
}

fn looks_like_date_prefix(before: &str) -> bool {
    let b = before.as_bytes();
    if b.len() < 8 {
        return false;
    }
    let s = &b[b.len() - 8..];
    s[0].is_ascii_digit()
        && s[1].is_ascii_digit()
        && s[2].is_ascii_digit()
        && s[3].is_ascii_digit()
        && s[4] == b'-'
        && s[5].is_ascii_digit()
        && s[6].is_ascii_digit()
        && s[7] == b'-'
}

fn parse_count_token(line: &str, p: usize) -> Option<(usize, u64)> {
    let bytes = line.as_bytes();
    if p >= bytes.len() || !bytes[p].is_ascii_digit() {
        return None;
    }
    if p > 0 && bytes[p - 1].is_ascii_digit() {
        return None;
    }
    let mut e = p;
    while e < bytes.len() && bytes[e].is_ascii_digit() {
        e += 1;
    }
    let n: u64 = line[p..e].parse().ok()?;
    Some((e, n))
}

fn bank_tail(line: &str, num_end: usize) -> Option<usize> {
    // 854-item [question] bank
    if let Some(e) = lit_ci(line, num_end, "-item") {
        let ws = run_ws(line, e);
        if let Some(q) = lit_ci(line, ws, "question") {
            let ws2 = run_ws(line, q);
            if let Some(b) = lit_ci(line, ws2, "bank") {
                return Some(b);
            }
        }
        let ws2 = run_ws(line, e);
        if let Some(b) = lit_ci(line, ws2, "bank") {
            return Some(b);
        }
    }
    let ws = run_ws(line, num_end);
    if ws == num_end {
        // (854 files) — no space required after '(' already consumed
    }
    // " original item files"
    if let Some(e) = lit_ci(line, ws, "original") {
        let w2 = run_ws(line, e);
        if let Some(e2) = lit_ci(line, w2, "item") {
            let w3 = run_ws(line, e2);
            if let Some(e3) = lit_ci(line, w3, "files") {
                return Some(e3);
            }
        }
    }
    // " bank item files" / " bank item keys" / " bank answer keys"
    if let Some(e) = lit_ci(line, ws, "bank") {
        let w2 = run_ws(line, e);
        if let Some(e2) = lit_ci(line, w2, "item") {
            let w3 = run_ws(line, e2);
            if let Some(e3) = lit_ci(line, w3, "files") {
                return Some(e3);
            }
            if let Some(e3) = lit_ci(line, w3, "keys") {
                return Some(e3);
            }
        }
        let w2 = run_ws(line, e);
        if let Some(e2) = lit_ci(line, w2, "answer") {
            let w3 = run_ws(line, e2);
            if let Some(e3) = lit_ci(line, w3, "keys") {
                return Some(e3);
            }
        }
    }
    // " item files"
    if let Some(e) = lit_ci(line, ws, "item") {
        let w2 = run_ws(line, e);
        if let Some(e2) = lit_ci(line, w2, "files") {
            return Some(e2);
        }
    }
    // " files /"  (engine README) or " files)" (data-flow)
    if let Some(e) = lit_ci(line, ws, "files") {
        let rest = &line[e..];
        if rest.starts_with(" /") || rest.starts_with('/') || rest.starts_with(')') {
            return Some(e);
        }
    }
    None
}

/// `854/829 is a file-set / approved-pool size` — two tokens, one line.
/// Context is required so dates and `15/14` domain splits stay uncounted.
fn slash_pair_hit(line: &str, p: usize, num_end: usize) -> Option<LedgerKey> {
    if !line_has_slash_pool_context(line) {
        return None;
    }
    let bytes = line.as_bytes();
    if num_end < bytes.len()
        && bytes[num_end] == b'/'
        && parse_count_token(line, num_end + 1).is_some()
    {
        return Some(LedgerKey::BankItemCount);
    }
    if p > 0 && bytes[p - 1] == b'/' {
        let mut i = p - 1;
        while i > 0 && bytes[i - 1].is_ascii_digit() {
            i -= 1;
        }
        if i < p - 1 {
            return Some(LedgerKey::ApprovedItemCount);
        }
    }
    None
}

fn line_has_slash_pool_context(line: &str) -> bool {
    let l = line.to_ascii_lowercase();
    l.contains("file-set") || l.contains("approved-pool")
}

fn approved_tail(line: &str, num_end: usize) -> Option<usize> {
    let ws = run_ws(line, num_end);
    lit_ci(line, ws, "approved").filter(|&e| at_word_boundary(line, e))
}

fn retired_tail(line: &str, num_end: usize) -> Option<usize> {
    let ws = run_ws(line, num_end);
    lit_ci(line, ws, "retired").filter(|&e| at_word_boundary(line, e))
}

fn unit_tail(line: &str, num_end: usize) -> Option<usize> {
    let ws = run_ws(line, num_end);
    let e = lit_ci(line, ws, "learn")?;
    let w2 = run_ws(line, e);
    let e2 = lit_ci(line, w2, "units")?;
    if at_word_boundary(line, e2) {
        Some(e2)
    } else {
        None
    }
}

fn wasm_tail(line: &str, num_end: usize) -> Option<usize> {
    let ws = run_ws(line, num_end);
    let e = lit_ci(line, ws, "kib")?;
    let w2 = run_ws(line, e);
    let e2 = lit_ci(line, w2, "wasm")?;
    if at_word_boundary(line, e2) {
        Some(e2)
    } else {
        None
    }
}

fn module_tail(line: &str, num_start: usize, num_end: usize) -> Option<usize> {
    // 15-module
    if let Some(e) = lit_ci(line, num_end, "-module") {
        return Some(e);
    }
    let ws = run_ws(line, num_end);
    if ws == num_end {
        return None;
    }
    // 15 module markdown files
    if let Some(e) = lit_ci(line, ws, "module") {
        let w2 = run_ws(line, e);
        if let Some(e2) = lit_ci(line, w2, "markdown") {
            let w3 = run_ws(line, e2);
            if let Some(e3) = lit_ci(line, w3, "files") {
                return Some(e3);
            }
        }
        // bare "15 module" is too loose (Module 15 already skipped)
    }
    // 15 modules
    if let Some(e) = lit_ci(line, ws, "modules") {
        if at_word_boundary(line, e) {
            // "14 public EPI domains" is not this pattern.
            let _ = num_start;
            return Some(e);
        }
    }
    None
}

fn try_decoration_at(line: &str, p: usize) -> Option<(usize, &'static str)> {
    let (num_end, _) = parse_count_token(line, p)?;
    let ws = run_ws(line, num_end);
    if let Some(e) = lit_ci(line, ws, "rust") {
        let w2 = run_ws(line, e);
        if let Some(e2) = lit_ci(line, w2, "crates") {
            return Some((e2, "rust-crates"));
        }
    }
    if let Some(e) = lit_ci(line, ws, "scripts") {
        if at_word_boundary(line, e) {
            return Some((e, "scripts"));
        }
    }
    if let Some(e) = lit_ci(line, ws, "files") {
        let rest = line.get(e..).unwrap_or("");
        if rest.starts_with(';') && rest.to_ascii_lowercase().contains("check.sh") {
            return Some((e, "script-files"));
        }
    }
    // ~67k lines / 67k lines
    if p > 0 && line.as_bytes()[p - 1] == b'~' {
        if let Some(k) = lit_ci(line, num_end, "k") {
            let w2 = run_ws(line, k);
            if let Some(e) = lit_ci(line, w2, "lines") {
                return Some((e, "k-lines"));
            }
        }
    }
    if let Some(k) = lit_ci(line, num_end, "k") {
        let w2 = run_ws(line, k);
        if let Some(e) = lit_ci(line, w2, "lines") {
            return Some((e, "k-lines"));
        }
    }
    None
}

fn lit_ci(s: &str, p: usize, lit: &str) -> Option<usize> {
    let end = p + lit.len();
    if end > s.len() || !s.is_char_boundary(end) {
        return None;
    }
    if s[p..end].eq_ignore_ascii_case(lit) {
        Some(end)
    } else {
        None
    }
}

fn run_ws(s: &str, mut p: usize) -> usize {
    let b = s.as_bytes();
    while p < b.len() && b[p].is_ascii_whitespace() {
        p += 1;
    }
    p
}

fn at_word_boundary(s: &str, p: usize) -> bool {
    let b = s.as_bytes();
    p == b.len() || !b[p].is_ascii_alphanumeric()
}

/// Live comparator. A mutant that always returns false makes drift GREEN.
pub fn site_disagrees(advertised: u64, actual: u64) -> bool {
    advertised != actual
}

pub fn rewrite_text(text: &str, hits: &[Hit], ledger: &Ledger) -> String {
    let mut ordered: Vec<&Hit> = hits.iter().collect();
    ordered.sort_by_key(|h| h.start);
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for h in ordered {
        if h.start < cursor || h.end > text.len() || h.start > h.end {
            continue;
        }
        out.push_str(&text[cursor..h.start]);
        let actual = ledger.get(h.key);
        if h.value == actual {
            out.push_str(&text[h.start..h.end]);
        } else {
            out.push_str(&actual.to_string());
        }
        cursor = h.end;
    }
    out.push_str(&text[cursor..]);
    out
}

/// Identity rewrite: replace each hit token with itself.
pub fn rewrite_identity(text: &str, hits: &[Hit]) -> String {
    let mut ordered: Vec<&Hit> = hits.iter().collect();
    ordered.sort_by_key(|h| h.start);
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for h in ordered {
        if h.start < cursor || h.end > text.len() {
            continue;
        }
        out.push_str(&text[cursor..h.start]);
        out.push_str(&text[h.start..h.end]);
        cursor = h.end;
    }
    out.push_str(&text[cursor..]);
    out
}

pub fn evaluate_file(
    rel: &str,
    text: &str,
    ledger: &Ledger,
    disagree: fn(u64, u64) -> bool,
) -> Vec<String> {
    let mut findings = Vec::new();
    for d in scan_decoration(text) {
        findings.push(format!(
            "{rel}:{} decoration {:?} survives — A10: delete, do not mechanize",
            d.line, d.kind
        ));
    }
    let hits = scan_text(text);
    let mut counts = std::collections::BTreeMap::<LedgerKey, usize>::new();
    for h in &hits {
        *counts.entry(h.key).or_insert(0) += 1;
        let actual = ledger.get(h.key);
        if disagree(h.value, actual) {
            findings.push(format!(
                "{rel}:{} advertised {}={} actual={}",
                h.line,
                h.key.as_str(),
                h.value,
                actual
            ));
        }
    }
    for &(file, key, min) in RATCHET {
        if file != rel {
            continue;
        }
        let n = counts.get(&key).copied().unwrap_or(0);
        if n < min {
            findings.push(format!(
                "{rel}: ratchet {key}={n} dropped below floor {min} (site left the scanner)",
                key = key.as_str()
            ));
        }
    }
    findings
}

pub fn sync_tree(engine_root: &Path, mode: Mode) -> Result<usize, SyncError> {
    sync_tree_with(engine_root, mode, site_disagrees)
}

pub fn sync_tree_with(
    engine_root: &Path,
    mode: Mode,
    disagree: fn(u64, u64) -> bool,
) -> Result<usize, SyncError> {
    let ledger = load_ledger(engine_root)?;
    let mut findings = Vec::new();
    let mut site_total = 0usize;
    let mut pending_writes: Vec<(PathBuf, String)> = Vec::new();
    for rel in SCANNED_FILES {
        let path = engine_root.join(rel);
        let text = fs::read_to_string(&path)
            .map_err(|e| SyncError::Io(format!("cannot read {}: {e}", path.display())))?;
        let hits = scan_text(&text);
        site_total += hits.len();
        findings.extend(evaluate_file(rel, &text, &ledger, disagree));
        if mode == Mode::Write {
            let next = rewrite_text(&text, &hits, &ledger);
            if next != text {
                pending_writes.push((path, next));
            }
        }
    }
    if site_total < MIN_ADVERTISEMENT_SITES {
        return Err(SyncError::Unsound(format!(
            "only {site_total} advertisement site(s) parsed; at least {MIN_ADVERTISEMENT_SITES} expected — empty coverage is an ERROR"
        )));
    }
    if !findings.is_empty() {
        if mode == Mode::Write {
            // Drift is what --write repairs. Decoration / ratchet / unsound
            // findings that are not simple value drift still refuse the write.
            let hard: Vec<String> = findings
                .iter()
                .filter(|f| f.contains("decoration") || f.contains("ratchet"))
                .cloned()
                .collect();
            if !hard.is_empty() {
                return Err(SyncError::Drift(hard));
            }
            for (path, next) in pending_writes {
                fs::write(&path, next).map_err(|e| {
                    SyncError::Io(format!(
                        "--write: could not rewrite {}: {e}",
                        path.display()
                    ))
                })?;
            }
            return Ok(site_total);
        }
        return Err(SyncError::Drift(findings));
    }
    if mode == Mode::Write {
        for (path, next) in pending_writes {
            fs::write(&path, next).map_err(|e| {
                SyncError::Io(format!(
                    "--write: could not rewrite {}: {e}",
                    path.display()
                ))
            })?;
        }
    }
    Ok(site_total)
}

/// CLI entry used by `cdcp docs sync`.
pub fn run(engine_root: &Path, mode: Mode) -> Result<String, SyncError> {
    let n = sync_tree(engine_root, mode)?;
    Ok(format!(
        "docs-sync: {} {n} advertisement site(s) against units_index + {WASM_REL}",
        match mode {
            Mode::Check => "ok",
            Mode::Write => "wrote",
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_readme() -> &'static str {
        "Fifteen modules of writing.\n\
         | **What** | 15-module curriculum |\n\
         | **Bank** | 854 original item files / 829 approved · 25 retired |\n\
         | **Engine** | `#![forbid(unsafe_code)]` · 518 KiB WASM |\n\
         All 854 item files are original.\n\
         ├── bank/items/           854 original item files\n\
         seed (854 files) of 829 approved\n\
         # the 15 modules\n\
         | M8 | Learn v2: 134 Learn units |\n\
         the 854 bank answer keys remain\n\
         854 original item files\n\
         (829 approved: a pool size)\n\
         All 854 item files are original\n"
    }

    #[test]
    fn fifteen_and_hyphen_and_plural_are_module_counts() {
        let hits = scan_text(sample_readme());
        let mods: Vec<_> = hits
            .iter()
            .filter(|h| h.key == LedgerKey::ModuleCount)
            .collect();
        assert_eq!(mods.len(), 3, "{mods:?}");
        assert!(mods.iter().all(|h| h.value == 15));
    }

    #[test]
    fn learn_dash_15_and_module_15_and_date_are_not_counts() {
        let text =
            "Learn-15 quiz. Module 15 is taught. modules/15-ops-adjacent.md. Date 2026-08-15.\n";
        let hits = scan_text(text);
        assert!(
            hits.iter().all(|h| h.key != LedgerKey::ModuleCount),
            "false module counts: {hits:?}"
        );
    }

    #[test]
    fn slash_file_set_pair_is_bank_and_approved() {
        let hits = scan_text("854/829 is a file-set / approved-pool size\n");
        assert_eq!(hits.len(), 2, "{hits:?}");
        assert_eq!(hits[0].key, LedgerKey::BankItemCount);
        assert_eq!(hits[0].value, 854);
        assert_eq!(hits[1].key, LedgerKey::ApprovedItemCount);
        assert_eq!(hits[1].value, 829);
    }

    #[test]
    fn bank_item_keys_is_a_bank_count() {
        let hits = scan_text("No external suite checks the 854 bank item keys\n");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].key, LedgerKey::BankItemCount);
        assert_eq!(hits[0].value, 854);
    }

    #[test]
    fn decoration_scripts_crates_lines_are_found() {
        let text = "18 Rust crates · ~67k lines · 33 scripts\n├── scripts/              33 files; check.sh is THE gate\n";
        let d = scan_decoration(text);
        let kinds: Vec<_> = d.iter().map(|h| h.kind).collect();
        assert!(kinds.contains(&"rust-crates"), "{kinds:?}");
        assert!(kinds.contains(&"k-lines"), "{kinds:?}");
        assert!(kinds.contains(&"scripts"), "{kinds:?}");
        assert!(kinds.contains(&"script-files"), "{kinds:?}");
    }

    // ── Metamorphic relations (strength ≥ 2.0) ────────────────────────────
    //
    // | MR | F | I | C | Score |
    // | identity rewrite | 4 | 5 | 1 | 20 |
    // | write-after-decrement restores ledger | 5 | 4 | 2 | 10 |
    // | unmatched prose does not drop sites | 3 | 4 | 1 | 12 |
    // | non-site numbers unchanged | 5 | 5 | 1 | 25 |
    // | compound: permute-unmatched ∘ decrement ∘ write | 4 | 4 | 2 | 8 |

    fn xorshift(mut x: u64) -> u64 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    }

    #[test]
    fn mr_identity_rewrite_is_byte_equal() {
        let mut seed = 0xC0FFEE_u64;
        for _ in 0..64 {
            seed = xorshift(seed);
            let mut text = sample_readme().to_string();
            if seed.is_multiple_of(3) {
                text.push_str("\nnoise ");
                text.push_str(&seed.to_string());
            }
            let hits = scan_text(&text);
            assert_eq!(rewrite_identity(&text, &hits), text);
        }
    }

    #[test]
    fn mr_write_after_decrement_restores_ledger() {
        let ledger = Ledger {
            bank_item_count: 854,
            approved_item_count: 829,
            retired_item_count: 25,
            module_count: 15,
            unit_count: 134,
            wasm_kib: 518,
        };
        let text = sample_readme();
        let hits = scan_text(text);
        let banks: Vec<_> = hits
            .iter()
            .filter(|h| h.key == LedgerKey::BankItemCount)
            .cloned()
            .collect();
        assert!(!banks.is_empty());
        for (i, h) in banks.iter().enumerate() {
            let mut drifted = text.to_string();
            drifted.replace_range(h.start..h.end, "853");
            let after = scan_text(&drifted);
            let restored = rewrite_text(&drifted, &after, &ledger);
            let restored_hits = scan_text(&restored);
            assert!(
                restored_hits
                    .iter()
                    .filter(|x| x.key == LedgerKey::BankItemCount)
                    .all(|x| x.value == 854),
                "site {i} did not restore: {restored}"
            );
        }
    }

    #[test]
    fn mr_unmatched_prose_does_not_drop_sites() {
        let base = sample_readme();
        let n0 = scan_text(base).len();
        let extra = format!("{base}\n# leftover heading 2026-08-15 Learn-15 Module 15\n");
        let n1 = scan_text(&extra).len();
        assert_eq!(n0, n1, "unmatched prose changed the hit set");
    }

    #[test]
    fn mr_non_site_numbers_unchanged_by_write() {
        let ledger = Ledger {
            bank_item_count: 854,
            approved_item_count: 829,
            retired_item_count: 25,
            module_count: 15,
            unit_count: 134,
            wasm_kib: 518,
        };
        let text = format!(
            "{}\nstudy bar 27 · mock 40 · 14 public EPI · 90 steps · 72 injections\n",
            sample_readme()
        );
        let hits = scan_text(&text);
        let out = rewrite_text(&text, &hits, &ledger);
        assert!(out.contains("27"), "{out}");
        assert!(out.contains("40"), "{out}");
        assert!(out.contains("14 public"), "{out}");
        assert!(out.contains("90 steps"), "{out}");
        assert!(out.contains("72 injections"), "{out}");
    }

    #[test]
    fn mr_compound_shuffle_noise_then_decrement_then_write() {
        let ledger = Ledger {
            bank_item_count: 854,
            approved_item_count: 829,
            retired_item_count: 25,
            module_count: 15,
            unit_count: 134,
            wasm_kib: 518,
        };
        let mut lines: Vec<String> = sample_readme().lines().map(|s| s.to_string()).collect();
        lines.push("noise A".into());
        lines.push("noise B".into());
        // permutative: reverse unmatched noise, keep content lines
        let last = lines.len() - 1;
        lines.swap(last, last - 1);
        let mut text = lines.join("\n");
        text.push('\n');
        let hits = scan_text(&text);
        let some = hits
            .iter()
            .find(|h| h.key == LedgerKey::ApprovedItemCount)
            .cloned()
            .expect("approved site");
        text.replace_range(some.start..some.end, "828");
        let after = scan_text(&text);
        let restored = rewrite_text(&text, &after, &ledger);
        assert!(scan_text(&restored)
            .iter()
            .filter(|h| h.key == LedgerKey::ApprovedItemCount)
            .all(|h| h.value == 829));
        assert!(restored.contains("noise A") && restored.contains("noise B"));
    }

    #[test]
    fn deleting_the_comparison_turns_drift_green() {
        let ledger = Ledger {
            bank_item_count: 854,
            approved_item_count: 829,
            retired_item_count: 25,
            module_count: 15,
            unit_count: 134,
            wasm_kib: 518,
        };
        let mut drifted = sample_readme().to_string();
        drifted = drifted.replacen("854", "853", 1);
        let live = evaluate_file("README.md", &drifted, &ledger, site_disagrees);
        assert!(
            live.iter()
                .any(|f| f.contains("advertised") && f.contains("853")),
            "live comparator missed drift: {live:?}"
        );
        let mutant = evaluate_file("README.md", &drifted, &ledger, |_a, _b| false);
        assert!(
            mutant.iter().all(|f| !f.contains("advertised")),
            "deleted comparison should hide value drift: {mutant:?}"
        );
        assert!(
            site_disagrees(853, 854),
            "site_disagrees itself is the prove-it-bites bit"
        );
    }

    #[test]
    fn fuzz_entry_never_panics_on_garbage() {
        for data in [
            &b""[..],
            b"\xff\xfe",
            b"854 item files\n",
            &[0u8; 64],
            b"Fifteen modules",
        ] {
            let _ = scan_document(data);
        }
    }
}
