//! Physical workspace identity for authoring and local verification commands.
//!
//! A path alias is safe when it resolves to the declared checkout; a second
//! checkout is not.  This module canonicalizes both sides before comparing
//! them, so a symlink remains a usable entry point while a copied checkout
//! cannot silently receive evidence for the canonical tree.

use std::fs;
use std::path::{Path, PathBuf};

use crate::{walk_engine_root, ENGINE_ANCHOR};

/// The first-read operating-file prefix that declares the physical root.
pub const DECLARATION_PREFIX: &str = "Canonical physical workspace root:";

/// The result of a successful physical-root comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIdentity {
    /// The physical current directory used for the preflight.
    pub cwd_physical: PathBuf,
    /// The physical engine root found from the current directory.
    pub engine_root: PathBuf,
    /// The physical path declared by `AGENTS.md`.
    pub declared_root: PathBuf,
}

impl WorkspaceIdentity {
    /// A path-bearing receipt line for logs and evidence.
    pub fn receipt_line(&self) -> String {
        format!(
            "workspace-identity: cwd={} root={} declared={} result=PASS",
            self.cwd_physical.display(),
            self.engine_root.display(),
            self.declared_root.display()
        )
    }
}

/// Why the physical workspace preflight refused to proceed.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WorkspaceIdentityError {
    #[error("workspace identity: {0}")]
    Message(String),
    #[error("workspace identity: cannot canonicalize {path}: {detail}")]
    Canonicalize { path: String, detail: String },
    #[error("workspace identity: physical root mismatch: declared={declared}, actual={actual}")]
    Mismatch { declared: String, actual: String },
}

/// Compare the current workspace with the root declared in its first-read
/// operating file (`AGENTS.md`).  The comparison is physical (`realpath` /
/// `pwd -P` semantics), so a symlink alias passes while a different checkout
/// fails before a reservation, build, receipt, or commit can use it.
pub fn verify_workspace_identity(
    start: &Path,
) -> Result<WorkspaceIdentity, WorkspaceIdentityError> {
    let cwd_physical = canonicalize(start)?;
    let engine_root = resolve_engine_root(start)?;
    let engine_root = canonicalize(&engine_root)?;
    let declared_raw = read_declaration(&engine_root.join("AGENTS.md"))?;
    let declared_root = canonicalize(Path::new(&declared_raw))?;

    if declared_root != engine_root {
        return Err(WorkspaceIdentityError::Mismatch {
            declared: declared_root.display().to_string(),
            actual: engine_root.display().to_string(),
        });
    }

    Ok(WorkspaceIdentity {
        cwd_physical,
        engine_root,
        declared_root,
    })
}

fn resolve_engine_root(start: &Path) -> Result<PathBuf, WorkspaceIdentityError> {
    if let Ok(root) = walk_engine_root(start) {
        return Ok(root);
    }

    // The outer git workspace is a supported invocation point.  Accept only
    // one direct child with the engine anchor; guessing among descendants is
    // precisely the identity failure this preflight is meant to prevent.
    let start_dir = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };
    let entries = fs::read_dir(start_dir).map_err(|e| {
        WorkspaceIdentityError::Message(format!(
            "cannot inspect {} for a direct course-engine child: {e}",
            start_dir.display()
        ))
    })?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| WorkspaceIdentityError::Message(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() && path.join(ENGINE_ANCHOR).is_file() {
            candidates.push(path);
        }
    }
    candidates.sort();
    match candidates.as_slice() {
        [root] => Ok(root.clone()),
        [] => Err(WorkspaceIdentityError::Message(format!(
            "no course-engine root at or above {} and no anchored direct child",
            start_dir.display()
        ))),
        _ => Err(WorkspaceIdentityError::Message(format!(
            "ambiguous anchored course-engine children under {}: {}",
            start_dir.display(),
            candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn read_declaration(path: &Path) -> Result<PathBuf, WorkspaceIdentityError> {
    let text = fs::read_to_string(path).map_err(|e| {
        WorkspaceIdentityError::Message(format!(
            "first-read operating file {} is unreadable: {e}",
            path.display()
        ))
    })?;
    let declarations: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix(DECLARATION_PREFIX))
        .map(str::trim)
        .collect();
    match declarations.as_slice() {
        [value] if !value.is_empty() => {
            let value = value
                .strip_prefix('`')
                .and_then(|value| value.strip_suffix('`'))
                .unwrap_or(value)
                .trim();
            if value.is_empty() {
                Err(WorkspaceIdentityError::Message(format!(
                    "{} has an empty canonical-root declaration",
                    path.display()
                )))
            } else {
                Ok(PathBuf::from(value))
            }
        }
        [] => Err(WorkspaceIdentityError::Message(format!(
            "{} is missing `{DECLARATION_PREFIX}`",
            path.display()
        ))),
        _ => Err(WorkspaceIdentityError::Message(format!(
            "{} has {} canonical-root declarations; exactly one is required",
            path.display(),
            declarations.len()
        ))),
    }
}

fn canonicalize(path: &Path) -> Result<PathBuf, WorkspaceIdentityError> {
    fs::canonicalize(path).map_err(|e| WorkspaceIdentityError::Canonicalize {
        path: path.display().to_string(),
        detail: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    fn write_fixture(root: &Path, declared: &Path) {
        fs::create_dir_all(root.join("registries")).unwrap();
        fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        fs::write(
            root.join("AGENTS.md"),
            format!(
                "# Operating file\n{DECLARATION_PREFIX} `{}`\n",
                declared.display()
            ),
        )
        .unwrap();
    }

    #[test]
    fn canonical_root_passes_from_nested_path() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("course-engine");
        write_fixture(&root, &root);
        let nested = root.join("crates/cdcp_root");
        fs::create_dir_all(&nested).unwrap();

        let identity = verify_workspace_identity(&nested).unwrap();
        assert_eq!(identity.engine_root, fs::canonicalize(&root).unwrap());
        assert_eq!(identity.declared_root, identity.engine_root);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_entrypoint_normalizes_to_one_identity() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("course-engine");
        write_fixture(&root, &root);
        let alias = td.path().join("alias");
        symlink(&root, &alias).unwrap();

        let identity = verify_workspace_identity(&alias).unwrap();
        assert_eq!(identity.engine_root, fs::canonicalize(&root).unwrap());
    }

    #[test]
    fn different_checkout_is_red() {
        let td = tempfile::tempdir().unwrap();
        let canonical = td.path().join("canonical/course-engine");
        let other = td.path().join("other/course-engine");
        write_fixture(&canonical, &canonical);
        write_fixture(&other, &canonical);

        let error = verify_workspace_identity(&other).unwrap_err();
        assert!(matches!(error, WorkspaceIdentityError::Mismatch { .. }));
        assert!(error.to_string().contains("physical root mismatch"));
    }

    #[test]
    fn missing_declaration_is_red() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("course-engine");
        fs::create_dir_all(root.join("registries")).unwrap();
        fs::write(root.join("registries/claims.toml"), "schema_version = 1\n").unwrap();
        fs::write(root.join("AGENTS.md"), "# Operating file\n").unwrap();

        let error = verify_workspace_identity(&root).unwrap_err();
        assert!(error
            .to_string()
            .contains("missing `Canonical physical workspace root:"));
    }
}
