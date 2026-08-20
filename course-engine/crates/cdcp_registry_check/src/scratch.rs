//! Owned scratch directories for probes that materialise repository trees.
//!
//! A probe owns one [`ScratchDir`].  Construction creates a uniquely named
//! child below `target/cdcp-scratch/`; `Drop` removes that child on ordinary
//! return, early error, and panic unwinding.  The process-level reaper in
//! `scripts/reap_scratch.sh` handles leftovers from interruption or SIGKILL.
//!
//! The manager deliberately does not expose a caller-selected path.  A label
//! is only a name inside the fixed root, so a probe cannot accidentally place a
//! tree beside the warm build or invent a second cleanup namespace.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;

/// The one namespace in which new materialised probe trees may live.
pub const ROOT_REL: &str = "target/cdcp-scratch";
const MARKER: &str = ".cdcp-scratch-owner";

/// An owned, ephemeral probe directory.
#[derive(Debug)]
pub struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    /// Create a uniquely named child under [`ROOT_REL`].
    pub fn new(engine_root: &Path, label: &str) -> std::io::Result<Self> {
        validate_label(label)?;
        let root = engine_root.join(ROOT_REL);
        fs::create_dir_all(&root)?;

        let pid = process::id();
        for attempt in 0..1000u32 {
            let path = root.join(format!("{label}-{pid}-{attempt}"));
            match fs::create_dir(&path) {
                Ok(()) => {
                    let marker = path.join(MARKER);
                    if let Err(error) = fs::write(
                        &marker,
                        format!("cdcp scratch v1\nlabel={label}\npid={pid}\n"),
                    ) {
                        let _ = fs::remove_dir_all(&path);
                        return Err(error);
                    }
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("could not allocate a unique scratch directory for {label}"),
        ))
    }

    /// The owned directory.  Callers may create children, but must not remove
    /// this path; the guard owns its lifecycle.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            eprintln!(
                "scratch-lifecycle: WARNING: could not remove {} on Drop: {error}; the next reaper must retry it",
                self.path.display()
            );
        }
    }
}

fn validate_label(label: &str) -> std::io::Result<()> {
    if label.is_empty()
        || !label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid scratch label {label:?}; use ASCII letters, digits, '-' or '_'"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_removes_owned_tree_on_drop() {
        let root = tempfile::tempdir().unwrap();
        let path = {
            let scratch = ScratchDir::new(root.path(), "drop-test").unwrap();
            fs::write(scratch.path().join("payload"), "probe").unwrap();
            scratch.path().to_path_buf()
        };
        assert!(!path.exists(), "Drop leaked {}", path.display());
    }

    #[test]
    fn labels_cannot_escape_the_named_root() {
        let root = tempfile::tempdir().unwrap();
        for label in ["", "../escape", "nested/name", "space here"] {
            let error = ScratchDir::new(root.path(), label).unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput, "{label:?}");
        }
    }

    #[test]
    fn panic_unwinding_drops_the_owned_tree() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(ROOT_REL);
        let result = std::panic::catch_unwind(|| {
            let scratch = ScratchDir::new(root.path(), "panic-test").unwrap();
            let owned = scratch.path().to_path_buf();
            fs::write(owned.join("payload"), "probe").unwrap();
            panic!("known test panic");
        });
        assert!(result.is_err());
        assert_eq!(fs::read_dir(path).unwrap().count(), 0);
    }
}
