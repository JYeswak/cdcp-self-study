//! goldens-couplings — B2 of milestone B (bd-hardening-b-ledgers-gvm.2).
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This gate raises exactly one floor: **a frozen artifact in this repo names
//! the semantic surfaces it was frozen against, and neither side can move
//! without the other being re-stated in the same change.**
//!
//! Concretely, `registries/goldens-couplings.toml` holds two kinds of row:
//!
//! * `[[surface]]` — a named semantic surface, with one or more PINS that are
//!   re-extracted on every run: a `const` literal, a `struct` field list, the
//!   object-key set of a serialised JSON artifact (`keys` — a property of the
//!   OUTPUT, never a syntax scrape of the emitter), a sha256 over a normalised
//!   named source region, or the resolved version of a locked dependency. A pin
//!   that no longer matches the tree is RED, naming the registry side and the
//!   source (or artifact) side.
//! * `[[golden]]` — a frozen artifact, pinned by the sha256 of its bytes, with a
//!   non-empty `depends_on` list naming surfaces AT A VERSION, and a written
//!   justification.
//!
//! The two are chained so the re-freeze cannot be silent:
//!
//! 1. Source moves ⇒ the surface's pin is RED (registry value vs source value).
//! 2. The author repairs the pin ⇒ the surface's `version` is RED, because
//!    `version` is DERIVED from the pin block (`derive_version`) and is not a
//!    label anyone can decline to bump.
//! 3. The author bumps the version ⇒ every golden that still affirms the old
//!    version is RED, and must be re-frozen and re-affirmed by hand.
//! 4. A golden re-frozen by `UPDATE_GOLDENS` without any of that is RED on its
//!    own `frozen` digest, which is the hole this bead exists to close.
//!
//! # WHAT THIS GATE CANNOT DECIDE — read this before quoting it
//!
//! It **cannot decide that a golden is CORRECT.** A digest frozen over a wrong
//! answer key is pinned exactly as firmly as a right one. Every comparator here
//! is internal; nothing outside this project is consulted.
//!
//! It **cannot decide that a justification is honest**, or that a
//! re-affirmation reflects a human who actually re-read anything. `affirmed` is
//! a date somebody typed; a re-freeze that updates the digests and the dates
//! without a thought passes. What changed is that the thought has a place to
//! happen and leaves a diff: the affirmation is a line in a review, not an
//! absence.
//!
//! It **cannot decide that the declared surfaces are the RIGHT surfaces.** If a
//! golden depends on something nobody registered, this gate is silent about it
//! and will stay green while that surface moves. The registry is an inventory
//! someone maintains, and an inventory is only as complete as its author.
//!
//! It **cannot decide what a source change MEANS.** `region` pins compare bytes
//! after dropping whole-line comments: a rename, a refactor with identical
//! behaviour, and a semantics change all read the same and all demand a
//! deliberate re-affirmation. That is a cost, chosen on purpose — the failure
//! being cured is a semantics change that read as nothing at all.
//!
//! `keys` is the one kind that does **not** scrape source. It was retired from
//! `"...".into()` / `json!` literal collection (bd-extract-key-literals-overmatch-4pak)
//! because that heuristic had both failure directions and taught authors to
//! route around it: an error-message `.into()` entered the pin (false
//! positive), and a key emitted via a const / `format!` / helper was silently
//! absent (false negative — the fooled certificate). The pin is now the
//! object-key set of the serialised artifact. A key the deriver cannot see
//! cannot exist in the artifact, so it cannot be omitted; a non-key string
//! cannot exist as an object key, so it cannot enter. An empty key set is an
//! ERROR.
//!
//! The same asymmetry on the other kinds, reported not papered over:
//! `fields` misses macro-generated and `#[serde(flatten)]` fields (fail open);
//! `const` records the assigned token stream, not a computed value; `region`
//! is source-identity of that item only — a helper *outside* the region can
//! change behaviour without moving the digest; `lockdep` reads `Cargo.lock`
//! (an artifact) and does not have this hole. `toml_array_contains` lives on
//! the doc-facts gate and fail-opens on a dynamically built array.
//!
//! It **cannot decide that the goldens are actually consulted by anything.**
//! Whether `scripts/check.sh` still runs `goldens check` is settled by the
//! wiring, not by this gate.
//!
//! The floor moves from *a golden is a file somebody regenerated* to *a golden
//! is a file pinned by digest, attributed to named surfaces at derived
//! versions, each of which is re-checked against source on every run*.
//!
//! # ANTI-VACUOUS (L4)
//!
//! Zero surfaces, zero goldens, zero pins, or zero couplings is an ERROR. Zero
//! golden FILES discovered under `goldens/` is an ERROR — a scan that found
//! nothing must never report the way a scan that checked everything does. A
//! discovered file with no row is RED, so a new golden cannot arrive uncovered,
//! and `REQUIRED_GOLDENS` is compiled in so deleting a row is not a way to pass.

#![forbid(unsafe_code)]

use crate::date::{self, Ymd};
use crate::gates::verify_content_lock::{sha256_file, sha256_hex_bytes};
use crate::registry::{GateCtx, GateError};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const NAME: &str = "goldens-couplings";
pub const SUMMARY: &str =
    "every golden names the semantic surfaces it was frozen against, and neither side moves alone";

/// Where the ledger lives, relative to the engine root.
pub const REGISTRY_PATH: &str = "registries/goldens-couplings.toml";

/// The directory whose contents are DISCOVERED. Anything found here that no row
/// declares is a finding: a golden must not be able to arrive uncovered.
pub const GOLDENS_DIR: &str = "goldens";

/// Extensions skipped by discovery. Prose next to the artifacts is not a golden.
pub const DISCOVERY_SKIP_EXT: &[&str] = &["md"];

/// Rows whose absence is an ERROR. Deleting the row for a golden that check.sh
/// pins is not a way to pass this gate.
pub const REQUIRED_GOLDENS: &[&str] = &[
    "goldens/fixtures/mock40_seed42.json",
    "goldens/mock40_seed42_all_correct.sha256",
    "goldens/mock40_seed42_all_wrong.sha256",
    "goldens/bank_hash.txt",
];

/// The pin kinds a surface may cite.
pub const KINDS: &[&str] = &["const", "fields", "keys", "region", "lockdep"];

/// A justification shorter than this says nothing a reviewer can disagree with.
pub const MIN_JUSTIFICATION_LEN: usize = 60;

/// The longest affirmation window the registry may configure. It may tighten
/// this; widening it would retire the expiry with a one-number diff.
pub const MAX_AFFIRMATION_DAYS: i64 = 365;

/// At least this many goldens must tie their path to a source constant. Zero
/// would let every path in this file drift from the code that opens it.
pub const MIN_PATH_CONSTS: usize = 1;

/// Derived-version shape: `v` plus this many hex characters of the pin digest.
pub const VERSION_HEX_LEN: usize = 12;

/// A sha256 hex digest is exactly this long.
pub const DIGEST_LEN: usize = 64;

const KNOWN_FLAGS: &[&str] = &["--quiet"];

// ── registry schema ────────────────────────────────────────────────────────

