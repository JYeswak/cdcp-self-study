//! export-anki — Rust port of `scripts/export_anki.py`
//! (bd-substrate-rust-migration-jhd.13).
//!
//! PRODUCES the learner artifact (`dist/anki/**`): TSV, optional CSV, README.
//! Field order, `\x1f`, newline flattening and the trailing newline decide
//! whether a re-import MERGES or DUPLICATES. `tests/diff_export_anki.rs`
//! compares stdout, stderr, exit, file set and bytes against the oracle.
//!
//! CAN DECIDE: those deterministic bytes match the oracle. CANNOT: whether
//! Anki imports the deck (Anki is not wired), whether cards are sound, or
//! whether a re-import merges. Status filter drops retired/draft; it cannot
//! judge quality. (Honesty scan is a substring match — see
//! bd-overclaim-scan-hits-disclaimers-xidi.)
//!
//! `.apkg` is BLOCKED, not skipped (bd-anki-apkg-not-reproducible-e13a).
//! Measured 2026-08-14: two oracle runs 2s apart, TSV identical, `.apkg`
//! 209382/212731 bytes different (98.4%) — `int(time.time())` in col/notes/
//! cards + zip mtimes, DEFLATE avalanches. A byte-exact `.apkg` differential
//! is UNSATISFIABLE. Pin `now` in the oracle first, then port the leg.
//!
//! Defects: RETIRED ITEMS FIXED (bd-anki-ships-retired-bbdr) — bank/seed42
//! drop retired+draft; keys/mock40 is already the approved draw. Still
//! reproduced: `correct in "ABCD"` is a SUBSTRING test (bd-anki-correct-
//! substring-crash-bstu); `int(module or -1)` hides module 0; `--limit`
//! without `--seed` takes the alphabetically-first N.
//!
//! Deviations (check.sh uses `--format tsv,csv` here and python for `.apkg`):
//! bad flags are Usage/3 vs argparse/2; tracebacks are Error/4; `str::trim`
//! vs `str.isspace`; last-key-wins dicts via reverse find; float shortest-
//! roundtrip + `.0`. RED does NOT go through `GateError` (oracle: `FAIL:`
//! on stderr, exit 1). Empty decks never exit 0.

use crate::registry::{GateCtx, GateError};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const NAME: &str = "export-anki";
pub const SUMMARY: &str =
    "export bank items to Anki TSV/CSV (byte-exact port of scripts/export_anki.py)";

// ── named constants (rule 9: every threshold is a named constant) ───────────

/// Directory, relative to the engine root, holding one TOML per bank item.
const ITEMS_DIR_REL: &str = "bank/items";
/// Directory, relative to the engine root, holding the generated web packs.
const WEB_DATA_REL: &str = "web/data";
/// Default `--out`, relative to the engine root.
const DEFAULT_OUT_REL: &str = "dist/anki";
/// Default `--format` list.
const DEFAULT_FORMAT: &str = "tsv,apkg";
/// Default `--deck-name` (only observable once the `.apkg` leg is unblocked).
const DEFAULT_DECK_NAME: &str = "CDCP Study";
/// The three formats the oracle recognises.
const KNOWN_FORMATS: [&str; 3] = ["tsv", "csv", "apkg"];
/// Output file stems, one per `--source`.
const STEM_BANK: &str = "cdcp_bank";
const STEM_SEED42: &str = "cdcp_seed42_bank";
const STEM_KEYS: &str = "cdcp_seed42_mock40";
/// The two operator comment lines the TSV opens with, before any card row.
const TSV_COMMENT_1: &str = "# CDCP Study Anki export — stem / answer / explanation / module\n";
const TSV_COMMENT_2: &str = "# Not a credential. Import as Basic (or map 4 fields).\n";
/// Header row of the CSV form. The TSV deliberately has none.
const CSV_HEADER: [&str; 4] = ["stem", "answer", "explanation", "module"];
/// Answer letters `format_answer` maps to a choice index. Kept as the oracle
/// wrote it — a `&str` — because the oracle tests membership with `in`, which on
/// a string is a SUBSTRING test and not a membership test. See the header.
const ANSWER_LETTERS: &str = "ABCD";

// ── a Python-shaped dynamic value ──────────────────────────────────────────
//
// Both input formats (TOML for the bank, JSON for the web packs) land here, so
// the filtering and formatting below is written once against Python semantics
// rather than twice against two Rust type systems.

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Val>),
    Map(Vec<(String, Val)>),
}

impl Val {
    /// Python truthiness: `None`, `False`, `0`, `""`, `[]` and `{}` are falsy.
    fn truthy(&self) -> bool {
        match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Int(i) => *i != 0,
            Val::Float(f) => *f != 0.0,
            Val::Str(s) => !s.is_empty(),
            Val::List(v) => !v.is_empty(),
            Val::Map(m) => !m.is_empty(),
        }
    }

    /// `str(x)` for the shapes these inputs can hold.
    fn py_str(&self) -> String {
        match self {
            Val::Null => "None".to_string(),
            Val::Bool(b) => (if *b { "True" } else { "False" }).to_string(),
            Val::Int(i) => i.to_string(),
            Val::Float(f) => py_float_str(*f),
            Val::Str(s) => s.clone(),
            Val::List(v) => format!(
                "[{}]",
                v.iter().map(|x| x.py_repr()).collect::<Vec<_>>().join(", ")
            ),
            Val::Map(_) => "{...}".to_string(),
        }
    }

    /// `repr(x)` — only ever needed for the element form inside a list repr.
    fn py_repr(&self) -> String {
        match self {
            Val::Str(s) => py_repr_str(s),
            other => other.py_str(),
        }
    }

    fn get(&self, key: &str) -> Option<&Val> {
        match self {
            Val::Map(m) => m.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// `d.get(key) or default` — the idiom the oracle uses everywhere.
    fn get_or_falsy<'a>(&'a self, key: &str) -> Option<&'a Val> {
        self.get(key).filter(|v| v.truthy())
    }
}

