//! Corpus-rights checker — the published corpus may not contain material
//! its own metadata says this project is not licensed to redistribute
//! (bd-corpus-public-captures-not-licensed-class-kej).
//!
//! EXTRACT-THEN-DELETE from the parked `cdcp_gate` sources. Product lives
//! here; the CLI is `cdcp corpus-rights`. Do not put this under
//! `cdcp_gate/src/gates/` — that glob grows the crate past the gate_shrink
//! ceiling.
//!
//! # CLAIM: FLOOR-RAISE
//!
//! This checker raises one floor: **every file inside the published corpus
//! tree is claimed by a metadata record, and no record that says "not
//! licensed" has a body sitting in that tree.** Concretely it reads
//! `knowledge/corpus/public/manifest.json` and every
//! `knowledge/corpus/free-pdfs/*.meta.toml`, and goes red when —
//!
//!   * **CR-R1** a record whose `redistribution` is anything other than
//!     `permitted` is `capture = "body-retained"`, or names a `path` that
//!     exists on disk under a published root;
//!   * **CR-R2** a record is missing `rights`, `redistribution` or
//!     `capture`, or carries a value the policy's vocabulary does not list;
//!   * **CR-R4** a record claims the R1 exemption (`rights_review = "OPEN"`)
//!     without both a bead and a non-empty reason;
//!   * **CR-R6** a record with `ai_ingestion = "PROHIBITED"` is
//!     body-retained;
//!   * **CR-R7** a body-retained record claims `redistribution =
//!     "permitted"` over rights that are not self-evidencing, without a
//!     `redistribution_evidence` table naming a licence, a url and a
//!     clause;
//!   * **CR-R8** a file exists under a published root that no record's
//!     `path` claims.
//!
//! The vocabulary, the published roots and the self-evidencing rights come
//! from `knowledge/corpus/rights-policy.toml`. **The string that means
//! permission is compiled in here, not read from that file**: the policy
//! can name new vocabulary values, and every one of them will fail the
//! permitted test, so no registry edit can mint permission. The policy may
//! also ADD published roots and NARROW the self-evidencing rights; it may
//! not drop a compiled-in root or widen self-evidencing, and an attempt
//! to is an ERROR.
//!
//! # WHAT THIS CHECKER CANNOT DECIDE
//!
//! **It reads metadata. It cannot decide that the metadata is honest.** A
//! record that says `redistribution = "permitted"` over material nobody
//! licensed passes exactly as a correctly recorded one does.
//!
//! **It cannot decide that a `redistribution_evidence` block is true.**
//! CR-R7 checks that a licence name, a url and a clause are present and
//! non-empty. It does not fetch the url.
//!
//! **It deliberately reads no capture body.** Classification is from
//! metadata only, and that is a requirement rather than an optimisation:
//! some sources record `ai_ingestion = "PROHIBITED"`. No code path here
//! reads, renders, echoes or hashes the contents of any capture — only
//! directory entries, file names, and the registry files themselves.
//!
//! # ANTI-VACUOUS (L4)
//!
//! Each of these is an ERROR, not a pass: zero manifest sources; zero
//! `.meta.toml` sidecars; zero records overall; a `manifest.json` that is
//! missing or does not parse; a published root that is missing or
//! unlistable; zero files examined under the roots; a policy file that is
//! missing, does not parse, or has an empty vocabulary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// CLI / log name. Compiled in so a rename is a test failure.
pub const NAME: &str = "corpus-rights";

/// The machine-readable policy this checker enforces.
pub const POLICY_PATH: &str = "knowledge/corpus/rights-policy.toml";

/// The manifest of public captures.
pub const MANIFEST_PATH: &str = "knowledge/corpus/public/manifest.json";

/// Where the per-PDF rights sidecars live.
pub const SIDECAR_DIR: &str = "knowledge/corpus/free-pdfs";

/// **The only string that means permission.** Compiled in on purpose: the
/// policy file may extend the vocabulary, and every value it adds fails
/// this test.
pub const PERMITTED: &str = "permitted";

/// The value that forbids AI ingestion, compared case-sensitively after
/// the same normalisation the vocabulary uses.
pub const PROHIBITED: &str = "prohibited";

/// Published roots that must appear in the policy. The policy may add
/// roots; it may not drop one of these.
pub const REQUIRED_ROOTS: &[&str] = &["knowledge/corpus/public", "knowledge/corpus/free-pdfs"];

/// The widest set of `rights` values that may excuse a body-retained
/// record from citing a licence (CR-R7). The policy may NARROW this.
pub const ALLOWED_SELF_EVIDENCING: &[&str] = &["public-domain", "own-work-this-repo"];

/// Registry files that live inside a published root and are not captures.
pub const REGISTRY_FILE_NAMES: &[&str] = &["manifest.json"];

/// Suffix marking a rights sidecar rather than a capture.
pub const SIDECAR_SUFFIX: &str = ".meta.toml";

/// Path prefix the policy spells its roots with (they are git-root-relative,
/// this checker is engine-root-relative).
pub const ENGINE_PREFIX: &str = "course-engine/";

/// Token interpolated on every CR-R7 finding. Deleting the R7 check
/// makes the matching selftest non-zero.
pub const CR_R7: &str = "CR-R7";

/// Token interpolated on every CR-R8 finding. Deleting the R8 walk
/// makes the matching selftest non-zero.
pub const CR_R8: &str = "CR-R8";