/// Every field is `#[serde(default)]` so a MISSING field arrives as an empty
/// value and is reported as the schema error it is, rather than as an opaque
/// TOML parse failure that names a line and not a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Ledger {
    pub schema_version: u32,
    #[serde(default)]
    pub policy: Policy,
    #[serde(default)]
    pub surface: Vec<Surface>,
    #[serde(default)]
    pub golden: Vec<Golden>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Policy {
    /// Absent lands as 0, which fails the floor check below. Blank is never
    /// permissive: a missing window must not read as "never expires".
    #[serde(default)]
    pub affirmation_days: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Surface {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    /// DERIVED from `pin` — see `derive_version`. Recorded here so a golden can
    /// quote it, and checked so it cannot lag the thing it names.
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub justification: String,
    #[serde(default)]
    pub pin: Vec<Pin>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Pin {
    #[serde(default)]
    pub kind: String,
    /// Engine-root-relative file the pin is read out of.
    #[serde(default)]
    pub file: String,
    /// The item to read: a Rust item name, a multi-word item head such as
    /// `impl Default for AssembleConfig`, or a package name for `lockdep`.
    #[serde(default)]
    pub symbol: String,
    /// `lockdep` only: the crate whose dependency edge selects the version.
    #[serde(default)]
    pub via: String,
    /// `const`, `fields`, `keys`, `lockdep`: the expected extraction, in order.
    #[serde(default)]
    pub expect: Vec<String>,
    /// `region` only: sha256 of the normalised source region.
    #[serde(default)]
    pub digest: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Golden {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file: String,
    /// Optional `<path>::<CONST>` whose literal must equal `file`, so the path
    /// this row pins and the path the code opens cannot drift apart.
    #[serde(default, rename = "const")]
    pub path_const: String,
    /// sha256 of the golden's bytes. This is the re-freeze tripwire.
    #[serde(default)]
    pub frozen: String,
    #[serde(default)]
    pub affirmed: String,
    #[serde(default)]
    pub justification: String,
    #[serde(default)]
    pub depends_on: Vec<Dep>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Dep {
    #[serde(default)]
    pub surface: String,
    #[serde(default)]
    pub version: String,
}

// ── civil-date arithmetic ──────────────────────────────────────────────────
// `crate::date` answers "what is today" and "is this before that"; expiry needs
// an AGE IN DAYS. Howard Hinnant's `days_from_civil`, the exact inverse of the
// one `crate::date` uses.

/// Days since 1970-01-01 for a proleptic-Gregorian civil date.
pub fn days_from_civil((y, m, d): Ymd) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Whole days from `from` to `to`; negative when `to` precedes `from`.
pub fn days_between(from: Ymd, to: Ymd) -> i64 {
    days_from_civil(to) - days_from_civil(from)
}

// ── the derived version ────────────────────────────────────────────────────

/// The version a surface MUST record, derived from its own pin block.
///
/// A hand-written label can be left alone while the thing it names moves — the
/// same defect as a maturity claim in a prose table. Deriving it means the only
/// way to keep a version stable is to leave the pins alone.
pub fn derive_version(s: &Surface) -> String {
    let mut buf = String::new();
    for p in &s.pin {
        buf.push_str(p.kind.trim());
        buf.push('|');
        buf.push_str(p.file.trim());
        buf.push('|');
        buf.push_str(p.symbol.trim());
        buf.push('|');
        buf.push_str(p.via.trim());
        buf.push('|');
        buf.push_str(p.digest.trim());
        buf.push('|');
        for e in &p.expect {
            buf.push_str(e.trim());
            buf.push(',');
        }
        buf.push('\n');
    }
    let hex = sha256_hex_bytes(buf.as_bytes());
    format!("v{}", &hex[..VERSION_HEX_LEN])
}

// ── source reading ─────────────────────────────────────────────────────────

/// Is this a normalised engine-root-relative path? A ref that can climb out of
/// the tree, or name an absolute location, is refused rather than resolved.
pub fn is_clean_relative_path(p: &str) -> bool {
    let p = p.trim();
    if p.is_empty() || p.starts_with('/') || p.contains('\\') {
        return false;
    }
    !p.split('/').any(|c| c.is_empty() || c == "." || c == "..")
}

/// A 64-character lowercase hex digest.
pub fn looks_like_digest(s: &str) -> bool {
    let s = s.trim();
    s.len() == DIGEST_LEN
        && s.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

/// The Rust item heads a single-word `symbol` may be introduced by.
const HEADS: &[&str] = &[
    "fn ", "struct ", "enum ", "const ", "static ", "type ", "trait ", "union ",
];

/// The byte range of the item `symbol` introduces in `text`, or `None`.
///
/// A `symbol` containing a space is matched as a literal item head (so
/// `impl Default for AssembleConfig` can be pinned); otherwise the symbol must
/// appear immediately after one of `HEADS` on a line that is not a comment.
/// The item ends at its balanced closing brace, or at the first `;` when it has
/// no body — computed by a scanner that skips strings, chars and comments, so a
/// `"{"` inside a literal does not move the end.
pub fn find_region(text: &str, symbol: &str) -> Option<(usize, usize)> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return None;
    }
    let bytes = text.as_bytes();
    let mut offset = 0usize;
    for line in text.lines() {
        let line_start = offset;
        offset += line.len() + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        let indent = line.len() - trimmed.len();
        let start = if symbol.contains(' ') {
            if trimmed.starts_with(symbol) {
                Some(line_start + indent)
            } else {
                None
            }
        } else {
            head_match(trimmed, symbol).map(|at| line_start + indent + at)
        };
        if let Some(start) = start {
            let end = item_end(bytes, start)?;
            return Some((start, end));
        }
    }
    None
}

/// Byte offset within `trimmed` where an item head introducing `symbol` starts.
fn head_match(trimmed: &str, symbol: &str) -> Option<usize> {
    for head in HEADS {
        let mut from = 0usize;
        while let Some(rel) = trimmed[from..].find(head) {
            let at = from + rel;
            from = at + head.len();
            // The head keyword must be its own token (`defn ` is not `fn `).
            if at > 0 {
                let prev = trimmed.as_bytes()[at - 1] as char;
                if prev.is_ascii_alphanumeric() || prev == '_' {
                    continue;
                }
            }
            let rest = &trimmed[at + head.len()..];
            if !rest.starts_with(symbol) {
                continue;
            }
            let after = rest[symbol.len()..].chars().next();
            if after.is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_')) {
                return Some(at);
            }
        }
    }
    None
}

/// One byte past the end of the item beginning at `start`.
fn item_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    let mut depth = 0usize;
    let mut seen_brace = false;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            b'r' if raw_string_hashes(bytes, i).is_some() => {
                i = skip_raw_string(bytes, i)?;
            }
            b'"' => i = skip_string(bytes, i)?,
            b'\'' => i = skip_char_or_lifetime(bytes, i),
            b'{' => {
                depth += 1;
                seen_brace = true;
                i += 1;
            }
            b'}' => {
                depth = depth.checked_sub(1)?;
                i += 1;
                if seen_brace && depth == 0 {
                    return Some(i);
                }
            }
            b';' if depth == 0 => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

/// `Some(n)` when a raw string with `n` hashes starts at `i`.
fn raw_string_hashes(bytes: &[u8], i: usize) -> Option<usize> {
    if bytes[i] != b'r' {
        return None;
    }
    let mut n = 0usize;
    let mut j = i + 1;
    while bytes.get(j) == Some(&b'#') {
        n += 1;
        j += 1;
    }
    if bytes.get(j) == Some(&b'"') {
        Some(n)
    } else {
        None
    }
}

fn skip_raw_string(bytes: &[u8], i: usize) -> Option<usize> {
    let n = raw_string_hashes(bytes, i)?;
    let mut j = i + 1 + n + 1;
    while j < bytes.len() {
        if bytes[j] == b'"' {
            let mut k = 0usize;
            while k < n && bytes.get(j + 1 + k) == Some(&b'#') {
                k += 1;
            }
            if k == n {
                return Some(j + 1 + n);
            }
        }
        j += 1;
    }
    None
}

fn skip_string(bytes: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    while j < bytes.len() {
        match bytes[j] {
            b'\\' => j += 2,
            b'"' => return Some(j + 1),
            _ => j += 1,
        }
    }
    None
}

/// `'a` (a lifetime) advances one byte; `'x'` and `'\n'` advance past the
/// closing quote.
fn skip_char_or_lifetime(bytes: &[u8], i: usize) -> usize {
    if bytes.get(i + 1) == Some(&b'\\') {
        let mut j = i + 2;
        while j < bytes.len() && j < i + 10 {
            if bytes[j] == b'\'' {
                return j + 1;
            }
            j += 1;
        }
        return i + 1;
    }
    if bytes.get(i + 2) == Some(&b'\'') {
        return i + 3;
    }
    i + 1
}

/// The digested form of a source region: whole-line comments dropped, trailing
/// whitespace trimmed, blank lines dropped. A doc edit must not force every
/// golden downstream to be re-frozen; a code edit must.
pub fn normalise_region(region: &str) -> String {
    let mut out = String::new();
    for line in region.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") {
            continue;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// Field names declared in a struct region, in declaration order.
pub fn extract_fields(region: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in region.lines().skip(1) {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || t.starts_with("#[") {
            continue;
        }
        let t = t.strip_prefix("pub ").unwrap_or(t);
        let Some((name, _)) = t.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() || name.contains(' ') || name.contains('(') {
            continue;
        }
        if name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '#')
        {
            out.push(name.trim_start_matches("r#").to_string());
        }
    }
    out
}

/// Is `symbol` a JSON selector a `keys` pin may carry: `$` (the whole
/// document) or a JSON pointer starting with `/`.
pub fn is_json_selector(symbol: &str) -> bool {
    let s = symbol.trim();
    s == "$" || s.starts_with('/')
}

/// Object keys of a serialised JSON artifact, unique, in first-seen preorder.
///
/// This is what a `keys` pin is derived FROM. It is a property of the
/// artifact, not of the emitter's syntax: a `"...".into()` in an error path
/// cannot enter (it is not an object key), and a key produced by a helper,
/// a `const`, or a `format!` cannot be omitted (it is in the JSON if it was
/// emitted). `serde_json::Map` is a `BTreeMap`, so each object's own keys
/// are visited in sorted order; first-seen across the walk is therefore
/// deterministic even when the file was written with a different key order.
///
/// `selector` is `$` (the whole document) or a JSON pointer. The walk starts
/// at the selected node and recurses through every object and array beneath
/// it. An empty key set is returned as `Ok([])` so the caller can treat it
/// as the anti-vacuous ERROR it is, distinct from a parse failure.
pub fn extract_json_keys(text: &str, selector: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| {
        format!(
            "not JSON ({e}) — a `keys` pin is derived from serialised output, not from source text"
        )
    })?;
    let node = json_select(&value, selector)?;
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    walk_json_keys(node, &mut out, &mut seen);
    Ok(out)
}

/// Resolve `$` or a JSON pointer against `root`. `~0` / `~1` unescaping is
/// applied; a token that does not exist is an error, never a silent empty.
pub fn json_select<'a>(
    root: &'a serde_json::Value,
    selector: &str,
) -> Result<&'a serde_json::Value, String> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err("empty JSON selector — a `keys` pin must name `$` or a JSON pointer".into());
    }
    if selector == "$" {
        return Ok(root);
    }
    if !selector.starts_with('/') {
        return Err(format!(
            "JSON selector {selector:?} is neither `$` nor a JSON pointer starting with `/`"
        ));
    }
    let mut cur = root;
    for raw in selector[1..].split('/') {
        let token = raw.replace("~1", "/").replace("~0", "~");
        cur = match cur {
            serde_json::Value::Object(map) => map
                .get(&token)
                .ok_or_else(|| format!("JSON pointer {selector:?} has no object key {token:?}"))?,
            serde_json::Value::Array(arr) => {
                let i: usize = token.parse().map_err(|_| {
                    format!(
                        "JSON pointer {selector:?} indexes an array with {token:?}, which is not a position"
                    )
                })?;
                arr.get(i).ok_or_else(|| {
                    format!(
                        "JSON pointer {selector:?} indexes array position {i}, which does not exist (len {})",
                        arr.len()
                    )
                })?
            }
            _ => {
                return Err(format!(
                    "JSON pointer {selector:?} walked into a non-container at token {token:?}"
                ))
            }
        };
    }
    Ok(cur)
}

