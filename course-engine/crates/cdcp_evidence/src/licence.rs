//! Licence policy: the three-field `rights` / `redistribution` / `ai_ingestion` split.
//!
//! This is a product check, not a gate. Snapshot loaders (E1) call
//! [`may_load`]. Public CI calls [`scan_engine`]. Classification reads
//! `.meta.toml` and directory listings only — never a capture body.
//! `ai_ingestion=PROHIBITED` sources forbid an AI tool from opening the
//! bytes; a rights check that had to read those bytes would be
//! self-defeating.

use crate::records::AiIngestion;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Token interpolated inside [`evaluate_artifact`]. Deleting the published-path
/// check makes the matching selftest non-zero.
pub const R1_PUBLISHED_UNLICENSED: &str = "redistribution != permitted under a published path";

/// Token interpolated inside [`evaluate_artifact`]. Deleting the missing-field
/// check makes the matching selftest non-zero.
pub const R2_MISSING_RIGHTS: &str =
    "missing licence/rights field is an ERROR, never default-permissive";

/// Token interpolated inside [`evaluate_artifact`]. Deleting the third-party
/// check makes the matching selftest non-zero.
pub const R3_THIRD_PARTY_PUBLIC_DOMAIN: &str =
    "an artifact declaring third_party_figures may NOT be marked redistribution=public-domain";

/// Token interpolated inside [`audit_index`] and
/// [`ArtifactMeta::eligible_for_agent_index`]. Deleting the exclusion makes
/// the matching selftest non-zero.
pub const R4_PROHIBITED_INDEX: &str =
    "ai_ingestion=PROHIBITED artifacts are excluded from any agent-reachable corpus index";

/// Token interpolated inside [`scan`]. Deleting the empty-scan check makes
/// the matching selftest non-zero.
pub const ANTI_VACUOUS: &str = "zero artifacts scanned is an ERROR";

/// Engine-root-relative published trees. Committing here is publication.
pub const DEFAULT_PUBLISHED_ROOTS: &[&str] =
    &["knowledge/corpus/public", "knowledge/corpus/free-pdfs"];

/// File names treated as an agent-reachable corpus index.
pub const DEFAULT_INDEX_NAMES: &[&str] = &["agent-index.toml", "corpus-index.toml"];

/// Suffix marking a rights sidecar rather than a capture body.
pub const SIDECAR_SUFFIX: &str = ".meta.toml";

/// The only string that means redistribution permission. Compared after [`norm`].
pub const PERMITTED: &str = "permitted";

/// The rights / (mis-filed) redistribution value that 17 USC 105 names.
pub const PUBLIC_DOMAIN: &str = "public-domain";

/// Engine-root anchor used by [`resolve_engine_root`].
pub const ENGINE_ANCHOR: &str = "registries/claims.toml";

/// Why a scan could not even start.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LicenceError {
    /// Filesystem failure on a path the checker has to read.
    #[error("cannot read {path}: {detail}")]
    Io {
        /// Path that failed.
        path: String,
        /// Underlying error.
        detail: String,
    },
}

/// One licence-policy finding. Absence of a field is never permission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenceFault {
    /// A body sits under a published root while its sidecar forbids redistribution.
    PublishedUnlicensed {
        /// Body path, engine-root-relative, named so the finding is actionable.
        path: String,
        /// Recorded redistribution value (not `permitted`).
        redistribution: String,
    },
    /// `.meta.toml` is missing the licence/rights line.
    MissingRights {
        /// Sidecar origin.
        origin: String,
        /// Which field was absent (`rights`/`licence`, `redistribution`, `ai_ingestion`).
        field: &'static str,
    },
    /// `third_party_figures` plus a public-domain / permitted redistribution mark.
    ThirdPartyPublicDomain {
        /// Sidecar origin.
        origin: String,
        /// The illegal mark (`public-domain` or `permitted`).
        marked: String,
    },
    /// A PROHIBITED artifact appears in an agent-reachable index.
    ProhibitedInAgentIndex {
        /// Source id that must not be indexed.
        id: String,
        /// Index file that listed it.
        index: String,
    },
    /// The scan looked at no `.meta.toml`. Green here would be vacuous.
    VacuousScan,
    /// A configured published root is not a directory.
    MissingPublishedRoot {
        /// Root that was configured.
        root: String,
    },
    /// A sidecar or index could not be parsed.
    Unparseable {
        /// Path that failed.
        path: String,
        /// Parser detail.
        detail: String,
    },
}