/// Token interpolated inside the no-body-read contract. Deleting the
/// `exists`-only probe makes the matching selftest non-zero.
pub const NEVER_OPENS_CAPTURE_BODIES: &str = "never opens capture bodies";

// ── tiny JSON reader ───────────────────────────────────────────────────────
//
// The manifest is JSON and this crate has no JSON dependency. Hand-rolled
// parsing keeps the extract to one new module and no lockfile growth.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The value as a string. A non-string where a string is required is
    /// reported by the caller as a schema error rather than silently coerced.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
}

struct P<'a> {
    ch: &'a [char],
    i: usize,
}

impl<'a> P<'a> {
    fn ws(&mut self) {
        while self.i < self.ch.len() && self.ch[self.i].is_whitespace() {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.ch.get(self.i).copied()
    }

    fn lit(&mut self, s: &str) -> Result<(), String> {
        for c in s.chars() {
            if self.peek() != Some(c) {
                return Err(format!("offset {}: expected {s:?}", self.i));
            }
            self.i += 1;
        }
        Ok(())
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.peek() {
            None => Err("unexpected end of input".to_string()),
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Json::Str(self.string()?)),
            Some('t') => {
                self.lit("true")?;
                Ok(Json::Bool(true))
            }
            Some('f') => {
                self.lit("false")?;
                Ok(Json::Bool(false))
            }
            Some('n') => {
                self.lit("null")?;
                Ok(Json::Null)
            }
            Some(_) => self.number(),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.lit("{")?;
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some('}') {
            self.i += 1;
            return Ok(Json::Obj(out));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.ws();
            self.lit(":")?;
            let v = self.value()?;
            out.push((k, v));
            self.ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {
                    self.i += 1;
                    return Ok(Json::Obj(out));
                }
                _ => return Err(format!("offset {}: expected ',' or '}}'", self.i)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.lit("[")?;
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(']') {
            self.i += 1;
            return Ok(Json::Arr(out));
        }
        loop {
            out.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some(']') => {
                    self.i += 1;
                    return Ok(Json::Arr(out));
                }
                _ => return Err(format!("offset {}: expected ',' or ']'", self.i)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.lit("\"")?;
        let mut s = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err("unterminated string".to_string());
            };
            self.i += 1;
            match c {
                '"' => return Ok(s),
                '\\' => {
                    let Some(e) = self.peek() else {
                        return Err("unterminated escape".to_string());
                    };
                    self.i += 1;
                    match e {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
                        'b' => s.push('\u{8}'),
                        'f' => s.push('\u{c}'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => {
                            let mut code = 0u32;
                            for _ in 0..4 {
                                let Some(h) = self.peek().and_then(|c| c.to_digit(16)) else {
                                    return Err(format!("offset {}: bad \\u escape", self.i));
                                };
                                code = code * 16 + h;
                                self.i += 1;
                            }
                            s.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        }
                        other => return Err(format!("offset {}: bad escape {other:?}", self.i)),
                    }
                }
                other => s.push(other),
            }
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') {
                self.i += 1;
            } else {
                break;
            }
        }
        let raw: String = self.ch[start..self.i].iter().collect();
        raw.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| format!("offset {start}: bad number {raw:?}"))
    }
}

/// Parse a complete JSON document. Trailing non-whitespace is an error.
pub fn parse_json(text: &str) -> Result<Json, String> {
    let ch: Vec<char> = text.chars().collect();
    let mut p = P { ch: &ch, i: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != p.ch.len() {
        return Err(format!("offset {}: trailing input", p.i));
    }
    Ok(v)
}

// ── the policy ─────────────────────────────────────────────────────────────

/// The parts of `rights-policy.toml` this checker is configured by.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub capture: BTreeSet<String>,
    pub rights: BTreeSet<String>,
    pub redistribution: BTreeSet<String>,
    pub ai_ingestion: BTreeSet<String>,
    pub self_evidencing: BTreeSet<String>,
    /// Engine-root-relative, `course-engine/` stripped.
    pub roots: Vec<String>,
    /// `[[open_violation]]` rows as `(bead, records)`.
    pub open_violations: Vec<(String, Vec<String>)>,
}

/// Vocabulary comparisons are case-insensitive and trimmed.
pub fn norm(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Documented legacy spellings and the value each one means.
pub const ALIASES: &[(&str, &str)] = &[("publisher-copyright", "publisher-retains-copyright")];

/// `norm`, then the alias map. Used only where two records are compared.
pub fn canon(s: &str) -> String {
    let n = norm(s);
    ALIASES
        .iter()
        .find(|(from, _)| *from == n)
        .map(|(_, to)| (*to).to_string())
        .unwrap_or(n)
}

fn str_list(v: Option<&toml::Value>) -> Option<Vec<String>> {
    let arr = v?.as_array()?;
    arr.iter().map(|e| e.as_str().map(str::to_string)).collect()
}

pub fn parse_policy(text: &str) -> Result<Policy, String> {
    let doc: toml::Value = toml::from_str(text).map_err(|e| format!("{POLICY_PATH}: {e}"))?;
    let vocab = doc
        .get("vocabulary")
        .ok_or_else(|| format!("{POLICY_PATH}: no [vocabulary] table"))?;

    let need = |key: &str| -> Result<BTreeSet<String>, String> {
        let list = str_list(vocab.get(key)).ok_or_else(|| {
            format!("{POLICY_PATH}: [vocabulary].{key} must be a list of strings")
        })?;
        if list.is_empty() {
            return Err(format!(
                "{POLICY_PATH}: [vocabulary].{key} is empty — an empty vocabulary accepts nothing and checks nothing"
            ));
        }
        Ok(list.iter().map(|s| norm(s)).collect())
    };

    let self_evidencing: BTreeSet<String> = str_list(vocab.get("self_evidencing_rights"))
        .ok_or_else(|| {
            format!("{POLICY_PATH}: [vocabulary].self_evidencing_rights must be a list of strings")
        })?
        .iter()
        .map(|s| norm(s))
        .collect();

    let roots_raw =
        str_list(doc.get("published_tree").and_then(|t| t.get("roots"))).ok_or_else(|| {
            format!("{POLICY_PATH}: [published_tree].roots must be a list of strings")
        })?;
    let roots: Vec<String> = roots_raw
        .iter()
        .map(|r| {
            r.trim()
                .strip_prefix(ENGINE_PREFIX)
                .unwrap_or(r.trim())
                .trim_end_matches('/')
                .to_string()
        })
        .collect();

    let mut open_violations = Vec::new();
    if let Some(arr) = doc.get("open_violation").and_then(|v| v.as_array()) {
        for row in arr {
            let bead = row
                .get("bead")
                .and_then(|b| b.as_str())
                .unwrap_or_default()
                .to_string();
            let records = str_list(row.get("records")).unwrap_or_default();
            open_violations.push((bead, records));
        }
    }

    Ok(Policy {
        capture: need("capture")?,
        rights: need("rights")?,
        redistribution: need("redistribution")?,
        ai_ingestion: need("ai_ingestion")?,
        self_evidencing,
        roots,
        open_violations,
    })
}

/// The registry may tighten this checker's configuration; it may not loosen it.
pub fn policy_floor_errors(p: &Policy) -> Vec<String> {
    let mut out = Vec::new();
    for r in REQUIRED_ROOTS {
        if !p.roots.iter().any(|x| x == r) {
            out.push(format!(
                "{POLICY_PATH}: [published_tree].roots dropped the compiled-in root {r:?} — the policy may add roots, never remove one"
            ));
        }
    }
    let allowed: BTreeSet<String> = ALLOWED_SELF_EVIDENCING.iter().map(|s| norm(s)).collect();
    for s in &p.self_evidencing {
        if !allowed.contains(s) {
            out.push(format!(
                "{POLICY_PATH}: [vocabulary].self_evidencing_rights lists {s:?}, which is outside the compiled-in set {ALLOWED_SELF_EVIDENCING:?} — this list may be narrowed, never widened"
            ));
        }
    }
    if !p.redistribution.contains(PERMITTED) {
        out.push(format!(
            "{POLICY_PATH}: [vocabulary].redistribution does not list {PERMITTED:?} — the vocabulary and the rule have diverged"
        ));
    }
    out
}

// ── records ────────────────────────────────────────────────────────────────

/// A licence citation standing behind a `permitted` claim (CR-R7).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Evidence {
    pub licence: String,
    pub url: String,
    pub clause: String,
}

impl Evidence {
    pub fn complete(&self) -> bool {
        !self.licence.trim().is_empty()
            && !self.url.trim().is_empty()
            && !self.clause.trim().is_empty()
    }
}

/// One corpus source record, from either registry.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Record {
    pub id: String,
    /// The registry file this row came from, for the finding text.
    pub origin: String,
    pub rights: Option<String>,
    pub redistribution: Option<String>,
    pub ai_ingestion: Option<String>,
    pub capture: Option<String>,
    pub path: Option<String>,
    pub rights_review: Option<String>,
    pub rights_review_bead: Option<String>,
    pub rights_review_reason: Option<String>,
    pub evidence: Option<Evidence>,
}

impl Record {
    fn field(&self, which: &str) -> Option<&str> {
        match which {
            "rights" => self.rights.as_deref(),
            "redistribution" => self.redistribution.as_deref(),
            "capture" => self.capture.as_deref(),
            _ => None,
        }
    }