fn walk_json_keys(v: &serde_json::Value, out: &mut Vec<String>, seen: &mut BTreeSet<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, child) in map {
                if seen.insert(k.clone()) {
                    out.push(k.clone());
                }
                walk_json_keys(child, out, seen);
            }
        }
        serde_json::Value::Array(xs) => {
            for x in xs {
                walk_json_keys(x, out, seen);
            }
        }
        _ => {}
    }
}

/// The literal a `const`/`static` region assigns, whitespace collapsed.
pub fn extract_const_value(region: &str) -> Option<String> {
    let bytes = region.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => i = skip_string(bytes, i)?,
            b'=' if bytes.get(i + 1) != Some(&b'=') => {
                let tail = region[i + 1..].trim();
                let tail = tail.strip_suffix(';').unwrap_or(tail);
                return Some(collapse_ws(tail));
            }
            _ => i += 1,
        }
    }
    None
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ── Cargo.lock reading ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct Lockfile {
    #[serde(default)]
    package: Vec<LockPkg>,
}

#[derive(Debug, Clone, Deserialize)]
struct LockPkg {
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    dependencies: Vec<String>,
}

/// The version of `package` that `via` resolves to, read out of a Cargo.lock.
///
/// Cargo writes the version into the edge only when the name is ambiguous, so
/// the unambiguous case falls back to the single `[[package]]` of that name —
/// and two candidates with no disambiguating edge is an error, never a guess.
pub fn lock_resolved_version(lock_text: &str, via: &str, package: &str) -> Result<String, String> {
    let lock: Lockfile = toml::from_str(lock_text).map_err(|e| format!("parse Cargo.lock: {e}"))?;
    if lock.package.is_empty() {
        return Err("Cargo.lock lists zero packages — a vacuous lock read is an error".into());
    }
    let vias: Vec<&LockPkg> = lock.package.iter().filter(|p| p.name == via).collect();
    if vias.len() != 1 {
        return Err(format!(
            "Cargo.lock names {} package(s) called {via:?}; exactly one is needed to resolve {package:?}",
            vias.len()
        ));
    }
    let edges: Vec<&String> = vias[0]
        .dependencies
        .iter()
        .filter(|d| d.split_whitespace().next() == Some(package))
        .collect();
    if edges.len() != 1 {
        return Err(format!(
            "{via} has {} dependency edge(s) on {package:?} in Cargo.lock",
            edges.len()
        ));
    }
    if let Some(v) = edges[0].split_whitespace().nth(1) {
        return Ok(v.to_string());
    }
    let named: Vec<&LockPkg> = lock.package.iter().filter(|p| p.name == package).collect();
    match named.len() {
        1 => Ok(named[0].version.clone()),
        n => Err(format!(
            "{via}'s edge on {package:?} carries no version and Cargo.lock holds {n} package(s) of that name — which one the build links is not decidable here"
        )),
    }
}

// ── pure schema validation ─────────────────────────────────────────────────

pub fn parse_ledger(text: &str) -> Result<Ledger, String> {
    let l: Ledger = toml::from_str(text).map_err(|e| format!("parse {REGISTRY_PATH}: {e}"))?;
    if l.schema_version != 1 {
        return Err(format!(
            "{REGISTRY_PATH}: schema_version {} unsupported (expected 1)",
            l.schema_version
        ));
    }
    Ok(l)
}