impl fmt::Display for LicenceFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LicenceFault::PublishedUnlicensed {
                path,
                redistribution,
            } => write!(
                f,
                "{R1_PUBLISHED_UNLICENSED}: {path} (redistribution={redistribution})"
            ),
            LicenceFault::MissingRights { origin, field } => {
                write!(f, "{R2_MISSING_RIGHTS}: {origin} missing `{field}`")
            }
            LicenceFault::ThirdPartyPublicDomain { origin, marked } => {
                write!(
                    f,
                    "{R3_THIRD_PARTY_PUBLIC_DOMAIN}: {origin} marked {marked}"
                )
            }
            LicenceFault::ProhibitedInAgentIndex { id, index } => {
                write!(f, "{R4_PROHIBITED_INDEX}: {id} in {index}")
            }
            LicenceFault::VacuousScan => write!(f, "{ANTI_VACUOUS}"),
            LicenceFault::MissingPublishedRoot { root } => {
                write!(f, "published root missing: {root}")
            }
            LicenceFault::Unparseable { path, detail } => {
                write!(f, "unparseable {path}: {detail}")
            }
        }
    }
}

/// Parsed `.meta.toml`. Missing fields stay `None` so the predicate can go RED
/// — a constructor that defaulted them to `permitted` would hide the plant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactMeta {
    id: String,
    origin: String,
    rights: Option<String>,
    licence: Option<String>,
    redistribution: Option<String>,
    ai_ingestion_raw: Option<String>,
    ai_ingestion: Option<AiIngestion>,
    capture: Option<String>,
    path: Option<String>,
    third_party_figures: Vec<String>,
}

impl ArtifactMeta {
    /// Source id (`source_id`).
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Sidecar path or other origin label.
    #[must_use]
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// `rights` as recorded.
    #[must_use]
    pub fn rights(&self) -> Option<&str> {
        self.rights.as_deref()
    }

    /// Explicit `licence` line, if any.
    #[must_use]
    pub fn licence(&self) -> Option<&str> {
        self.licence.as_deref()
    }

    /// `redistribution` as recorded.
    #[must_use]
    pub fn redistribution(&self) -> Option<&str> {
        self.redistribution.as_deref()
    }

    /// Parsed AI-ingestion vocabulary.
    #[must_use]
    pub fn ai_ingestion(&self) -> Option<AiIngestion> {
        self.ai_ingestion
    }

    /// `capture` as recorded.
    #[must_use]
    pub fn capture(&self) -> Option<&str> {
        self.capture.as_deref()
    }

    /// Declared body path, if any.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Third-party figure credit lines. Content of those figures is not here.
    #[must_use]
    pub fn third_party_figures(&self) -> &[String] {
        &self.third_party_figures
    }

    /// True when `rights` or `licence` is a non-empty line.
    #[must_use]
    pub fn has_licence_or_rights(&self) -> bool {
        nonempty(self.rights.as_deref()) || nonempty(self.licence.as_deref())
    }

    /// True only for the compiled-in permission string.
    #[must_use]
    pub fn redistribution_is_permitted(&self) -> bool {
        self.redistribution
            .as_deref()
            .is_some_and(|v| norm(v) == PERMITTED)
    }

    /// True when an agent-reachable index may list this artifact.
    ///
    /// Fail-closed: missing fields, `unknown`, and `PROHIBITED` are all
    /// exclusions. Deleting the `Prohibited` arm is what the R4 selftest
    /// keys on.
    #[must_use]
    pub fn eligible_for_agent_index(&self) -> bool {
        let _ = R4_PROHIBITED_INDEX;
        if !self.has_licence_or_rights() || !self.redistribution_is_permitted() {
            return false;
        }
        match self.ai_ingestion {
            Some(AiIngestion::Permitted) => true,
            Some(AiIngestion::Prohibited) | Some(AiIngestion::Unknown) | None => false,
        }
    }
}

