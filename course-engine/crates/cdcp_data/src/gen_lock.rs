//! Write `content.lock` (L7-S9 ecosystem pin).
//!
//! Extracted from `scripts/gen_content_lock.py` by
//! `bd-substrate-rust-migration-jhd.30`. The Python is DELETED. Product
//! writer, not a gate: `cdcp content-lock` authors the lock;
//! `cdcp_gate verify-content-lock` and [`crate::verify_data_lock`] check it.
//!
//! # Contract
//!
//! Pins:
//!   - `bank_hash` (caller-supplied; the CLI computes it via `cdcp_bank`)
//!   - knowledge pack top-level `*.toml` (ONE LEVEL, bd-zhnd)
//!   - module markdown under `web/content/modules/*.md` and parent
//!     `../modules/*.md` when present (ONE LEVEL)
//!   - `[data]` every body + sidecar named by `snapshots.toml`
//!     (not a walk of `knowledge/corpus/`)
//!
//! Empty knowledge, empty modules, or a non-empty `snapshots.toml` that
//! yields zero data files is RED. A lock that pins nothing certifies
//! nothing. Write-after-verdict: [`generate_content_lock`] builds the
//! full text before [`write_content_lock`] touches the destination.
//!
//! # What this cannot decide
//!
//! It cannot decide that a hash is the *right* hash — a regen over
//! corrupted bytes is internally consistent. It does not recurse into
//! `knowledge/corpus/` (NOAA / USGS / eGRID stay unvendored unless a
//! snapshot row names them). It does not compute `bank_hash`; a caller
//! that passes a stale digest writes a stale pin.

use crate::{
    join_rel, load_pins_from_disk, referenced_data_paths, sha256_hex, DataError, LOCK_REL,
};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Lock schema this writer emits. Must stay 1 until the verifier grows.
pub const SCHEMA_VERSION: u32 = 1;
/// Domain tag written as `canonical`. Label for `cdcp_core::BANK_HASH_DOMAIN`.
///
/// A partial bump (this const without the core constant, or the reverse)
/// is worse than none. The three sites move in one commit.
pub const CANONICAL: &str = "cdcp-bank-v3";
/// Digest algorithm recorded in the lock.
pub const HASH_ALG: &str = "sha256";

/// ONE LEVEL (bd-zhnd). `list_one_level` is read_dir of one directory.
/// `knowledge/corpus/*.toml` is deliberately unpinned (external blobs).
pub const KNOWLEDGE_DIR_REL: &str = "knowledge";
/// Suffix the knowledge glob accepts. Not recursive.
pub const KNOWLEDGE_SUFFIX: &str = ".toml";
/// Product-surface module markdown, engine-relative.
pub const WEB_MODULES_REL: &str = "web/content/modules";
/// Source-corpus module markdown, parent-relative.
pub const PARENT_MODULES_REL: &str = "modules";
/// Suffix the module globs accept. Not recursive.
pub const MODULE_SUFFIX: &str = ".md";

/// Counts from a successful write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenLockReport {
    /// Destination that was written.
    pub dest: PathBuf,
    /// `bank_hash` recorded in the lock.
    pub bank_hash: String,
    /// Rows under `[knowledge]`.
    pub knowledge: usize,
    /// Rows under `[modules]`.
    pub modules: usize,
    /// Rows under `[data]`.
    pub data: usize,
}

impl fmt::Display for GenLockReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bh = if self.bank_hash.len() >= 12 {
            &self.bank_hash[..12]
        } else {
            &self.bank_hash
        };
        write!(
            f,
            "content_lock: wrote {} bank_hash={bh}\u{2026} knowledge={} modules={} data={}",
            self.dest.display(),
            self.knowledge,
            self.modules,
            self.data
        )
    }
}

/// Build the lock text. Does not write.
pub fn generate_content_lock(root: &Path, bank_hash: &str) -> Result<String, DataError> {
    let bank = normalize_bank_hash(bank_hash)?;
    let knowledge = hash_knowledge(root)?;
    let modules = hash_modules(root)?;
    let (data, registered) = hash_snapshot_files(root)?;

    if knowledge.is_empty() {
        return Err(DataError::EmptyKnowledge);
    }
    if modules.is_empty() {
        return Err(DataError::EmptyModules);
    }
    if data.is_empty() {
        return Err(DataError::EmptyDataLock { registered });
    }

    Ok(render_lock(&bank, &knowledge, &modules, &data))
}