/// `repr(float)`: shortest round-trip, but integral values keep a `.0`.
fn py_float_str(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let s = format!("{f}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

/// `repr(str)` — single-quoted unless the value contains a single quote and no
/// double quote. Used only for the `unknown format(s)` report, which prints a
/// `sorted()` list of strings.
fn py_repr_str(s: &str) -> String {
    let has_sq = s.contains('\'');
    let has_dq = s.contains('"');
    let (q, esc_q) = if has_sq && !has_dq {
        ('"', '"')
    } else {
        ('\'', '\'')
    };
    let mut out = String::new();
    out.push(q);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == esc_q && c == q => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(q);
    out
}

// ── minimal JSON reader (hand-rolled; see the dependency note below) ────────
//
// DEPENDENCY DECISION, stated explicitly rather than reached for: `cdcp_gate`
// depends on `serde` and `toml` and nothing else, and its Cargo.toml is a shared
// file eight concurrent agents are working around. The three JSON packs this
// gate reads are machine-generated, small, and consumed for four scalar fields
// each, so a ~120-line reader is cheaper than a shared-manifest edit. `serde_json`
// would buy nothing this needs.
//
// The `.apkg` leg is the one place where a real dependency argument exists — it
// needs a SQLite file writer and a DEFLATE encoder — and that leg is blocked on
// bd-anki-apkg-not-reproducible-e13a for reasons that have nothing to do with
// dependencies. When it lands, `rusqlite` + `zip` is the argument to make, and
// making it then is honest; making it now would be reaching.

struct JsonReader<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> JsonReader<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self { b, i: 0 }
    }

    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn parse(&mut self) -> Result<Val, String> {
        self.ws();
        let v = self.value()?;
        self.ws();
        if self.i != self.b.len() {
            return Err(format!("trailing bytes at offset {}", self.i));
        }
        Ok(v)
    }

    fn value(&mut self) -> Result<Val, String> {
        self.ws();
        let Some(&c) = self.b.get(self.i) else {
            return Err("unexpected end of JSON".into());
        };
        match c {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => Ok(Val::Str(self.string()?)),
            b't' => self.lit("true", Val::Bool(true)),
            b'f' => self.lit("false", Val::Bool(false)),
            b'n' => self.lit("null", Val::Null),
            _ => self.number(),
        }
    }

    fn lit(&mut self, word: &str, v: Val) -> Result<Val, String> {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(format!("bad literal at offset {}", self.i))
        }
    }

    fn object(&mut self) -> Result<Val, String> {
        self.i += 1; // '{'
        let mut out = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Val::Map(out));
        }
        loop {
            self.ws();
            if self.b.get(self.i) != Some(&b'"') {
                return Err(format!("object key expected at offset {}", self.i));
            }
            let k = self.string()?;
            self.ws();
            if self.b.get(self.i) != Some(&b':') {
                return Err(format!("':' expected at offset {}", self.i));
            }
            self.i += 1;
            let v = self.value()?;
            // Later duplicate keys win, as in Python's json.
            if let Some(slot) = out.iter_mut().find(|(kk, _): &&mut (String, Val)| *kk == k) {
                slot.1 = v;
            } else {
                out.push((k, v));
            }
            self.ws();
            match self.b.get(self.i) {
                Some(&b',') => self.i += 1,
                Some(&b'}') => {
                    self.i += 1;
                    return Ok(Val::Map(out));
                }
                _ => return Err(format!("',' or '}}' expected at offset {}", self.i)),
            }
        }
    }

    fn array(&mut self) -> Result<Val, String> {
        self.i += 1; // '['
        let mut out = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Val::List(out));
        }
        loop {
            out.push(self.value()?);
            self.ws();
            match self.b.get(self.i) {
                Some(&b',') => self.i += 1,
                Some(&b']') => {
                    self.i += 1;
                    return Ok(Val::List(out));
                }
                _ => return Err(format!("',' or ']' expected at offset {}", self.i)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.i += 1; // '"'
        let mut out = String::new();
        loop {
            let Some(&c) = self.b.get(self.i) else {
                return Err("unterminated string".into());
            };
            match c {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.i += 1;
                    let Some(&e) = self.b.get(self.i) else {
                        return Err("unterminated escape".into());
                    };
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hi = self.hex4()?;
                            if (0xD800..0xDC00).contains(&hi) {
                                if self.b.get(self.i) == Some(&b'\\')
                                    && self.b.get(self.i + 1) == Some(&b'u')
                                {
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    let cp = 0x10000
                                        + (((hi - 0xD800) as u32) << 10)
                                        + (lo - 0xDC00) as u32;
                                    out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                                } else {
                                    out.push('\u{fffd}');
                                }
                            } else {
                                out.push(char::from_u32(hi as u32).unwrap_or('\u{fffd}'));
                            }
                        }
                        _ => return Err("bad escape".into()),
                    }
                }
                _ => {
                    // Copy one whole UTF-8 sequence.
                    let start = self.i;
                    let len = utf8_len(c);
                    self.i += len;
                    let slice = self
                        .b
                        .get(start..self.i)
                        .ok_or_else(|| "truncated UTF-8".to_string())?;
                    out.push_str(std::str::from_utf8(slice).map_err(|e| e.to_string())?);
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u16, String> {
        let s = self
            .b
            .get(self.i..self.i + 4)
            .ok_or_else(|| "short \\u escape".to_string())?;
        let s = std::str::from_utf8(s).map_err(|e| e.to_string())?;
        self.i += 4;
        u16::from_str_radix(s, 16).map_err(|e| e.to_string())
    }

    fn number(&mut self) -> Result<Val, String> {
        let start = self.i;
        while self.i < self.b.len()
            && matches!(
                self.b[self.i],
                b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
            )
        {
            self.i += 1;
        }
        let s = std::str::from_utf8(&self.b[start..self.i]).map_err(|e| e.to_string())?;
        if s.is_empty() {
            return Err(format!("number expected at offset {start}"));
        }
        if s.contains('.') || s.contains('e') || s.contains('E') {
            s.parse::<f64>().map(Val::Float).map_err(|e| e.to_string())
        } else {
            s.parse::<i64>().map(Val::Int).map_err(|e| e.to_string())
        }
    }
}

fn utf8_len(b: u8) -> usize {
    match b {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        _ => 4,
    }
}

fn toml_to_val(v: &toml::Value) -> Val {
    match v {
        toml::Value::String(s) => Val::Str(s.clone()),
        toml::Value::Integer(i) => Val::Int(*i),
        toml::Value::Float(f) => Val::Float(*f),
        toml::Value::Boolean(b) => Val::Bool(*b),
        toml::Value::Datetime(d) => Val::Str(d.to_string()),
        toml::Value::Array(a) => Val::List(a.iter().map(toml_to_val).collect()),
        toml::Value::Table(t) => {
            Val::Map(t.iter().map(|(k, v)| (k.clone(), toml_to_val(v))).collect())
        }
    }
}

// ── MT19937, so `--limit` with `--seed` samples exactly as Python does ──────
//
// `filter_items` reaches for `random.Random(seed).shuffle(...)`. Reproducing the
// SAMPLE, not merely "a" sample, needs CPython's Mersenne Twister, its
// `init_by_array` seeding from the absolute value of an int seed, its
// `getrandbits` and its rejection-sampling `_randbelow`. The differential is
// what CHECKS this emulation against CPython on every case it runs — see
// `a_seeded_limit_samples_identically`. It does not establish agreement on
// inputs no case exercises.

const MT_N: usize = 624;
const MT_M: usize = 397;
const MT_MATRIX_A: u32 = 0x9908_b0df;
const MT_UPPER_MASK: u32 = 0x8000_0000;
const MT_LOWER_MASK: u32 = 0x7fff_ffff;
const MT_INIT_SEED: u32 = 19_650_218;
const MT_INIT_MULT: u32 = 1_812_433_253;
const MT_ARRAY_MULT_A: u32 = 1_664_525;
const MT_ARRAY_MULT_B: u32 = 1_566_083_941;
// The four tempering parameters of MT19937, named per rule 9 rather than left
// as inline literals. These are the PUBLISHED constants of Matsumoto &
// Nishimura (1998) — the algorithm's definition, not a threshold anyone chose,
// and emphatically not a module bound. `MT_TEMPER_SHIFT_T` lands in 13..=16 and
// therefore owes a NamedBound verdict row in tests/rebase_module_bounds.rs; the
// row is REPORTED to that file's owner rather than written here.
const MT_TEMPER_SHIFT_U: u32 = 11;
const MT_TEMPER_SHIFT_S: u32 = 7;
const MT_TEMPER_MASK_B: u32 = 0x9d2c_5680;
const MT_TEMPER_SHIFT_T: u32 = 15;
const MT_TEMPER_MASK_C: u32 = 0xefc6_0000;
const MT_TEMPER_SHIFT_L: u32 = 18;

struct Mt19937 {
    mt: [u32; MT_N],
    idx: usize,
}

impl Mt19937 {
    fn init_genrand(s: u32) -> Self {
        let mut mt = [0u32; MT_N];
        mt[0] = s;
        for i in 1..MT_N {
            mt[i] = MT_INIT_MULT
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { mt, idx: MT_N }
    }

    /// CPython seeds an int seed as `init_by_array(little-endian 32-bit words of
    /// abs(seed))`, with a single zero word when the seed is 0.
    fn from_python_int_seed(seed: i64) -> Self {
        let n = (seed as i128).unsigned_abs();
        let mut key: Vec<u32> = Vec::new();
        let mut rem = n;
        while rem > 0 {
            key.push((rem & 0xffff_ffff) as u32);
            rem >>= 32;
        }
        if key.is_empty() {
            key.push(0);
        }
        let mut r = Mt19937::init_genrand(MT_INIT_SEED);
        r.init_by_array(&key);
        r
    }

    fn init_by_array(&mut self, key: &[u32]) {
        let mut i: usize = 1;
        let mut j: usize = 0;
        let mut k = MT_N.max(key.len());
        while k > 0 {
            self.mt[i] = (self.mt[i]
                ^ ((self.mt[i - 1] ^ (self.mt[i - 1] >> 30)).wrapping_mul(MT_ARRAY_MULT_A)))
            .wrapping_add(key[j])
            .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= MT_N {
                self.mt[0] = self.mt[MT_N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
            k -= 1;
        }
        let mut k = MT_N - 1;
        while k > 0 {
            self.mt[i] = (self.mt[i]
                ^ ((self.mt[i - 1] ^ (self.mt[i - 1] >> 30)).wrapping_mul(MT_ARRAY_MULT_B)))
            .wrapping_sub(i as u32);
            i += 1;
            if i >= MT_N {
                self.mt[0] = self.mt[MT_N - 1];
                i = 1;
            }
            k -= 1;
        }
        self.mt[0] = MT_UPPER_MASK;
        self.idx = MT_N;
    }

    fn genrand_u32(&mut self) -> u32 {
        if self.idx >= MT_N {
            for i in 0..MT_N {
                let y = (self.mt[i] & MT_UPPER_MASK) | (self.mt[(i + 1) % MT_N] & MT_LOWER_MASK);
                let mut next = self.mt[(i + MT_M) % MT_N] ^ (y >> 1);
                if y & 1 != 0 {
                    next ^= MT_MATRIX_A;
                }
                self.mt[i] = next;
            }
            self.idx = 0;
        }
        let mut y = self.mt[self.idx];
        self.idx += 1;
        y ^= y >> MT_TEMPER_SHIFT_U;
        y ^= (y << MT_TEMPER_SHIFT_S) & MT_TEMPER_MASK_B;
        y ^= (y << MT_TEMPER_SHIFT_T) & MT_TEMPER_MASK_C;
        y ^= y >> MT_TEMPER_SHIFT_L;
        y
    }

    /// `getrandbits(k)` for the `k <= 32` case, which is the only one a list
    /// index can reach (a bank of 2^32 items is not a thing).
    fn getrandbits(&mut self, k: u32) -> u32 {
        if k == 0 {
            return 0;
        }
        debug_assert!(k <= 32, "getrandbits: only the k<=32 path is ported");
        self.genrand_u32() >> (32 - k)
    }

    /// `_randbelow_with_getrandbits`: rejection sampling, never modulo.
    fn randbelow(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let k = 32 - n.leading_zeros();
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }

    /// `random.shuffle(x)`.
    fn shuffle<T>(&mut self, x: &mut [T]) {
        if x.len() < 2 {
            return;
        }
        for i in (1..x.len()).rev() {
            let j = self.randbelow(i as u32 + 1) as usize;
            x.swap(i, j);
        }
    }
}

// ── loaders ────────────────────────────────────────────────────────────────

fn read_text(path: &Path) -> Result<String, GateError> {
    std::fs::read_to_string(path)
        .map_err(|e| GateError::error(format!("read {}: {e}", path.display())))
}

/// `load_bank_items()` — every `bank/items/*.toml` that carries an `id`, in
/// filename order. A missing directory yields NOTHING, which the caller turns
/// into the anti-vacuous ERROR rather than a green empty deck.
fn load_bank_items(root: &Path) -> Result<Vec<Val>, GateError> {
    let dir = root.join(ITEMS_DIR_REL);
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(&dir) {
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
        let text = read_text(&p)?;
        let parsed: toml::Value = text
            .parse()
            .map_err(|e| GateError::error(format!("parse {}: {e}", p.display())))?;
        let v = toml_to_val(&parsed);
        if v.get("id").is_some() {
            out.push(v);
        }
    }
    Ok(out)
}

fn load_json(path: &Path) -> Result<Val, GateError> {
    let bytes = std::fs::read(path)
        .map_err(|e| GateError::error(format!("read {}: {e}", path.display())))?;
    JsonReader::new(&bytes)
        .parse()
        .map_err(|e| GateError::error(format!("parse {}: {e}", path.display())))
}

/// `load_seed42_bank_items()` — the web pack, either a bare list or `{"items": [..]}`.
fn load_seed42_bank_items(root: &Path) -> Result<Option<Vec<Val>>, GateError> {
    let path = root.join(WEB_DATA_REL).join("bank_items_seed42.json");
    if !path.is_file() {
        return Ok(None);
    }
    let data = load_json(&path)?;
    match &data {
        Val::List(v) => Ok(Some(v.clone())),
        Val::Map(_) => match data.get("items") {
            Some(Val::List(v)) => Ok(Some(v.clone())),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn row(id: &Val, stem: Val, choices: Val, correct: Val, explanation: Val, module: Val) -> Val {
    Val::Map(vec![
        ("id".into(), id.clone()),
        ("stem".into(), stem),
        ("choices".into(), choices),
        ("correct".into(), correct),
        ("explanation".into(), explanation),
        ("module".into(), module),
    ])
}

fn str_or_empty(v: Option<&Val>) -> Val {
    match v {
        Some(v) if v.truthy() => v.clone(),
        _ => Val::Str(String::new()),
    }
}

fn list_or_empty(v: Option<&Val>) -> Val {
    match v {
        Some(v) if v.truthy() => v.clone(),
        _ => Val::List(Vec::new()),
    }
}

/// `it.get("module", "")` — absent key yields `""`, a present `None`/`null`
/// stays `None`. Distinct from `get(..) or ""`, and the oracle uses both.
fn module_or_blank(v: &Val) -> Val {
    match v.get("module") {
        Some(m) => m.clone(),
        None => Val::Str(String::new()),
    }
}

/// `load_keys_seed42_pack()` — the 40-card learner deck, from the live packs if
/// present and from the golden fixture otherwise, enriched from the bank either
/// way.
fn load_keys_seed42_pack(root: &Path) -> Result<Option<Vec<Val>>, GateError> {
    let mock_path = root.join(WEB_DATA_REL).join("mock40_seed42.json");
    let keys_path = root.join(WEB_DATA_REL).join("keys_seed42.json");

    if !mock_path.is_file() || !keys_path.is_file() {
        let fix = root.join("goldens/fixtures/mock40_seed42.json");
        if !fix.is_file() {
            return Ok(None);
        }
        let data = load_json(&fix)?;
        let items = match data.get("items") {
            Some(Val::List(v)) if !v.is_empty() => v.clone(),
            _ => Vec::new(),
        };
        let mut rows: Vec<Val> = Vec::new();
        for it in &items {
            // `it["id"]`, `it["stem"]` and `it["correct"]` are SUBSCRIPTS in the
            // oracle, not `.get`, so an absent key is a KeyError traceback.
            let id = it
                .get("id")
                .ok_or_else(|| GateError::error("goldens fixture item has no 'id'"))?;
            let stem = it
                .get("stem")
                .ok_or_else(|| GateError::error("goldens fixture item has no 'stem'"))?
                .clone();
            let correct = it
                .get("correct")
                .ok_or_else(|| GateError::error("goldens fixture item has no 'correct'"))?
                .clone();
            rows.push(row(
                id,
                stem,
                list_or_empty(it.get("choices")),
                correct,
                str_or_empty(it.get("explanation")),
                module_or_blank(it),
            ));
        }
        if rows.is_empty() {
            return Ok(None);
        }
        // Enrich blank explanations (and, nested inside that branch, blank
        // modules) from the bank. The module enrich sits INSIDE the explanation
        // branch in the oracle; that nesting is load-bearing and reproduced.
        let bank = load_bank_items(root)?;
        for r in rows.iter_mut() {
            let needs = !r.get("explanation").map(|v| v.truthy()).unwrap_or(false);
            if !needs {
                continue;
            }
            let id = r.get("id").cloned().unwrap_or(Val::Null);
            let Some(b) = bank.iter().rev().find(|b| b.get("id") == Some(&id)) else {
                continue;
            };
            let expl = str_or_empty(b.get("explanation"));
            set(r, "explanation", expl);
            let m = r.get("module").cloned().unwrap_or(Val::Null);
            if m == Val::Str(String::new()) || m == Val::Null {
                set(r, "module", module_or_blank(b));
            }
        }
        return Ok(Some(rows));
    }

    let mock = load_json(&mock_path)?;
    let keys = load_json(&keys_path)?;
    let key_rows = match keys.get("keys") {
        Some(Val::List(v)) => v.clone(),
        _ => Vec::new(),
    };
    let mock_items = match mock.get("items") {
        Some(Val::List(v)) => v.clone(),
        _ => Vec::new(),
    };
    let mut rows: Vec<Val> = Vec::new();
    for it in &mock_items {
        let id = it
            .get("id")
            .ok_or_else(|| GateError::error("mock40 item has no 'id'"))?;
        // `{k["item_id"]: k for k in ...}` — a duplicate item_id means the LAST row wins.
        let k = key_rows.iter().rev().find(|k| k.get("item_id") == Some(id));
        rows.push(row(
            id,
            str_or_empty(it.get("stem")),
            list_or_empty(it.get("choices")),
            k.map(|k| str_or_empty(k.get("correct")))
                .unwrap_or(Val::Str(String::new())),
            k.map(|k| str_or_empty(k.get("explanation")))
                .unwrap_or(Val::Str(String::new())),
            module_or_blank(it),
        ));
    }
    let blank_module = |r: &Val| {
        matches!(r.get("module"), Some(Val::Str(s)) if s.is_empty())
            || matches!(r.get("module"), Some(Val::Null) | None)
    };
    if rows.iter().any(blank_module) {
        let bank = load_bank_items(root)?;
        for r in rows.iter_mut() {
            if !blank_module(r) {
                continue;
            }
            let id = r.get("id").cloned().unwrap_or(Val::Null);
            if let Some(b) = bank.iter().rev().find(|b| b.get("id") == Some(&id)) {
                set(r, "module", module_or_blank(b));
            }
        }
    }
    Ok(Some(rows))
}

fn set(v: &mut Val, key: &str, val: Val) {
    if let Val::Map(m) = v {
        if let Some(slot) = m.iter_mut().find(|(k, _)| k == key) {
            slot.1 = val;
        } else {
            m.push((key.to_string(), val));
        }
    }
}

// ── formatting ─────────────────────────────────────────────────────────────

/// `format_answer(item)`.
///
/// Faithful to a defect: `correct in "ABCD"` is SUBSTRING containment, so `""`
/// and `"AB"` both take the `ord()` branch and the oracle raises `TypeError`.
/// bd-anki-correct-substring-crash-bstu.
fn format_answer(item: &Val) -> Result<String, GateError> {
    let correct_raw = item
        .get_or_falsy("correct")
        .map(|v| v.py_str())
        .unwrap_or_default();
    let correct = correct_raw.trim().to_uppercase();
    let choices = match item.get_or_falsy("choices") {
        Some(Val::List(v)) => v.clone(),
        Some(other) => match other {
            // `len()`/subscript on a str works in Python, so a string `choices`
            // is indexable. Reproduced rather than rejected.
            Val::Str(s) => s.chars().map(|c| Val::Str(c.to_string())).collect(),
            _ => Vec::new(),
        },
        None => Vec::new(),
    };
    let idx: i64 = if ANSWER_LETTERS.contains(correct.as_str()) {
        let mut cs = correct.chars();
        match (cs.next(), cs.next()) {
            (Some(c), None) => c as i64 - 'A' as i64,
            _ => {
                return Err(GateError::error(format!(
                    "format_answer: correct={correct:?} is a SUBSTRING of {ANSWER_LETTERS:?} but \
                     not a single character, so the oracle reaches ord() and dies with a \
                     TypeError traceback. Reproduced as an ERROR because a traceback is not \
                     portable output. bd-anki-correct-substring-crash-bstu"
                )));
            }
        }
    } else {
        -1
    };
    if idx >= 0 && (idx as usize) < choices.len() {
        return Ok(format!("{correct}) {}", choices[idx as usize].py_str()));
    }
    Ok(if correct.is_empty() {
        "?".to_string()
    } else {
        correct
    })
}

/// `card_fields(item)` — the four fields, in the order Anki will see them.
fn card_fields(item: &Val) -> Result<[String; 4], GateError> {
    let stem = item
        .get_or_falsy("stem")
        .map(|v| v.py_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    let answer = format_answer(item)?;
    let explanation = item
        .get_or_falsy("explanation")
        .map(|v| v.py_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    let module = match item.get("module") {
        Some(Val::Null) | None => String::new(),
        Some(v) => v.py_str(),
    };
    Ok([stem, answer, explanation, module])
}

// ── csv writer, Python's dialect rules ─────────────────────────────────────

/// `csv.writer(..., quoting=QUOTE_MINIMAL)`: a field is quoted when it contains
/// the delimiter, the quote char, any character of the line terminator, or a
/// bare `\r` or `\n`.
///
/// THE `\r` LEG WAS WRONG HERE UNTIL THE DIFFERENTIAL CAUGHT IT, and the way it
/// was wrong is worth keeping written down. Reading CPython's `_csv.c` suggests
/// the quoting test is `delimiter | escapechar | quotechar | chars-of-
/// lineterminator`, which with `lineterminator="\n"` would leave a bare `\r`
/// UNQUOTED. Measured instead of believed, CPython 3 quotes it:
///
///     csv.writer(s, lineterminator="\n").writerow(["a\rb", "x"])
///       -> '"a\rb",x\n'
///
/// The port shipped the source-reading, the harness compared the bytes, and the
/// CSV came out two bytes short. That is the entire argument for a byte
/// differential over a careful reimplementation: a "functionally equivalent"
/// writer agrees on every value and disagrees on the bytes, and only one of
/// those two is what a learner's Anki collection actually consumes.
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

fn write_tsv_body(items: &[Val]) -> Result<String, GateError> {
    let mut out = String::new();
    out.push_str(TSV_COMMENT_1);
    out.push_str(TSV_COMMENT_2);
    for it in items {
        let [stem, answer, explanation, module] = card_fields(it)?;
        // Collapse tabs and newlines so the TSV stays one row per card. `\r` is
        // deliberately NOT collapsed — the oracle does not touch it.
        let flat = |s: String| s.replace('\t', " ").replace('\n', " ");
        out.push_str(&csv_row(
            &[flat(stem), flat(answer), flat(explanation), module],
            '\t',
            "\n",
        ));
    }
    Ok(out)
}

fn write_csv_body(items: &[Val]) -> Result<String, GateError> {
    let mut out = String::new();
    out.push_str(&csv_row(
        &CSV_HEADER.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        ',',
        "\n",
    ));
    for it in items {
        // NOTE: unlike the TSV path, the oracle does NOT flatten newlines here.
        let f = card_fields(it)?;
        out.push_str(&csv_row(&f, ',', "\n"));
    }
    Ok(out)
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

// ── filtering ──────────────────────────────────────────────────────────────

/// `int(x)` for the shapes a `module` field can hold. A shape Python would
/// reject with a traceback becomes an ERROR here.
fn py_int(v: &Val) -> Result<i64, GateError> {
    match v {
        Val::Int(i) => Ok(*i),
        Val::Float(f) => Ok(*f as i64),
        Val::Bool(b) => Ok(*b as i64),
        Val::Str(s) => s.trim().parse::<i64>().map_err(|_| {
            GateError::error(format!(
                "int({s:?}) — the oracle raises ValueError and dies with a traceback here"
            ))
        }),
        other => Err(GateError::error(format!(
            "int() on {other:?} — the oracle raises TypeError and dies with a traceback here"
        ))),
    }
}

fn join_strs(v: Option<&Val>) -> String {
    match v {
        Some(Val::List(items)) => items
            .iter()
            .map(|x| x.py_str())
            .collect::<Vec<_>>()
            .join(" "),
        // Iterating a Python str yields its characters, so `" ".join(str(t) for
        // t in "abc")` is `"a b c"`. Reproduced.
        Some(Val::Str(s)) => s
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

struct Filters<'a> {
    module: Option<i64>,
    tag: Option<&'a str>,
    limit: Option<i64>,
    seed: Option<i64>,
}

fn filter_items(items: &[Val], f: &Filters) -> Result<Vec<Val>, GateError> {
    let mut out: Vec<Val> = items.to_vec();

    if let Some(want) = f.module {
        let mut keep = Vec::new();
        for it in out {
            // `int(it.get("module") or -1)`: a FALSY module (absent, None, 0 or
            // "") becomes -1. Module 0 is therefore unreachable by --module 0.
            let n = match it.get_or_falsy("module") {
                Some(v) => py_int(v)?,
                None => -1,
            };
            if n == want {
                keep.push(it);
            }
        }
        out = keep;
    }

    if let Some(tag) = f.tag {
        let want = tag.to_lowercase();
        let mut keep = Vec::new();
        for it in out {
            let topic_blob = join_strs(it.get_or_falsy("topic_ids")).to_lowercase();
            let tags = match it.get_or_falsy("tags") {
                Some(v @ Val::Str(_)) => Val::List(vec![v.clone()]),
                Some(v) => v.clone(),
                None => Val::List(Vec::new()),
            };
            let tag_blob = join_strs(Some(&tags)).to_lowercase();
            if tag_blob.contains(&want) || topic_blob.contains(&want) {
                keep.push(it);
            }
        }
        out = keep;
    }

    let id_key = |x: &Val| x.get_or_falsy("id").map(|v| v.py_str()).unwrap_or_default();
    out.sort_by(|a, b| id_key(a).cmp(&id_key(b)));

    if let Some(limit) = f.limit {
        if limit > 0 && out.len() > limit as usize {
            match f.seed {
                Some(seed) => {
                    let mut rng = Mt19937::from_python_int_seed(seed);
                    rng.shuffle(&mut out);
                    out.truncate(limit as usize);
                    out.sort_by(|a, b| id_key(a).cmp(&id_key(b)));
                }
                None => out.truncate(limit as usize),
            }
        }
    }
    Ok(out)
}

// ── argument parsing ───────────────────────────────────────────────────────

struct Args {
    source: String,
    out: PathBuf,
    format: String,
    module: Option<i64>,
    tag: Option<String>,
    limit: Option<i64>,
    seed: Option<i64>,
    #[allow(dead_code)]
    deck_name: String,
}

fn parse_args(root: &Path, argv: &[String]) -> Result<Args, GateError> {
    let mut a = Args {
        source: "bank".into(),
        out: root.join(DEFAULT_OUT_REL),
        format: DEFAULT_FORMAT.into(),
        module: None,
        tag: None,
        limit: None,
        seed: None,
        deck_name: DEFAULT_DECK_NAME.into(),
    };
    let mut i = 0;
    while i < argv.len() {
        let raw = &argv[i];
        let (name, inline) = match raw.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n.to_string(), Some(v.to_string())),
            _ => (raw.clone(), None),
        };
        let take = |i: &mut usize| -> Result<String, GateError> {
            if let Some(v) = inline.clone() {
                *i += 1;
                return Ok(v);
            }
            let v = argv
                .get(*i + 1)
                .ok_or_else(|| GateError::usage(format!("{name}: expected one argument")))?
                .clone();
            *i += 2;
            Ok(v)
        };
        match name.as_str() {
            "--source" => {
                let v = take(&mut i)?;
                if !["bank", "seed42", "keys"].contains(&v.as_str()) {
                    return Err(GateError::usage(format!(
                        "--source: invalid choice {v:?} (choose from 'bank', 'seed42', 'keys')"
                    )));
                }
                a.source = v;
            }
            "--out" => {
                let v = take(&mut i)?;
                a.out = PathBuf::from(v);
            }
            "--format" => a.format = take(&mut i)?,
            "--module" => {
                let v = take(&mut i)?;
                a.module = Some(
                    v.parse::<i64>()
                        .map_err(|_| GateError::usage(format!("--module: invalid int {v:?}")))?,
                );
            }
            "--tag" => a.tag = Some(take(&mut i)?),
            "--limit" => {
                let v = take(&mut i)?;
                a.limit = Some(
                    v.parse::<i64>()
                        .map_err(|_| GateError::usage(format!("--limit: invalid int {v:?}")))?,
                );
            }
            "--seed" => {
                let v = take(&mut i)?;
                a.seed = Some(
                    v.parse::<i64>()
                        .map_err(|_| GateError::usage(format!("--seed: invalid int {v:?}")))?,
                );
            }
            "--deck-name" => a.deck_name = take(&mut i)?,
            other => {
                return Err(GateError::usage(format!(
                    "unrecognized argument {other:?}; known: --source --out --format --module \
                     --tag --limit --seed --deck-name"
                )))
            }
        }
    }
    Ok(a)
}

// ── the gate ───────────────────────────────────────────────────────────────

/// Everything the run decided, before anything is written. Holding the verdict
/// and the bytes apart is what makes "a RED run writes nothing" checkable rather
/// than hoped for.
#[derive(Debug)]
pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub files: Vec<(PathBuf, String)>,
}

fn is_drawable(it: &Val) -> bool {
    let s = it.get("status").map(|v| v.py_str()).unwrap_or_default();
    let s = s.trim().to_ascii_lowercase();
    s != "retired" && s != "draft"
}

pub fn evaluate(root: &Path, argv: &[String]) -> Result<Outcome, GateError> {
    let args = parse_args(root, argv)?;

    let fail = |msg: &str| Outcome {
        code: 1,
        stdout: String::new(),
        stderr: format!("{msg}\n"),
        files: Vec::new(),
    };

    let (mut items, stem, stderr_pre) = match args.source.as_str() {
        "bank" => (load_bank_items(root)?, STEM_BANK.to_string(), String::new()),
        "seed42" => match load_seed42_bank_items(root)? {
            Some(v) => (v, STEM_SEED42.to_string(), String::new()),
            None => (
                load_bank_items(root)?,
                STEM_BANK.to_string(),
                "WARN: bank_items_seed42.json missing — falling back to bank\n".to_string(),
            ),
        },
        _ => match load_keys_seed42_pack(root)? {
            Some(v) => (v, STEM_KEYS.to_string(), String::new()),
            None => return Ok(fail("FAIL: keys/seed42 packs not found")),
        },
    };

    let scanned = items.len();
    if scanned == 0 {
        let mut o = fail("FAIL: zero items to export");
        o.stderr = format!("{stderr_pre}{}", o.stderr);
        return Ok(o);
    }

    // Bank/seed42 carry status. keys/mock40 is already the approved draw.
    if args.source == "bank" || args.source == "seed42" {
        items.retain(is_drawable);
        if items.is_empty() {
            let mut o = fail("FAIL: zero approved items to export");
            o.stderr = format!("{stderr_pre}{}", o.stderr);
            return Ok(o);
        }
    }

    items = filter_items(
        &items,
        &Filters {
            module: args.module,
            tag: args.tag.as_deref(),
            limit: args.limit,
            seed: args.seed,
        },
    )?;
    if items.is_empty() {
        let mut o = fail("FAIL: filter removed all items");
        o.stderr = format!("{stderr_pre}{}", o.stderr);
        return Ok(o);
    }

    // `{f.strip().lower() for f in args.format.split(",") if f.strip()}` — a SET,
    // so duplicates collapse and order is irrelevant.
    let mut formats: Vec<String> = Vec::new();
    for f in args.format.split(',') {
        let f = f.trim();
        if f.is_empty() {
            continue;
        }
        let f = f.to_lowercase();
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
        let listed = unknown
            .iter()
            .map(|s| py_repr_str(s))
            .collect::<Vec<_>>()
            .join(", ");
        let mut o = fail(&format!("FAIL: unknown format(s): [{listed}]"));
        o.stderr = format!("{stderr_pre}{}", o.stderr);
        return Ok(o);
    }
    if formats.is_empty() {
        let mut o = fail("FAIL: no formats requested");
        o.stderr = format!("{stderr_pre}{}", o.stderr);
        return Ok(o);
    }

    // ── the blocked leg ────────────────────────────────────────────────────
    // Placed AFTER every check the oracle performs first, so each of those
    // cases stays byte-identical, and BEFORE any write, so the refusal leaves
    // nothing behind.
    if formats.iter().any(|f| f == "apkg") {
        return Err(GateError::error(
            "the .apkg leg of this port is BLOCKED, not skipped. Measured 2026-08-14, two runs of \
             scripts/export_anki.py two seconds apart on byte-identical inputs produced .apkg \
             files differing in 209382 of 212731 bytes (98.4%): the deck embeds int(time.time()) \
             in col.crt/col.mod/col.scm and every notes.mod, and the DEFLATE stream avalanches. A \
             byte-exact differential against that oracle is UNSATISFIABLE, and shipping an \
             unverified hand-rolled SQLite writer to learners is worse than not porting it. Make \
             the ORACLE reproducible first (bd-anki-apkg-not-reproducible-e13a), then this leg \
             lands against a real oracle. Until then run `python3 scripts/export_anki.py` for the \
             deck; `--format tsv` and `--format csv` are ported and byte-verified."
                .to_string(),
        ));
    }

    let out_dir = args.out.clone();
    let mut files: Vec<(PathBuf, String)> = Vec::new();
    let mut written: Vec<String> = Vec::new();

    // `str(p)` — the oracle prints the path exactly as `--out` gave it, so a
    // relative --out prints relative and the default prints the absolute path
    // derived from the script's own location.
    let shown = |p: &Path| p.to_string_lossy().into_owned();

    if formats.iter().any(|f| f == "tsv") {
        let p = out_dir.join(format!("{stem}.tsv"));
        files.push((p.clone(), write_tsv_body(&items)?));
        written.push(shown(&p));
    }
    if formats.iter().any(|f| f == "csv") {
        let p = out_dir.join(format!("{stem}.csv"));
        files.push((p.clone(), write_csv_body(&items)?));
        written.push(shown(&p));
    }
    let note = out_dir.join("README.txt");
    files.push((note.clone(), readme_body(items.len(), &args.source)));
    written.push(shown(&note));

    let mut stdout = String::new();
    stdout.push_str("export_anki ok\n");
    stdout.push_str(&format!("  cards={}\n", items.len()));
    stdout.push_str(&format!("  {scanned} scanned, {} exported\n", items.len()));
    stdout.push_str(&format!("  source={}\n", args.source));
    for w in &written {
        stdout.push_str(&format!("  wrote {w}\n"));
    }

    Ok(Outcome {
        code: 0,
        stdout,
        // The `--source seed42` fallback WARN is the only thing the oracle ever
        // writes to stderr on a green run, and it is written BEFORE the report.
        stderr: stderr_pre,
        files,
    })
}

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    // The Python resolves its own location, so every path it derives is
    // symlink-free. Do the same to the engine root.
    let root = ctx.root.canonicalize().unwrap_or_else(|_| ctx.root.clone());
    let outcome = evaluate(&root, &ctx.args)?;

    // A RED outcome carries NO files. The oracle returns before every write on
    // every failure path; this is the belt to that braces, and it is asserted
    // rather than assumed — a contradiction must not be resolvable by writing.
    debug_assert!(
        outcome.code == 0 || outcome.files.is_empty(),
        "a failing export must not carry files"
    );

    if outcome.code == 0 {
        for (path, body) in &outcome.files {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| GateError::error(format!("mkdir {}: {e}", parent.display())))?;
            }
            std::fs::write(path, body.as_bytes())
                .map_err(|e| GateError::error(format!("write {}: {e}", path.display())))?;
        }
    }

    print!("{}", outcome.stdout);
    let _ = std::io::stdout().flush();
    eprint!("{}", outcome.stderr);
    let _ = std::io::stderr().flush();
    if outcome.code != 0 {
        // See the module header: the oracle writes `FAIL: …` to stderr and exits
        // 1. Byte-identical output including the exit code is the acceptance
        // bar, and `GateError` cannot express exit 1.
        std::process::exit(outcome.code);
    }
    Ok(())
}

// ── unit tests (the differential is the acceptance bar; these pin the pieces
//    the differential can only reach indirectly) ────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn item(pairs: &[(&str, Val)]) -> Val {
        Val::Map(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        )
    }

    #[test]
    fn mt19937_matches_cpython_for_seed_42() {
        // `random.Random(42)` then `getrandbits(32)` three times, taken from
        // CPython 3. If this drifts, `--limit --seed` silently samples a
        // DIFFERENT deck while every other byte matches.
        let mut r = Mt19937::from_python_int_seed(42);
        assert_eq!(r.getrandbits(32), 2_746_317_213);
        assert_eq!(r.getrandbits(32), 478_163_327);
        assert_eq!(r.getrandbits(32), 107_420_369);
    }

    #[test]
    fn shuffle_matches_cpython_for_seed_42() {
        // `x = list(range(10)); random.Random(42).shuffle(x)` in CPython 3.
        let mut x: Vec<u32> = (0..10).collect();
        Mt19937::from_python_int_seed(42).shuffle(&mut x);
        assert_eq!(x, vec![7, 3, 2, 8, 5, 6, 9, 4, 0, 1]);
    }

    #[test]
    fn csv_quoting_follows_the_python_dialect_measured_not_inferred() {
        // A bare CR DOES force quoting — measured against CPython, not inferred
        // from _csv.c. This assertion held the opposite until the differential
        // proved it wrong; see the doc comment on `csv_row`.
        assert_eq!(csv_row(&["a\rb".into()], '\t', "\n"), "\"a\rb\"\n");
        assert_eq!(csv_row(&["a\rb".into()], ',', "\n"), "\"a\rb\"\n");
        // A tab does, in the tab dialect.
        assert_eq!(csv_row(&["a\tb".into()], '\t', "\n"), "\"a\tb\"\n");
        // A quote is doubled and the field quoted.
        assert_eq!(csv_row(&["a\"b".into()], ',', "\n"), "\"a\"\"b\"\n");
        // A newline forces quoting in both dialects.
        assert_eq!(csv_row(&["a\nb".into()], ',', "\n"), "\"a\nb\"\n");
        // A comma is inert in the tab dialect.
        assert_eq!(csv_row(&["a,b".into()], '\t', "\n"), "a,b\n");
    }

    #[test]
    fn format_answer_reproduces_the_substring_defect_rather_than_fixing_it() {
        // The GOOD path.
        let ok = item(&[
            ("correct", Val::Str("b".into())),
            (
                "choices",
                Val::List(vec![
                    Val::Str("one".into()),
                    Val::Str("two".into()),
                    Val::Str("three".into()),
                ]),
            ),
        ]);
        assert_eq!(format_answer(&ok).unwrap(), "B) two");

        // A MISSING key is a substring of "ABCD", so the oracle reaches ord("")
        // and raises TypeError. Reproduced as an ERROR, not silently repaired.
        let missing = item(&[("choices", Val::List(vec![Val::Str("one".into())]))]);
        assert!(
            format_answer(&missing).is_err(),
            "an absent answer key must not quietly produce a card"
        );

        // Two letters: also a substring, also a crash in the oracle.
        let two = item(&[
            ("correct", Val::Str("AB".into())),
            ("choices", Val::List(vec![Val::Str("one".into())])),
        ]);
        assert!(format_answer(&two).is_err());

        // NOT a substring: falls through to the literal, no crash.
        let e = item(&[
            ("correct", Val::Str("E".into())),
            ("choices", Val::List(vec![Val::Str("one".into())])),
        ]);
        assert_eq!(format_answer(&e).unwrap(), "E");

        // A letter past the end of `choices` also falls through.
        let past = item(&[
            ("correct", Val::Str("D".into())),
            ("choices", Val::List(vec![Val::Str("one".into())])),
        ]);
        assert_eq!(format_answer(&past).unwrap(), "D");
    }

    #[test]
    fn module_zero_is_unreachable_by_the_module_filter() {
        // `int(it.get("module") or -1)` maps a falsy 0 to -1. Reproduced.
        let items = vec![item(&[
            ("id", Val::Str("x".into())),
            ("module", Val::Int(0)),
        ])];
        let f = Filters {
            module: Some(0),
            tag: None,
            limit: None,
            seed: None,
        };
        assert!(
            filter_items(&items, &f).unwrap().is_empty(),
            "the oracle cannot select module 0; a port that could would diverge"
        );
    }

    #[test]
    fn a_falsy_module_renders_as_empty_but_zero_renders_as_zero() {
        // `card_fields` uses `is not None`, not truthiness, so 0 keeps its "0".
        let zero = item(&[("module", Val::Int(0)), ("correct", Val::Str("A".into()))]);
        assert_eq!(card_fields(&zero).unwrap()[3], "0");
        let none = item(&[("module", Val::Null), ("correct", Val::Str("A".into()))]);
        assert_eq!(card_fields(&none).unwrap()[3], "");
        let absent = item(&[("correct", Val::Str("A".into()))]);
        assert_eq!(card_fields(&absent).unwrap()[3], "");
    }

    #[test]
    fn unknown_formats_are_reported_as_a_python_sorted_list_repr() {
        assert_eq!(py_repr_str("xyz"), "'xyz'");
        assert_eq!(py_repr_str("it's"), "\"it's\"");
    }

    #[test]
    fn json_reader_handles_the_shapes_the_packs_use() {
        let v = JsonReader::new(br#"{"items":[{"id":"a","module":6,"choices":["x","y"]}]}"#)
            .parse()
            .unwrap();
        let items = v.get("items").unwrap();
        let Val::List(rows) = items else {
            panic!("items")
        };
        assert_eq!(rows[0].get("id"), Some(&Val::Str("a".into())));
        assert_eq!(rows[0].get("module"), Some(&Val::Int(6)));
        // \u escapes and surrogate pairs round-trip.
        let s = JsonReader::new(r#""a—b😀""#.as_bytes()).parse().unwrap();
        assert_eq!(s, Val::Str("a—b😀".into()));
    }

    #[test]
    fn an_empty_bank_directory_is_never_a_green_export() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
        let o = evaluate(td.path(), &["--format".into(), "tsv".into()]).unwrap();
        assert_eq!(o.code, 1, "a zero-card deck must never exit 0");
        assert_eq!(o.stderr, "FAIL: zero items to export\n");
        assert!(o.files.is_empty(), "a failing export writes nothing");
        assert!(!o.stdout.contains("ok"), "no success token on a RED run");
    }

    #[test]
    fn a_missing_bank_directory_is_never_a_green_export() {
        let td = tempfile::tempdir().unwrap();
        let o = evaluate(td.path(), &["--format".into(), "tsv".into()]).unwrap();
        assert_eq!(o.code, 1);
        assert_eq!(o.stderr, "FAIL: zero items to export\n");
        assert!(o.files.is_empty());
    }

    #[test]
    fn an_all_retired_bank_is_error_and_writes_nothing() {
        // Distinct from "zero items": files were scanned, the approved pool is empty.
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
        std::fs::write(
            td.path().join(ITEMS_DIR_REL).join("r.toml"),
            "id = \"r\"\nstatus = \"retired\"\nstem = \"gone\"\ncorrect = \"A\"\nchoices = [\"x\"]\n",
        )
        .unwrap();
        let o = evaluate(td.path(), &["--format".into(), "tsv".into()]).unwrap();
        assert_eq!(o.code, 1);
        assert_eq!(o.stderr, "FAIL: zero approved items to export\n");
        assert!(o.files.is_empty(), "must not write a retired deck");
        assert!(!o.stdout.contains("ok"));
    }

    #[test]
    fn the_apkg_leg_refuses_loudly_instead_of_writing_an_unverified_deck() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join(ITEMS_DIR_REL)).unwrap();
        std::fs::write(
            td.path().join(ITEMS_DIR_REL).join("a.toml"),
            "id = \"a\"\nstem = \"s\"\ncorrect = \"A\"\nchoices = [\"x\"]\n",
        )
        .unwrap();
        let e = evaluate(td.path(), &[]).unwrap_err();
        let msg = format!("{e}");
        assert!(msg.contains("bd-anki-apkg-not-reproducible-e13a"), "{msg}");
        assert!(
            !td.path().join(DEFAULT_OUT_REL).exists(),
            "the refusal must leave no output directory behind"
        );
    }
}
