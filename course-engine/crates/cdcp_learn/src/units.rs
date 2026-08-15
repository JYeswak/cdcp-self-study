//! build_units — compile `web/data/units_index.json` (learner-visible product).
//!
//! One lesson unit per ATX `##` section of every module the Learn index carries,
//! with domain topic ids and two-to-three approved bank item ids on each.
//!
//! Extracted from `cdcp_gate` by bd-engine-not-gate-ar39.2. The compilers are
//! product: a learner is scored against `check_item_ids`. The contract is the
//! artifact schema (declared keys, correct counts, approved-only checks, stable
//! bytes), not a Python `json.dumps` replica.
//!
//! Anti-vacuous: zero modules, zero units, zero modules matching the module-id
//! shape, or zero units across those modules are each an ERROR. A status filter
//! that removes the whole bank is an ERROR, not an empty build.
//!
//! `generated_by` is `cdcp_learn`. Write-after-verdict: a RED compile writes
//! nothing.

#![forbid(unsafe_code)]

use crate::{join_rel, BuildOutcome, LearnError, GENERATED_BY};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const NAME: &str = "build-units";
pub const SUMMARY: &str = "build web/data/units_index.json: one unit per module ## section";

/// Engine-root-relative paths, matching the Python module constants.
pub const CONTENT_REL: &str = "web/content/modules";
pub const TOPICS_REL: &str = "knowledge/topics.toml";
pub const OUT_REL: &str = "web/data/units_index.json";
pub const MOD_INDEX_REL: &str = "web/data/modules_index.json";
pub const BANK_JSON_REL: &str = "web/data/bank_items_seed42.json";
pub const BANK_DIR_REL: &str = "bank/items";

/// The one `BankItem.status` a unit check may draw (`APPROVED`).
///
/// `web/data/bank_items_seed42.json` is the content-addressed MANIFEST of the
/// whole bank — 804 rows, 779 approved, 25 retired — and it CANNOT be filtered
/// at the source: `cdcp_wasm::grade_digest_json` recomputes `bank_hash` over
/// those exact bytes and `cdcp_grade::grade` hard-fails on a mismatch. The
/// obligation lands on every consumer that DRAWS, and this gate is one.
/// See the oracle's module header and `web/data/README.md`.
pub const APPROVED: &str = "approved";

/// Target questions per unit (`CHECK_N`).
pub const CHECK_N: usize = 3;
/// A unit shorter than this is dropped unless its title names an objective.
pub const MIN_UNIT_WORDS: usize = 40;
/// A unit needs this many bank items to count towards the floor.
pub const MIN_CHECKS_PER_UNIT: usize = 2;
/// A module with fewer units than this is a WARN-level shortfall.
pub const MIN_UNITS_PER_MODULE: usize = 3;
/// The fraction of units per module that must clear `MIN_CHECKS_PER_UNIT`.
pub const COVERAGE_FLOOR: f64 = 0.8;
/// Cap on matched topic ids before the full-domain fill.
pub const MAX_TOPIC_MATCHES: usize = 6;
/// The two named spot-check modules and the unit counts they must reach. Not
/// the general floor — these two carry the heaviest syllabus weight and are
/// where a content regression shows up first.
pub const SPOT_CHECKS: &[(&str, usize)] = &[("01-mission-critical", 4), ("06-power", 3)];

/// The stop list the topic matcher drops before scoring (`_STOP`).
pub const STOP: &[&str] = &[
    "a", "an", "and", "as", "at", "for", "in", "of", "on", "or", "the", "to", "vs", "with",
];

// ── Python-behaviour emulations ────────────────────────────────────────────

/// The `\s` character class of Python's `re` on `str` patterns, which is also
/// the set `str.strip()` removes: Unicode `White_Space` plus the four ASCII
/// information separators (0x1C-0x1F) that `str.isspace()` counts and Rust's
/// `char::is_whitespace` does not.
pub fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `str.strip()` with no argument.
pub fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

/// The `\w` class of Python's `re` on `str` patterns: `str.isalnum()` plus `_`.
pub fn py_word(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// `str.casefold()`, to the depth this gate can reach.
pub fn py_casefold(s: &str) -> String {
    s.to_lowercase().replace('ß', "ss")
}

/// `len(re.findall(r"\b\w+\b", text))` — `\w+` is maximal, so the boundaries
/// are redundant and this is exactly the number of maximal word-character runs.
pub fn count_words(s: &str) -> usize {
    let mut n = 0usize;
    let mut inside = false;
    for c in s.chars() {
        if py_word(c) {
            if !inside {
                n += 1;
                inside = true;
            }
        } else {
            inside = false;
        }
    }
    n
}

/// `repr()` of a Python `str`. Single quotes unless the value contains `'` and
/// no `"`. Non-ASCII passes through, which matches CPython for printable code
/// points; reaching the difference needs an unprintable non-ASCII character in
/// a module id.
pub fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || (c as u32) == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `repr()` of a `list[str]`.
pub fn py_repr_str_list(v: &[String]) -> String {
    let inner: Vec<String> = v.iter().map(|s| py_repr(s)).collect();
    format!("[{}]", inner.join(", "))
}

/// `repr()` of a `list[int]`.
pub fn py_repr_int_list(v: &[usize]) -> String {
    let inner: Vec<String> = v.iter().map(|n| n.to_string()).collect();
    format!("[{}]", inner.join(", "))
}

/// `round(x)` on a float with no `ndigits`: C `round` (half away from zero),
/// then ties corrected to even. This is CPython's `float.__round__` verbatim.
pub fn py_round(x: f64) -> f64 {
    let r = x.round();
    if (x - r).abs() == 0.5 {
        2.0 * (x / 2.0).round()
    } else {
        r
    }
}

/// `max(3, min(20, round(word_count / 200 * 1.35)))`.
pub fn estimate_minutes(word_count: usize) -> i64 {
    let x = (word_count as f64) / 200.0 * 1.35;
    let r = py_round(x) as i64;
    r.clamp(3, 20)
}

/// `slugify` — lowercase, drop emphasis marks, drop everything outside
/// `[\w\s-]`, collapse whitespace/underscores to `-`, collapse runs of `-`,
/// trim `-`, and fall back to `section` when nothing survives.
pub fn slugify(text: &str) -> String {
    let lowered = text.to_lowercase();
    let no_emphasis: String = lowered
        .chars()
        .filter(|c| !matches!(c, '*' | '_' | '`'))
        .collect();
    let kept: String = no_emphasis
        .chars()
        .filter(|c| py_word(*c) || py_space(*c) || *c == '-')
        .collect();
    let trimmed = py_strip(&kept);

    let chars: Vec<char> = trimmed.chars().collect();
    let mut dashed = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        if py_space(chars[i]) || chars[i] == '_' {
            while i < chars.len() && (py_space(chars[i]) || chars[i] == '_') {
                i += 1;
            }
            dashed.push('-');
        } else {
            dashed.push(chars[i]);
            i += 1;
        }
    }

    let mut collapsed = String::new();
    let mut prev_dash = false;
    for c in dashed.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push('-');
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }
    let out = collapsed.trim_matches('-').to_string();
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

/// `re.match(r"^(\d{2})-", mid)` then `int(group(1))`. ASCII digits only; the
/// oracle's `\d` also accepts other Unicode decimal digits, a shape no module
/// id in this corpus has.
pub fn module_num_from_id(mid: &str) -> Option<i64> {
    let c: Vec<char> = mid.chars().collect();
    if c.len() < 3 || !c[0].is_ascii_digit() || !c[1].is_ascii_digit() || c[2] != '-' {
        return None;
    }
    let n = (c[0] as i64 - '0' as i64) * 10 + (c[1] as i64 - '0' as i64);
    Some(n)
}

/// `re.match(r"^\d{2}-", mid)` as a predicate.
pub fn has_module_id_shape(mid: &str) -> bool {
    let c: Vec<char> = mid.chars().collect();
    c.len() >= 3 && c[0].is_ascii_digit() && c[1].is_ascii_digit() && c[2] == '-'
}