/// Generate, then write `dest`. A RED generate writes nothing.
pub fn write_content_lock(
    root: &Path,
    bank_hash: &str,
    dest: &Path,
) -> Result<GenLockReport, DataError> {
    let text = generate_content_lock(root, bank_hash)?;
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| DataError::Io {
                path: parent.display().to_string(),
                detail: e.to_string(),
            })?;
        }
    }
    std::fs::write(dest, text.as_bytes()).map_err(|e| DataError::Io {
        path: dest.display().to_string(),
        detail: e.to_string(),
    })?;

    // Recount from the text we just wrote (the maps are not returned).
    let parsed = parse_written_counts(&text);
    Ok(GenLockReport {
        dest: dest.to_path_buf(),
        bank_hash: normalize_bank_hash(bank_hash)?,
        knowledge: parsed.0,
        modules: parsed.1,
        data: parsed.2,
    })
}

/// Write `<root>/content.lock`.
pub fn write_content_lock_at_root(
    root: &Path,
    bank_hash: &str,
) -> Result<GenLockReport, DataError> {
    write_content_lock(root, bank_hash, &join_rel(root, LOCK_REL))
}

/// Lock text with comment lines removed. Identity of the pin tables
/// ignores the generator banner.
#[must_use]
pub fn lock_pin_body(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Bytes of one `[section]` table, without the following table.
#[must_use]
pub fn lock_section(text: &str, name: &str) -> String {
    let header = format!("[{name}]");
    let mut lines = text.lines();
    let mut found = false;
    for line in lines.by_ref() {
        if line.trim() == header {
            found = true;
            break;
        }
    }
    if !found {
        return String::new();
    }
    let mut out = vec![header];
    for line in lines {
        if line.starts_with('[') {
            break;
        }
        out.push(line.to_string());
    }
    while out.last().is_some_and(|s| s.is_empty()) {
        out.pop();
    }
    out.join("\n")
}

fn render_lock(
    bank: &str,
    knowledge: &BTreeMap<String, String>,
    modules: &BTreeMap<String, String>,
    data: &BTreeMap<String, String>,
) -> String {
    let mut lines: Vec<String> = vec![
        "# content.lock — ecosystem pin for bank + knowledge + modules (L7-S9)".into(),
        "# Generated by `cdcp content-lock` — do not hand-edit hashes.".into(),
        "# Regenerate: cdcp content-lock".into(),
        "# Verify:     cdcp_gate verify-content-lock".into(),
        String::new(),
        format!("schema_version = {SCHEMA_VERSION}"),
        format!("canonical = \"{CANONICAL}\""),
        format!("hash_alg = \"{HASH_ALG}\""),
        format!("bank_hash = \"{bank}\""),
        String::new(),
        "[knowledge]".into(),
    ];
    push_table(
        &mut lines,
        knowledge,
        "# (empty — no knowledge/*.toml found)",
    );
    lines.push(String::new());
    lines.push("[modules]".into());
    push_table(&mut lines, modules, "# (empty — no module markdown found)");
    lines.push(String::new());
    lines.push("[data]".into());
    push_table(&mut lines, data, "# (empty — no snapshot-referenced files)");
    lines.push(String::new());
    lines.join("\n")
}

fn push_table(lines: &mut Vec<String>, table: &BTreeMap<String, String>, empty_comment: &str) {
    if table.is_empty() {
        lines.push(empty_comment.into());
        return;
    }
    for (path, hx) in table {
        lines.push(format!("\"{path}\" = \"{hx}\""));
    }
}

fn normalize_bank_hash(bank_hash: &str) -> Result<String, DataError> {
    let hx = bank_hash.trim().to_ascii_lowercase();
    if hx.len() != 64 || !hx.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return Err(DataError::InvalidBankHash);
    }
    Ok(hx)
}

fn hash_knowledge(root: &Path) -> Result<BTreeMap<String, String>, DataError> {
    let dir = join_rel(root, KNOWLEDGE_DIR_REL);
    hash_one_level(root, &dir, KNOWLEDGE_SUFFIX)
}

