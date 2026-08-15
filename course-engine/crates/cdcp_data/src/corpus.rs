//! Offline public-corpus planner.
//!
//! Extracted from `scripts/fetch_public_corpus.py` by
//! `bd-substrate-rust-migration-jhd.36`. The Python is DELETED.
//!
//! The retired script fetched HTTP(S) for every `access` in
//! `{public_summary, free, local}` and wrote `knowledge/corpus/public/<id>.txt`.
//! That network path is **not** the product. This crate never opens a
//! socket (E1 / `tests/network.rs`). Public-page bodies were already
//! reduced to citation rows (bd-corpus-public-captures-not-licensed-class-kej);
//! re-fetching them would republish bytes this tree is not licensed to
//! keep. `cdcp fetch-corpus --dry-run` is the CI / check.sh path.
//!
//! # Contract
//!
//! - Parse `knowledge/sources.toml`. Zero `[[source]]` rows is ERROR.
//! - `access=paid` is refused and **never written** (token [`REFUSED_PAID`]).
//! - Allowed access is only `{public_summary, free, local}`.
//! - `file://` may be copied under `--write`. HTTP(S) is planned
//!   (`would-fetch` / `would-stub-pdf`) and never retrieved.
//! - Write-after-verdict: payloads are built before any dest is created.
//! - Does not write `manifest.json` (the live file is the v2 rights
//!   ledger, not a fetch artifact). Does not edit `content.lock` /
//!   `snapshots.toml`.
//!
//! # What this cannot decide
//!
//! It cannot decide that a `public_summary` page is licensed to keep.
//! It cannot retrieve HTTP. A dry-run that lists `would-fetch` is not
//! a snapshot.

use crate::join_rel;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Engine-root-relative default source ledger.
pub const SOURCES_REL: &str = "knowledge/sources.toml";
/// Engine-root-relative default write directory.
pub const OUT_DIR_REL: &str = "knowledge/corpus/public";

/// Access values the retired script would fetch. Paid is not in this set.
pub const ALLOWED_ACCESS: &[&str] = &["public_summary", "free", "local"];

/// Token interpolated on every paid refusal. Deleting the paid check
/// makes the matching selftest non-zero.
pub const REFUSED_PAID: &str = "access=paid";

/// Token interpolated on the write path. A paid dest that appears on
/// disk after `--write` is a contract break.
pub const NEVER_WRITTEN: &str = "never written";

/// Token interpolated in the HTTP skip. This crate has no transport.
pub const NO_SOCKET: &str = "this crate never opens a socket";

/// Empty `[[source]]` list is ERROR, never a vacuous pass.
pub const ANTI_VACUOUS_EMPTY_SOURCES: &str = "zero sources is an ERROR";

/// Every row paid / unknown is ERROR — a planner that allows nothing
/// certifies nothing.
pub const ANTI_VACUOUS_NONE_ALLOWED: &str = "zero allowed sources is an ERROR";

/// `--write` produced no local copies. HTTP is not a write.
pub const ANTI_VACUOUS_NOTHING_WRITTEN: &str = "zero local copies written is an ERROR";

/// Access classification from `sources.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessKind {
    /// Publisher summary page. Price/visibility fact, not a licence.
    PublicSummary,
    /// Free-to-read at the publisher's URL. Still not a licence.
    Free,
    /// `file://` in this study tree.
    Local,
    /// Paid catalog / full-text. Never fetched, never written.
    Paid,
    /// Anything else. Skipped, never written.
    Unknown(String),
}

impl AccessKind {
    /// True for the three access values the retired script would fetch.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            AccessKind::PublicSummary | AccessKind::Free | AccessKind::Local
        )
    }

    /// Wire spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            AccessKind::PublicSummary => "public_summary",
            AccessKind::Free => "free",
            AccessKind::Local => "local",
            AccessKind::Paid => "paid",
            AccessKind::Unknown(s) => s.as_str(),
        }
    }
}