    pub fn is_body_retained(&self) -> bool {
        self.capture.as_deref().map(norm).as_deref() == Some("body-retained")
    }

    pub fn redistribution_permitted(&self) -> bool {
        self.redistribution.as_deref().map(norm).as_deref() == Some(PERMITTED)
    }

    pub fn ai_prohibited(&self) -> bool {
        self.ai_ingestion.as_deref().map(norm).as_deref() == Some(PROHIBITED)
    }
}

fn json_field(o: &Json, key: &str) -> Option<String> {
    o.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

/// `sources[]` of the manifest, as records.
pub fn records_from_manifest(doc: &Json, origin: &str) -> Result<Vec<Record>, String> {
    let Some(Json::Arr(sources)) = doc.get("sources") else {
        return Err(format!("{origin}: `sources` must be an array"));
    };
    let mut out = Vec::with_capacity(sources.len());
    for (i, s) in sources.iter().enumerate() {
        if !matches!(s, Json::Obj(_)) {
            return Err(format!("{origin}: sources[{i}] is not an object"));
        }
        let Some(id) = json_field(s, "id") else {
            return Err(format!("{origin}: sources[{i}] has no string `id`"));
        };
        let evidence = s.get("redistribution_evidence").map(|e| Evidence {
            licence: json_field(e, "licence").unwrap_or_default(),
            url: json_field(e, "url").unwrap_or_default(),
            clause: json_field(e, "clause").unwrap_or_default(),
        });
        out.push(Record {
            id,
            origin: origin.to_string(),
            rights: json_field(s, "rights"),
            redistribution: json_field(s, "redistribution"),
            ai_ingestion: json_field(s, "ai_ingestion"),
            capture: json_field(s, "capture"),
            path: json_field(s, "path"),
            rights_review: json_field(s, "rights_review"),
            rights_review_bead: json_field(s, "rights_review_bead"),
            rights_review_reason: json_field(s, "rights_review_reason"),
            evidence,
        });
    }
    Ok(out)
}

fn toml_field(v: &toml::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
}

/// One `*.meta.toml` sidecar, as a record.
pub fn record_from_sidecar(text: &str, origin: &str) -> Result<Record, String> {
    let doc: toml::Value = toml::from_str(text).map_err(|e| format!("{origin}: {e}"))?;
    let id = toml_field(&doc, "source_id")
        .ok_or_else(|| format!("{origin}: no `source_id` — a sidecar must name its source"))?;
    let evidence = doc.get("redistribution_evidence").map(|e| Evidence {
        licence: toml_field(e, "licence").unwrap_or_default(),
        url: toml_field(e, "url").unwrap_or_default(),
        clause: toml_field(e, "clause").unwrap_or_default(),
    });
    Ok(Record {
        id,
        origin: origin.to_string(),
        rights: toml_field(&doc, "rights"),
        redistribution: toml_field(&doc, "redistribution"),
        ai_ingestion: toml_field(&doc, "ai_ingestion"),
        capture: toml_field(&doc, "capture"),
        path: toml_field(&doc, "path"),
        rights_review: toml_field(&doc, "rights_review"),
        rights_review_bead: toml_field(&doc, "rights_review_bead"),
        rights_review_reason: toml_field(&doc, "rights_review_reason"),
        evidence,
    })
}

// ── evaluation ─────────────────────────────────────────────────────────────

/// What one pass over the corpus found.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Report {
    /// Assertion failures: the tree contradicts its own metadata.
    pub violations: Vec<String>,
    /// The checker could not honestly evaluate.
    pub errors: Vec<String>,
    pub records_scanned: usize,
    pub files_scanned: usize,
    pub bodies_retained: usize,
}

/// Is `p` inside one of the published roots?
pub fn under_published_root(policy: &Policy, p: &str) -> bool {
    let p = p.trim().trim_start_matches("./");
    policy
        .roots
        .iter()
        .any(|r| p == r.as_str() || p.starts_with(&format!("{r}/")))
}

/// Every regular file under the published roots, engine-root-relative, sorted.
///
/// A root that is missing or cannot be listed is returned as an error rather
/// than contributing zero files in silence.
pub fn walk_published(root: &Path, policy: &Policy) -> (Vec<String>, Vec<String>) {
    let mut files = Vec::new();
    let mut errs = Vec::new();
    for rel in &policy.roots {
        let dir = root.join(rel);
        if !dir.is_dir() {
            errs.push(format!(
                "published root missing: {rel} — a root that is not there contributes zero files in silence"
            ));
            continue;
        }
        let mut stack = vec![(dir, rel.clone())];
        let mut listed_any = false;
        while let Some((d, disp)) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else {
                errs.push(format!("published root unreadable: {disp}"));
                continue;
            };
            listed_any = true;
            let mut entries: Vec<(String, PathBuf)> = rd
                .flatten()
                .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
                .collect();
            entries.sort();
            for (name, p) in entries {
                let child = format!("{disp}/{name}");
                // ABSENT-OK: walk type-filter; only directories are descended
                // and only regular files are collected. Other entry types are
                // not inputs this checker checks. Names only — never open.
                if p.is_dir() {
                    stack.push((p, child));
                } else if p.is_file() {
                    files.push(child);
                }
            }
        }
        if !listed_any {
            errs.push(format!("published root unreadable: {rel}"));
        }
    }
    files.sort();
    (files, errs)
}