/// The registry configures its own affirmation window, so it may only TIGHTEN
/// the compiled-in ceiling.
pub fn check_policy(p: &Policy) -> Vec<String> {
    let mut v = Vec::new();
    if p.affirmation_days <= 0 {
        v.push(format!(
            "{REGISTRY_PATH}: [policy].affirmation_days is {} — a window that cannot expire is no window; blank and zero are never permissive",
            p.affirmation_days
        ));
    } else if p.affirmation_days > MAX_AFFIRMATION_DAYS {
        v.push(format!(
            "{REGISTRY_PATH}: [policy].affirmation_days {} exceeds the compiled-in ceiling of {MAX_AFFIRMATION_DAYS} — the registry may tighten the window, never widen it",
            p.affirmation_days
        ));
    }
    v
}

/// Everything decidable without touching the filesystem. These are SCHEMA
/// errors: a ledger that cannot be read as a set of couplings exempts nothing.
pub fn schema_errors(l: &Ledger) -> Vec<String> {
    let mut v = check_policy(&l.policy);

    if l.surface.is_empty() {
        v.push(format!(
            "{REGISTRY_PATH}: zero [[surface]] rows — a coupling registry with no surfaces is an ERROR, not a pass"
        ));
    }
    if l.golden.is_empty() {
        v.push(format!(
            "{REGISTRY_PATH}: zero [[golden]] rows — a coupling registry with no goldens is an ERROR, not a pass"
        ));
    }

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut pin_total = 0usize;
    for (i, s) in l.surface.iter().enumerate() {
        let id = s.id.trim();
        let where_ = if id.is_empty() {
            format!("[[surface]] #{}", i + 1)
        } else {
            format!("[[surface]] {id}")
        };
        if id.is_empty() {
            v.push(format!("{where_}: missing or empty `id`"));
        } else if !seen.insert(id) {
            v.push(format!("{where_}: duplicate `id`"));
        }
        if s.title.trim().is_empty() {
            v.push(format!("{where_}: missing or empty `title`"));
        }
        if s.version.trim().is_empty() {
            v.push(format!(
                "{where_}: missing or empty `version` — a surface goldens cannot quote cannot be re-affirmed"
            ));
        }
        justification_errors(&where_, &s.justification, &mut v);
        if s.pin.is_empty() {
            v.push(format!(
                "{where_}: empty `pin` — a surface with nothing read out of source is a name, not a surface; empty is a SCHEMA ERROR, never a pass"
            ));
        }
        pin_total += s.pin.len();
        for (j, p) in s.pin.iter().enumerate() {
            pin_schema_errors(&format!("{where_} pin #{}", j + 1), p, &mut v);
        }
    }

    let mut seen_g: BTreeSet<&str> = BTreeSet::new();
    let mut seen_f: BTreeSet<&str> = BTreeSet::new();
    let mut coupling_total = 0usize;
    let mut path_consts = 0usize;
    let surface_ids: BTreeSet<&str> = l.surface.iter().map(|s| s.id.trim()).collect();
    for (i, g) in l.golden.iter().enumerate() {
        let id = g.id.trim();
        let where_ = if id.is_empty() {
            format!("[[golden]] #{}", i + 1)
        } else {
            format!("[[golden]] {id}")
        };
        if id.is_empty() {
            v.push(format!("{where_}: missing or empty `id`"));
        } else if !seen_g.insert(id) {
            v.push(format!("{where_}: duplicate `id`"));
        }
        let file = g.file.trim();
        if file.is_empty() {
            v.push(format!("{where_}: missing or empty `file`"));
        } else {
            if !is_clean_relative_path(file) {
                v.push(format!(
                    "{where_}: `file` {file:?} is not a normalised engine-root-relative path"
                ));
            }
            if !seen_f.insert(file) {
                v.push(format!("{where_}: duplicate `file` {file:?}"));
            }
        }
        if !looks_like_digest(&g.frozen) {
            v.push(format!(
                "{where_}: `frozen` {:?} is not a 64-character lowercase sha256 — a golden with no content pin can be re-frozen without a diff here, which is the defect this file exists for",
                g.frozen.trim()
            ));
        }
        if g.affirmed.trim().is_empty() {
            v.push(format!(
                "{where_}: missing or empty `affirmed` — an affirmation that cannot go stale is permanent by another name"
            ));
        } else if let Err(e) = date::parse_ymd(g.affirmed.trim()) {
            v.push(format!("{where_}: `affirmed` {e}"));
        }
        justification_errors(&where_, &g.justification, &mut v);
        if !g.path_const.trim().is_empty() {
            path_consts += 1;
            if let Err(e) = split_symbol_ref(&g.path_const) {
                v.push(format!("{where_}: `const` {e}"));
            }
        }
        if g.depends_on.is_empty() {
            v.push(format!(
                "{where_}: empty `depends_on` — a golden that depends on nothing is a golden nothing can invalidate; empty is a SCHEMA ERROR, never a pass"
            ));
        }
        coupling_total += g.depends_on.len();
        let mut seen_d: BTreeSet<&str> = BTreeSet::new();
        for (j, d) in g.depends_on.iter().enumerate() {
            let at = format!("{where_} depends_on #{}", j + 1);
            let sid = d.surface.trim();
            if sid.is_empty() {
                v.push(format!("{at}: missing or empty `surface`"));
            } else if !seen_d.insert(sid) {
                v.push(format!("{at}: duplicate dependency on {sid:?}"));
            } else if !surface_ids.contains(sid) {
                v.push(format!(
                    "{at}: names surface {sid:?}, which no [[surface]] row declares"
                ));
            }
            if d.version.trim().is_empty() {
                v.push(format!(
                    "{at}: missing or empty `version` — an unversioned coupling cannot record that the surface moved"
                ));
            }
        }
    }

    if !l.surface.is_empty() && pin_total == 0 {
        v.push(format!(
            "{REGISTRY_PATH}: zero pins across {} surface(s) — a registry that reads nothing out of source is an ERROR, not a pass",
            l.surface.len()
        ));
    }
    if !l.golden.is_empty() && coupling_total == 0 {
        v.push(format!(
            "{REGISTRY_PATH}: zero couplings across {} golden(s) — an uncoupled coupling registry is an ERROR, not a pass",
            l.golden.len()
        ));
    }
    if !l.golden.is_empty() && path_consts < MIN_PATH_CONSTS {
        v.push(format!(
            "{REGISTRY_PATH}: {path_consts} golden(s) tie their path to a source constant; at least {MIN_PATH_CONSTS} must, or every path here is free to drift from the code that opens it"
        ));
    }
    for req in REQUIRED_GOLDENS {
        if !l.golden.iter().any(|g| g.file.trim() == *req) {
            v.push(format!(
                "{REGISTRY_PATH}: required golden {req:?} has no row — deleting the row for a pinned artifact is not a way to pass this gate"
            ));
        }
    }
    v
}

fn justification_errors(where_: &str, text: &str, v: &mut Vec<String>) {
    let j = text.trim();
    if j.is_empty() {
        v.push(format!(
            "{where_}: missing or empty `justification` — a coupling nobody wrote a reason for is a line of TOML, not a record; empty is a SCHEMA ERROR, never a pass"
        ));
    } else if j.len() < MIN_JUSTIFICATION_LEN {
        v.push(format!(
            "{where_}: `justification` is {} chars; at least {MIN_JUSTIFICATION_LEN} are needed to say something a reviewer can disagree with",
            j.len()
        ));
    }
}

