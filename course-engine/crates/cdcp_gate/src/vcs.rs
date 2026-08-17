//! Thin git plumbing adapter.
//!
//! `git` is invoked by argv (`Command::new("git")`), never through a shell — the
//! guard that bans shell does not get to spawn one. Every helper returns
//! engine-root-relative paths, because that is what the registries speak.
//!
//! # TWO SNAPSHOTS, NEVER MIXED
//!
//! A gate that reads its SUBJECT from one snapshot and its POLICY from another is
//! judging a hybrid state that no commit ever has (bd-how, confirmed by injection
//! 2026-08-14). This module therefore exposes the index and HEAD as first-class
//! readable snapshots — `index_text`, `head_text`, `materialise_index` — so a gate
//! can read the allowlist from the same place it read the file list.
//!
//! `run` deliberately INHERITS the caller's `GIT_INDEX_FILE`: when an operator
//! points the gate at a throwaway index, every read must follow it. `run_isolated`
//! is the opposite and is used only for repos this process creates itself, where
//! an inherited `GIT_*` would silently redirect writes at the caller's real repo.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Environment that redirects git at a different repo/index. Cleared for the
/// scratch repos this module creates; never cleared for reads of the caller's.
const GIT_REDIRECT_ENV: &[&str] = &[
    "GIT_INDEX_FILE",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

fn run_with(root: &Path, args: &[&str], isolated: bool) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.current_dir(root).args(args);
    if isolated {
        for k in GIT_REDIRECT_ENV {
            cmd.env_remove(k);
        }
    }
    let out = cmd
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

fn run(root: &Path, args: &[&str]) -> Result<String, String> {
    run_with(root, args, false)
}

/// Raw stdout bytes, for blobs that are not necessarily UTF-8.
///
/// `run` returns `Err` on non-UTF-8 output, which is right for the text it reads
/// (registries, shell scripts) and wrong for an arbitrary tracked blob: a gate
/// that wants to look at a file's first two bytes must not be turned into an
/// ERROR by a tracked `.wasm`. Same environment rules as `run` — the caller's
/// `GIT_*` is inherited deliberately.
fn run_bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
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
    Ok(out.stdout)
}

fn run_isolated(root: &Path, args: &[&str]) -> Result<String, String> {
    run_with(root, args, true)
}

/// Exit-status-only probe. Used where "absent" is a legitimate answer rather
/// than a failure, so the caller can tell it apart from "git broke".
fn run_ok(root: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn split_nul(s: &str) -> Vec<String> {
    s.split('\0')
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

/// A `<rev>:<path>` spec that git resolves RELATIVE TO THE CURRENT DIRECTORY.
///
/// The leading `./` is load-bearing: without it git reads `<path>` from the
/// repository root, which is the parent of the engine root here, and every
/// registry path would silently resolve to nothing.
fn rev_path(rev: &str, path: &str) -> String {
    format!("{rev}:./{path}")
}

/// Files git tracks under `root` (index contents), relative to `root`.
///
/// Index contents — not HEAD — so a file that has merely been `git add`ed is
/// already "in the tree" for this gate's purposes. That is the moment the
/// bleeding starts.
pub fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    Ok(split_nul(&run(root, &["ls-files", "-z"])?))
}

/// One index entry: git's own MODE plus the path.
///
/// The mode is the part `tracked_files` throws away, and it is the part that
/// says whether the entry is an ordinary file (`100644`), an executable
/// (`100755`), a SYMLINK (`120000`) or a SUBMODULE gitlink (`160000`). A gate
/// that only sees paths cannot tell a directory symlink from a directory, and
/// git reports the link — never the tree beneath it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub mode: String,
    pub path: String,
}

/// `git ls-files -s -z`, parsed. Records are `<mode> SP <sha> SP <stage> TAB <path>`.
pub fn tracked_entries(root: &Path) -> Result<Vec<IndexEntry>, String> {
    let raw = run(root, &["ls-files", "-s", "-z"])?;
    let mut out = Vec::new();
    for rec in raw.split('\0').filter(|r| !r.is_empty()) {
        let (meta, path) = rec
            .split_once('\t')
            .ok_or_else(|| format!("git ls-files -s: record has no TAB: {rec:?}"))?;
        let mode = meta
            .split_whitespace()
            .next()
            .ok_or_else(|| format!("git ls-files -s: record has no mode: {rec:?}"))?;
        out.push(IndexEntry {
            mode: mode.to_string(),
            path: path.to_string(),
        });
    }
    Ok(out)
}