/// Is this file under a published root a registry file rather than a capture?
pub fn is_registry_file(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    REGISTRY_FILE_NAMES.contains(&name) || name.ends_with(SIDECAR_SUFFIX)
}

/// The whole verdict, given the records, the policy, and a way to ask whether
/// a declared path exists. `exists` is injected so unit tests can drive every
/// leg without a temp tree; the checker passes the real filesystem.
pub fn evaluate(
    records: &[Record],
    policy: &Policy,
    published_files: &[String],
    exists: &dyn Fn(&str) -> bool,
) -> Report {
    let _ = NEVER_OPENS_CAPTURE_BODIES;
    let mut rep = Report {
        records_scanned: records.len(),
        files_scanned: published_files.len(),
        ..Default::default()
    };

    let mut seen: BTreeMap<&str, &Record> = BTreeMap::new();
    for r in records {
        if let Some(prev) = seen.get(r.id.as_str()) {
            let a = (
                prev.rights.as_deref().map(canon),
                prev.redistribution.as_deref().map(canon),
                prev.ai_ingestion.as_deref().map(canon),
            );
            let b = (
                r.rights.as_deref().map(canon),
                r.redistribution.as_deref().map(canon),
                r.ai_ingestion.as_deref().map(canon),
            );
            if a != b {
                rep.errors.push(format!(
                    "{}: id {:?} also appears in {} with different rights/redistribution/ai_ingestion — one of the two is wrong and the tree does not say which",
                    r.origin, r.id, prev.origin
                ));
            }
        } else {
            seen.insert(r.id.as_str(), r);
        }
    }

    let mut claimed: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for r in records {
        let where_ = format!("{}: {}", r.origin, r.id);

        let mut vocab_ok = true;
        for (field, allowed) in [
            ("rights", &policy.rights),
            ("redistribution", &policy.redistribution),
            ("capture", &policy.capture),
        ] {
            match r.field(field) {
                None => {
                    rep.errors.push(format!(
                        "{where_}: CR-R2 missing `{field}` — absence is never permission"
                    ));
                    vocab_ok = false;
                }
                Some(v) if v.trim().is_empty() => {
                    rep.errors.push(format!(
                        "{where_}: CR-R2 empty `{field}` — absence is never permission"
                    ));
                    vocab_ok = false;
                }
                Some(v) if !allowed.contains(&norm(v)) => {
                    rep.errors.push(format!(
                        "{where_}: CR-R2 `{field}` = {v:?} is not in [vocabulary].{field} of {POLICY_PATH} — an unrecognised value must never read as permission"
                    ));
                    vocab_ok = false;
                }
                Some(_) => {}
            }
        }
        if let Some(ai) = r.ai_ingestion.as_deref() {
            if !policy.ai_ingestion.contains(&norm(ai)) {
                rep.errors.push(format!(
                    "{where_}: CR-R2 `ai_ingestion` = {ai:?} is not in [vocabulary].ai_ingestion of {POLICY_PATH}"
                ));
                vocab_ok = false;
            }
        }

        if r.rights_review.as_deref().map(norm).as_deref() == Some("open") {
            let bead = r.rights_review_bead.as_deref().unwrap_or("").trim();
            let reason = r.rights_review_reason.as_deref().unwrap_or("").trim();
            if bead.is_empty() || reason.is_empty() {
                rep.errors.push(format!(
                    "{where_}: CR-R4 rights_review=\"OPEN\" needs a non-empty rights_review_bead AND rights_review_reason — a bare exemption is a schema error, not a pass"
                ));
            }
        }

        let declared = r.path.as_deref().map(str::trim).filter(|p| !p.is_empty());
        let in_tree = declared
            .map(|p| under_published_root(policy, p))
            .unwrap_or(false);
        let on_disk = matches!(declared, Some(p) if exists(p));
        if let Some(p) = declared {
            claimed
                .entry(p.trim_start_matches("./").to_string())
                .or_default()
                .push(r.id.clone());
        }
        if r.is_body_retained() {
            rep.bodies_retained += 1;
        }

        if !vocab_ok {
            continue;
        }

        if !r.redistribution_permitted() {
            let value = r.redistribution.as_deref().unwrap_or("");
            if r.is_body_retained() {
                rep.violations.push(format!(
                    "{where_}: CR-R1 redistribution={value:?} with capture=\"body-retained\" — retaining a capture and publishing it are different acts, and this tree is published"
                ));
            }
            if in_tree && on_disk {
                rep.violations.push(format!(
                    "{where_}: CR-R1 redistribution={value:?} but the body is present at {} — remove it or record the licence that permits it",
                    declared.unwrap_or("")
                ));
            }
        }

        if r.ai_prohibited() {
            if r.is_body_retained() {
                rep.violations.push(format!(
                    "{where_}: CR-R6 ai_ingestion=\"PROHIBITED\" with capture=\"body-retained\" — this repository is AI-built, so an in-tree body is ingested by construction"
                ));
            }
            if in_tree && on_disk {
                rep.violations.push(format!(
                    "{where_}: CR-R6 ai_ingestion=\"PROHIBITED\" but the body is present at {}",
                    declared.unwrap_or("")
                ));
            }
        }

        if r.is_body_retained() && r.redistribution_permitted() {
            let rights = r.rights.as_deref().map(norm).unwrap_or_default();
            if !policy.self_evidencing.contains(&rights) {
                let complete = r.evidence.as_ref().map(Evidence::complete).unwrap_or(false);
                if !complete {
                    let _ = CR_R7;
                    rep.violations.push(format!(
                        "{where_}: {CR_R7} rights={rights:?} with redistribution=\"permitted\" and no complete redistribution_evidence {{licence, url, clause}} — a bare \"permitted\" over someone else's copyright is an assertion, not a licence"
                    ));
                }
            }
            match declared {
                None => rep.errors.push(format!(
                    "{where_}: capture=\"body-retained\" with no `path` — the record cannot be checked against the tree"
                )),
                Some(p) if !on_disk => rep.violations.push(format!(
                    "{where_}: capture=\"body-retained\" but {p} is not in the tree — the record and the tree disagree"
                )),
                Some(_) => {}
            }
        }
    }

    for f in published_files {
        if is_registry_file(f) {
            continue;
        }
        if !claimed.contains_key(f) {
            let _ = CR_R8;
            rep.errors.push(format!(
                "{CR_R8} {f} is in the published tree and no corpus record claims it — a capture with no metadata is an ERROR, never a pass"
            ));
        }
    }
    for (p, ids) in &claimed {
        if ids.len() > 1 {
            let _ = CR_R8;
            rep.errors.push(format!(
                "{CR_R8} {p} is claimed by {} records ({}) — a body must have exactly one owning record",
                ids.len(),
                ids.join(", ")
            ));
        }
    }

    let ids: BTreeSet<&str> = records.iter().map(|r| r.id.as_str()).collect();
    for (bead, recs) in &policy.open_violations {
        if bead.trim().is_empty() {
            rep.errors.push(format!(
                "{POLICY_PATH}: an [[open_violation]] row has no bead — an exception without an owner is not tracked"
            ));
        }
        for r in recs {
            if !ids.contains(r.as_str()) {
                rep.errors.push(format!(
                    "{POLICY_PATH}: [[open_violation]] names {r:?}, which is not a corpus record — the list has rotted"
                ));
            }
        }
    }

    rep
}

