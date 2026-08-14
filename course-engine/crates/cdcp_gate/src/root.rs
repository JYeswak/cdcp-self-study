//! Project-root resolution.
//!
//! The course-engine project root is the directory holding `registries/`. It is
//! NOT the git repository root (`cdcp-self-study/`) — the engine lives in a
//! subdirectory, and every registry path is relative to the engine root.

use std::path::{Path, PathBuf};

/// The file whose presence defines the engine root.
pub const ANCHOR: &str = "registries/claims.toml";

/// Resolve the engine root by walking up from `start`, then falling back to the
/// crate's own compile-time location. Mirrors `cdcp_registry_check::resolve_repo_root`
/// so both gates agree on what "the repo" means.
pub fn resolve(start: &Path) -> Result<PathBuf, String> {
    let mut cur = start.to_path_buf();
    if cur.is_file() {
        cur.pop();
    }
    for _ in 0..12 {
        if cur.join(ANCHOR).is_file() {
            return Ok(cur);
        }
        if !cur.pop() {
            break;
        }
    }
    // crates/cdcp_gate -> engine root
    let from_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if let Ok(canon) = from_manifest.canonicalize() {
        if canon.join(ANCHOR).is_file() {
            return Ok(canon);
        }
    }
    Err(format!(
        "could not locate the course-engine root (no {ANCHOR} at or above {})",
        start.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_root_from_a_nested_dir() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("engine");
        std::fs::create_dir_all(root.join("registries")).unwrap();
        std::fs::write(root.join(ANCHOR), "schema_version = 1\n").unwrap();
        let nested = root.join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        assert_eq!(resolve(&nested).unwrap(), root);
    }

    #[test]
    fn real_repo_root_has_the_anchor() {
        let root = resolve(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(root.join(ANCHOR).is_file());
    }
}