impl fmt::Display for AccessKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One `[[source]]` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Stable id (`src-…`). Filename stem under the out dir.
    pub id: String,
    /// Publisher / org label.
    pub org: String,
    /// Human title.
    pub title: String,
    /// `https://…` or `file://…`.
    pub url: String,
    /// Access classification.
    pub access: AccessKind,
    /// Ledger `fetch_date` (informational).
    pub fetch_date: String,
    /// Optional note.
    pub note: String,
}

/// What the planner will do with one row. HTTP is never performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanAction {
    /// `file://` exists and may be copied under `--write`.
    CopyLocal {
        /// Absolute path that passed the root/parent confinement check.
        path: PathBuf,
    },
    /// `file://` named a missing file. Skipped, not an ERROR by itself.
    MissingLocal,
    /// HTTP(S) HTML/text. Planned only.
    WouldFetch,
    /// HTTP(S) URL ending in `.pdf`. Planned stub only; body not stored.
    WouldStubPdf,
    /// [`REFUSED_PAID`] — dest must not exist after `--write`.
    RefusePaid,
    /// Access not in the allow set and not paid.
    SkipUnknownAccess,
    /// URL scheme is not `file` / `http` / `https`.
    SkipUnknownScheme,
}

/// One planned row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRow {
    /// Source as parsed.
    pub source: Source,
    /// Action.
    pub action: PlanAction,
    /// Engine-relative dest (`knowledge/corpus/public/<id>.txt`) or the
    /// equivalent under a custom `--out`.
    pub dest: PathBuf,
}

/// Classified ledger. Constructor refuses the empty / none-allowed cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchPlan {
    /// Rows in ledger order.
    pub rows: Vec<PlanRow>,
    /// Count with [`AccessKind::is_allowed`].
    pub allowed: usize,
    /// Count of [`PlanAction::RefusePaid`].
    pub refused_paid: usize,
}

/// Inputs for [`fetch_corpus`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    /// Engine root (directory holding `registries/`).
    pub root: PathBuf,
    /// Path to `sources.toml`.
    pub sources: PathBuf,
    /// Directory that would receive `<id>.txt` under `--write`.
    pub out_dir: PathBuf,
    /// When true, plan only. Default for the CLI.
    pub dry_run: bool,
    /// `YYYY-MM-DD` written into snapshot headers. Tests pass a pin.
    pub fetched: String,
}

/// Outcome of a dry-run or a local-only write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchReport {
    /// True when no dest was created.
    pub dry_run: bool,
    /// Planned rows (includes refuses).
    pub plan: FetchPlan,
    /// Local copies actually written. Always 0 on dry-run.
    pub wrote: usize,
    /// Dest paths that were written.
    pub written: Vec<PathBuf>,
}

impl FetchReport {
    /// True when the run classified at least one allowed source and
    /// (if writing) produced at least one local copy.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.plan.allowed > 0 && (self.dry_run || self.wrote > 0)
    }
}

impl fmt::Display for FetchReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = if self.dry_run { "DRY-RUN" } else { "WRITE" };
        writeln!(
            f,
            "fetch_corpus: {mode} sources={} allowed={} refused_paid={} wrote={} ({NO_SOCKET})",
            self.plan.rows.len(),
            self.plan.allowed,
            self.plan.refused_paid,
            self.wrote
        )?;
        for row in &self.plan.rows {
            writeln!(
                f,
                "  {} {} {} {}",
                action_tag(&row.action),
                row.source.id,
                row.source.access,
                row.source.url
            )?;
        }
        Ok(())
    }
}

fn action_tag(action: &PlanAction) -> &'static str {
    match action {
        PlanAction::CopyLocal { .. } => "local",
        PlanAction::MissingLocal => "local-missing",
        PlanAction::WouldFetch => "would-fetch",
        PlanAction::WouldStubPdf => "would-stub-pdf",
        PlanAction::RefusePaid => "refuse-paid",
        PlanAction::SkipUnknownAccess => "skip-access",
        PlanAction::SkipUnknownScheme => "skip-scheme",
    }
}

