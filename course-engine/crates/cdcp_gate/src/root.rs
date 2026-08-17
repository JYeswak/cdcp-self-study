//! Project-root resolution.
//!
//! The course-engine project root is the directory holding `registries/`. It is
//! NOT the git repository root (`cdcp-self-study/`) — the engine lives in a
//! subdirectory, and every registry path is relative to the engine root.
//!
//! Walks only. There is no compile-time crate-directory fallback; the
//! installed-tree resolver lives in `cdcp_root` and is used by `cdcp serve`.

use std::path::{Path, PathBuf};

/// The file whose presence defines the engine root.
pub const ANCHOR: &str = cdcp_root::ENGINE_ANCHOR;

/// Resolve the engine root by walking up from `start` for [`ANCHOR`].
pub fn resolve(start: &Path) -> Result<PathBuf, String> {
    cdcp_root::walk_engine_root(start).map_err(|e| e.to_string())
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