/// `re.sub(r"\s+#*\s*$", "", s)` — drop a closing ATX hash run.
pub fn strip_trailing_hashes(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    for i in 0..c.len() {
        let mut p = i;
        let start = p;
        while p < c.len() && py_space(c[p]) {
            p += 1;
        }
        if p == start {
            continue;
        }
        while p < c.len() && c[p] == '#' {
            p += 1;
        }
        while p < c.len() && py_space(c[p]) {
            p += 1;
        }
        if p == c.len() {
            return c[..i].iter().collect();
        }
    }
    s.to_string()
}

/// `re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", s)` — inline links to their text.
pub fn strip_links(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    let n = c.len();
    let mut out = String::new();
    let mut i = 0usize;
    while i < n {
        if c[i] == '[' {
            let mut j = i + 1;
            while j < n && c[j] != ']' {
                j += 1;
            }
            if j < n && j > i + 1 && j + 1 < n && c[j + 1] == '(' {
                let mut k = j + 2;
                while k < n && c[k] != ')' {
                    k += 1;
                }
                if k < n && k > j + 2 {
                    out.extend(c[i + 1..j].iter());
                    i = k + 1;
                    continue;
                }
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// `re.match(r"^(#{1,6})\s+(.*)$", stripped)` restricted to what the caller
/// needs: `Some(title)` when the line is an `##` heading. A seven-hash run or a
/// hash run with no following whitespace does not match, and the oracle then
/// treats the line as body text — so does this.
pub fn h2_title(stripped: &str) -> Option<String> {
    let c: Vec<char> = stripped.chars().collect();
    let mut k = 0usize;
    while k < c.len() && c[k] == '#' {
        k += 1;
    }
    if k == 0 || k > 6 || k != 2 {
        return None;
    }
    if k >= c.len() || !py_space(c[k]) {
        return None;
    }
    let mut p = k;
    while p < c.len() && py_space(c[p]) {
        p += 1;
    }
    Some(c[p..].iter().collect())
}

// ── minimal JSON reader ────────────────────────────────────────────────────
// The crate has no `serde_json` and this gate may not edit `Cargo.toml`
// (siblings share it), so the two JSON inputs are parsed by hand.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

struct JsonParser<'a> {
    c: &'a [char],
    i: usize,
}

impl JsonParser<'_> {
    fn ws(&mut self) {
        while self.i < self.c.len() && matches!(self.c[self.i], ' ' | '\t' | '\n' | '\r') {
            self.i += 1;
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), String> {
        if self.c.get(self.i) == Some(&ch) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected {ch:?} at offset {}", self.i))
        }
    }

    fn literal(&mut self, word: &str) -> Result<(), String> {
        for ch in word.chars() {
            self.expect(ch)?;
        }
        Ok(())
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            let Some(&ch) = self.c.get(self.i) else {
                return Err("unterminated string".into());
            };
            self.i += 1;
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let Some(&esc) = self.c.get(self.i) else {
                        return Err("unterminated escape".into());
                    };
                    self.i += 1;
                    match esc {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let hi = self.hex4()?;
                            let cp = if (0xd800..0xdc00).contains(&hi)
                                && self.c.get(self.i) == Some(&'\\')
                                && self.c.get(self.i + 1) == Some(&'u')
                            {
                                self.i += 2;
                                let lo = self.hex4()?;
                                if (0xdc00..0xe000).contains(&lo) {
                                    0x10000 + ((hi - 0xd800) << 10) + (lo - 0xdc00)
                                } else {
                                    return Err("bad surrogate pair".into());
                                }
                            } else {
                                hi
                            };
                            out.push(char::from_u32(cp).ok_or("bad code point")?);
                        }
                        other => return Err(format!("bad escape {other:?}")),
                    }
                }
                c => out.push(c),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let mut v = 0u32;
        for _ in 0..4 {
            let Some(&ch) = self.c.get(self.i) else {
                return Err("short \\u escape".into());
            };
            let d = ch.to_digit(16).ok_or("bad hex digit")?;
            v = v * 16 + d;
            self.i += 1;
        }
        Ok(v)
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while self
            .c
            .get(self.i)
            .is_some_and(|c| matches!(c, '-' | '+' | '.' | 'e' | 'E') || c.is_ascii_digit())
        {
            self.i += 1;
        }
        let text: String = self.c[start..self.i].iter().collect();
        if text.is_empty() {
            return Err(format!("expected a value at offset {start}"));
        }
        if text.contains(['.', 'e', 'E']) {
            text.parse::<f64>()
                .map(Json::Float)
                .map_err(|e| e.to_string())
        } else {
            match text.parse::<i64>() {
                Ok(n) => Ok(Json::Int(n)),
                Err(_) => text
                    .parse::<f64>()
                    .map(Json::Float)
                    .map_err(|e| e.to_string()),
            }
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.c.get(self.i) {
            Some('{') => {
                self.i += 1;
                let mut kv = Vec::new();
                self.ws();
                if self.c.get(self.i) == Some(&'}') {
                    self.i += 1;
                    return Ok(Json::Obj(kv));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.ws();
                    self.expect(':')?;
                    let v = self.value()?;
                    kv.push((k, v));
                    self.ws();
                    match self.c.get(self.i) {
                        Some(',') => self.i += 1,
                        Some('}') => {
                            self.i += 1;
                            return Ok(Json::Obj(kv));
                        }
                        _ => return Err(format!("bad object at offset {}", self.i)),
                    }
                }
            }
            Some('[') => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.c.get(self.i) == Some(&']') {
                    self.i += 1;
                    return Ok(Json::Arr(items));
                }
                loop {
                    let v = self.value()?;
                    items.push(v);
                    self.ws();
                    match self.c.get(self.i) {
                        Some(',') => self.i += 1,
                        Some(']') => {
                            self.i += 1;
                            return Ok(Json::Arr(items));
                        }
                        _ => return Err(format!("bad array at offset {}", self.i)),
                    }
                }
            }
            Some('"') => self.string().map(Json::Str),
            Some('t') => {
                self.literal("true")?;
                Ok(Json::Bool(true))
            }
            Some('f') => {
                self.literal("false")?;
                Ok(Json::Bool(false))
            }
            Some('n') => {
                self.literal("null")?;
                Ok(Json::Null)
            }
            _ => self.number(),
        }
    }
}

pub fn json_parse(text: &str) -> Result<Json, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut p = JsonParser { c: &chars, i: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != chars.len() {
        return Err(format!("trailing data at offset {}", p.i));
    }
    Ok(v)
}

pub fn obj_get<'a>(v: &'a Json, key: &str) -> Option<&'a Json> {
    match v {
        Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, val)| val),
        _ => None,
    }
}

/// Python truthiness of a value that may be absent.
pub fn truthy(v: Option<&Json>) -> bool {
    match v {
        None | Some(Json::Null) => false,
        Some(Json::Bool(b)) => *b,
        Some(Json::Int(n)) => *n != 0,
        Some(Json::Float(f)) => *f != 0.0,
        Some(Json::Str(s)) => !s.is_empty(),
        Some(Json::Arr(a)) => !a.is_empty(),
        Some(Json::Obj(o)) => !o.is_empty(),
    }
}

/// `str(x or fallback)` for the scalar shapes this corpus contains.
pub fn py_str_or(v: Option<&Json>, fallback: &str) -> String {
    if !truthy(v) {
        return fallback.to_string();
    }
    match v {
        Some(Json::Str(s)) => s.clone(),
        Some(Json::Int(n)) => n.to_string(),
        Some(Json::Bool(true)) => "True".to_string(),
        Some(Json::Bool(false)) => "False".to_string(),
        Some(Json::Float(f)) => format!("{f}"),
        // Containers here are a schema violation `verify_bank` owns; rendered
        // deterministically rather than reproduced.
        Some(Json::Arr(_)) => "[...]".to_string(),
        Some(Json::Obj(_)) => "{...}".to_string(),
        None | Some(Json::Null) => fallback.to_string(),
    }
}