fn pin_schema_errors(at: &str, p: &Pin, v: &mut Vec<String>) {
    let kind = p.kind.trim();
    if kind.is_empty() {
        v.push(format!("{at}: missing or empty `kind`"));
    } else if !KINDS.contains(&kind) {
        v.push(format!(
            "{at}: `kind` {kind:?} is not one of: {}",
            KINDS.join(", ")
        ));
    }
    let file = p.file.trim();
    if file.is_empty() {
        v.push(format!("{at}: missing or empty `file`"));
    } else if !is_clean_relative_path(file) {
        v.push(format!(
            "{at}: `file` {file:?} is not a normalised engine-root-relative path"
        ));
    }
    if p.symbol.trim().is_empty() {
        v.push(format!(
            "{at}: missing or empty `symbol` — a pin that names no item reads the whole file or nothing"
        ));
    }
    match kind {
        "region" => {
            if !looks_like_digest(&p.digest) {
                v.push(format!(
                    "{at}: `digest` {:?} is not a 64-character lowercase sha256",
                    p.digest.trim()
                ));
            }
            if !p.expect.is_empty() {
                v.push(format!(
                    "{at}: a `region` pin carries `digest`, not `expect` — a field that is never read must not sit here looking load-bearing"
                ));
            }
        }
        "lockdep" => {
            if p.via.trim().is_empty() {
                v.push(format!(
                    "{at}: a `lockdep` pin needs `via` — which crate's edge resolves the version is not guessable"
                ));
            }
            expect_len(at, p, 1, v);
        }
        "const" => expect_len(at, p, 1, v),
        "fields" | "keys" => {
            if p.expect.is_empty() {
                v.push(format!(
                    "{at}: empty `expect` — an empty expectation matches an emptied surface; empty is a SCHEMA ERROR, never a pass"
                ));
            }
            if !p.digest.trim().is_empty() {
                v.push(format!("{at}: `digest` belongs to a `region` pin only"));
            }
        }
        _ => {}
    }
    if kind == "keys" {
        let file = p.file.trim();
        if file.ends_with(".rs") {
            v.push(format!(
                "{at}: a `keys` pin is derived from serialised JSON, not from a Rust source scrape — pointing it at {file:?} is the overmatch this kind was retired to end"
            ));
        }
        if !is_json_selector(&p.symbol) {
            v.push(format!(
                "{at}: a `keys` pin names a JSON selector (`$` for the whole document, or a JSON pointer), not a Rust item; got {:?}",
                p.symbol.trim()
            ));
        }
    }
    if kind != "lockdep" && !p.via.trim().is_empty() {
        v.push(format!("{at}: `via` belongs to a `lockdep` pin only"));
    }
    for (i, e) in p.expect.iter().enumerate() {
        if e.trim().is_empty() {
            v.push(format!("{at}: `expect` entry #{} is empty", i + 1));
        }
    }
}

fn expect_len(at: &str, p: &Pin, want: usize, v: &mut Vec<String>) {
    if p.expect.len() != want {
        v.push(format!(
            "{at}: a `{}` pin takes exactly {want} `expect` entry; {} given",
            p.kind.trim(),
            p.expect.len()
        ));
    }
    if !p.digest.trim().is_empty() {
        v.push(format!("{at}: `digest` belongs to a `region` pin only"));
    }
}

/// Split `<path>::<SYMBOL>`.
pub fn split_symbol_ref(r: &str) -> Result<(&str, &str), String> {
    let r = r.trim();
    let Some((path, sym)) = r.split_once("::") else {
        return Err(format!(
            "{r:?} is not <path>::<SYMBOL> — it must name the file AND the item"
        ));
    };
    if sym.contains("::") {
        return Err(format!("{r:?} holds more than one `::` separator"));
    }
    if !is_clean_relative_path(path) {
        return Err(format!(
            "{r:?}: {path:?} is not a normalised engine-root-relative path"
        ));
    }
    if sym.trim().is_empty() {
        return Err(format!("{r:?}: names no item"));
    }
    Ok((path, sym))
}

// ── evaluation ─────────────────────────────────────────────────────────────

/// Everything the evaluation needs from the outside world, injected so the
/// rules stay testable without a filesystem and so no leg can quietly borrow
/// another's answer.
pub struct World<'a> {
    /// File text, or `None` when the path names nothing readable.
    pub read: &'a dyn Fn(&str) -> Option<String>,
    /// sha256 of a file's bytes; `Err` when it could not be read at all, which
    /// is a finding and never a pass.
    pub digest: &'a dyn Fn(&str) -> Result<String, String>,
    /// Files discovered under `GOLDENS_DIR`, engine-root-relative.
    pub discovered: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Report {
    pub violations: Vec<String>,
    pub errors: Vec<String>,
    pub pins_checked: usize,
    pub goldens_checked: usize,
    pub couplings_checked: usize,
    pub oldest_affirmation: Option<String>,
}

/// The verdict pass. Assumes `schema_errors` ran clean, so every field is
/// present and well formed; what is left is whether the world agrees.
pub fn evaluate(l: &Ledger, today: Ymd, w: &World<'_>) -> Report {
    let mut rep = Report::default();

    // ── surfaces: the registry against source ──────────────────────────────
    for s in &l.surface {
        let where_ = format!("[[surface]] {}", s.id.trim());
        let want = derive_version(s);
        if s.version.trim() != want {
            rep.violations.push(format!(
                "{where_}: `version` records {:?} but its pin block derives {want:?} — a pin moved and the version did not. Bump it, then re-affirm every golden that depends on this surface",
                s.version.trim()
            ));
        }
        for (j, p) in s.pin.iter().enumerate() {
            rep.pins_checked += 1;
            let at = format!("{where_} pin #{} ({})", j + 1, p.kind.trim());
            evaluate_pin(&at, p, w, &mut rep);
        }
    }

    // ── goldens: the registry against the frozen bytes ─────────────────────
    let versions: Vec<(&str, &str)> = l
        .surface
        .iter()
        .map(|s| (s.id.trim(), s.version.trim()))
        .collect();
    let mut declared: BTreeSet<&str> = BTreeSet::new();
    let mut oldest: Option<(i64, String)> = None;

    for g in &l.golden {
        let where_ = format!("[[golden]] {}", g.id.trim());
        let file = g.file.trim();
        declared.insert(file);
        rep.goldens_checked += 1;

        match (w.digest)(file) {
            Err(e) => rep.violations.push(format!(
                "{where_}: {file} could not be read ({e}) — a pinned artifact that is not here is not pinned"
            )),
            Ok(actual) if actual != g.frozen.trim() => rep.violations.push(format!(
                "{where_}: {file} is sha256 {actual}, the registry pins {} — the artifact was RE-FROZEN. Re-state the couplings below in this same change, or restore the file",
                g.frozen.trim()
            )),
            Ok(_) => {}
        }

        if let Ok(d) = date::parse_ymd(g.affirmed.trim()) {
            let age = days_between(d, today);
            if age < 0 {
                rep.violations.push(format!(
                    "{where_}: `affirmed` {} is in the future",
                    g.affirmed.trim()
                ));
            } else if age > l.policy.affirmation_days {
                rep.violations.push(format!(
                    "{where_}: `affirmed` {} is {age} days old (window {}) — EXPIRED. Re-read the couplings, then re-date the row",
                    g.affirmed.trim(),
                    l.policy.affirmation_days
                ));
            }
            match &oldest {
                Some((a, _)) if *a >= age => {}
                _ => oldest = Some((age, g.affirmed.trim().to_string())),
            }
        }

        if !g.path_const.trim().is_empty() {
            check_path_const(&where_, g, w, &mut rep);
        }

        for d in &g.depends_on {
            rep.couplings_checked += 1;
            let sid = d.surface.trim();
            let Some((_, live)) = versions.iter().find(|(id, _)| *id == sid) else {
                continue; // schema_errors already named the dangling surface
            };
            if *live != d.version.trim() {
                rep.violations.push(format!(
                    "{where_}: affirms {sid} at {:?}, and the registry now carries that surface at {live:?} — the surface MOVED and this golden was not re-affirmed. Re-freeze it deliberately, or say why it did not have to move",
                    d.version.trim()
                ));
            }
        }
    }

    // ── discovery: nothing frozen may sit outside the ledger ───────────────
    if w.discovered.is_empty() {
        rep.errors.push(format!(
            "discovery found zero files under {GOLDENS_DIR}/ — a scan that checked nothing must never report the way one that checked everything does"
        ));
    }
    for f in &w.discovered {
        if !declared.contains(f.as_str()) {
            rep.violations.push(format!(
                "{f} is under {GOLDENS_DIR}/ and no [[golden]] row declares it — a frozen artifact nothing couples to is the state this registry exists to end"
            ));
        }
    }

    rep.oldest_affirmation = oldest.map(|(_, d)| d);
    rep
}

