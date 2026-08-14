//! Thin git plumbing adapter.
//!
//! `git` is invoked by argv (`Command::new("git")`), never through a shell — the
//! guard that bans shell does not get to spawn one. Every helper returns
//! engine-root-relative paths, because that is what the registries speak.

use std::path::{Path, PathBuf};
use std::process::Command;

fn run(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed (status {:?}): {}",
            args.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout)
        .map_err(|e| format!("git {}: non-utf8 output: {e}", args.join(" ")))
}

fn split_nul(s: &str) -> Vec<String> {
    s.split('\0')
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

/// Files git tracks under `root` (index contents), relative to `root`.
///
/// Index contents — not HEAD — so a file that has merely been `git add`ed is
/// already "in the tree" for this gate's purposes. That is the moment the
/// bleeding starts.
pub fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    Ok(split_nul(&run(root, &["ls-files", "-z"])?))
}

/// Files this commit would ADD (or copy/rename into place), relative to `root`.
///
/// `--diff-filter=ACR` deliberately omits `M`: editing a file that is already
/// allowlisted must not trip the gate. Modified-but-unlisted files are still
/// caught, by the presence scan.
pub fn staged_additions(root: &Path) -> Result<Vec<String>, String> {
    let has_head = run(root, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_ok();
    if !has_head {
        // Pre-first-commit: every staged path is an addition.
        return tracked_files(root);
    }
    Ok(split_nul(&run(
        root,
        &[
            "diff",
            "--cached",
            "--name-only",
            "--relative",
            "--diff-filter=ACR",
            "-M",
            "-z",
            "HEAD",
        ],
    )?))
}

/// The shared git directory (worktree-safe), absolute.
pub fn common_dir(root: &Path) -> Result<PathBuf, String> {
    let s = run(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let p = PathBuf::from(s.trim());
    if p.as_os_str().is_empty() {
        return Err("git rev-parse --git-common-dir returned nothing".to_string());
    }
    Ok(p)
}

/// True when `root` is inside a git working tree.
pub fn is_repo(root: &Path) -> bool {
    run(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|s| s.trim() == "true")
        .unwrap_or(false)
}