/// `int(x or 0)` for the scalar shapes this corpus contains.
pub fn py_int_or_zero(v: Option<&Json>) -> i64 {
    if !truthy(v) {
        return 0;
    }
    match v {
        Some(Json::Int(n)) => *n,
        Some(Json::Float(f)) => f.trunc() as i64,
        Some(Json::Bool(b)) => i64::from(*b),
        Some(Json::Str(s)) => py_strip(s).parse::<i64>().unwrap_or(0),
        _ => 0,
    }
}

/// `list(x or [])` reduced to the strings it holds.
pub fn py_str_list(v: Option<&Json>) -> Vec<String> {
    match v {
        Some(Json::Arr(a)) => a.iter().map(|e| py_str_or(Some(e), "")).collect(),
        _ => Vec::new(),
    }
}

/// `len(list(x or []))`.
pub fn py_seq_len(v: Option<&Json>) -> usize {
    match v {
        Some(Json::Arr(a)) => a.len(),
        Some(Json::Str(s)) => s.chars().count(),
        Some(Json::Obj(o)) => o.len(),
        _ => 0,
    }
}

// ── minimal JSON writer ────────────────────────────────────────────────────

/// A value on the way out. Deliberately has no float arm: every number the
/// oracle emits is an `int`, so no float repr can differ between the two sides.
#[derive(Debug, Clone)]
pub enum Jv {
    Null,
    Int(i64),
    Str(String),
    Arr(Vec<Jv>),
    Obj(Vec<(String, Jv)>),
}

/// `json.dumps` string encoding with `ensure_ascii=True`: everything outside
/// printable ASCII becomes `\uXXXX`, astral code points as a surrogate pair.
pub fn json_string_ascii(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (' '..='~').contains(&c) => out.push(c),
            c => {
                let n = c as u32;
                if n < 0x10000 {
                    out.push_str(&format!("\\u{n:04x}"));
                } else {
                    let v = n - 0x10000;
                    out.push_str(&format!("\\u{:04x}", 0xd800 + (v >> 10)));
                    out.push_str(&format!("\\u{:04x}", 0xdc00 + (v & 0x3ff)));
                }
            }
        }
    }
    out.push('"');
}

fn pad(out: &mut String, level: usize) {
    for _ in 0..level * 2 {
        out.push(' ');
    }
}

/// `json.dumps(v, indent=2, sort_keys=True)`. Empty containers render inline as
/// `{}` / `[]`, exactly as CPython's encoder does under `indent`.
pub fn dump(v: &Jv, level: usize, out: &mut String) {
    match v {
        Jv::Null => out.push_str("null"),
        Jv::Int(n) => out.push_str(&n.to_string()),
        Jv::Str(s) => json_string_ascii(s, out),
        Jv::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            let last = items.len() - 1;
            for (i, item) in items.iter().enumerate() {
                pad(out, level + 1);
                dump(item, level + 1, out);
                if i != last {
                    out.push(',');
                }
                out.push('\n');
            }
            pad(out, level);
            out.push(']');
        }
        Jv::Obj(kv) => {
            if kv.is_empty() {
                out.push_str("{}");
                return;
            }
            let mut entries: Vec<&(String, Jv)> = kv.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            out.push_str("{\n");
            let last = entries.len() - 1;
            for (i, (k, val)) in entries.iter().enumerate() {
                pad(out, level + 1);
                json_string_ascii(k, out);
                out.push_str(": ");
                dump(val, level + 1, out);
                if i != last {
                    out.push(',');
                }
                out.push('\n');
            }
            pad(out, level);
            out.push('}');
        }
    }
}

// ── the model ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Topic {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub module: i64,
    pub topic_ids: Vec<String>,
    pub stem: String,
    pub explanation: String,
    pub choices_len: usize,
    /// `BankItem.status`. `""` when the row carried none — see [`is_approved`].
    pub status: String,
}

impl Item {
    /// The drawable predicate (`is_approved`). Absent status is WITHHELD, never
    /// permitted: `export-web` refuses to write a manifest row without a
    /// `status`, so a row that reached here without one came from somewhere
    /// that gate does not cover, and guessing in its favour is how a withdrawn
    /// item reaches a learner.
    pub fn is_approved(&self) -> bool {
        self.status == APPROVED
    }
}

#[derive(Debug, Clone)]
pub struct RawUnit {
    pub title: String,
    pub heading_id: String,
    pub word_count: usize,
    pub order: usize,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub id: String,
    pub module_id: String,
    pub module_num: Option<i64>,
    pub order: usize,
    pub title: String,
    pub heading_id: String,
    pub word_count: usize,
    pub estimate_minutes: i64,
    pub topic_ids: Vec<String>,
    pub check_item_ids: Vec<String>,
    pub check_count: usize,
}

impl Row {
    pub fn to_jv(&self) -> Jv {
        Jv::Obj(vec![
            ("id".into(), Jv::Str(self.id.clone())),
            ("module_id".into(), Jv::Str(self.module_id.clone())),
            (
                "module_num".into(),
                match self.module_num {
                    Some(n) => Jv::Int(n),
                    None => Jv::Null,
                },
            ),
            ("order".into(), Jv::Int(self.order as i64)),
            ("title".into(), Jv::Str(self.title.clone())),
            ("heading_id".into(), Jv::Str(self.heading_id.clone())),
            ("word_count".into(), Jv::Int(self.word_count as i64)),
            ("estimate_minutes".into(), Jv::Int(self.estimate_minutes)),
            (
                "topic_ids".into(),
                Jv::Arr(self.topic_ids.iter().cloned().map(Jv::Str).collect()),
            ),
            (
                "check_item_ids".into(),
                Jv::Arr(self.check_item_ids.iter().cloned().map(Jv::Str).collect()),
            ),
            ("check_count".into(), Jv::Int(self.check_count as i64)),
        ])
    }
}

/// `split_h2_units` — one unit per `##` section, with the short-section filter
/// applied AFTER heading ids are assigned (so a dropped section still consumes
/// its slug, which is load-bearing for the `-2` suffixes downstream).
pub fn split_h2_units(md: &str) -> Vec<RawUnit> {
    let normalised = md.replace("\r\n", "\n").replace('\r', "\n");
    let mut units: Vec<RawUnit> = Vec::new();
    let mut in_fence = false;
    let mut current: Option<(String, String)> = None;
    let mut body: Vec<String> = Vec::new();
    let mut used: HashMap<String, usize> = HashMap::new();

    fn flush(
        current: &mut Option<(String, String)>,
        body: &mut Vec<String>,
        out: &mut Vec<RawUnit>,
    ) {
        let Some((title, heading_id)) = current.take() else {
            body.clear();
            return;
        };
        let text = py_strip(&body.join("\n")).to_string();
        out.push(RawUnit {
            title,
            heading_id,
            word_count: count_words(&text),
            order: 0,
        });
        body.clear();
    }

    fn uniq(used: &mut HashMap<String, usize>, base: &str) -> String {
        if !used.contains_key(base) {
            used.insert(base.to_string(), 1);
            return base.to_string();
        }
        let mut n = used[base] + 1;
        while used.contains_key(&format!("{base}-{n}")) {
            n += 1;
        }
        used.insert(base.to_string(), n);
        let candidate = format!("{base}-{n}");
        used.insert(candidate.clone(), 1);
        candidate
    }

    for raw in normalised.split('\n') {
        let stripped = py_strip(raw);
        if stripped.starts_with("```") {
            in_fence = !in_fence;
            if current.is_some() {
                body.push(raw.to_string());
            }
            continue;
        }
        if !in_fence {
            if let Some(rest) = h2_title(stripped) {
                flush(&mut current, &mut body, &mut units);
                let title = py_strip(&strip_trailing_hashes(&rest)).to_string();
                let plain = strip_links(&title);
                let plain: String = plain
                    .chars()
                    .filter(|c| !matches!(c, '*' | '_' | '`'))
                    .collect();
                let heading_id = uniq(&mut used, &slugify(&plain));
                current = Some((title, heading_id));
                body.clear();
                continue;
            }
        }
        if current.is_some() {
            body.push(raw.to_string());
        }
    }
    flush(&mut current, &mut body, &mut units);

    let mut out: Vec<RawUnit> = Vec::new();
    for mut u in units {
        if u.word_count < MIN_UNIT_WORDS {
            let lowered = u.title.to_lowercase();
            if !lowered.contains("objective") && !lowered.contains("learning") {
                continue;
            }
        }
        u.order = out.len() + 1;
        out.push(u);
    }
    out
}