fn check_path_const(where_: &str, g: &Golden, w: &World<'_>, rep: &mut Report) {
    let r = g.path_const.trim();
    let Ok((path, sym)) = split_symbol_ref(r) else {
        return;
    };
    let Some(text) = (w.read)(path) else {
        rep.violations.push(format!(
            "{where_}: `const` {r:?} names {path:?}, which is not a readable file in this tree"
        ));
        return;
    };
    let Some((a, b)) = find_region(&text, sym) else {
        rep.violations.push(format!(
            "{where_}: `const` {r:?} names {sym:?}, which {path} does not declare"
        ));
        return;
    };
    let Some(value) = extract_const_value(&text[a..b]) else {
        rep.violations
            .push(format!("{where_}: `const` {r:?} assigns nothing readable"));
        return;
    };
    let value = value.trim().trim_matches('"');
    if value != g.file.trim() {
        rep.violations.push(format!(
            "{where_}: `const` {r:?} is {value:?} and this row pins {:?} — the path the code opens and the path this registry covers have drifted apart",
            g.file.trim()
        ));
    }
}

fn evaluate_pin(at: &str, p: &Pin, w: &World<'_>, rep: &mut Report) {
    let file = p.file.trim();
    let symbol = p.symbol.trim();
    let Some(text) = (w.read)(file) else {
        rep.violations.push(format!(
            "{at}: {file:?} is not a readable file in this tree — the surface's definition is gone, so every golden below it is frozen against nothing"
        ));
        return;
    };

    if p.kind.trim() == "lockdep" {
        match lock_resolved_version(&text, p.via.trim(), symbol) {
            Err(e) => rep.violations.push(format!("{at}: {e}")),
            Ok(v) if v != p.expect[0].trim() => rep.violations.push(format!(
                "{at}: {} resolves {symbol} to {v:?}; the registry pins {:?} — the goldens below were frozen under the pinned one",
                p.via.trim(),
                p.expect[0].trim()
            )),
            Ok(_) => {}
        }
        return;
    }

    // `keys` is a property of the serialised artifact. Do not fall through to
    // find_region — that would scrape the emitter, which is the hole this
    // kind was retired from.
    if p.kind.trim() == "keys" {
        match extract_json_keys(&text, symbol) {
            Err(e) => rep.violations.push(format!("{at}: {file}::{symbol} {e}")),
            Ok(actual) => compare_list(at, file, symbol, "emitted key", &actual, p, rep),
        }
        return;
    }

    let Some((a, b)) = find_region(&text, symbol) else {
        rep.violations.push(format!(
            "{at}: {file} no longer declares {symbol:?} — a coupling that names a surface constant which does not exist cannot be checked, and must never read as checked"
        ));
        return;
    };
    let region = &text[a..b];

    match p.kind.trim() {
        "region" => {
            let actual = sha256_hex_bytes(normalise_region(region).as_bytes());
            if actual != p.digest.trim() {
                rep.violations.push(format!(
                    "{at}: {file}::{symbol} normalises to sha256 {actual}, the registry pins {} — this surface MOVED. Re-pin it, bump the surface version, and re-affirm every golden that names it",
                    p.digest.trim()
                ));
            }
        }
        "const" => match extract_const_value(region) {
            None => rep.violations.push(format!(
                "{at}: {file}::{symbol} assigns nothing this reader can extract"
            )),
            Some(v) if v.trim() != p.expect[0].trim() => rep.violations.push(format!(
                "{at}: {file}::{symbol} is {v:?}; the registry pins {:?} — a version constant moved under artifacts frozen against it",
                p.expect[0].trim()
            )),
            Some(_) => {}
        },
        "fields" => compare_list(at, file, symbol, "field", &extract_fields(region), p, rep),
        _ => {}
    }
}

fn compare_list(
    at: &str,
    file: &str,
    symbol: &str,
    noun: &str,
    actual: &[String],
    p: &Pin,
    rep: &mut Report,
) {
    let want: Vec<String> = p.expect.iter().map(|e| e.trim().to_string()).collect();
    if actual.is_empty() {
        rep.violations.push(format!(
            "{at}: {file}::{symbol} yields zero {noun}s — an empty extraction must never compare clean against a non-empty pin"
        ));
        return;
    }
    if *actual == want {
        return;
    }
    let added: Vec<&String> = actual.iter().filter(|a| !want.contains(a)).collect();
    let removed: Vec<&String> = want.iter().filter(|e| !actual.contains(e)).collect();
    let how = if added.is_empty() && removed.is_empty() {
        "the same names in a different order".to_string()
    } else {
        format!("added {added:?}, removed {removed:?}")
    };
    rep.violations.push(format!(
        "{at}: {file}::{symbol} now has {noun}s {actual:?}; the registry pins {want:?} ({how}) — this surface MOVED under artifacts frozen against it"
    ));
}

// ── discovery ──────────────────────────────────────────────────────────────

/// Every file under `root/GOLDENS_DIR`, engine-root-relative, sorted, minus the
/// prose extensions.
pub fn discover(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk(&root.join(GOLDENS_DIR), root, &mut out);
    out.sort();
    out
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, root, out);
            continue;
        }
        let ext = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if DISCOVERY_SKIP_EXT.contains(&ext.as_str()) {
            continue;
        }
        if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

// ── the gate ───────────────────────────────────────────────────────────────