/// Why a fetch/plan could not succeed.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CorpusError {
    /// No `[[source]]` rows.
    #[error("{ANTI_VACUOUS_EMPTY_SOURCES}")]
    EmptySources,
    /// Every row is paid or otherwise not allowed.
    #[error(
        "{ANTI_VACUOUS_NONE_ALLOWED} ({REFUSED_PAID} refused={refused_paid} skipped={skipped}; {NEVER_WRITTEN})"
    )]
    NoneAllowed {
        /// Paid rows.
        refused_paid: usize,
        /// Other skipped rows.
        skipped: usize,
    },
    /// `--write` found no `file://` copies to materialise.
    #[error("{ANTI_VACUOUS_NOTHING_WRITTEN} ({NO_SOCKET}; {REFUSED_PAID} is {NEVER_WRITTEN})")]
    NothingWritten,
    /// `id` is not a safe filename stem.
    #[error("source id {id:?} is not a safe filename stem")]
    UnsafeId {
        /// Offending id.
        id: String,
    },
    /// Ledger could not be parsed.
    #[error("unparseable {path}: {detail}")]
    Unparseable {
        /// Path that failed.
        path: String,
        /// Parser detail.
        detail: String,
    },
    /// Filesystem failure.
    #[error("cannot read {path}: {detail}")]
    Io {
        /// Path that failed.
        path: String,
        /// Underlying error.
        detail: String,
    },
    /// `file://` resolved outside the engine root / parent study root.
    #[error("file url escapes the study tree: {url}")]
    PathEscape {
        /// The file URL.
        url: String,
    },
    /// `--fetched` was not a strict `YYYY-MM-DD`.
    #[error("fetched date {got:?} is not a strict YYYY-MM-DD")]
    BadFetchedDate {
        /// Caller-supplied value.
        got: String,
    },
    /// Paid row reached the write helper. This is a programmer error
    /// if it fires — the planner must have filtered it first.
    #[error("{REFUSED_PAID} for {id} is {NEVER_WRITTEN}")]
    RefusedPaidWrite {
        /// Source id.
        id: String,
    },
}

/// Parse a source ledger. Zero rows is [`CorpusError::EmptySources`].
pub fn parse_sources(text: &str, origin: &str) -> Result<Vec<Source>, CorpusError> {
    let _ = ANTI_VACUOUS_EMPTY_SOURCES;
    let doc: toml::Value = toml::from_str(text).map_err(|e| CorpusError::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;
    let rows = match doc.get("source") {
        Some(toml::Value::Array(items)) => items,
        Some(other) => {
            return Err(CorpusError::Unparseable {
                path: origin.to_string(),
                detail: format!("`source` must be an array, got {other}"),
            });
        }
        None => return Err(CorpusError::EmptySources),
    };
    if rows.is_empty() {
        return Err(CorpusError::EmptySources);
    }
    let mut out = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let id = toml_string(row, "id").ok_or_else(|| CorpusError::Unparseable {
            path: origin.to_string(),
            detail: format!("source[{i}] missing `id`"),
        })?;
        if !is_safe_id(&id) {
            return Err(CorpusError::UnsafeId { id });
        }
        let url = toml_string(row, "url").ok_or_else(|| CorpusError::Unparseable {
            path: origin.to_string(),
            detail: format!("source[{i}] ({id}) missing `url`"),
        })?;
        let access = parse_access(toml_string(row, "access").as_deref().unwrap_or(""));
        out.push(Source {
            id,
            org: toml_string(row, "org").unwrap_or_default(),
            title: toml_string(row, "title").unwrap_or_default(),
            url,
            access,
            fetch_date: toml_string(row, "fetch_date").unwrap_or_default(),
            note: toml_string(row, "note").unwrap_or_default(),
        });
    }
    Ok(out)
}

/// Load and parse `path`.
pub fn load_sources(path: &Path) -> Result<Vec<Source>, CorpusError> {
    let origin = path.display().to_string();
    let text = std::fs::read_to_string(path).map_err(|e| CorpusError::Io {
        path: origin.clone(),
        detail: e.to_string(),
    })?;
    parse_sources(&text, &origin)
}

