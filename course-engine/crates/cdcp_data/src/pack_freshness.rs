//! Freshness coupling between the authored bank and the shipped learner pack.
//!
//! This proves only that a pack commit is newer than the newest bank commit.
//! It does not compare pack contents with the bank, so a fresh but incorrect
//! regeneration remains possible and is covered by the content-lock/golden
//! checks instead.

use std::path::Path;
use std::process::Command;

pub const REQUIRED_PACKS: [&str; 3] = [
    "web/data/bank_items_seed42.json",
    "web/data/keys_seed42.json",
    "web/data/mock40_seed42.json",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFreshnessReport {
    pub bank_commit: String,
    pub bank_epoch: i64,
    pub pack_commit: String,
    pub pack_epoch: i64,
    pub bank_files: usize,
    pub pack_files: usize,
}

impl PackFreshnessReport {
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        self.pack_epoch >= self.bank_epoch
    }
}

pub fn evaluate_pack_freshness(root: &Path) -> Result<PackFreshnessReport, String> {
    if !root.is_dir() {
        return Err(format!(
            "engine root is not a directory: {}",
            root.display()
        ));
    }

    let bank_dir = root.join("bank/items");
    if !bank_dir.is_dir() {
        return Err(format!(
            "missing bank/items directory: {}",
            bank_dir.display()
        ));
    }
    let bank_files = std::fs::read_dir(&bank_dir)
        .map_err(|e| format!("cannot read {}: {e}", bank_dir.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("toml"))
        .count();
    if bank_files == 0 {
        return Err("zero bank/items TOML files is an ERROR".to_string());
    }

    for rel in REQUIRED_PACKS {
        let path = root.join(rel);
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("missing required learner pack {rel}: {e}"))?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(format!(
                "required learner pack is empty or not a file: {rel}"
            ));
        }
    }

    let (bank_commit, bank_epoch) = latest_commit(root, &["bank/items"])?;
    let (pack_commit, pack_epoch) = latest_commit(root, &REQUIRED_PACKS)?;
    Ok(PackFreshnessReport {
        bank_commit,
        bank_epoch,
        pack_commit,
        pack_epoch,
        bank_files,
        pack_files: REQUIRED_PACKS.len(),
    })
}

fn latest_commit(root: &Path, paths: &[&str]) -> Result<(String, i64), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%H%x09%ct", "--"])
        .args(paths)
        .output()
        .map_err(|e| format!("cannot execute git for {:?}: {e}", paths))?;
    if !output.status.success() {
        return Err(format!(
            "git history unavailable for {:?}: {}",
            paths,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let (sha, epoch) = line
        .split_once('\t')
        .ok_or_else(|| format!("git returned no usable commit for {paths:?}"))?;
    let epoch = epoch
        .parse::<i64>()
        .map_err(|e| format!("invalid git commit timestamp for {paths:?}: {e}"))?;
    if sha.is_empty() {
        return Err(format!("git returned an empty commit for {paths:?}"));
    }
    Ok((sha.to_string(), epoch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn git(dir: &Path, args: &[&str], date: &str) {
        let status = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    fn fixture() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q"], "2026-08-20T00:00:00Z");
        git(
            dir.path(),
            &["config", "user.email", "test@example.com"],
            "2026-08-20T00:00:00Z",
        );
        git(
            dir.path(),
            &["config", "user.name", "test"],
            "2026-08-20T00:00:00Z",
        );
        fs::create_dir_all(dir.path().join("bank/items")).unwrap();
        fs::write(
            dir.path().join("bank/items/a.toml"),
            "status=\"approved\"\n",
        )
        .unwrap();
        for rel in REQUIRED_PACKS {
            let path = dir.path().join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "{}\n").unwrap();
        }
        git(dir.path(), &["add", "."], "2026-08-20T00:01:00Z");
        git(
            dir.path(),
            &["commit", "-qm", "pack"],
            "2026-08-20T00:01:00Z",
        );
        dir
    }

    #[test]
    fn stale_bank_commit_is_detected_and_refresh_is_green() {
        let dir = fixture();
        fs::write(
            dir.path().join("bank/items/a.toml"),
            "status=\"approved\"\ntopic=\"new\"\n",
        )
        .unwrap();
        git(
            dir.path(),
            &["add", "bank/items/a.toml"],
            "2026-08-20T00:02:00Z",
        );
        git(
            dir.path(),
            &["commit", "-qm", "bank"],
            "2026-08-20T00:02:00Z",
        );
        let stale = evaluate_pack_freshness(dir.path()).unwrap();
        assert!(!stale.is_fresh());
        assert!(stale.bank_epoch > stale.pack_epoch);

        fs::write(dir.path().join(REQUIRED_PACKS[0]), "refreshed\n").unwrap();
        git(
            dir.path(),
            &["add", "web/data/bank_items_seed42.json"],
            "2026-08-20T00:03:00Z",
        );
        git(
            dir.path(),
            &["commit", "-qm", "refresh"],
            "2026-08-20T00:03:00Z",
        );
        let fresh = evaluate_pack_freshness(dir.path()).unwrap();
        assert!(fresh.is_fresh());
        assert_eq!(fresh.pack_files, REQUIRED_PACKS.len());
    }

    #[test]
    fn missing_pack_is_an_error_not_a_vacuous_pass() {
        let dir = fixture();
        fs::remove_file(dir.path().join(REQUIRED_PACKS[1])).unwrap();
        let error = evaluate_pack_freshness(dir.path()).unwrap_err();
        assert!(error.contains("missing required learner pack"), "{error}");
    }
}