/// Agent-reachable corpus index: source ids an agent may ingest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorpusIndex {
    ids: BTreeSet<String>,
}

impl CorpusIndex {
    /// Empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from an iterator of ids. Does **not** filter — [`audit_index`]
    /// is what goes RED on a planted PROHIBITED id.
    #[must_use]
    pub fn from_ids<I, S>(ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CorpusIndex {
            ids: ids.into_iter().map(Into::into).collect(),
        }
    }

    /// Recorded ids.
    #[must_use]
    pub fn ids(&self) -> &BTreeSet<String> {
        &self.ids
    }

    /// True when `id` is listed.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains(id)
    }
}

/// Scan configuration. Tests point this at in-tree fixtures.
#[derive(Debug, Clone, Copy)]
pub struct ScanRequest<'a> {
    /// Tree root (engine root, or a fixture root with the same layout).
    pub root: &'a Path,
    /// Engine-root-relative published directories.
    pub published_roots: &'a [&'a str],
    /// File names treated as agent-reachable indexes.
    pub index_names: &'a [&'a str],
}

/// Outcome of one scan. [`LicenceReport::is_clean`] is the verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenceReport {
    /// Number of `.meta.toml` sidecars parsed (or attempted).
    pub scanned: usize,
    /// Findings. Empty means clean.
    pub faults: Vec<LicenceFault>,
}

impl LicenceReport {
    /// True when there are no faults.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.faults.is_empty()
    }

    /// Process exit status: 0 clean, 1 red.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        if self.is_clean() {
            0
        } else {
            1
        }
    }
}

impl fmt::Display for LicenceReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_clean() {
            writeln!(f, "check_licence: PASS scanned={} faults=0", self.scanned)?;
        } else {
            writeln!(
                f,
                "check_licence: FAIL scanned={} faults={}",
                self.scanned,
                self.faults.len()
            )?;
            for fault in &self.faults {
                writeln!(f, "  {fault}")?;
            }
        }
        Ok(())
    }
}