fn hash_modules(root: &Path) -> Result<BTreeMap<String, String>, DataError> {
    let mut out = hash_one_level(root, &join_rel(root, WEB_MODULES_REL), MODULE_SUFFIX)?;
    if let Some(parent) = root.parent() {
        let parent_dir = join_rel(parent, PARENT_MODULES_REL);
        let more = hash_one_level(root, &parent_dir, MODULE_SUFFIX)?;
        out.extend(more);
    }
    Ok(out)
}

fn hash_snapshot_files(root: &Path) -> Result<(BTreeMap<String, String>, usize), DataError> {
    let pins = load_pins_from_disk(root)?;
    let required = referenced_data_paths(&pins);
    if required.is_empty() {
        return Err(DataError::EmptyRegistry);
    }
    let mut missing = Vec::new();
    let mut out = BTreeMap::new();
    for rel in required {
        let abs = join_rel(root, &rel);
        if !abs.is_file() {
            missing.push(rel);
            continue;
        }
        out.insert(rel, hash_file(&abs)?);
    }
    if !missing.is_empty() {
        return Err(DataError::SnapshotFilesMissing {
            paths: missing.join(", "),
        });
    }
    Ok((out, pins.len()))
}

/// ONE LEVEL: files matching `*<suffix>` DIRECTLY under `dir`.
/// Not a recursive walk. Dotfiles are skipped (Python `glob` does not
/// match a leading `.`).
fn list_one_level(dir: &Path, suffix: &str) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for ent in rd.flatten() {
        let p = ent.path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if !name.ends_with(suffix) {
            continue;
        }
        if !p.is_file() {
            continue;
        }
        out.push(p);
    }
    out
}

fn hash_one_level(
    engine_root: &Path,
    dir: &Path,
    suffix: &str,
) -> Result<BTreeMap<String, String>, DataError> {
    let mut out = BTreeMap::new();
    for path in list_one_level(dir, suffix) {
        let key = rel_posix(engine_root, &path);
        out.insert(key, hash_file(&path)?);
    }
    Ok(out)
}

fn hash_file(path: &Path) -> Result<String, DataError> {
    let bytes = std::fs::read(path).map_err(|e| DataError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    Ok(sha256_hex(&bytes))
}

/// Path relative to the engine root when under it; else parent-relative.
fn rel_posix(engine_root: &Path, path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let root = engine_root
        .canonicalize()
        .unwrap_or_else(|_| engine_root.to_path_buf());
    if let Ok(rel) = path.strip_prefix(&root) {
        return to_posix(rel);
    }
    if let Some(parent) = root.parent() {
        if let Ok(rel) = path.strip_prefix(parent) {
            return to_posix(rel);
        }
    }
    to_posix(&path)
}

fn to_posix(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn parse_written_counts(text: &str) -> (usize, usize, usize) {
    (
        count_pin_rows(&lock_section(text, "knowledge")),
        count_pin_rows(&lock_section(text, "modules")),
        count_pin_rows(&lock_section(text, "data")),
    )
}

fn count_pin_rows(section: &str) -> usize {
    section
        .lines()
        .filter(|l| l.starts_with('"') && l.contains(" = \""))
        .count()
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn production_walk_is_one_level() {
        let src = include_str!("gen_lock.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        assert!(src.contains("ONE LEVEL"));
        assert!(src.contains("fn list_one_level"));
        assert!(src.contains("KNOWLEDGE_DIR_REL"));
        assert!(src.contains("WEB_MODULES_REL"));
        assert!(src.contains("PARENT_MODULES_REL"));
        assert!(!src.contains("WalkDir"));
        assert!(!src.contains("rglob"));
        assert!(!src.contains("visit_dirs"));
    }

    #[test]
    fn invalid_bank_hash_is_red() {
        let err = normalize_bank_hash("abcd").unwrap_err();
        assert!(matches!(err, DataError::InvalidBankHash), "{err:?}");
    }

    #[test]
    fn lock_section_stops_at_the_next_table() {
        let text = "[knowledge]\n\"a\" = \"b\"\n\n[modules]\n\"c\" = \"d\"\n";
        assert_eq!(
            lock_section(text, "knowledge"),
            "[knowledge]\n\"a\" = \"b\""
        );
        assert_eq!(lock_section(text, "modules"), "[modules]\n\"c\" = \"d\"");
    }
}