/// Classify every row. Empty / none-allowed is ERROR.
pub fn plan_sources(
    root: &Path,
    sources: &[Source],
    out_dir: &Path,
) -> Result<FetchPlan, CorpusError> {
    if sources.is_empty() {
        return Err(CorpusError::EmptySources);
    }
    let mut rows = Vec::with_capacity(sources.len());
    let mut allowed = 0usize;
    let mut refused_paid = 0usize;
    let mut skipped = 0usize;
    for source in sources {
        if source.access.is_allowed() {
            allowed += 1;
        } else if source.access == AccessKind::Paid {
            refused_paid += 1;
        } else {
            skipped += 1;
        }
        let dest = out_dir.join(format!("{}.txt", source.id));
        let action = classify_action(root, source);
        rows.push(PlanRow {
            source: source.clone(),
            action,
            dest,
        });
    }
    if allowed == 0 {
        return Err(CorpusError::NoneAllowed {
            refused_paid,
            skipped,
        });
    }
    Ok(FetchPlan {
        rows,
        allowed,
        refused_paid,
    })
}

/// Plan, optionally copy `file://` sources. Never opens a socket.
///
/// Paid rows cannot reach the write helper: [`classify_action`] maps
/// them to [`PlanAction::RefusePaid`] and [`write_one_local`] refuses
/// them again if called directly.
pub fn fetch_corpus(req: &FetchRequest) -> Result<FetchReport, CorpusError> {
    let _ = NO_SOCKET;
    validate_fetched(&req.fetched)?;
    let sources = load_sources(&req.sources)?;
    let plan = plan_sources(&req.root, &sources, &req.out_dir)?;
    if req.dry_run {
        return Ok(FetchReport {
            dry_run: true,
            plan,
            wrote: 0,
            written: Vec::new(),
        });
    }

    let mut payloads: Vec<(PathBuf, String, String)> = Vec::new();
    for row in &plan.rows {
        if let PlanAction::CopyLocal { path } = &row.action {
            let text = render_local_snapshot(&row.source, path, &req.fetched)?;
            payloads.push((row.dest.clone(), text, row.source.id.clone()));
        }
    }
    if payloads.is_empty() {
        return Err(CorpusError::NothingWritten);
    }

    let mut written = Vec::with_capacity(payloads.len());
    for (dest, text, id) in &payloads {
        write_one_local(&plan, dest, text, id)?;
        written.push(dest.clone());
    }
    let wrote = written.len();
    Ok(FetchReport {
        dry_run: false,
        plan,
        wrote,
        written,
    })
}

/// UTC today as `YYYY-MM-DD`. Tests should pass an explicit pin.
#[must_use]
pub fn today_utc_ymd() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// Default request against `root`. Dry-run, UTC date, default paths.
#[must_use]
pub fn dry_run_request(root: &Path) -> FetchRequest {
    FetchRequest {
        root: root.to_path_buf(),
        sources: join_rel(root, SOURCES_REL),
        out_dir: join_rel(root, OUT_DIR_REL),
        dry_run: true,
        fetched: today_utc_ymd(),
    }
}

fn classify_action(root: &Path, source: &Source) -> PlanAction {
    if source.access == AccessKind::Paid {
        let _ = REFUSED_PAID;
        let _ = NEVER_WRITTEN;
        return PlanAction::RefusePaid;
    }
    if !source.access.is_allowed() {
        return PlanAction::SkipUnknownAccess;
    }
    if let Some(rest) = source.url.strip_prefix("file://") {
        match resolve_local(root, rest, &source.url) {
            Ok(path) => PlanAction::CopyLocal { path },
            Err(CorpusError::PathEscape { .. }) => PlanAction::SkipUnknownScheme,
            Err(_) => PlanAction::MissingLocal,
        }
    } else if is_http_url(&source.url) {
        if source.url.to_ascii_lowercase().ends_with(".pdf") {
            PlanAction::WouldStubPdf
        } else {
            PlanAction::WouldFetch
        }
    } else {
        PlanAction::SkipUnknownScheme
    }
}