/// Case-insensitive, trimmed comparison key. `NOT-licensed` and
/// `not-licensed` are one value; a case difference must never mint permission.
#[must_use]
pub fn norm(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Parse a `.meta.toml`. Invalid TOML is an error. Missing rights stay
/// missing — that is the R2 plant.
pub fn parse_meta_toml(text: &str, origin: &str) -> Result<ArtifactMeta, LicenceFault> {
    let doc: toml::Value = toml::from_str(text).map_err(|e| LicenceFault::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;
    let id = toml_string(&doc, "source_id").ok_or_else(|| LicenceFault::Unparseable {
        path: origin.to_string(),
        detail: "no `source_id` — a sidecar must name its source".into(),
    })?;
    let ai_raw = toml_string(&doc, "ai_ingestion");
    let ai_ingestion = parse_ai_ingestion(ai_raw.as_deref());
    Ok(ArtifactMeta {
        id,
        origin: origin.to_string(),
        rights: toml_string(&doc, "rights"),
        licence: toml_string(&doc, "licence"),
        redistribution: toml_string(&doc, "redistribution"),
        ai_ingestion_raw: ai_raw,
        ai_ingestion,
        capture: toml_string(&doc, "capture"),
        path: toml_string(&doc, "path"),
        third_party_figures: toml_string_array(&doc, "third_party_figures"),
    })
}

/// Snapshot-loader refusal. Missing licence/rights is a refusal, never a warning.
///
/// Also refuses a missing three-field companion (`redistribution`,
/// `ai_ingestion`), `PROHIBITED` ingestion, and anything that is not
/// `redistribution=permitted`. E1 calls this; it does not default-permissive.
pub fn may_load(meta: &ArtifactMeta) -> Result<(), LicenceFault> {
    let faults = evaluate_artifact(meta, None);
    // Load refusal is the first structural fault. A clean citation-only
    // sidecar (not-licensed, no body) is still not a snapshot — E1 vendors
    // permitted bodies only.
    if let Some(fault) = faults.into_iter().next() {
        return Err(fault);
    }
    if !meta.redistribution_is_permitted() {
        return Err(LicenceFault::PublishedUnlicensed {
            path: meta.origin.clone(),
            redistribution: meta
                .redistribution
                .clone()
                .unwrap_or_else(|| "<missing>".into()),
        });
    }
    if matches!(meta.ai_ingestion, Some(AiIngestion::Prohibited) | None) {
        return Err(LicenceFault::ProhibitedInAgentIndex {
            id: meta.id.clone(),
            index: "may_load".into(),
        });
    }
    Ok(())
}

/// The four-rule predicate over one sidecar plus an optional published body.
///
/// `body` is a published-tree-relative path that exists on disk. The
/// checker never opens that file.
#[must_use]
pub fn evaluate_artifact(meta: &ArtifactMeta, body: Option<&str>) -> Vec<LicenceFault> {
    let mut faults = Vec::new();

    // R2 — missing licence/rights is an ERROR, never default-permissive.
    // The three-field split also refuses a missing redistribution or
    // ai_ingestion: absence of either companion is how `access=free` used
    // to be read as permission.
    let _ = R2_MISSING_RIGHTS;
    if !meta.has_licence_or_rights() {
        faults.push(LicenceFault::MissingRights {
            origin: meta.origin.clone(),
            field: "rights",
        });
    }
    if !nonempty(meta.redistribution.as_deref()) {
        faults.push(LicenceFault::MissingRights {
            origin: meta.origin.clone(),
            field: "redistribution",
        });
    }
    if !nonempty(meta.ai_ingestion_raw.as_deref()) {
        faults.push(LicenceFault::MissingRights {
            origin: meta.origin.clone(),
            field: "ai_ingestion",
        });
    }

    // R1 — a published body whose sidecar forbids redistribution is RED,
    // naming the file. Citation-only sidecars (no body) are not artifacts
    // under the published path.
    let _ = R1_PUBLISHED_UNLICENSED;
    if !meta.redistribution_is_permitted() {
        if let Some(path) = body.filter(|p| !p.is_empty()) {
            faults.push(LicenceFault::PublishedUnlicensed {
                path: path.to_string(),
                redistribution: meta
                    .redistribution
                    .clone()
                    .unwrap_or_else(|| "<missing>".into()),
            });
        }
    }

    // R3 — third-party figures do not become public domain by being bound
    // into a government wrapper, and they do not authorise `permitted`.
    let _ = R3_THIRD_PARTY_PUBLIC_DOMAIN;
    if !meta.third_party_figures.is_empty() {
        let redist = meta.redistribution.as_deref().map(norm).unwrap_or_default();
        let rights = meta.rights.as_deref().map(norm).unwrap_or_default();
        if redist == PUBLIC_DOMAIN {
            faults.push(LicenceFault::ThirdPartyPublicDomain {
                origin: meta.origin.clone(),
                marked: format!("redistribution={PUBLIC_DOMAIN}"),
            });
        } else if rights == PUBLIC_DOMAIN {
            faults.push(LicenceFault::ThirdPartyPublicDomain {
                origin: meta.origin.clone(),
                marked: format!("rights={PUBLIC_DOMAIN}"),
            });
        } else if redist == PERMITTED {
            faults.push(LicenceFault::ThirdPartyPublicDomain {
                origin: meta.origin.clone(),
                marked: format!("redistribution={PERMITTED}"),
            });
        }
    }

    faults
}

/// Build the index agents may consult. PROHIBITED records never appear.
#[must_use]
pub fn build_agent_reachable_index(records: &[ArtifactMeta]) -> CorpusIndex {
    let _ = R4_PROHIBITED_INDEX;
    CorpusIndex::from_ids(
        records
            .iter()
            .filter(|r| r.eligible_for_agent_index())
            .map(|r| r.id.clone()),
    )
}

/// RED when a proposed index lists a `PROHIBITED` record. The plant is an
/// index that includes such an id — [`build_agent_reachable_index`] will
/// not produce one.
#[must_use]
pub fn audit_index(
    index: &CorpusIndex,
    records: &[ArtifactMeta],
    index_name: &str,
) -> Vec<LicenceFault> {
    let _ = R4_PROHIBITED_INDEX;
    let mut faults = Vec::new();
    for rec in records {
        let prohibited = matches!(rec.ai_ingestion, Some(AiIngestion::Prohibited));
        if prohibited && index.contains(&rec.id) {
            faults.push(LicenceFault::ProhibitedInAgentIndex {
                id: rec.id.clone(),
                index: index_name.to_string(),
            });
        }
    }
    faults
}

/// Walk the published roots under `req.root`.
#[must_use]
pub fn scan(req: ScanRequest<'_>) -> LicenceReport {
    let _ = ANTI_VACUOUS;
    let mut faults = Vec::new();
    let mut records = Vec::new();
    let mut scanned = 0usize;
    let mut index_paths = Vec::new();

    for rel in req.published_roots {
        let dir = join_rel(req.root, rel);
        if !dir.is_dir() {
            faults.push(LicenceFault::MissingPublishedRoot {
                root: (*rel).to_string(),
            });
            continue;
        }
        let mut metas = Vec::new();
        let mut indexes = Vec::new();
        walk_published(
            &dir,
            rel,
            &mut metas,
            &mut indexes,
            req.index_names,
            &mut faults,
        );
        index_paths.extend(indexes);
        for meta_path in metas {
            scanned += 1;
            let origin = rel_display(req.root, &meta_path);
            let text = match std::fs::read_to_string(&meta_path) {
                Ok(t) => t,
                Err(e) => {
                    faults.push(LicenceFault::Unparseable {
                        path: origin,
                        detail: e.to_string(),
                    });
                    continue;
                }
            };
            match parse_meta_toml(&text, &origin) {
                Ok(meta) => {
                    let body = resolve_body(req.root, req.published_roots, &meta_path, &meta);
                    faults.extend(evaluate_artifact(&meta, body.as_deref()));
                    records.push(meta);
                }
                Err(fault) => faults.push(fault),
            }
        }
    }

    for index_path in index_paths {
        let origin = rel_display(req.root, &index_path);
        match load_index(&index_path, &origin) {
            Ok(index) => faults.extend(audit_index(&index, &records, &origin)),
            Err(fault) => faults.push(fault),
        }
    }

    if scanned == 0 {
        faults.push(LicenceFault::VacuousScan);
    }

    // The builder is the index the product will hand to agents. Audit it
    // too — a regression that started listing PROHIBITED ids is RED even
    // when no index file exists yet.
    let built = build_agent_reachable_index(&records);
    faults.extend(audit_index(&built, &records, "agent-reachable-index"));

    LicenceReport { scanned, faults }
}

/// Scan the live engine layout ([`DEFAULT_PUBLISHED_ROOTS`]).
#[must_use]
pub fn scan_engine(root: &Path) -> LicenceReport {
    scan(ScanRequest {
        root,
        published_roots: DEFAULT_PUBLISHED_ROOTS,
        index_names: DEFAULT_INDEX_NAMES,
    })
}

/// Walk up from `start` looking for [`ENGINE_ANCHOR`].
pub fn resolve_engine_root(start: &Path) -> Result<PathBuf, LicenceError> {
    let mut cur = start.to_path_buf();
    if cur.is_file() {
        cur.pop();
    }
    for _ in 0..12 {
        if cur.join(ENGINE_ANCHOR).is_file() {
            return Ok(cur);
        }
        if !cur.pop() {
            break;
        }
    }
    let from_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if let Ok(canon) = from_manifest.canonicalize() {
        if canon.join(ENGINE_ANCHOR).is_file() {
            return Ok(canon);
        }
    }
    Err(LicenceError::Io {
        path: start.display().to_string(),
        detail: format!("no {ENGINE_ANCHOR} at or above start"),
    })
}

fn nonempty(s: Option<&str>) -> bool {
    s.is_some_and(|v| !v.trim().is_empty())
}

fn parse_ai_ingestion(raw: Option<&str>) -> Option<AiIngestion> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    match norm(s).as_str() {
        "permitted" => Some(AiIngestion::Permitted),
        "prohibited" => Some(AiIngestion::Prohibited),
        _ => Some(AiIngestion::Unknown),
    }
}

fn toml_string(v: &toml::Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn toml_string_array(v: &toml::Value, key: &str) -> Vec<String> {
    match v.get(key) {
        Some(toml::Value::Array(items)) => items
            .iter()
            .filter_map(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        Some(toml::Value::String(s)) if !s.trim().is_empty() => vec![s.trim().to_string()],
        _ => Vec::new(),
    }
}

fn join_rel(root: &Path, rel: &str) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        p.push(part);
    }
    p
}

fn rel_display(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

fn under_published(rel: &str, roots: &[&str]) -> bool {
    let rel = rel.trim().trim_start_matches("./");
    roots
        .iter()
        .any(|r| rel == *r || rel.starts_with(&format!("{r}/")))
}

fn walk_published(
    dir: &Path,
    disp: &str,
    metas: &mut Vec<PathBuf>,
    indexes: &mut Vec<PathBuf>,
    index_names: &[&str],
    faults: &mut Vec<LicenceFault>,
) {
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            faults.push(LicenceFault::Unparseable {
                path: disp.to_string(),
                detail: e.to_string(),
            });
            return;
        }
    };
    let mut entries: Vec<(String, PathBuf)> = rd
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, p) in entries {
        if name.starts_with('.') {
            continue;
        }
        let child = format!("{disp}/{name}");
        if p.is_dir() {
            walk_published(&p, &child, metas, indexes, index_names, faults);
        } else if p.is_file() {
            if name.ends_with(SIDECAR_SUFFIX) {
                metas.push(p);
            } else if index_names.contains(&name.as_str()) {
                indexes.push(p);
            }
        }
    }
}