// ── the product check ──────────────────────────────────────────────────────

/// Why the published tree could not be certified.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RightsError {
    /// Schema / vacuity / unreadable input. Not a verdict about a body.
    #[error("{0}")]
    Error(String),
    /// The tree contradicts its own metadata.
    #[error("{}", .0.join(" | "))]
    Violation(Vec<String>),
}

/// Clean result of [`check_corpus_rights`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RightsReport {
    /// Records scanned (manifest + sidecars).
    pub records_scanned: usize,
    /// Manifest `sources[]` count.
    pub manifest_sources: usize,
    /// `*.meta.toml` sidecars loaded.
    pub sidecars: usize,
    /// Regular files under published roots.
    pub files_scanned: usize,
    /// Records with `capture = "body-retained"`.
    pub bodies_retained: usize,
    /// Engine-root-relative published roots.
    pub roots: Vec<String>,
    /// `[[open_violation]]` rows (grant no exemption; checked for rot).
    pub open_violation_rows: usize,
}

impl RightsReport {
    /// A live report always scanned at least one record and one file.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.records_scanned > 0 && self.files_scanned > 0
    }
}

impl fmt::Display for RightsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{NAME}: ok: records={} (manifest={} sidecars={}) published_files={} bodies_retained={} roots={} open_violation_rows={}",
            self.records_scanned,
            self.manifest_sources,
            self.sidecars,
            self.files_scanned,
            self.bodies_retained,
            self.roots.join(","),
            self.open_violation_rows,
        )?;
        if self.open_violation_rows > 0 {
            writeln!(
                f,
                "{NAME}: note: {} [[open_violation]] row(s) recorded in {POLICY_PATH}. They grant no exemption here and were checked only for rot.",
                self.open_violation_rows
            )?;
        }
        Ok(())
    }
}

