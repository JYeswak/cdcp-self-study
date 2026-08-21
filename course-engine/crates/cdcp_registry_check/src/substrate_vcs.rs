//! Git snapshot plumbing used by the extracted substrate policy.
//!
//! The substrate scan must judge the same index and bytes. This small adapter
//! is kept beside that policy so moving the assertion logic out of
//! `cdcp_gate` does not reintroduce a gate-to-product dependency.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

const GIT_REDIRECT_ENV: &[&str] = &[
    "GIT_INDEX_FILE",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

fn run_with(root: &Path, args: &[&str], isolated: bool) -> Result<String, String> {
    let mut command = Command::new("git");
    command.current_dir(root).args(args);
    if isolated {
        for variable in GIT_REDIRECT_ENV {
            command.env_remove(variable);
        }
    }
    let output = command
        .output()
        .map_err(|error| format!("git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed (status {:?}): {}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("git {}: non-utf8 output: {error}", args.join(" ")))
}

fn run(root: &Path, args: &[&str]) -> Result<String, String> {
    run_with(root, args, false)
}

fn run_bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|error| format!("git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed (status {:?}): {}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

fn run_isolated(root: &Path, args: &[&str]) -> Result<String, String> {
    run_with(root, args, true)
}

fn run_ok(root: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn split_nul(value: &str) -> Vec<String> {
    value
        .split('\0')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn rev_path(revision: &str, path: &str) -> String {
    format!("{revision}:./{path}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub mode: String,
    pub path: String,
}

pub fn tracked_entries(root: &Path) -> Result<Vec<IndexEntry>, String> {
    let raw = run(root, &["ls-files", "-s", "-z"])?;
    let mut entries = Vec::new();
    for record in raw.split('\0').filter(|record| !record.is_empty()) {
        let (metadata, path) = record
            .split_once('\t')
            .ok_or_else(|| format!("git ls-files -s: record has no TAB: {record:?}"))?;
        let mode = metadata
            .split_whitespace()
            .next()
            .ok_or_else(|| format!("git ls-files -s: record has no mode: {record:?}"))?;
        entries.push(IndexEntry {
            mode: mode.to_string(),
            path: path.to_string(),
        });
    }
    Ok(entries)
}

pub fn index_bytes(root: &Path, path: &str) -> Result<Option<Vec<u8>>, String> {
    let spec = rev_path("", path);
    if !run_ok(root, &["cat-file", "-e", &spec]) {
        return Ok(None);
    }
    run_bytes(root, &["show", &spec]).map(Some)
}

pub fn staged_additions(root: &Path) -> Result<Vec<String>, String> {
    if !has_head(root) {
        let tracked = run(root, &["ls-files", "-z"])?;
        return Ok(split_nul(&tracked));
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

pub fn index_text(root: &Path, path: &str) -> Result<Option<String>, String> {
    let spec = rev_path("", path);
    if !run_ok(root, &["cat-file", "-e", &spec]) {
        return Ok(None);
    }
    run(root, &["show", &spec]).map(Some)
}

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

pub fn materialise_index(root: &Path, destination: &Path) -> Result<PathBuf, String> {
    let destination = destination
        .to_str()
        .ok_or_else(|| format!("non-utf8 destination path {}", destination.display()))?;
    let prefix = format!("--prefix={}/", destination.trim_end_matches('/'));
    let top_level = PathBuf::from(run(root, &["rev-parse", "--show-toplevel"])?.trim());
    if top_level.as_os_str().is_empty() {
        return Err("git rev-parse --show-toplevel returned nothing".into());
    }
    run(&top_level, &["checkout-index", "-a", "-f", &prefix])?;
    let prefix = run(root, &["rev-parse", "--show-prefix"])?;
    let prefix = prefix.trim();
    if prefix.is_empty() {
        Ok(PathBuf::from(destination))
    } else {
        Ok(Path::new(destination).join(prefix))
    }
}

pub fn prepare_probe_lockfile(engine: &Path) -> Result<(), String> {
    if !engine.join("Cargo.toml").is_file() {
        return Ok(());
    }
    let output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(engine)
        .output()
        .map_err(|error| format!("scratch lockfile: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "scratch lockfile failed ({:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub fn materialise_probe_index(root: &Path, destination: &Path) -> Result<PathBuf, String> {
    let engine = materialise_index(root, destination)?;
    prepare_probe_lockfile(&engine)?;
    Ok(engine)
}

pub fn init_and_stage_all(directory: &Path) -> Result<(), String> {
    run_isolated(directory, &["init", "-q"])?;
    run_isolated(directory, &["add", "-A", "-f", "--", "."])?;
    run_isolated(
        directory,
        &[
            "-c",
            "user.email=cdcp-probe@example.invalid",
            "-c",
            "user.name=cdcp substrate probe",
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "cdcp substrate probe snapshot",
        ],
    )?;
    Ok(())
}

pub fn has_head(root: &Path) -> bool {
    run_ok(root, &["rev-parse", "--verify", "--quiet", "HEAD"])
}

pub fn is_repo(root: &Path) -> bool {
    run(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|value| value.trim() == "true")
        .unwrap_or(false)
}