fn resolve_body(
    root: &Path,
    published_roots: &[&str],
    sidecar: &Path,
    meta: &ArtifactMeta,
) -> Option<String> {
    if let Some(declared) = meta.path.as_deref() {
        let from_root = join_rel(root, declared);
        if from_root.is_file() {
            let rel = rel_display(root, &from_root);
            if under_published(&rel, published_roots) {
                return Some(rel);
            }
        }
    }
    let name = sidecar.file_name()?.to_str()?;
    let stem = name.strip_suffix(SIDECAR_SUFFIX)?;
    let dir = sidecar.parent()?;
    for ext in [".pdf", ".txt", ".bin", ".json", ""] {
        let candidate = if ext.is_empty() {
            dir.join(stem)
        } else {
            dir.join(format!("{stem}{ext}"))
        };
        if candidate.is_file() {
            let rel = rel_display(root, &candidate);
            if under_published(&rel, published_roots) {
                return Some(rel);
            }
        }
    }
    None
}

fn load_index(path: &Path, origin: &str) -> Result<CorpusIndex, LicenceFault> {
    let text = std::fs::read_to_string(path).map_err(|e| LicenceFault::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;
    let doc: toml::Value = toml::from_str(&text).map_err(|e| LicenceFault::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;
    let ids = if let Some(arr) = doc.get("ids") {
        toml_string_array_value(arr)
    } else if let Some(arr) = doc.get("source_ids") {
        toml_string_array_value(arr)
    } else {
        return Err(LicenceFault::Unparseable {
            path: origin.to_string(),
            detail: "index has no `ids` or `source_ids` array".into(),
        });
    };
    Ok(CorpusIndex::from_ids(ids))
}

fn toml_string_array_value(v: &toml::Value) -> Vec<String> {
    match v {
        toml::Value::Array(items) => items
            .iter()
            .filter_map(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}
