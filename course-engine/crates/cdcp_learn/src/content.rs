//! Learn content-copy orphan sweep (bd-zhnd).
//!
//! `scripts/build_learn.py` writes `web/content/modules/{id}.md` and then
//! deletes names that look like leftover generated copies. Until 2026-08-14
//! that sweep matched **any** `*.md`, so a tracked doc (`README.md`) was
//! deleted on every run. The keep set is the product: a non-module `.md` is
//! documentation and must survive.
//!
//! This module is the live rust compiler for that keep set. The python
//! builder (still the page generator) implements the same predicate; the
//! CHARTER pair in `tests/build_learn_charter_pair.rs` runs whichever path
//! is live and asserts a planted non-module `.md` SURVIVES. Deleting a
//! tracked doc is RED.
//!
//! # What it cannot decide
//!
//! It does not generate Learn pages, the hub, or `modules_index.json`.
//! Those stay with `scripts/build_learn.py` until that compiler is ported.
//! It cannot decide that a surviving file is *correct* — only that it was
//! not unlinked. It cannot see files one directory down (the python glob
//! is one level; this walk matches).

#![forbid(unsafe_code)]

use crate::LearnError;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Engine-root-relative content-copy directory.
pub const CONTENT_DIR_REL: &str = "web/content/modules";

/// Tracked documentation that is never a generated module copy.
///
/// A literal here is right: these names are not derived from
/// `knowledge/domains.toml`. They are hand-written docs that share a
/// directory with generated copies. Adding a name is a deliberate edit;
/// removing one is too. `README.md` is the file the unguarded sweep
/// deleted (observed 2026-08-15).
pub const PROTECTED_CONTENT_DOCS: &[&str] = &["README.md"];

/// Report of one sweep. `scanned == 0` is never returned — that is an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepReport {
    pub scanned: usize,
    pub unlinked: Vec<String>,
    pub kept: Vec<String>,
}

/// A content-dir `.md` is a generated module copy iff it matches the domain-id
/// filename shape (`NN-slug.md`). Everything else is documentation.
pub fn is_generated_module_copy(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".md") else {
        return false;
    };
    let bytes = stem.as_bytes();
    if bytes.len() < 4 {
        return false;
    }
    bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2] == b'-'
        && bytes[3..]
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

/// True when the orphan sweep must unlink `name`.
///
/// Keep: the current navigable `{id}.md` set, plus every protected doc, plus
/// any name that is not a generated module copy. Delete: leftover generated
/// copies only.
pub fn should_unlink_content_copy(name: &str, navigable_md: &BTreeSet<String>) -> bool {
    if PROTECTED_CONTENT_DOCS.contains(&name) {
        return false;
    }
    if navigable_md.contains(name) {
        return false;
    }
    is_generated_module_copy(name)
}

/// Sweep `dir` in place. An empty scan (zero `.md` files) is an ERROR, never
/// a pass — a sweep that looked at nothing reports exactly like a sweep that
/// kept everything.
pub fn sweep_content_copies(
    dir: &Path,
    navigable_md: &BTreeSet<String>,
) -> Result<SweepReport, LearnError> {
    if !dir.is_dir() {
        return Err(LearnError::io(format!(
            "content dir is not a directory: {}",
            dir.display()
        )));
    }
    let rd =
        fs::read_dir(dir).map_err(|e| LearnError::io(format!("read {}: {e}", dir.display())))?;
    let mut scanned = 0usize;
    let mut unlinked = Vec::new();
    let mut kept = Vec::new();
    for ent in rd {
        let ent = ent.map_err(|e| LearnError::io(format!("dirent: {e}")))?;
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) if n.ends_with(".md") => n.to_string(),
            _ => continue,
        };
        scanned += 1;
        if should_unlink_content_copy(&name, navigable_md) {
            fs::remove_file(&path)
                .map_err(|e| LearnError::io(format!("unlink {}: {e}", path.display())))?;
            unlinked.push(name);
        } else {
            kept.push(name);
        }
    }
    if scanned == 0 {
        return Err(LearnError::io(format!(
            "content dir matched zero .md files under {} — an empty sweep is an ERROR, not a pass",
            dir.display()
        )));
    }
    unlinked.sort();
    kept.sort();
    Ok(SweepReport {
        scanned,
        unlinked,
        kept,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn nav(ids: &[&str]) -> BTreeSet<String> {
        ids.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn protected_inventory_is_non_empty_and_names_readme() {
        // Anti-vacuous: an empty protect set would make the keep policy a
        // no-op on the documented name and report like a working guard.
        assert!(
            !PROTECTED_CONTENT_DOCS.is_empty(),
            "PROTECTED_CONTENT_DOCS is empty — a keep set of nothing is not a keep set"
        );
        assert!(
            PROTECTED_CONTENT_DOCS.contains(&"README.md"),
            "README.md is the tracked doc the unguarded sweep deleted"
        );
    }

    #[test]
    fn generated_copy_shape_is_nn_slug() {
        assert!(is_generated_module_copy("01-mission-critical.md"));
        assert!(is_generated_module_copy("15-ops-adjacent.md"));
        assert!(is_generated_module_copy("99-stale.md"));
        assert!(!is_generated_module_copy("README.md"));
        assert!(!is_generated_module_copy("NOTES.md"));
        assert!(!is_generated_module_copy("01.md"));
        assert!(!is_generated_module_copy("m01-foo.md"));
        assert!(!is_generated_module_copy("01-Mission.md"));
    }

    #[test]
    fn readme_and_other_docs_are_never_unlinked() {
        let empty = nav(&[]);
        let some = nav(&["01-mission-critical.md"]);
        for set in [&empty, &some] {
            assert!(!should_unlink_content_copy("README.md", set));
            assert!(!should_unlink_content_copy("NOTES.md", set));
            assert!(!should_unlink_content_copy("CHANGELOG.md", set));
        }
    }

    #[test]
    fn stale_generated_copies_are_unlinked_and_live_ones_are_not() {
        let some = nav(&["01-mission-critical.md"]);
        assert!(should_unlink_content_copy("99-stale.md", &some));
        assert!(should_unlink_content_copy("15-ops-adjacent.md", &some));
        assert!(!should_unlink_content_copy("01-mission-critical.md", &some));
    }

    #[test]
    fn an_empty_content_dir_is_an_error_not_a_pass() {
        let td = tempfile::tempdir().unwrap();
        let err = sweep_content_copies(td.path(), &nav(&["01-foo.md"])).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("zero .md files"),
            "empty scan must be named: {msg}"
        );
    }

    #[test]
    fn sweep_keeps_docs_and_deletes_stale_copies() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path();
        std::fs::write(dir.join("README.md"), "tracked\n").unwrap();
        std::fs::write(dir.join("NOTES.md"), "notes\n").unwrap();
        std::fs::write(dir.join("01-mission-critical.md"), "# m\n").unwrap();
        std::fs::write(dir.join("99-stale.md"), "stale\n").unwrap();
        let report = sweep_content_copies(dir, &nav(&["01-mission-critical.md"])).unwrap();
        assert_eq!(report.scanned, 4, "a scan that skipped files is vacuous");
        assert_eq!(report.unlinked, vec!["99-stale.md".to_string()]);
        assert!(dir.join("README.md").is_file());
        assert!(dir.join("NOTES.md").is_file());
        assert!(dir.join("01-mission-critical.md").is_file());
        assert!(!dir.join("99-stale.md").exists());
    }
}