/// `re.findall(r"[a-z0-9]+", s)`.
fn ascii_alnum_runs(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Best-effort topic attachment by label/slug overlap.
pub fn match_topics(unit_title: &str, heading_id: &str, topics: &[Topic]) -> Vec<String> {
    if topics.is_empty() {
        return Vec::new();
    }
    let title_cf = py_casefold(unit_title);
    let slug = heading_id;
    let mut scored: Vec<(i64, String)> = Vec::new();
    for t in topics {
        let mut score: i64 = 0;
        let lab_slug = slugify(&t.label);
        if !lab_slug.is_empty() && (slug.contains(&lab_slug) || lab_slug.contains(slug)) {
            score = score.max(70);
        }
        let lowered = t.label.to_lowercase();
        let words: Vec<String> = ascii_alnum_runs(&lowered)
            .into_iter()
            .filter(|w| !STOP.contains(&w.as_str()) && w.chars().count() > 2)
            .collect();
        if !words.is_empty() {
            let hits = words
                .iter()
                .filter(|w| title_cf.contains(w.as_str()) || slug.contains(w.as_str()))
                .count();
            if hits == words.len() {
                score = score.max(80);
            } else if hits >= std::cmp::max(1, words.len() / 2) {
                score = score.max(40 + hits as i64 * 5);
            }
        }
        // `tid.split("-", 1)[-1]`: everything after the first hyphen, or the
        // whole id when there is none.
        let tail = match t.id.split_once('-') {
            Some((_, rest)) => rest,
            None => t.id.as_str(),
        };
        for part in tail.split(['-', '_']) {
            if !part.is_empty() && part.chars().count() > 3 && slug.contains(part) {
                score = score.max(45);
            }
        }
        if score >= 40 {
            scored.push((score, t.id.clone()));
        }
    }
    // `list.sort(reverse=True)` on `(score, id)` tuples: descending on both.
    scored.sort_by(|a, b| b.cmp(a));

    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for (_, tid) in scored {
        if !seen.insert(tid.clone()) {
            continue;
        }
        out.push(tid);
        if out.len() >= MAX_TOPIC_MATCHES {
            break;
        }
    }
    out
}

/// Matched topics first, then the whole domain, so Quick check can always map.
pub fn assign_topic_ids(unit_title: &str, heading_id: &str, topics: &[Topic]) -> Vec<String> {
    let matched = match_topics(unit_title, heading_id, topics);
    if topics.is_empty() {
        return matched;
    }
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for tid in matched
        .into_iter()
        .chain(topics.iter().map(|t| t.id.clone()))
    {
        if seen.insert(tid.clone()) {
            out.push(tid);
        }
    }
    out
}

/// Higher = better for Quick check samples.
pub fn item_quality(it: &Item) -> i64 {
    let mut score = 0i64;
    if !it.explanation.is_empty() && it.explanation.chars().count() >= 20 {
        score += 50;
    }
    if it.choices_len >= 4 {
        score += 20;
    }
    let stem_len = it.stem.chars().count();
    if (40..=280).contains(&stem_len) {
        score += 15;
    }
    if !it.topic_ids.is_empty() {
        score += 10;
    }
    let lowered = it.stem.to_lowercase();
    if ["why", "most", "best", "risk", "fail", "when", "which"]
        .iter()
        .any(|w| lowered.contains(w))
    {
        score += 5;
    }
    score
}

/// Pick `n` diversified bank ids for a unit. Deterministic.
pub fn pick_check_items(
    bank_by_module: &HashMap<i64, Vec<Item>>,
    module_num: Option<i64>,
    topic_ids: &[String],
    unit_order: usize,
    n: usize,
) -> Vec<String> {
    let Some(mnum) = module_num else {
        return Vec::new();
    };
    if mnum == 0 {
        return Vec::new();
    }
    let Some(pool) = bank_by_module.get(&mnum) else {
        return Vec::new();
    };
    if pool.is_empty() {
        return Vec::new();
    }

    let tid_set: HashSet<&str> = topic_ids.iter().map(String::as_str).collect();
    let mut primary: Vec<&Item> = if tid_set.is_empty() {
        Vec::new()
    } else {
        pool.iter()
            .filter(|it| it.topic_ids.iter().any(|t| tid_set.contains(t.as_str())))
            .collect()
    };
    let key = |it: &&Item| (-item_quality(it), it.id.clone());
    primary.sort_by_key(key);
    // `it not in primary` compares dicts by VALUE on the Python side; bank ids
    // are unique (`verify_bank` holds that), so identity by id is the same set.
    let chosen: HashSet<&str> = primary.iter().map(|it| it.id.as_str()).collect();
    let mut rest: Vec<&Item> = pool
        .iter()
        .filter(|it| !chosen.contains(it.id.as_str()))
        .collect();
    rest.sort_by_key(key);

    let mut ordered = primary;
    ordered.extend(rest);
    if ordered.is_empty() {
        return Vec::new();
    }

    // Offset by unit order so adjacent units get different questions.
    let start = ((unit_order - 1) * n) % ordered.len();
    let mut rotated: Vec<&Item> = ordered[start..].to_vec();
    rotated.extend_from_slice(&ordered[..start]);

    let mut picked: Vec<&Item> = Vec::new();
    let mut used_topics: HashSet<String> = HashSet::new();
    let mut used_ids: HashSet<String> = HashSet::new();
    for require_new_topic in [true, false] {
        for it in &rotated {
            if picked.len() >= n {
                break;
            }
            if used_ids.contains(&it.id) {
                continue;
            }
            let head = it.topic_ids.first().cloned().unwrap_or_default();
            if require_new_topic && !head.is_empty() && used_topics.contains(&head) {
                continue;
            }
            picked.push(it);
            used_ids.insert(it.id.clone());
            if !head.is_empty() {
                used_topics.insert(head);
            }
        }
    }
    picked.iter().take(n).map(|it| it.id.clone()).collect()
}

// ── input loading ──────────────────────────────────────────────────────────

fn read_utf8(p: &Path) -> Result<String, LearnError> {
    let bytes =
        std::fs::read(p).map_err(|e| LearnError::io(format!("read {}: {e}", p.display())))?;
    String::from_utf8(bytes)
        .map_err(|e| LearnError::io(format!("{} is not valid UTF-8: {e}", p.display())))
}

fn toml_truthy(v: Option<&toml::Value>) -> bool {
    match v {
        None => false,
        Some(toml::Value::Boolean(b)) => *b,
        Some(toml::Value::Integer(n)) => *n != 0,
        Some(toml::Value::Float(f)) => *f != 0.0,
        Some(toml::Value::String(s)) => !s.is_empty(),
        Some(toml::Value::Array(a)) => !a.is_empty(),
        Some(toml::Value::Table(t)) => !t.is_empty(),
        Some(toml::Value::Datetime(_)) => true,
    }
}

fn toml_str_or(v: Option<&toml::Value>, fallback: &str) -> String {
    if !toml_truthy(v) {
        return fallback.to_string();
    }
    match v {
        Some(toml::Value::String(s)) => s.clone(),
        Some(toml::Value::Integer(n)) => n.to_string(),
        Some(toml::Value::Float(f)) => format!("{f}"),
        Some(toml::Value::Boolean(true)) => "True".to_string(),
        Some(toml::Value::Boolean(false)) => "False".to_string(),
        Some(other) => other.to_string(),
        None => fallback.to_string(),
    }
}

fn toml_int_or_zero(v: Option<&toml::Value>) -> i64 {
    if !toml_truthy(v) {
        return 0;
    }
    match v {
        Some(toml::Value::Integer(n)) => *n,
        Some(toml::Value::Float(f)) => f.trunc() as i64,
        Some(toml::Value::Boolean(b)) => i64::from(*b),
        Some(toml::Value::String(s)) => py_strip(s).parse::<i64>().unwrap_or(0),
        _ => 0,
    }
}

fn toml_str_list(v: Option<&toml::Value>) -> Vec<String> {
    match v {
        Some(toml::Value::Array(a)) => a.iter().map(|e| toml_str_or(Some(e), "")).collect(),
        _ => Vec::new(),
    }
}

fn toml_seq_len(v: Option<&toml::Value>) -> usize {
    match v {
        Some(toml::Value::Array(a)) => a.len(),
        Some(toml::Value::String(s)) => s.chars().count(),
        Some(toml::Value::Table(t)) => t.len(),
        _ => 0,
    }
}

/// `load_topics_by_domain`. The caller has already refused to run without the
/// file: the `is_file` guard that used to sit here returned an empty map for an
/// absent registry, which the picker then read as "no preference".
pub fn load_topics_by_domain(root: &Path) -> Result<HashMap<String, Vec<Topic>>, LearnError> {
    let mut by: HashMap<String, Vec<Topic>> = HashMap::new();
    let path = join_rel(root, TOPICS_REL);
    let text = read_utf8(&path)?;
    let data: toml::Value = text
        .parse()
        .map_err(|e| LearnError::io(format!("{}: {e}", path.display())))?;
    let Some(toml::Value::Array(rows)) = data.get("topic") else {
        return Ok(by);
    };
    for t in rows {
        let dom = py_strip(&toml_str_or(t.get("domain"), "")).to_string();
        let tid = py_strip(&toml_str_or(t.get("id"), "")).to_string();
        if dom.is_empty() || tid.is_empty() {
            continue;
        }
        let label = toml_str_or(t.get("label"), &tid);
        by.entry(dom).or_default().push(Topic { id: tid, label });
    }
    Ok(by)
}

/// `load_bank` — the exported seed JSON first, then the per-item TOMLs.
pub fn load_bank(root: &Path) -> Result<Vec<Item>, LearnError> {
    let bank_json = join_rel(root, BANK_JSON_REL);
    if bank_json.is_file() {
        let text = read_utf8(&bank_json)?;
        let data = json_parse(&text)
            .map_err(|e| LearnError::io(format!("{}: {e}", bank_json.display())))?;
        let empty: Vec<Json> = Vec::new();
        let items: &Vec<Json> = match &data {
            Json::Arr(v) => v,
            other => match obj_get(other, "items") {
                Some(Json::Arr(v)) => v,
                _ => &empty,
            },
        };
        let mut out: Vec<Item> = Vec::new();
        for it in items {
            if !matches!(it, Json::Obj(_)) {
                continue;
            }
            if !truthy(obj_get(it, "id")) {
                continue;
            }
            out.push(Item {
                id: py_str_or(obj_get(it, "id"), ""),
                module: py_int_or_zero(obj_get(it, "module")),
                topic_ids: py_str_list(obj_get(it, "topic_ids")),
                stem: py_str_or(obj_get(it, "stem"), ""),
                explanation: py_str_or(obj_get(it, "explanation"), ""),
                choices_len: py_seq_len(obj_get(it, "choices")),
                status: py_str_or(obj_get(it, "status"), ""),
            });
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }

    let mut items: Vec<Item> = Vec::new();
    let bank_dir = join_rel(root, BANK_DIR_REL);
    if !bank_dir.is_dir() {
        return Ok(items);
    }
    // `sorted(BANK_DIR.glob("*.toml"))` — pathlib's `*` is fnmatch, so a
    // leading dot is NOT excluded; sorting is on the full path string, which
    // under a shared parent is filename order.
    let mut paths: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(&bank_dir)
        .map_err(|e| LearnError::io(format!("read {}: {e}", bank_dir.display())))?;
    for e in entries.flatten() {
        let p = e.path();
        if p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".toml"))
        {
            paths.push(p);
        }
    }
    paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    for path in paths {
        // The oracle swallows a parse error here with a bare `except`.
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let Ok(t) = text.parse::<toml::Value>() else {
            continue;
        };
        if !toml_truthy(t.get("id")) {
            continue;
        }
        items.push(Item {
            id: toml_str_or(t.get("id"), ""),
            module: toml_int_or_zero(t.get("module")),
            topic_ids: toml_str_list(t.get("topic_ids")),
            stem: toml_str_or(t.get("stem"), ""),
            explanation: toml_str_or(t.get("explanation"), ""),
            choices_len: toml_seq_len(t.get("choices")),
            status: toml_str_or(t.get("status"), ""),
        });
    }
    Ok(items)
}