fn write_one_local(plan: &FetchPlan, dest: &Path, text: &str, id: &str) -> Result<(), CorpusError> {
    // Belt: refuse if this dest belongs to a paid row.
    if plan.rows.iter().any(|r| {
        r.source.id == id
            && (r.action == PlanAction::RefusePaid || r.source.access == AccessKind::Paid)
    }) {
        return Err(CorpusError::RefusedPaidWrite { id: id.to_string() });
    }
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| CorpusError::Io {
                path: parent.display().to_string(),
                detail: e.to_string(),
            })?;
        }
    }
    std::fs::write(dest, text.as_bytes()).map_err(|e| CorpusError::Io {
        path: dest.display().to_string(),
        detail: e.to_string(),
    })?;
    Ok(())
}

fn render_local_snapshot(
    source: &Source,
    path: &Path,
    fetched: &str,
) -> Result<String, CorpusError> {
    if source.access == AccessKind::Paid {
        return Err(CorpusError::RefusedPaidWrite {
            id: source.id.clone(),
        });
    }
    if !source.url.starts_with("file://") {
        return Err(CorpusError::NothingWritten);
    }
    let bytes = std::fs::read(path).map_err(|e| CorpusError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    let body = collapse_blank_lines(&String::from_utf8_lossy(&bytes));
    Ok(format!(
        "# source_id: {}\n# url: {}\n# access: {}\n# fetched: {fetched}\n# title: {}\n\n{body}",
        source.id, source.url, source.access, source.title
    ))
}

fn resolve_local(root: &Path, stripped: &str, url: &str) -> Result<PathBuf, CorpusError> {
    if stripped.is_empty() {
        return Err(CorpusError::Io {
            path: url.to_string(),
            detail: "empty file url".into(),
        });
    }
    let candidate = if Path::new(stripped).is_absolute() {
        PathBuf::from(stripped)
    } else {
        join_rel(root, stripped)
    };
    let resolved = if candidate.is_file() {
        candidate
    } else if let Some(parent) = root.parent() {
        let alt = join_rel(parent, stripped.trim_start_matches("./"));
        if alt.is_file() {
            alt
        } else {
            return Err(CorpusError::Io {
                path: url.to_string(),
                detail: "local file missing".into(),
            });
        }
    } else {
        return Err(CorpusError::Io {
            path: url.to_string(),
            detail: "local file missing".into(),
        });
    };
    confine_to_study_tree(root, &resolved, url)
}

fn confine_to_study_tree(root: &Path, path: &Path, url: &str) -> Result<PathBuf, CorpusError> {
    let canon = std::fs::canonicalize(path).map_err(|e| CorpusError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    let root_ok = std::fs::canonicalize(root)
        .ok()
        .is_some_and(|r| canon.starts_with(r));
    let parent_ok = root
        .parent()
        .and_then(|p| std::fs::canonicalize(p).ok())
        .is_some_and(|p| canon.starts_with(p));
    if root_ok || parent_ok {
        Ok(canon)
    } else {
        Err(CorpusError::PathEscape {
            url: url.to_string(),
        })
    }
}

fn parse_access(raw: &str) -> AccessKind {
    match raw {
        "public_summary" => AccessKind::PublicSummary,
        "free" => AccessKind::Free,
        "local" => AccessKind::Local,
        "paid" => AccessKind::Paid,
        other => AccessKind::Unknown(other.to_string()),
    }
}

fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && !id.starts_with('.')
        && !id.contains("..")
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn is_http_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

fn toml_string(v: &toml::Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn collapse_blank_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut nls = 0u32;
    for ch in text.chars() {
        if ch == '\n' {
            nls += 1;
            if nls <= 2 {
                out.push('\n');
            }
        } else {
            nls = 0;
            out.push(ch);
        }
    }
    out
}

fn validate_fetched(s: &str) -> Result<(), CorpusError> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return Err(CorpusError::BadFetchedDate { got: s.to_string() });
    }
    if !b
        .iter()
        .enumerate()
        .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    {
        return Err(CorpusError::BadFetchedDate { got: s.to_string() });
    }
    Ok(())
}

/// Howard Hinnant's `civil_from_days` (UTC). Same algorithm as the
/// gate's date helper — copied so this crate does not grow the gate.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}
