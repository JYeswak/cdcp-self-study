//! `cdcp_learn` — the Learn compilers, chrome smoke, and Learn-surface smoke are product, not gate.
//!
//! `web/data/units_index.json` and `web/data/glossary.json` are LEARNER-VISIBLE.
//! A learner reads the glossary and is scored against unit `check_item_ids`.
//! If a learner can see it or be scored by it, it is not a gate.
//!
//! # Contract (bd-engine-not-gate-ar39.2)
//!
//! The artifact SCHEMA, not a Python `json.dumps` replica:
//!
//! * declared keys are present
//! * `unit_count` / `module_count` / `term_count` match the collections
//! * `check_item_ids` are approved-only (a retired id is an ERROR)
//! * emitted bytes are stable across runs
//! * zero modules or zero units is an ERROR
//! * a red compile writes nothing
//!
//! #![forbid(unsafe_code)] is load-bearing: this crate is on the learner path.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod chrome;
pub mod glossary;
pub mod smoke;
pub mod units;

/// Engine-root anchor: the directory holding `registries/claims.toml`.
pub const ENGINE_ANCHOR: &str = "registries/claims.toml";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LearnError {
    #[error("{0}")]
    Msg(String),
}

impl LearnError {
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Msg(msg.into())
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Msg(msg.into())
    }
}

/// Result of compiling a Learn artifact. `code != 0` means RED and `artifact`
/// is always `None` — write-after-verdict, never the reverse.
#[derive(Debug, Clone)]
pub struct BuildOutcome {
    pub stdout: String,
    pub code: i32,
    /// `(path, bytes)` the run would write. `None` on every RED path.
    pub artifact: Option<(PathBuf, String)>,
}

/// Join an engine-root-relative `/`-separated path.
pub fn join_rel(root: &Path, rel: &str) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in rel.split('/') {
        p.push(part);
    }
    p
}

/// Resolve the course-engine root by walking up from `start`.
pub fn resolve_engine_root(start: &Path) -> Result<PathBuf, LearnError> {
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
    Err(LearnError::io(format!(
        "could not locate the course-engine root (no {ENGINE_ANCHOR} at or above {})",
        start.display()
    )))
}

/// Top-level declared keys of `web/data/units_index.json`.
pub const UNITS_INDEX_KEYS: &[&str] = &[
    "schema_version",
    "generated_by",
    "unit_count",
    "module_count",
    "approved_item_count",
    "bank_item_count",
    "units_with_checks",
    "units_zero_checks",
    "units",
    "by_module",
    "shortfalls",
];

/// Per-unit keys inside `units` / `by_module`.
pub const UNIT_ROW_KEYS: &[&str] = &[
    "id",
    "module_id",
    "module_num",
    "order",
    "title",
    "heading_id",
    "word_count",
    "estimate_minutes",
    "topic_ids",
    "check_item_ids",
    "check_count",
];

/// Top-level declared keys of `web/data/glossary.json`.
pub const GLOSSARY_KEYS: &[&str] = &[
    "schema_version",
    "generated_by",
    "source",
    "term_count",
    "terms",
];

/// Provenance string written into both artifacts.
pub const GENERATED_BY: &str = "cdcp_learn";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_root_resolves_from_this_crate() {
        let root = resolve_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(root.join(ENGINE_ANCHOR).is_file());
        assert!(root.join("web/data").is_dir());
    }

    #[test]
    fn schema_key_lists_are_not_empty() {
        assert!(!UNITS_INDEX_KEYS.is_empty());
        assert!(!UNIT_ROW_KEYS.is_empty());
        assert!(!GLOSSARY_KEYS.is_empty());
        assert_eq!(GENERATED_BY, "cdcp_learn");
    }
}