/// Scan `root` for the published corpus vs `rights-policy.toml`.
///
/// Reads policy, manifest, sidecar metadata, and directory entries.
/// **Never opens a capture body** — only `exists()` on a declared `path`.
pub fn check_corpus_rights(root: &Path) -> Result<RightsReport, RightsError> {
    let _ = NEVER_OPENS_CAPTURE_BODIES;
    let policy_text = std::fs::read_to_string(root.join(POLICY_PATH))
        .map_err(|e| RightsError::Error(format!("read {POLICY_PATH}: {e}")))?;
    let policy = parse_policy(&policy_text).map_err(RightsError::Error)?;
    let floor = policy_floor_errors(&policy);
    if !floor.is_empty() {
        return Err(RightsError::Error(floor.join(" | ")));
    }

    let manifest_text = std::fs::read_to_string(root.join(MANIFEST_PATH))
        .map_err(|e| RightsError::Error(format!("read {MANIFEST_PATH}: {e}")))?;
    let manifest = parse_json(&manifest_text)
        .map_err(|e| RightsError::Error(format!("{MANIFEST_PATH}: {e}")))?;
    let mut records =
        records_from_manifest(&manifest, MANIFEST_PATH).map_err(RightsError::Error)?;
    if records.is_empty() {
        return Err(RightsError::Error(format!(
            "{MANIFEST_PATH}: zero sources — an empty manifest is an ERROR, never a vacuous pass"
        )));
    }
    let manifest_sources = records.len();

    let sidecar_dir = root.join(SIDECAR_DIR);
    let rd = std::fs::read_dir(&sidecar_dir)
        .map_err(|e| RightsError::Error(format!("read {SIDECAR_DIR}: {e}")))?;
    let mut sidecar_paths: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(SIDECAR_SUFFIX))
                    .unwrap_or(false)
        })
        .collect();
    sidecar_paths.sort();
    if sidecar_paths.is_empty() {
        return Err(RightsError::Error(format!(
            "{SIDECAR_DIR}: zero *{SIDECAR_SUFFIX} sidecars — a corpus that was never checked must not report like one that passed"
        )));
    }
    let sidecars = sidecar_paths.len();
    for p in &sidecar_paths {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let origin = format!("{SIDECAR_DIR}/{name}");
        let text = std::fs::read_to_string(p)
            .map_err(|e| RightsError::Error(format!("read {origin}: {e}")))?;
        records.push(record_from_sidecar(&text, &origin).map_err(RightsError::Error)?);
    }

    let (files, walk_errs) = walk_published(root, &policy);
    if !walk_errs.is_empty() {
        return Err(RightsError::Error(walk_errs.join(" | ")));
    }
    if files.is_empty() {
        return Err(RightsError::Error(
            "zero files found under the published roots — a walk that found nothing is an ERROR, not a clean tree".into(),
        ));
    }

    // exists() only — NEVER open the capture. AI-ingestion forbid.
    let exists = |rel: &str| root.join(rel).exists();
    let rep = evaluate(&records, &policy, &files, &exists);

    if !rep.errors.is_empty() {
        return Err(RightsError::Error(rep.errors.join(" | ")));
    }
    if !rep.violations.is_empty() {
        return Err(RightsError::Violation(rep.violations));
    }

    Ok(RightsReport {
        records_scanned: rep.records_scanned,
        manifest_sources,
        sidecars,
        files_scanned: rep.files_scanned,
        bodies_retained: rep.bodies_retained,
        roots: policy.roots,
        open_violation_rows: policy.open_violations.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Policy {
        Policy {
            capture: ["body-retained", "citation-only", "not-vendored"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            rights: [
                "public-domain",
                "open-licence",
                "own-work-this-repo",
                "publisher-retains-copyright",
                "unknown",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            redistribution: ["permitted", "not-licensed", "unknown"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ai_ingestion: ["permitted", "prohibited", "unknown"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            self_evidencing: ["public-domain", "own-work-this-repo"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            roots: vec![
                "knowledge/corpus/public".into(),
                "knowledge/corpus/free-pdfs".into(),
            ],
            open_violations: vec![],
        }
    }

    fn rec(id: &str) -> Record {
        Record {
            id: id.into(),
            origin: MANIFEST_PATH.into(),
            rights: Some("publisher-retains-copyright".into()),
            redistribution: Some("not-licensed".into()),
            capture: Some("citation-only".into()),
            ..Default::default()
        }
    }

    const P: &str = "knowledge/corpus/public/src-x.txt";

    fn here(_: &str) -> bool {
        true
    }
    fn nowhere(_: &str) -> bool {
        false
    }

    fn production_src() -> &'static str {
        include_str!("corpus_rights.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests")
    }

    #[test]
    fn production_never_opens_capture_bodies() {
        let src = production_src();
        assert!(src.contains("NEVER_OPENS_CAPTURE_BODIES"));
        assert!(
            src.contains(".exists()"),
            "body presence must be exists(), not a read"
        );
        assert!(
            !src.contains("std::fs::read("),
            "std::fs::read would open a capture body"
        );
        assert!(!src.contains("File::open"), "File::open would open a body");
        // The only read_to_string targets are policy, manifest, sidecar.
        assert!(src.contains("read_to_string(root.join(POLICY_PATH))"));
        assert!(src.contains("read_to_string(root.join(MANIFEST_PATH))"));
        assert!(src.contains("read_to_string(p)"));
    }

    #[test]
    fn clean_citation_row_passes() {
        let r = evaluate(&[rec("src-x")], &policy(), &[], &nowhere);
        assert!(r.errors.is_empty() && r.violations.is_empty(), "{r:?}");
    }

    #[test]
    fn not_licensed_body_in_the_tree_is_a_violation_naming_file_and_field() {
        let mut r = rec("src-x");
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(out.violations.iter().any(|v| v.contains("CR-R1")));
        assert!(out.violations.iter().any(|v| v.contains(P)));
        assert!(out.violations.iter().any(|v| v.contains("redistribution")));
    }

    #[test]
    fn missing_redistribution_is_an_error_not_a_pass() {
        let mut r = rec("src-x");
        r.redistribution = None;
        let out = evaluate(&[r], &policy(), &[], &nowhere);
        assert!(out
            .errors
            .iter()
            .any(|e| e.contains("CR-R2") && e.contains("redistribution")));
        assert!(out.violations.is_empty(), "R2 is an ERROR, not a verdict");
    }

    #[test]
    fn an_invented_redistribution_value_is_not_permission() {
        let mut r = rec("src-x");
        r.redistribution = Some("probably-fine".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(out.errors.iter().any(|e| e.contains("probably-fine")));
    }

    #[test]
    fn ai_prohibited_body_is_a_violation() {
        let mut r = rec("src-x");
        r.ai_ingestion = Some("PROHIBITED".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(out.violations.iter().any(|v| v.contains("CR-R6")));
    }

    #[test]
    fn bare_permitted_over_publisher_copyright_needs_a_licence() {
        let mut r = rec("src-x");
        r.redistribution = Some("permitted".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(out.violations.iter().any(|v| v.contains(CR_R7)));
    }

    #[test]
    fn a_recorded_licence_justification_passes() {
        let mut r = rec("src-x");
        r.rights = Some("open-licence".into());
        r.redistribution = Some("permitted".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        r.evidence = Some(Evidence {
            licence: "CC-BY-4.0".into(),
            url: "https://creativecommons.org/licenses/by/4.0/legalcode".into(),
            clause: "§2(a)(1)".into(),
        });
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(
            out.errors.is_empty() && out.violations.is_empty(),
            "{out:?}"
        );
    }

    #[test]
    fn own_work_needs_no_licence_citation() {
        let mut r = rec("src-x");
        r.rights = Some("own-work-this-repo".into());
        r.redistribution = Some("permitted".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(
            out.errors.is_empty() && out.violations.is_empty(),
            "{out:?}"
        );
    }

    #[test]
    fn public_domain_body_retained_needs_no_licence_citation() {
        let mut r = rec("src-x");
        r.rights = Some("public-domain".into());
        r.redistribution = Some("permitted".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(
            out.errors.is_empty() && out.violations.is_empty(),
            "known-good public-domain body-retained must PASS: {out:?}"
        );
    }

    #[test]
    fn an_unclaimed_file_is_an_error() {
        let out = evaluate(
            &[rec("src-x")],
            &policy(),
            &["knowledge/corpus/public/src-stowaway.txt".to_string()],
            &here,
        );
        assert!(out
            .errors
            .iter()
            .any(|e| e.contains(CR_R8) && e.contains("src-stowaway")));
    }

    #[test]
    fn registry_files_are_not_unclaimed_captures() {
        let out = evaluate(
            &[rec("src-x")],
            &policy(),
            &[
                "knowledge/corpus/public/manifest.json".to_string(),
                "knowledge/corpus/free-pdfs/a.meta.toml".to_string(),
            ],
            &here,
        );
        assert!(out.errors.is_empty(), "{out:?}");
    }

    #[test]
    fn bare_exemption_is_a_schema_error() {
        let mut r = rec("src-x");
        r.rights_review = Some("OPEN".into());
        r.rights_review_reason = Some("".into());
        let out = evaluate(&[r], &policy(), &[], &nowhere);
        assert!(out.errors.iter().any(|e| e.contains("CR-R4")));
    }

    #[test]
    fn two_rows_for_one_id_that_disagree_are_an_error() {
        let a = rec("src-dup");
        let mut b = rec("src-dup");
        b.origin = "knowledge/corpus/free-pdfs/x.meta.toml".into();
        b.redistribution = Some("permitted".into());
        b.rights = Some("public-domain".into());
        let out = evaluate(&[a, b], &policy(), &[], &nowhere);
        assert!(out.errors.iter().any(|e| e.contains("src-dup")));
    }

    #[test]
    fn legacy_uppercase_spelling_is_the_same_value() {
        let mut r = rec("src-x");
        r.redistribution = Some("NOT-licensed".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[P.to_string()], &here);
        assert!(
            out.errors.is_empty(),
            "vocabulary is case-insensitive: {out:?}"
        );
        assert!(out.violations.iter().any(|v| v.contains("CR-R1")));
    }

    #[test]
    fn policy_may_not_drop_a_root_or_widen_self_evidencing() {
        let mut p = policy();
        p.roots = vec!["knowledge/corpus/public".into()];
        assert!(policy_floor_errors(&p)
            .iter()
            .any(|e| e.contains("free-pdfs")));

        let mut p = policy();
        p.self_evidencing
            .insert("publisher-retains-copyright".into());
        assert!(policy_floor_errors(&p)
            .iter()
            .any(|e| e.contains("publisher-retains-copyright")));

        assert!(policy_floor_errors(&policy()).is_empty());
    }

    #[test]
    fn a_stale_open_violation_entry_is_an_error() {
        let mut p = policy();
        p.open_violations = vec![("bd-x".into(), vec!["src-ghost".into()])];
        let out = evaluate(&[rec("src-x")], &p, &[], &nowhere);
        assert!(out.errors.iter().any(|e| e.contains("src-ghost")));
    }

    #[test]
    fn an_open_violation_grants_no_exemption() {
        let mut p = policy();
        p.open_violations = vec![("bd-x".into(), vec!["src-x".into()])];
        let mut r = rec("src-x");
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &p, &[P.to_string()], &here);
        assert!(
            out.violations.iter().any(|v| v.contains("CR-R1")),
            "a tracked violation is still a violation: {out:?}"
        );
    }

    #[test]
    fn body_retained_pointing_at_nothing_is_a_violation() {
        let mut r = rec("src-x");
        r.rights = Some("public-domain".into());
        r.redistribution = Some("permitted".into());
        r.capture = Some("body-retained".into());
        r.path = Some(P.into());
        let out = evaluate(&[r], &policy(), &[], &nowhere);
        assert!(out.violations.iter().any(|v| v.contains("disagree")));
    }

    #[test]
    fn json_round_trips_the_shapes_the_manifest_uses() {
        let doc = parse_json(
            r#"{"sources":[{"id":"a","rights":"public-domain","bytes":12,"ok":true,
                 "redistribution_evidence":{"licence":"L","url":"U","clause":"C"}}]}"#,
        )
        .expect("parses");
        let recs = records_from_manifest(&doc, "m").expect("records");
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].id, "a");
        assert_eq!(recs[0].evidence.as_ref().unwrap().licence, "L");
    }

    #[test]
    fn truncated_json_is_an_error_not_a_short_manifest() {
        assert!(parse_json(r#"{"sources":[{"id":"a"}"#).is_err());
        assert!(parse_json(r#"{"sources":[]} trailing"#).is_err());
    }

    #[test]
    fn empty_vocabulary_is_rejected() {
        let text = r#"
[vocabulary]
capture = []
rights = ["public-domain"]
redistribution = ["permitted"]
ai_ingestion = ["permitted"]
self_evidencing_rights = ["public-domain"]
[published_tree]
roots = ["course-engine/knowledge/corpus/public"]
"#;
        let e = parse_policy(text).unwrap_err();
        assert!(e.contains("empty"), "{e}");
    }

    #[test]
    fn policy_roots_are_engine_relative() {
        let text = r#"
[vocabulary]
capture = ["citation-only"]
rights = ["public-domain"]
redistribution = ["permitted"]
ai_ingestion = ["permitted"]
self_evidencing_rights = ["public-domain"]
[published_tree]
roots = ["course-engine/knowledge/corpus/public", "course-engine/knowledge/corpus/free-pdfs"]
"#;
        let p = parse_policy(text).expect("parses");
        assert_eq!(p.roots, REQUIRED_ROOTS);
        assert!(under_published_root(&p, "knowledge/corpus/public/x.txt"));
        assert!(!under_published_root(&p, "knowledge/other/x.txt"));
    }
}