pub fn run(ctx: &GateCtx) -> Result<(), GateError> {
    ctx.reject_unknown_flags(KNOWN_FLAGS)?;
    let quiet = ctx.has_flag("--quiet");
    let root: &Path = &ctx.root;

    let reg = root.join(REGISTRY_PATH);
    let text = std::fs::read_to_string(&reg)
        .map_err(|e| GateError::error(format!("read {}: {e}", reg.display())))?;
    let ledger = parse_ledger(&text).map_err(GateError::error)?;

    let schema = schema_errors(&ledger);
    if !schema.is_empty() {
        return Err(GateError::Error(format!(
            "{} schema error(s) in {REGISTRY_PATH}: {}",
            schema.len(),
            schema.join(" | ")
        )));
    }

    let read = |rel: &str| -> Option<String> { std::fs::read_to_string(root.join(rel)).ok() };
    let digest = |rel: &str| -> Result<String, String> {
        sha256_file(&root.join(rel)).map_err(|e| e.to_string())
    };
    let world = World {
        read: &read,
        digest: &digest,
        discovered: discover(root),
    };

    let rep = evaluate(&ledger, date::today(), &world);

    // Anti-vacuous, from the other side: rows that exist but were never really
    // looked at must not report the way looked-at ones do.
    if rep.pins_checked == 0 || rep.goldens_checked == 0 || rep.couplings_checked == 0 {
        return Err(GateError::error(format!(
            "checked {} pin(s), {} golden(s), {} coupling(s) — a vacuous scan is an ERROR, not a pass",
            rep.pins_checked, rep.goldens_checked, rep.couplings_checked
        )));
    }
    if !rep.errors.is_empty() {
        return Err(GateError::Error(rep.errors.join(" | ")));
    }
    if !rep.violations.is_empty() {
        return Err(GateError::Violation(rep.violations));
    }

    if !quiet {
        println!(
            "{NAME}: ok: surfaces={} pins={} goldens={} couplings={} discovered={} oldest_affirmation={}",
            ledger.surface.len(),
            rep.pins_checked,
            rep.goldens_checked,
            rep.couplings_checked,
            world.discovered.len(),
            rep.oldest_affirmation.as_deref().unwrap_or("none")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
pub const SCHEMA_VERSION: u32 = 1;

/// doc comment with a { brace
pub struct GradeReport {
    pub schema_version: u32,
    #[serde(default)]
    pub bank_hash: String,
}

pub fn payload() -> Map {
    let mut m = Map::new();
    m.insert("id".into(), json!(self.id));
    // a comment mentioning "ghost".into()
    m.insert("module".into(), json!(self.module));
    let s = "a } brace in a string";
    json!({ "wrapped": s, "n": 1 })
}
"#;

    fn pin(kind: &str, symbol: &str, expect: &[&str], digest: &str) -> Pin {
        Pin {
            kind: kind.into(),
            file: "src/lib.rs".into(),
            symbol: symbol.into(),
            via: String::new(),
            expect: expect.iter().map(|s| (*s).to_string()).collect(),
            digest: digest.into(),
        }
    }

    fn surface(pins: Vec<Pin>) -> Surface {
        let mut s = Surface {
            id: "x.y".into(),
            title: "t".into(),
            version: String::new(),
            justification: "a justification long enough that a reviewer has something concrete to disagree with".into(),
            pin: pins,
        };
        s.version = derive_version(&s);
        s
    }

    fn golden(deps: Vec<(&str, &str)>) -> Golden {
        Golden {
            id: "g".into(),
            file: "goldens/bank_hash.txt".into(),
            path_const: String::new(),
            frozen: "a".repeat(64),
            affirmed: "2026-08-14".into(),
            justification: "a justification long enough that a reviewer has something concrete to disagree with".into(),
            depends_on: deps
                .into_iter()
                .map(|(s, v)| Dep {
                    surface: s.into(),
                    version: v.into(),
                })
                .collect(),
        }
    }

    fn world_of<'a>(
        read: &'a dyn Fn(&str) -> Option<String>,
        digest: &'a dyn Fn(&str) -> Result<String, String>,
        discovered: &[&str],
    ) -> World<'a> {
        World {
            read,
            digest,
            discovered: discovered.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn days_arithmetic_round_trips() {
        assert_eq!(days_from_civil((1970, 1, 1)), 0);
        assert_eq!(days_between((2026, 8, 14), (2026, 8, 14)), 0);
        assert_eq!(days_between((2025, 8, 14), (2026, 8, 14)), 365);
        assert_eq!(days_between((2026, 8, 14), (2026, 8, 13)), -1);
        assert_eq!(days_between((2024, 2, 28), (2024, 3, 1)), 2);
    }

    #[test]
    fn a_region_ends_at_its_balanced_brace_not_at_one_in_a_string() {
        let (a, b) = find_region(SRC, "payload").expect("payload");
        let region = &SRC[a..b];
        assert!(region.starts_with("fn payload"));
        assert!(region.contains("a } brace in a string"));
        assert!(region.trim_end().ends_with('}'));
        assert!(!region.contains("pub const SCHEMA_VERSION"));
    }

    #[test]
    fn a_const_region_ends_at_its_semicolon() {
        let (a, b) = find_region(SRC, "SCHEMA_VERSION").expect("const");
        assert_eq!(SRC[a..b].trim(), "const SCHEMA_VERSION: u32 = 1;");
        assert_eq!(extract_const_value(&SRC[a..b]).as_deref(), Some("1"));
    }

    #[test]
    fn fields_come_out_in_declaration_order_and_skip_attributes() {
        let (a, b) = find_region(SRC, "GradeReport").expect("struct");
        assert_eq!(extract_fields(&SRC[a..b]), ["schema_version", "bank_hash"]);
    }

    #[test]
    fn json_keys_are_object_keys_not_string_values() {
        // First-seen preorder, BTreeMap order at each object: envelope then
        // the first nested object's keys. String VALUES (the error message)
        // must not enter. A key produced only as JSON (the helper route the
        // old scrape could not see) must.
        let json = r#"{
            "id": "x",
            "module": 1,
            "note": "missing from bank",
            "items": [{"from_helper": true, "id": "x"}]
        }"#;
        let keys = extract_json_keys(json, "$").expect("parse");
        assert_eq!(keys, ["id", "items", "from_helper", "module", "note"]);
        assert!(
            !keys.iter().any(|k| k.contains("missing from bank")),
            "an error-message string value must not enter the key pin: {keys:?}"
        );
        assert!(
            keys.iter().any(|k| k == "from_helper"),
            "a key that exists only on the artifact (helper/const/format!) must enter: {keys:?}"
        );
    }

    #[test]
    fn json_keys_of_an_empty_object_are_empty_so_compare_list_can_fail() {
        assert_eq!(extract_json_keys("{}", "$").unwrap(), [] as [String; 0]);
        assert_eq!(extract_json_keys("[]", "$").unwrap(), [] as [String; 0]);
        assert_eq!(extract_json_keys("\"id\"", "$").unwrap(), [] as [String; 0]);
    }

    #[test]
    fn a_keys_pin_on_rust_source_is_a_schema_error() {
        let s = surface(vec![pin("keys", "$", &["id"], "")]);
        // pin() defaults file to src/lib.rs — that is the retired scrape.
        let g = golden(vec![("x.y", &s.version.clone())]);
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            surface: vec![s],
            golden: vec![g],
        };
        let errs = schema_errors(&l);
        assert!(
            errs.iter()
                .any(|e| e.contains("serialised JSON") && e.contains("src/lib.rs")),
            "{errs:?}"
        );
    }

    #[test]
    fn rust_source_is_not_silently_read_as_a_key_set() {
        assert!(extract_json_keys(SRC, "$").is_err());
    }

    #[test]
    fn a_missing_symbol_does_not_resolve_to_a_neighbour() {
        assert!(find_region(SRC, "GradeReportV2").is_none());
        assert!(find_region(SRC, "payloadx").is_none());
    }

    #[test]
    fn normalisation_drops_whole_line_comments_only() {
        let n = normalise_region("fn a() {\n    // gone\n\n    let x = 1; // kept\n}\n");
        assert!(!n.contains("gone"));
        assert!(n.contains("// kept"));
        assert!(!n.contains("\n\n"));
    }

    #[test]
    fn the_derived_version_moves_when_any_pin_moves() {
        let a = surface(vec![pin("const", "SCHEMA_VERSION", &["1"], "")]);
        let b = surface(vec![pin("const", "SCHEMA_VERSION", &["2"], "")]);
        assert_ne!(derive_version(&a), derive_version(&b));
        assert!(a.version.starts_with('v'));
        assert_eq!(a.version.len(), VERSION_HEX_LEN + 1);
    }

    #[test]
    fn a_lockdep_edge_carries_the_version_when_the_name_is_ambiguous() {
        let lock = "\
[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\"rand 0.8.7\"]\n\n\
[[package]]\nname = \"rand\"\nversion = \"0.8.7\"\n\n\
[[package]]\nname = \"rand\"\nversion = \"0.9.5\"\n";
        assert_eq!(
            lock_resolved_version(lock, "app", "rand").unwrap(),
            "0.8.7".to_string()
        );
        assert!(lock_resolved_version(lock, "app", "serde").is_err());
        assert!(lock_resolved_version(lock, "nope", "rand").is_err());
    }

    #[test]
    fn an_unversioned_edge_into_two_candidates_is_an_error_not_a_guess() {
        let lock = "\
[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\"rand\"]\n\n\
[[package]]\nname = \"rand\"\nversion = \"0.8.7\"\n\n\
[[package]]\nname = \"rand\"\nversion = \"0.9.5\"\n";
        assert!(lock_resolved_version(lock, "app", "rand").is_err());
    }

    #[test]
    fn a_blank_field_is_a_schema_error_never_permission() {
        let s = surface(vec![pin("const", "SCHEMA_VERSION", &["1"], "")]);
        let g = golden(vec![("x.y", &s.version.clone())]);
        let mut l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            surface: vec![s],
            golden: vec![g],
        };
        // The compiled-in floors are the only outstanding complaint: a one-row
        // fixture cannot satisfy REQUIRED_GOLDENS or MIN_PATH_CONSTS, and both
        // of those are about the SHIPPED registry, not about row shape.
        let base = schema_errors(&l);
        assert!(
            base.iter()
                .all(|e| e.contains("required golden") || e.contains("source constant")),
            "{base:?}"
        );

        l.golden[0].justification = "  ".into();
        assert!(
            schema_errors(&l)
                .iter()
                .any(|e| e.contains("empty `justification`")),
            "a blank justification must be an error"
        );
        l.golden[0].justification = "x".repeat(MIN_JUSTIFICATION_LEN);
        l.golden[0].depends_on.clear();
        assert!(schema_errors(&l)
            .iter()
            .any(|e| e.contains("empty `depends_on`")));
        l.surface[0].pin.clear();
        assert!(schema_errors(&l).iter().any(|e| e.contains("empty `pin`")));
    }

    #[test]
    fn zero_rows_and_a_widened_window_are_errors() {
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: MAX_AFFIRMATION_DAYS + 1,
            },
            surface: vec![],
            golden: vec![],
        };
        let e = schema_errors(&l);
        assert!(e.iter().any(|x| x.contains("zero [[surface]]")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("zero [[golden]]")), "{e:?}");
        assert!(e.iter().any(|x| x.contains("never widen it")), "{e:?}");
        assert!(check_policy(&Policy {
            affirmation_days: 30
        })
        .is_empty());
        assert!(!check_policy(&Policy {
            affirmation_days: 0
        })
        .is_empty());
    }

    #[test]
    fn a_moved_surface_is_red_and_names_both_sides() {
        let read = |_: &str| Some(SRC.to_string());
        let dig = |_: &str| Ok("a".repeat(64));
        let w = world_of(&read, &dig, &["goldens/bank_hash.txt"]);

        let good = surface(vec![pin("const", "SCHEMA_VERSION", &["1"], "")]);
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![golden(vec![("x.y", &good.version.clone())])],
            surface: vec![good],
        };
        let rep = evaluate(&l, (2026, 8, 14), &w);
        assert!(rep.violations.is_empty(), "{:?}", rep.violations);
        assert_eq!(rep.pins_checked, 1);
        assert_eq!(rep.couplings_checked, 1);

        let moved = surface(vec![pin("const", "SCHEMA_VERSION", &["2"], "")]);
        let l2 = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![golden(vec![("x.y", &moved.version.clone())])],
            surface: vec![moved],
        };
        let rep = evaluate(&l2, (2026, 8, 14), &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("\"1\"")
                && v.contains("\"2\"")
                && v.contains("version constant moved")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn a_golden_that_did_not_re_affirm_a_moved_surface_is_red() {
        let read = |_: &str| Some(SRC.to_string());
        let dig = |_: &str| Ok("a".repeat(64));
        let w = world_of(&read, &dig, &["goldens/bank_hash.txt"]);
        let s = surface(vec![pin("const", "SCHEMA_VERSION", &["1"], "")]);
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![golden(vec![("x.y", "v000000000000")])],
            surface: vec![s],
        };
        let rep = evaluate(&l, (2026, 8, 14), &w);
        assert!(
            rep.violations
                .iter()
                .any(|v| v.contains("was not re-affirmed")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn a_re_frozen_golden_is_red_even_when_every_surface_held() {
        let read = |_: &str| Some(SRC.to_string());
        let dig = |_: &str| Ok("b".repeat(64));
        let w = world_of(&read, &dig, &["goldens/bank_hash.txt"]);
        let s = surface(vec![pin("const", "SCHEMA_VERSION", &["1"], "")]);
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![golden(vec![("x.y", &s.version.clone())])],
            surface: vec![s],
        };
        let rep = evaluate(&l, (2026, 8, 14), &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("RE-FROZEN")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn an_undeclared_discovery_is_red_and_an_empty_one_is_an_error() {
        let read = |_: &str| Some(SRC.to_string());
        let dig = |_: &str| Ok("a".repeat(64));
        let s = surface(vec![pin("const", "SCHEMA_VERSION", &["1"], "")]);
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![golden(vec![("x.y", &s.version.clone())])],
            surface: vec![s],
        };

        let w = world_of(
            &read,
            &dig,
            &["goldens/bank_hash.txt", "goldens/a_new_one.sha256"],
        );
        let rep = evaluate(&l, (2026, 8, 14), &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("a_new_one")),
            "{:?}",
            rep.violations
        );

        let w2 = world_of(&read, &dig, &[]);
        let rep = evaluate(&l, (2026, 8, 14), &w2);
        assert!(
            rep.errors.iter().any(|e| e.contains("zero files")),
            "{:?}",
            rep.errors
        );
    }

    #[test]
    fn an_expired_affirmation_is_red_and_a_future_one_is_too() {
        let read = |_: &str| Some(SRC.to_string());
        let dig = |_: &str| Ok("a".repeat(64));
        let w = world_of(&read, &dig, &["goldens/bank_hash.txt"]);
        let s = surface(vec![pin("const", "SCHEMA_VERSION", &["1"], "")]);
        let mut g = golden(vec![("x.y", &s.version.clone())]);
        g.affirmed = "2020-01-01".into();
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![g.clone()],
            surface: vec![s.clone()],
        };
        let rep = evaluate(&l, (2026, 8, 14), &w);
        assert!(rep.violations.iter().any(|v| v.contains("EXPIRED")));

        g.affirmed = "2099-01-01".into();
        let l2 = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![g],
            surface: vec![s],
        };
        let rep = evaluate(&l2, (2026, 8, 14), &w);
        assert!(rep.violations.iter().any(|v| v.contains("in the future")));
    }

    #[test]
    fn a_surface_whose_file_or_symbol_is_gone_is_red() {
        let dig = |_: &str| Ok("a".repeat(64));
        let s = surface(vec![pin("fields", "GradeReport", &["schema_version"], "")]);
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![golden(vec![("x.y", &s.version.clone())])],
            surface: vec![s],
        };

        let gone = |_: &str| None;
        let w = world_of(&gone, &dig, &["goldens/bank_hash.txt"]);
        let rep = evaluate(&l, (2026, 8, 14), &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("not a readable")),
            "{:?}",
            rep.violations
        );

        let renamed = |_: &str| Some("pub struct Other { pub a: u32 }".to_string());
        let w2 = world_of(&renamed, &dig, &["goldens/bank_hash.txt"]);
        let rep = evaluate(&l, (2026, 8, 14), &w2);
        assert!(
            rep.violations
                .iter()
                .any(|v| v.contains("no longer declares")),
            "{:?}",
            rep.violations
        );
    }

    #[test]
    fn an_emptied_extraction_never_compares_clean() {
        let read = |_: &str| Some("pub struct GradeReport {}".to_string());
        let dig = |_: &str| Ok("a".repeat(64));
        let w = world_of(&read, &dig, &["goldens/bank_hash.txt"]);
        let s = surface(vec![pin("fields", "GradeReport", &["schema_version"], "")]);
        let l = Ledger {
            schema_version: 1,
            policy: Policy {
                affirmation_days: 365,
            },
            golden: vec![golden(vec![("x.y", &s.version.clone())])],
            surface: vec![s],
        };
        let rep = evaluate(&l, (2026, 8, 14), &w);
        assert!(
            rep.violations.iter().any(|v| v.contains("zero fields")),
            "{:?}",
            rep.violations
        );
    }
}