/// The BYTES of `path` as the index holds them. `Ok(None)` means the path is not
/// in the index — a real answer, not an error. Unlike `index_text` this survives
/// a binary blob, because "what are this file's first two bytes" is a question
/// that must be answerable about a `.wasm` as well as about a shell script.
pub fn index_bytes(root: &Path, path: &str) -> Result<Option<Vec<u8>>, String> {
    let spec = rev_path("", path);
    if !run_ok(root, &["cat-file", "-e", &spec]) {
        return Ok(None);
    }
    run_bytes(root, &["show", &spec]).map(Some)
}

/// Files this commit would ADD (or copy/rename into place), relative to `root`.
///
/// `--diff-filter=ACR` deliberately omits `M`: editing a file that is already
/// allowlisted must not trip the gate. Modified-but-unlisted files are still
/// caught, by the presence scan.
pub fn staged_additions(root: &Path) -> Result<Vec<String>, String> {
    if !has_head(root) {
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

/// The text of `path` AS THE INDEX HOLDS IT — i.e. as the next commit will
/// contain it. `Ok(None)` means the path is not in the index at all; that is a
/// real answer (the commit deletes the file), not an error, and the caller
/// decides what it means.
pub fn index_text(root: &Path, path: &str) -> Result<Option<String>, String> {
    let spec = rev_path("", path);
    if !run_ok(root, &["cat-file", "-e", &spec]) {
        return Ok(None);
    }
    run(root, &["show", &spec]).map(Some)
}

/// The text of `path` at HEAD. `Ok(None)` for an unborn HEAD or a path HEAD does
/// not carry — both are ordinary states (a fresh repo, a newly added file).
pub fn head_text(root: &Path, path: &str) -> Result<Option<String>, String> {
    if !has_head(root) {
        return Ok(None);
    }
    let spec = rev_path("HEAD", path);
    if !run_ok(root, &["cat-file", "-e", &spec]) {
        return Ok(None);
    }
    run(root, &["show", &spec]).map(Some)
}

/// Write the ENTIRE index out under `dest`, then return the path that
/// corresponds to `root` inside it.
///
/// This is the tree the next commit creates, materialised. Nothing in it comes
/// from the working tree, so a gate run against it cannot be fooled by an edit
/// the author declined to stage.
pub fn materialise_index(root: &Path, dest: &Path) -> Result<PathBuf, String> {
    let dest_s = dest
        .to_str()
        .ok_or_else(|| format!("non-utf8 destination path {}", dest.display()))?;
    let prefix = format!("--prefix={}/", dest_s.trim_end_matches('/'));
    // `checkout-index -a` from a subdirectory only writes that prefix of the
    // index (measured 2026-08-17: dest contained `course-engine/` and no
    // sibling README.md / CHARTER.md). claims_lint roots `../README.md` and
    // `../CHARTER.md` then fail in the nested --prove-wired check.sh before
    // the plant is seen (bd-installability-sm4g.20). The docstring says the
    // ENTIRE index — run checkout-index from the repository root.
    let toplevel = run(root, &["rev-parse", "--show-toplevel"])?;
    let toplevel = PathBuf::from(toplevel.trim());
    if toplevel.as_os_str().is_empty() {
        return Err("git rev-parse --show-toplevel returned nothing".to_string());
    }
    run(&toplevel, &["checkout-index", "-a", "-f", &prefix])?;
    let sub = run(root, &["rev-parse", "--show-prefix"])?;
    let sub = sub.trim();
    if sub.is_empty() {
        Ok(dest.to_path_buf())
    } else {
        Ok(dest.join(sub))
    }
}

/// Make `dir` a self-contained git repo whose index holds everything under it.
///
/// Isolated on purpose: an inherited `GIT_DIR`/`GIT_INDEX_FILE` would point these
/// writes at the caller's real repository.
pub fn init_and_stage_all(dir: &Path) -> Result<(), String> {
    run_isolated(dir, &["init", "-q"])?;
    run_isolated(dir, &["add", "-A", "-f", "--", "."])?;
    Ok(())
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

/// True when this repo has at least one commit.
pub fn has_head(root: &Path) -> bool {
    run_ok(root, &["rev-parse", "--verify", "--quiet", "HEAD"])
}

/// True when `root` is inside a git working tree.
pub fn is_repo(root: &Path) -> bool {
    run(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|s| s.trim() == "true")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rev_paths_are_cwd_relative() {
        // Without the `./` git resolves against the repository root, which is the
        // PARENT of the engine root in this project — every registry read would
        // then miss. Regression pin for bd-how.
        assert_eq!(rev_path("", "registries/a.toml"), ":./registries/a.toml");
        assert_eq!(
            rev_path("HEAD", "scripts/check.sh"),
            "HEAD:./scripts/check.sh"
        );
    }

    #[test]
    fn index_and_head_reads_follow_the_snapshot_they_name() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        std::fs::write(root.join("a.txt"), "committed\n").unwrap();
        run_isolated(&root, &["init", "-q"]).unwrap();
        run_isolated(&root, &["add", "-A"]).unwrap();
        run_isolated(
            &root,
            &[
                "-c",
                "user.email=t@example.invalid",
                "-c",
                "user.name=t",
                "commit",
                "-q",
                "-m",
                "base",
            ],
        )
        .unwrap();

        // Three different contents in three different snapshots.
        std::fs::write(root.join("a.txt"), "staged\n").unwrap();
        run_isolated(&root, &["add", "a.txt"]).unwrap();
        std::fs::write(root.join("a.txt"), "worktree\n").unwrap();

        assert_eq!(head_text(&root, "a.txt").unwrap().unwrap(), "committed\n");
        assert_eq!(index_text(&root, "a.txt").unwrap().unwrap(), "staged\n");
        assert_eq!(
            std::fs::read_to_string(root.join("a.txt")).unwrap(),
            "worktree\n"
        );
        assert_eq!(index_text(&root, "nope.txt").unwrap(), None);
    }

    #[test]
    fn materialise_index_writes_the_staged_content_not_the_worktree_one() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().canonicalize().unwrap();
        std::fs::write(root.join("a.txt"), "staged\n").unwrap();
        run_isolated(&root, &["init", "-q"]).unwrap();
        run_isolated(&root, &["add", "-A"]).unwrap();
        std::fs::write(root.join("a.txt"), "worktree\n").unwrap();

        let out = tempfile::tempdir().unwrap();
        let engine = materialise_index(&root, out.path()).unwrap();
        assert_eq!(
            std::fs::read_to_string(engine.join("a.txt")).unwrap(),
            "staged\n",
            "the materialised tree is the commit's, not the author's desk"
        );
    }

    #[test]
    fn materialise_index_from_subdir_includes_sibling_index_files() {
        // Nested-repo shape of cdcp-self-study: parent README + engine/.
        // checkout-index from the subdirectory used to omit the sibling,
        // so claims_lint `../README.md` died in --prove-wired (sm4g.20).
        let td = tempfile::tempdir().unwrap();
        let repo = td.path().canonicalize().unwrap();
        std::fs::write(repo.join("README.md"), "parent-readme\n").unwrap();
        std::fs::create_dir_all(repo.join("engine")).unwrap();
        std::fs::write(repo.join("engine").join("a.txt"), "engine-file\n").unwrap();
        run_isolated(&repo, &["init", "-q"]).unwrap();
        run_isolated(&repo, &["add", "-A"]).unwrap();

        let out = tempfile::tempdir().unwrap();
        let engine = materialise_index(&repo.join("engine"), out.path()).unwrap();
        assert_eq!(
            engine,
            out.path().join("engine"),
            "engine path is dest + show-prefix"
        );
        assert_eq!(
            std::fs::read_to_string(out.path().join("README.md")).unwrap(),
            "parent-readme\n",
            "parent-corpus files must land next to the engine prefix"
        );
        assert_eq!(
            std::fs::read_to_string(engine.join("a.txt")).unwrap(),
            "engine-file\n"
        );
        assert!(
            !out.path().join("engine").join("README.md").is_file(),
            "parent README is a sibling of engine/, not inside it"
        );
    }
}