// ── the compiler ───────────────────────────────────────────────────────────

pub type Outcome = BuildOutcome;

pub fn evaluate(root: &Path) -> Result<Outcome, LearnError> {
    let content = join_rel(root, CONTENT_REL);
    if !content.is_dir() {
        return Ok(Outcome {
            stdout: "FAIL: missing web/content/modules — run build_learn.py first\n".to_string(),
            code: 1,
            artifact: None,
        });
    }

    // ANTI-VACUOUS, ON THE INPUTS. Each of these registries could vanish and
    // the build stayed GREEN — one by degrading to an empty topic map, the
    // other by globbing a different module set. Refuse, name the file, write
    // nothing.
    let topics_path = join_rel(root, TOPICS_REL);
    let mod_index = join_rel(root, MOD_INDEX_REL);
    let mut missing_registries: Vec<String> = Vec::new();
    if !topics_path.is_file() {
        missing_registries.push(format!(
            "{TOPICS_REL} (topic registry; absent, every unit gets topic_ids=[] \
             and the picker reads that as 'no preference')"
        ));
    }
    if !mod_index.is_file() {
        missing_registries.push(format!(
            "{MOD_INDEX_REL} (Learn index; the module set is DERIVED from it — \
             there is no glob fallback)"
        ));
    }
    if !missing_registries.is_empty() {
        let mut report = vec!["FAIL: build_units missing required input registries".to_string()];
        report.extend(missing_registries.iter().map(|m| format!("  - {m}")));
        report.push(format!(
            "  out={OUT_REL} NOT WRITTEN (a failing build leaves no artifact)"
        ));
        return Ok(Outcome {
            stdout: format!("{}\n", report.join("\n")),
            code: 1,
            artifact: None,
        });
    }

    let mut domain_ids: Vec<String> = Vec::new();
    let text = read_utf8(&mod_index)?;
    let mi =
        json_parse(&text).map_err(|e| LearnError::io(format!("{}: {e}", mod_index.display())))?;
    if let Some(Json::Arr(modules)) = obj_get(&mi, "modules") {
        for m in modules {
            if !truthy(obj_get(m, "empty")) && truthy(obj_get(m, "id")) {
                domain_ids.push(py_str_or(obj_get(m, "id"), ""));
            }
        }
    }

    let topics_by = load_topics_by_domain(root)?;
    let bank = load_bank(root)?;
    // THE DRAW POOL IS THE APPROVED POOL. `bank` stays whole so
    // `bank_item_count` keeps meaning "manifest rows"; only approved rows may
    // reach a unit's check_item_ids.
    let approved: Vec<&Item> = bank.iter().filter(|it| it.is_approved()).collect();
    let mut bank_by_module: HashMap<i64, Vec<Item>> = HashMap::new();
    for it in &approved {
        bank_by_module
            .entry(it.module)
            .or_default()
            .push((*it).clone());
    }

    let mut all_units: Vec<Row> = Vec::new();
    // Insertion-ordered, because `by_module.items()` order is observable in the
    // shortfall list even though the artifact sorts its keys.
    let mut by_module: Vec<(String, Vec<Row>)> = Vec::new();
    let mut units_with_checks = 0usize;
    let mut units_zero_checks = 0usize;

    let no_topics: Vec<Topic> = Vec::new();
    for mid in &domain_ids {
        let path = content.join(format!("{mid}.md"));
        if !path.is_file() {
            continue;
        }
        let md = read_utf8(&path)?;
        let units = split_h2_units(&md);
        let topics = topics_by.get(mid).unwrap_or(&no_topics);
        let mnum = module_num_from_id(mid);
        let mut mod_units: Vec<Row> = Vec::new();
        for u in &units {
            let topic_ids = assign_topic_ids(&u.title, &u.heading_id, topics);
            let check_ids = pick_check_items(&bank_by_module, mnum, &topic_ids, u.order, CHECK_N);
            if check_ids.len() >= MIN_CHECKS_PER_UNIT {
                units_with_checks += 1;
            } else {
                units_zero_checks += 1;
            }
            let row = Row {
                id: format!("{mid}__{}", u.heading_id),
                module_id: mid.clone(),
                module_num: mnum,
                order: u.order,
                title: u.title.clone(),
                heading_id: u.heading_id.clone(),
                word_count: u.word_count,
                estimate_minutes: estimate_minutes(u.word_count),
                topic_ids,
                check_count: check_ids.len(),
                check_item_ids: check_ids,
            };
            mod_units.push(row.clone());
            all_units.push(row);
        }
        by_module.push((mid.clone(), mod_units));
    }

    let mut shortfalls: Vec<String> = Vec::new();
    for (mid, us) in &by_module {
        if has_module_id_shape(mid) && us.len() < MIN_UNITS_PER_MODULE {
            shortfalls.push(format!("{mid}: {} units", us.len()));
        }
        let mnum = module_num_from_id(mid);
        if mnum.is_some_and(|n| n != 0) && has_module_id_shape(mid) {
            let weak = us
                .iter()
                .filter(|u| u.check_count < MIN_CHECKS_PER_UNIT)
                .count();
            if weak != 0 {
                shortfalls.push(format!(
                    "{mid}: {weak}/{} units with <2 check items",
                    us.len()
                ));
            }
        }
    }

    let payload = Jv::Obj(vec![
        ("schema_version".into(), Jv::Int(2)),
        (
            "generated_by".into(),
            Jv::Str(GENERATED_BY.into()),
        ),
        ("unit_count".into(), Jv::Int(all_units.len() as i64)),
        ("module_count".into(), Jv::Int(by_module.len() as i64)),
        ("approved_item_count".into(), Jv::Int(approved.len() as i64)),
        ("bank_item_count".into(), Jv::Int(bank.len() as i64)),
        (
            "units_with_checks".into(),
            Jv::Int(units_with_checks as i64),
        ),
        (
            "units_zero_checks".into(),
            Jv::Int(units_zero_checks as i64),
        ),
        (
            "units".into(),
            Jv::Arr(all_units.iter().map(Row::to_jv).collect()),
        ),
        (
            "by_module".into(),
            Jv::Obj(
                by_module
                    .iter()
                    .map(|(mid, us)| {
                        (
                            mid.clone(),
                            Jv::Arr(us.iter().map(Row::to_jv).collect::<Vec<_>>()),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "shortfalls".into(),
            Jv::Arr(shortfalls.iter().cloned().map(Jv::Str).collect()),
        ),
    ]);
    let mut body = String::new();
    dump(&payload, 0, &mut body);
    body.push('\n');

    // Everything below is COLLECTED, then reported once. The verdict is the
    // first line of a report composed only after every check is done — see
    // bd-lt7; this block used to print PASS first and FAIL underneath it.
    let mut failures: Vec<String> = Vec::new();
    let mut detail: Vec<String> = Vec::new();

    if by_module.is_empty() {
        failures.push("zero modules discovered (vacuous unit build is ERROR)".to_string());
    }
    if all_units.is_empty() {
        failures.push("zero units discovered (vacuous unit build is ERROR)".to_string());
    }

    // A FILTER THAT REMOVES EVERYTHING IS AN ERROR, NOT AN EMPTY BUILD. A bank
    // that loaded rows but whose approved pool is empty would otherwise emit an
    // artifact with every check_item_ids=[] and let the coverage floor do the
    // complaining — a shortfall message about CONTENT, for what is actually a
    // status-filter fault.
    if !bank.is_empty() && approved.is_empty() {
        failures.push(format!(
            "bank loaded {} rows and NONE are status={} \
             (a status filter that removes the whole pool is ERROR, not an empty build)",
            bank.len(),
            py_repr(APPROVED)
        ));
    }

    let lookup = |need: &str| -> Vec<Row> {
        by_module
            .iter()
            .find(|(m, _)| m == need)
            .map(|(_, us)| us.clone())
            .unwrap_or_default()
    };
    for (need, want) in SPOT_CHECKS {
        let us = lookup(need);
        let got = us.len();
        if got < *want {
            failures.push(format!("{need} has {got} units, need ≥{want}"));
            continue;
        }
        let checks: Vec<usize> = us.iter().map(|u| u.check_count).collect();
        if checks.iter().any(|c| *c < MIN_CHECKS_PER_UNIT) {
            failures.push(format!(
                "{need} has units with <2 check items: {}",
                py_repr_int_list(&checks)
            ));
            continue;
        }
        detail.push(format!(
            "  ok: {need} units={got} check_counts={}",
            py_repr_int_list(&checks)
        ));
    }

    // The general floor. The set is DERIVED from by_module — i.e. from
    // modules_index.json — and not from a numeric bound.
    let mut primary: Vec<&String> = by_module
        .iter()
        .map(|(m, _)| m)
        .filter(|m| has_module_id_shape(m))
        .collect();
    primary.sort();
    let total_u: usize = primary
        .iter()
        .map(|m| lookup(m.as_str()).len())
        .sum::<usize>();
    let good_u: usize = primary
        .iter()
        .map(|m| {
            lookup(m.as_str())
                .iter()
                .filter(|u| u.check_count >= MIN_CHECKS_PER_UNIT)
                .count()
        })
        .sum::<usize>();
    if primary.is_empty() {
        failures
            .push("zero modules matched the module-id shape (vacuous check floor is ERROR)".into());
    } else if total_u == 0 {
        failures.push(format!(
            "{} modules carry zero units between them (vacuous check floor is ERROR)",
            primary.len()
        ));
    } else if (good_u as f64) / (total_u as f64) < COVERAGE_FLOOR {
        failures.push(format!(
            "only {good_u}/{total_u} units across {} modules have ≥2 checks",
            primary.len()
        ));
    } else {
        let names: Vec<String> = primary.iter().map(|m| (*m).clone()).collect();
        detail.push(format!(
            "  ok: check coverage {good_u}/{total_u} across {} modules ({})",
            primary.len(),
            names.join(" ")
        ));
    }

    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" };
    let mut report: Vec<String> = vec![
        format!(
            "{verdict}: build_units units={} modules={}",
            all_units.len(),
            by_module.len()
        ),
        format!(
            "  bank_items={} approved_pool={} \
             units_with_checks≥2={units_with_checks} zero={units_zero_checks}",
            bank.len(),
            approved.len()
        ),
        if failures.is_empty() {
            format!("  out={OUT_REL}")
        } else {
            format!("  out={OUT_REL} NOT WRITTEN (a failing build leaves no artifact)")
        },
    ];
    if !shortfalls.is_empty() {
        report.push(format!(
            "  WARN shortfalls: {}",
            py_repr_str_list(&shortfalls)
        ));
    }
    report.extend(detail);
    report.extend(failures.iter().map(|f| format!("  - {f}")));

    // THE SIDE EFFECT DEPENDS ON THE VERDICT, never the reverse. This used to
    // hand back `Some(..)` unconditionally and `run` wrote it before printing,
    // so a RED run left a units_index.json behind.
    Ok(Outcome {
        stdout: format!("{}\n", report.join("\n")),
        code: i32::from(!failures.is_empty()),
        artifact: if failures.is_empty() {
            Some((join_rel(root, OUT_REL), body))
        } else {
            None
        },
    })
}

/// Compile units and write the artifact on the GREEN path only.
///
/// A RED compile writes nothing. The caller prints `outcome.stdout` and maps
/// `outcome.code` onto the process exit — this function does not call
/// `process::exit`.
pub fn write_units(root: &Path) -> Result<Outcome, LearnError> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let outcome = evaluate(&root)?;
    debug_assert!(
        outcome.code == 0 || outcome.artifact.is_none(),
        "a failing run must not carry an artifact"
    );
    if outcome.code == 0 {
        if let Some((path, body)) = &outcome.artifact {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LearnError::io(format!("mkdir {}: {e}", parent.display())))?;
            }
            std::fs::write(path, body.as_bytes())
                .map_err(|e| LearnError::io(format!("write {}: {e}", path.display())))?;
        }
    }
    Ok(outcome)
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(title: &str, words: usize) -> RawUnit {
        RawUnit {
            title: title.to_string(),
            heading_id: slugify(title),
            word_count: words,
            order: 1,
        }
    }

    #[test]
    fn slugify_strips_emphasis_punctuation_and_collapses_runs() {
        assert_eq!(
            slugify("Why it matters — ops & design"),
            "why-it-matters-ops-design"
        );
        assert_eq!(slugify("**Bold**"), "bold");
        assert_eq!(slugify("A_B"), "ab");
        assert_eq!(slugify("!!!"), "section");
        assert_eq!(slugify(""), "section");
    }

    #[test]
    fn word_count_is_the_number_of_word_character_runs() {
        assert_eq!(count_words("one two  three"), 3);
        assert_eq!(count_words("a-b"), 2);
        assert_eq!(count_words("snake_case"), 1);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn h2_is_the_only_heading_level_that_opens_a_unit() {
        assert_eq!(h2_title("## Title").as_deref(), Some("Title"));
        assert_eq!(h2_title("##   Spaced").as_deref(), Some("Spaced"));
        assert!(h2_title("# One").is_none());
        assert!(h2_title("### Three").is_none());
        assert!(h2_title("##NoSpace").is_none());
        assert!(h2_title("plain").is_none());
    }

    #[test]
    fn closing_hashes_and_links_are_removed_from_titles() {
        assert_eq!(strip_trailing_hashes("Title ##"), "Title");
        assert_eq!(strip_trailing_hashes("Title"), "Title");
        assert_eq!(strip_trailing_hashes("A B C"), "A B C");
        assert_eq!(strip_links("see [docs](x.html) now"), "see docs now");
        assert_eq!(strip_links("[a](b)"), "a");
        assert_eq!(strip_links("[unclosed"), "[unclosed");
    }

    #[test]
    fn a_fenced_block_cannot_open_a_unit() {
        let long = "word ".repeat(MIN_UNIT_WORDS + 1);
        let md = format!("## Real\n\n```\n## Fake\n```\n\n{long}\n");
        let units = split_h2_units(&md);
        assert_eq!(units.len(), 1, "{units:?}");
        assert_eq!(units[0].title, "Real");
        // and the fence lines themselves stay in the body, so they count.
        assert!(units[0].word_count > MIN_UNIT_WORDS);
    }

    #[test]
    fn duplicate_headings_get_suffixed_ids() {
        let md = "## Same\n\nLearning objectives body\n\n## Same\n\nmore\n";
        let units = split_h2_units(md);
        assert_eq!(units.len(), 0, "both are short and neither title qualifies");

        let long = "word ".repeat(MIN_UNIT_WORDS + 1);
        let md = format!("## Same\n\n{long}\n\n## Same\n\n{long}\n");
        let units = split_h2_units(&md);
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].heading_id, "same");
        assert_eq!(units[1].heading_id, "same-2");
    }

    #[test]
    fn a_short_section_survives_only_if_its_title_names_an_objective() {
        let md = "## Learning objectives\n\nshort\n\n## Filler\n\nshort\n";
        let units = split_h2_units(md);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].title, "Learning objectives");
        assert_eq!(units[0].order, 1);
    }

    #[test]
    fn a_dropped_section_still_consumes_its_slug() {
        // "Same" is short and dropped, but it took the `same` id with it, so
        // the surviving second section is `same-2`. Reordering the filter would
        // silently renumber every heading anchor the web app links to.
        let long = "word ".repeat(MIN_UNIT_WORDS + 1);
        let md = format!("## Same\n\nshort\n\n## Same\n\n{long}\n");
        let units = split_h2_units(&md);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].heading_id, "same-2");
    }

    #[test]
    fn estimate_minutes_is_clamped_and_rounds_ties_to_even() {
        assert_eq!(estimate_minutes(0), 3);
        assert_eq!(estimate_minutes(100_000), 20);
        assert_eq!(py_round(0.5), 0.0);
        assert_eq!(py_round(1.5), 2.0);
        assert_eq!(py_round(2.5), 2.0);
        assert_eq!(py_round(-0.5), 0.0);
    }

    #[test]
    fn module_ids_yield_their_number_or_nothing() {
        assert_eq!(module_num_from_id("01-mission-critical"), Some(1));
        assert_eq!(module_num_from_id("06-power"), Some(6));
        assert_eq!(module_num_from_id("README"), None);
        assert_eq!(module_num_from_id("1-x"), None);
        assert!(has_module_id_shape("00-x"));
        assert!(!has_module_id_shape("x"));
    }

    #[test]
    fn json_round_trips_the_shapes_the_inputs_use() {
        let v = json_parse(r#"{"a": [1, 2.5, "xé", true, null], "b": {}}"#).unwrap();
        assert_eq!(obj_get(&v, "b"), Some(&Json::Obj(vec![])));
        let Some(Json::Arr(items)) = obj_get(&v, "a") else {
            panic!("{v:?}")
        };
        assert_eq!(items[0], Json::Int(1));
        assert_eq!(items[2], Json::Str("xé".to_string()));
        assert_eq!(items[3], Json::Bool(true));
        assert_eq!(items[4], Json::Null);
        assert!(json_parse("{").is_err());
        assert!(json_parse("{} junk").is_err());
    }

    #[test]
    fn surrogate_pairs_decode_and_re_encode() {
        let v = json_parse(r#""😀""#).unwrap();
        assert_eq!(v, Json::Str("😀".to_string()));
        let mut s = String::new();
        json_string_ascii("😀", &mut s);
        assert_eq!(s, "\"\\ud83d\\ude00\"");
    }

    #[test]
    fn ascii_escaping_matches_python_ensure_ascii_true() {
        let mut s = String::new();
        json_string_ascii("a\"b\\c\td — e\u{7f}", &mut s);
        assert_eq!(s, "\"a\\\"b\\\\c\\td \\u2014 e\\u007f\"");
    }

    #[test]
    fn the_writer_sorts_keys_and_renders_empty_containers_inline() {
        let v = Jv::Obj(vec![
            ("b".into(), Jv::Arr(vec![])),
            ("a".into(), Jv::Obj(vec![])),
            ("c".into(), Jv::Arr(vec![Jv::Int(1), Jv::Null])),
        ]);
        let mut s = String::new();
        dump(&v, 0, &mut s);
        assert_eq!(
            s,
            "{\n  \"a\": {},\n  \"b\": [],\n  \"c\": [\n    1,\n    null\n  ]\n}"
        );
    }

    #[test]
    fn python_reprs_match_for_the_two_list_shapes_the_report_prints() {
        assert_eq!(py_repr_int_list(&[3, 3, 3]), "[3, 3, 3]");
        assert_eq!(py_repr_int_list(&[]), "[]");
        assert_eq!(
            py_repr_str_list(&["a: 1 units".to_string()]),
            "['a: 1 units']"
        );
        assert_eq!(py_repr("it's"), "\"it's\"");
    }

    #[test]
    fn topic_matching_prefers_full_label_overlap() {
        let topics = vec![
            Topic {
                id: "m01-dc-types".into(),
                label: "Data centre types".into(),
            },
            Topic {
                id: "m01-importance".into(),
                label: "Importance".into(),
            },
        ];
        let ids = match_topics("Data centre types", "data-centre-types", &topics);
        assert_eq!(ids.first().map(String::as_str), Some("m01-dc-types"));
        // The full-domain fill always follows, so a unit is never left unable
        // to map — which is exactly why attachment is not evidence of fit.
        let all = assign_topic_ids("Nothing alike", "nothing-alike", &topics);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn an_empty_topic_registry_leaves_topic_ids_empty() {
        assert!(assign_topic_ids("Anything", "anything", &[]).is_empty());
    }

    fn item(id: &str, module: i64, topics: &[&str]) -> Item {
        Item {
            id: id.into(),
            module,
            topic_ids: topics.iter().map(|s| (*s).to_string()).collect(),
            stem: "Which of these is the most likely failure mode on this run?".into(),
            explanation: "because the explanation is long enough to score".into(),
            choices_len: 4,
            // These fixtures exercise the PICKER, so they must be drawable.
            // Absent status is WITHHELD (see `Item::is_approved`), which would
            // make every picker test vacuous — zero candidates, trivially
            // deterministic, asserting nothing.
            status: "approved".into(),
        }
    }

    #[test]
    fn the_picker_is_deterministic_and_rotates_by_unit_order() {
        let mut bank: HashMap<i64, Vec<Item>> = HashMap::new();
        bank.insert(
            1,
            vec![
                item("q1", 1, &["t1"]),
                item("q2", 1, &["t2"]),
                item("q3", 1, &["t3"]),
                item("q4", 1, &["t4"]),
                item("q5", 1, &["t5"]),
                item("q6", 1, &["t6"]),
            ],
        );
        let tids: Vec<String> = vec!["t1".into()];
        let a = pick_check_items(&bank, Some(1), &tids, 1, CHECK_N);
        let b = pick_check_items(&bank, Some(1), &tids, 2, CHECK_N);
        assert_eq!(a.len(), CHECK_N);
        assert_eq!(a, pick_check_items(&bank, Some(1), &tids, 1, CHECK_N));
        assert_ne!(a, b, "adjacent units must not draw the same questions");
    }

    #[test]
    fn the_picker_returns_nothing_without_a_module_or_a_pool() {
        let bank: HashMap<i64, Vec<Item>> = HashMap::new();
        assert!(pick_check_items(&bank, None, &[], 1, CHECK_N).is_empty());
        assert!(pick_check_items(&bank, Some(0), &[], 1, CHECK_N).is_empty());
        assert!(pick_check_items(&bank, Some(9), &[], 1, CHECK_N).is_empty());
    }

    #[test]
    fn an_empty_topic_set_still_fills_from_the_module_pool() {
        // This is the finding, asserted so it cannot be "fixed" by accident: a
        // missing topic registry does NOT starve the picker, which is why the
        // oracle stays GREEN without one.
        let mut bank: HashMap<i64, Vec<Item>> = HashMap::new();
        bank.insert(1, vec![item("q1", 1, &[]), item("q2", 1, &[])]);
        let picked = pick_check_items(&bank, Some(1), &[], 1, CHECK_N);
        assert_eq!(picked.len(), 2, "{picked:?}");
    }

    /// A tree `evaluate` can run against, written into a temp dir.
    fn tree(dir: &Path, with_topics: bool, with_index: bool) {
        let put = |rel: &str, body: &str| {
            let p = join_rel(dir, rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, body).unwrap();
        };
        let long: String = (0..MIN_UNIT_WORDS + 5)
            .map(|i| format!("word{i} "))
            .collect();
        put(
            "web/content/modules/01-mission-critical.md",
            &format!("## A\n\n{long}\n\n## B\n\n{long}\n"),
        );
        if with_topics {
            put(
                TOPICS_REL,
                "[[topic]]\nid = \"m01-a\"\ndomain = \"01-mission-critical\"\nlabel = \"A\"\n",
            );
        }
        if with_index {
            put(
                MOD_INDEX_REL,
                "{\"modules\": [{\"id\": \"01-mission-critical\", \"empty\": false}]}\n",
            );
        }
    }

    #[test]
    fn a_red_outcome_never_carries_an_artifact() {
        // WRITE-AFTER-VERDICT as an invariant of `evaluate` itself, not merely
        // of `run`: the artifact is `None` on every RED path, so `run` has
        // nothing it *could* write even if its guard were removed. The three
        // RED shapes below are the three doors into a failing build.
        let td = tempfile::tempdir().unwrap();

        // (a) a missing content tree — the earliest return.
        let a = td.path().join("a");
        std::fs::create_dir_all(&a).unwrap();
        let o = evaluate(&a).unwrap();
        assert_ne!(o.code, 0);
        assert!(o.artifact.is_none(), "{}", o.stdout);
        assert!(!o.stdout.contains("PASS"), "{}", o.stdout);

        // (b) a missing input registry — the new anti-vacuous leg. Each
        // registry is dropped on its own so neither can be the one doing the
        // work for both.
        for (with_topics, with_index, want) in [
            (false, true, TOPICS_REL),
            (true, false, MOD_INDEX_REL),
            (false, false, TOPICS_REL),
        ] {
            let d = td.path().join(format!("b{with_topics}{with_index}"));
            tree(&d, with_topics, with_index);
            let o = evaluate(&d).unwrap();
            assert_eq!(o.code, 1, "{}", o.stdout);
            assert!(
                o.stdout.contains(want),
                "the absent registry must be NAMED: {}",
                o.stdout
            );
            assert!(o.artifact.is_none(), "{}", o.stdout);
            assert!(!o.stdout.contains("PASS"), "{}", o.stdout);
        }

        // (c) a complete tree that fails a threshold — the report path, which
        // is the one that used to hand back an artifact anyway. 06-power is
        // absent, so the spot check fires.
        let c = td.path().join("c");
        tree(&c, true, true);
        let o = evaluate(&c).unwrap();
        assert_ne!(o.code, 0, "{}", o.stdout);
        assert!(o.stdout.starts_with("FAIL: build_units"), "{}", o.stdout);
        assert!(o.stdout.contains("NOT WRITTEN"), "{}", o.stdout);
        assert!(
            o.artifact.is_none(),
            "a failing build must not carry an artifact: {}",
            o.stdout
        );
        assert!(!o.stdout.contains("PASS"), "{}", o.stdout);
    }

    #[test]
    fn there_is_no_glob_fallback_left_to_find_a_module_the_learn_index_omits() {
        // The fallback is DELETED, not gated: with the Learn index absent there
        // is no second derivation that could sweep in a README.md and print
        // `modules=16` over a 15-module index. Measured 2026-08-14, that is
        // exactly what it did.
        let td = tempfile::tempdir().unwrap();
        let d = td.path().join("noindex");
        tree(&d, true, false);
        std::fs::write(join_rel(&d, "web/content/modules/README.md"), "## R\n\nx\n").unwrap();
        let o = evaluate(&d).unwrap();
        assert_eq!(o.code, 1, "{}", o.stdout);
        assert!(!o.stdout.contains("modules="), "{}", o.stdout);
        assert!(!o.stdout.contains("README"), "{}", o.stdout);
    }

    #[test]
    fn a_bank_less_build_cannot_clear_the_coverage_floor() {
        // The anti-vacuous leg, asserted through the same expression
        // `evaluate` uses: units that drew no items must not report GREEN.
        let checks = [0usize, 0, 0];
        let good = checks.iter().filter(|c| **c >= MIN_CHECKS_PER_UNIT).count();
        assert!(
            (good as f64) / (checks.len() as f64) < COVERAGE_FLOOR,
            "a build with zero attached items would report GREEN"
        );
        assert!(
            SPOT_CHECKS.iter().all(|(_, want)| *want != 0),
            "a spot check that wants zero units checks nothing"
        );
        assert_eq!(
            unit("Learning objectives", 1).heading_id,
            "learning-objectives"
        );
    }
}
