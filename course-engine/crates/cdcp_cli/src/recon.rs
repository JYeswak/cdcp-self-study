//! Helpers for `scripts/selftest_reconstructed.sh`.
//!
//! Not a gate. The reconstructed L5–V11 selftest used to spawn `python3 -c`
//! for isolation snapshots, inode identity, HEAD archive, JSON plants, and
//! cargo-artifact mtimes. Those jobs live here so the script body has no
//! live python3. A missing watch set, an empty archive, or a JSON plant that
//! cannot name its target is RED: silent no-op is how an isolation proof
//! lies.
//!
//! EXTRACT-THEN-DELETE (`bd-extract-reconstructed-python-dxgj`).

use cdcp_core::sha256_hex;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, UNIX_EPOCH};

/// `cdcp recon snapshot-live`.
pub(crate) fn snapshot_live(
    root: &Path,
    fp: &Path,
    clean_out: &Path,
    rels: &[String],
) -> Result<(), String> {
    if rels.is_empty() {
        return Err(
            "recon snapshot-live: no files to snapshot — an empty watch set certifies nothing"
                .into(),
        );
    }
    let missing: Vec<&str> = rels
        .iter()
        .map(String::as_str)
        .filter(|rel| !root.join(rel).is_file())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "recon snapshot-live: live snapshot missing: {}",
            missing.join(" ")
        ));
    }
    let top = git_toplevel(root)?;
    let prefix = repo_prefix(root, &top)?;
    let mut fp_body = String::new();
    let mut clean_body = String::new();
    for rel in rels {
        let path = root.join(rel);
        let digest = sha256_file(&path)?;
        fp_body.push_str(&digest);
        fp_body.push(' ');
        fp_body.push_str(rel);
        fp_body.push('\n');
        let repo_rel = if prefix.is_empty() {
            rel.clone()
        } else {
            format!("{prefix}/{rel}")
        };
        let porcelain = git_output(&top, &["status", "--porcelain", "--", &repo_rel])?;
        if porcelain.trim().is_empty() {
            clean_body.push_str(rel);
            clean_body.push('\n');
        }
    }
    fs::write(fp, fp_body)
        .map_err(|e| format!("recon snapshot-live: cannot write {}: {e}", fp.display()))?;
    fs::write(clean_out, clean_body).map_err(|e| {
        format!(
            "recon snapshot-live: cannot write {}: {e}",
            clean_out.display()
        )
    })?;
    Ok(())
}

/// `cdcp recon assert-unmoved`.
pub(crate) fn assert_unmoved(root: &Path, fp: &Path, label: &str) -> Result<(), String> {
    let raw = fs::read_to_string(fp)
        .map_err(|e| format!("recon assert-unmoved: cannot read {}: {e}", fp.display()))?;
    let mut rows = 0usize;
    let mut bad = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((digest, rel)) = split_once_ws(line) else {
            return Err(format!(
                "recon assert-unmoved: {}:{i} is not '<sha256> <rel>'",
                fp.display()
            ));
        };
        rows += 1;
        let path = root.join(rel);
        if !path.is_file() {
            bad.push(format!("live {rel} vanished during {label}"));
            continue;
        }
        let now = sha256_file(&path)?;
        if now != digest {
            bad.push(format!("live {rel} bytes moved during {label}"));
        }
    }
    if rows == 0 {
        return Err(format!(
            "recon assert-unmoved: {} has no fingerprint rows — nothing to prove unmoved",
            fp.display()
        ));
    }
    if !bad.is_empty() {
        return Err(bad.join("\n"));
    }
    Ok(())
}

/// `cdcp recon assert-git-unmoved`.
pub(crate) fn assert_git_unmoved(live: &Path, clean: &Path, label: &str) -> Result<(), String> {
    let raw = fs::read_to_string(clean).map_err(|e| {
        format!(
            "recon assert-git-unmoved: cannot read {}: {e}",
            clean.display()
        )
    })?;
    let started: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    // An empty clean set is allowed: every watched file may have started dirty.
    // Bytes (`assert-unmoved`) are the isolation property.
    if started.is_empty() {
        return Ok(());
    }
    let top = git_toplevel(live)?;
    let prefix = repo_prefix(live, &top)?;
    let mut bad = Vec::new();
    for rel in started {
        let repo_rel = if prefix.is_empty() {
            rel.to_string()
        } else {
            format!("{prefix}/{rel}")
        };
        let porcelain = git_output(&top, &["status", "--porcelain", "--", &repo_rel])?;
        if !porcelain.trim().is_empty() {
            bad.push(format!(
                "live {rel} was clean and is now dirty ({label}): {porcelain:?}"
            ));
        }
    }
    if !bad.is_empty() {
        return Err(bad.join("\n"));
    }
    Ok(())
}

/// `cdcp recon samefile`. Always prints `0` or `1`. OS errors are `0`.
pub(crate) fn emit_samefile(a: &Path, b: &Path) -> Result<(), String> {
    println!("{}", i32::from(same_inode(a, b)));
    Ok(())
}

/// `cdcp recon archive-head`. Prints the PRIVATE_TREE_COPY_S receipt.
pub(crate) fn archive_head(top: &Path, snap: &Path) -> Result<(), String> {
    if !top.is_dir() {
        return Err(format!(
            "recon archive-head: top {} is not a directory",
            top.display()
        ));
    }
    fs::create_dir_all(snap)
        .map_err(|e| format!("recon archive-head: cannot create {}: {e}", snap.display()))?;
    let tar_path = sibling_tar(snap);
    let t0 = Instant::now();
    let git = Command::new("git")
        .arg("-C")
        .arg(top)
        .args(["archive", "--format=tar", "-o"])
        .arg(&tar_path)
        .arg("HEAD")
        .output()
        .map_err(|e| format!("recon archive-head: spawn git: {e}"))?;
    if !git.status.success() {
        let _ = fs::remove_file(&tar_path);
        return Err(format!(
            "recon archive-head: git archive failed: {}",
            String::from_utf8_lossy(&git.stderr)
        ));
    }
    let tar = Command::new("tar")
        .arg("-C")
        .arg(snap)
        .arg("-xf")
        .arg(&tar_path)
        .output()
        .map_err(|e| format!("recon archive-head: tar: {e}"))?;
    let _ = fs::remove_file(&tar_path);
    if !tar.status.success() {
        return Err(format!(
            "recon archive-head: tar extract failed: {}",
            String::from_utf8_lossy(&tar.stderr)
        ));
    }
    let n = count_files(snap)?;
    if n < 1 {
        return Err("git archive HEAD was empty".into());
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("PRIVATE_TREE_COPY_S={dt:.2} FILES={n} REV=HEAD");
    Ok(())
}

/// Jobs `cdcp recon json-set` can apply. Exactly one per invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JsonJob {
    SetNItems(u64),
    PlantCorrect(String),
    FlipFirstKey,
}

/// `cdcp recon json-set`.
pub(crate) fn json_set(path: &Path, job: &JsonJob) -> Result<(), String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("recon json-set: cannot read {}: {e}", path.display()))?;
    if raw.trim().is_empty() {
        return Err(format!(
            "recon json-set: {} is empty — a 0-byte plant certifies nothing",
            path.display()
        ));
    }
    let mut value: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("recon json-set: {} is not JSON: {e}", path.display()))?;
    let obj = value.as_object_mut().ok_or_else(|| {
        format!(
            "recon json-set: {} root is not a JSON object",
            path.display()
        )
    })?;
    match job {
        JsonJob::SetNItems(n) => {
            obj.insert("n_items".into(), Value::from(*n));
        }
        JsonJob::PlantCorrect(letter) => {
            if letter.is_empty() {
                return Err("recon json-set: empty --plant-correct".into());
            }
            let items = obj
                .get_mut("items")
                .ok_or_else(|| format!("recon json-set: {} has no items array", path.display()))?;
            let arr = items.as_array_mut().ok_or_else(|| {
                format!("recon json-set: {} items is not an array", path.display())
            })?;
            let first = arr.first_mut().ok_or_else(|| {
                format!(
                    "recon json-set: {} items is empty — cannot plant correct",
                    path.display()
                )
            })?;
            let row = first.as_object_mut().ok_or_else(|| {
                format!(
                    "recon json-set: {} items[0] is not an object",
                    path.display()
                )
            })?;
            row.insert("correct".into(), Value::String(letter.clone()));
        }
        JsonJob::FlipFirstKey => {
            let keys = obj
                .get_mut("keys")
                .ok_or_else(|| format!("recon json-set: {} has no keys array", path.display()))?;
            let arr = keys.as_array_mut().ok_or_else(|| {
                format!("recon json-set: {} keys is not an array", path.display())
            })?;
            let first = arr.first_mut().ok_or_else(|| {
                format!(
                    "recon json-set: {} keys is empty — cannot flip correct",
                    path.display()
                )
            })?;
            let row = first.as_object_mut().ok_or_else(|| {
                format!(
                    "recon json-set: {} keys[0] is not an object",
                    path.display()
                )
            })?;
            let current = row.get("correct").and_then(Value::as_str).ok_or_else(|| {
                format!(
                    "recon json-set: {} keys[0].correct is not a string",
                    path.display()
                )
            })?;
            let next = if current != "A" { "A" } else { "B" };
            row.insert("correct".into(), Value::String(next.into()));
        }
    }
    let mut body = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("recon json-set: serialize {}: {e}", path.display()))?;
    body.push('\n');
    fs::write(path, body)
        .map_err(|e| format!("recon json-set: cannot write {}: {e}", path.display()))?;
    Ok(())
}

/// `cdcp recon newest-bin`. Empty stdout when no candidate exists.
pub(crate) fn emit_newest_bin(dir: &Path, name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("recon newest-bin: empty --name".into());
    }
    if let Some(p) = newest_bin(dir, name)? {
        println!("{}", p.display());
    }
    Ok(())
}

/// `cdcp recon mtime-ns`.
pub(crate) fn emit_mtime_ns(path: &Path) -> Result<(), String> {
    println!("{}", mtime_ns(path)?);
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|e| format!("recon: cannot read {}: {e}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

fn same_inode(a: &Path, b: &Path) -> bool {
    let Ok(ma) = fs::metadata(a) else {
        return false;
    };
    let Ok(mb) = fs::metadata(b) else {
        return false;
    };
    ma.dev() == mb.dev() && ma.ino() == mb.ino()
}

fn newest_bin(dir: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    if !dir.is_dir() {
        return Ok(None);
    }
    let prefix = format!("{name}-");
    let mut best: Option<(u128, PathBuf)> = None;
    let entries =
        fs::read_dir(dir).map_err(|e| format!("recon newest-bin: read {}: {e}", dir.display()))?;
    for ent in entries {
        let ent = ent.map_err(|e| format!("recon newest-bin: read {}: {e}", dir.display()))?;
        let path = ent.path();
        let Some(fname) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !fname.starts_with(&prefix) {
            continue;
        }
        if path.extension().is_some() {
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        if meta.permissions().mode() & 0o111 == 0 {
            continue;
        }
        let stamp = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        match &best {
            Some((prev, _)) if stamp <= *prev => {}
            _ => best = Some((stamp, path)),
        }
    }
    Ok(best.map(|(_, p)| p))
}

fn mtime_ns(path: &Path) -> Result<u128, String> {
    let meta =
        fs::metadata(path).map_err(|e| format!("recon mtime-ns: stat {}: {e}", path.display()))?;
    let modified = meta
        .modified()
        .map_err(|e| format!("recon mtime-ns: mtime {}: {e}", path.display()))?;
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos())
}

fn git_toplevel(dir: &Path) -> Result<PathBuf, String> {
    let out = git_output(dir, &["rev-parse", "--show-toplevel"])?;
    let top = out.trim();
    if top.is_empty() {
        return Err(format!(
            "recon: git -C {} rev-parse --show-toplevel was empty",
            dir.display()
        ));
    }
    Ok(PathBuf::from(top))
}

fn repo_prefix(root: &Path, top: &Path) -> Result<String, String> {
    let root_c = fs::canonicalize(root)
        .map_err(|e| format!("recon: canonicalize {}: {e}", root.display()))?;
    let top_c =
        fs::canonicalize(top).map_err(|e| format!("recon: canonicalize {}: {e}", top.display()))?;
    if root_c == top_c {
        return Ok(String::new());
    }
    let rel = root_c.strip_prefix(&top_c).map_err(|_| {
        format!(
            "recon: {} is not inside git toplevel {}",
            root.display(),
            top.display()
        )
    })?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn git_output(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("recon: git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "recon: git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn split_once_ws(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let (digest, rest) = line.split_once(char::is_whitespace)?;
    let rel = rest.trim();
    if digest.is_empty() || rel.is_empty() {
        return None;
    }
    Some((digest, rel))
}

fn sibling_tar(snap: &Path) -> PathBuf {
    match snap.parent() {
        Some(p) if !p.as_os_str().is_empty() => {
            p.join(format!("cdcp-recon-head-{}.tar", std::process::id()))
        }
        _ => std::env::temp_dir().join(format!("cdcp-recon-head-{}.tar", std::process::id())),
    }
}

fn count_files(dir: &Path) -> Result<usize, String> {
    let mut n = 0usize;
    walk_files(dir, &mut n)?;
    Ok(n)
}

fn walk_files(dir: &Path, n: &mut usize) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("recon archive-head: read {}: {e}", dir.display()))?;
    for ent in entries {
        let ent = ent.map_err(|e| format!("recon archive-head: read {}: {e}", dir.display()))?;
        let path = ent.path();
        let ft = ent
            .file_type()
            .map_err(|e| format!("recon archive-head: stat {}: {e}", path.display()))?;
        if ft.is_dir() {
            walk_files(&path, n)?;
        } else if ft.is_file() {
            *n += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::SystemTime;

    fn scratch(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "cdcp_recon_{}_{}_{name}",
            std::process::id(),
            nanos
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn wipe(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    fn git(dir: &Path, args: &[&str]) {
        let st = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .expect("git");
        assert!(st.success(), "git {args:?} failed in {}", dir.display());
    }

    fn init_repo(dir: &Path) {
        git(dir, &["init", "-q", "-b", "main"]);
        git(
            dir,
            &[
                "-c",
                "user.email=cdcp-recon@test",
                "-c",
                "user.name=cdcp-recon",
                "add",
                "-A",
            ],
        );
        git(
            dir,
            &[
                "-c",
                "user.email=cdcp-recon@test",
                "-c",
                "user.name=cdcp-recon",
                "commit",
                "-qm",
                "cdcp-recon fixture",
            ],
        );
    }

    #[test]
    fn snapshot_live_empty_watch_is_red() {
        let dir = scratch("empty_watch");
        let err = snapshot_live(&dir, &dir.join("fp"), &dir.join("clean"), &[]).unwrap_err();
        assert!(err.contains("empty watch"), "{err}");
        wipe(&dir);
    }

    #[test]
    fn snapshot_live_missing_file_is_red() {
        let dir = scratch("missing");
        fs::write(dir.join("keep.txt"), "x\n").unwrap();
        init_repo(&dir);
        let err = snapshot_live(
            &dir,
            &dir.join("fp"),
            &dir.join("clean"),
            &["keep.txt".into(), "gone.txt".into()],
        )
        .unwrap_err();
        assert!(err.contains("gone.txt"), "{err}");
        wipe(&dir);
    }

    #[test]
    fn snapshot_and_assert_round_trip() {
        let dir = scratch("round");
        fs::write(dir.join("keep.txt"), "alpha\n").unwrap();
        init_repo(&dir);
        let fp = dir.join("live.fp");
        let clean = dir.join("live.clean");
        snapshot_live(&dir, &fp, &clean, &["keep.txt".into()]).unwrap();
        let body = fs::read_to_string(&fp).unwrap();
        assert!(body.contains("keep.txt"), "{body}");
        assert_eq!(fs::read_to_string(&clean).unwrap(), "keep.txt\n");
        assert_unmoved(&dir, &fp, "clean").unwrap();
        assert_git_unmoved(&dir, &clean, "clean").unwrap();
        fs::write(dir.join("keep.txt"), "moved\n").unwrap();
        let err = assert_unmoved(&dir, &fp, "after-edit").unwrap_err();
        assert!(err.contains("bytes moved"), "{err}");
        wipe(&dir);
    }

    #[test]
    fn assert_unmoved_empty_fp_is_red() {
        let dir = scratch("empty_fp");
        let fp = dir.join("fp");
        fs::write(&fp, "").unwrap();
        let err = assert_unmoved(&dir, &fp, "x").unwrap_err();
        assert!(err.contains("no fingerprint rows"), "{err}");
        wipe(&dir);
    }

    #[test]
    fn same_inode_self_is_true_missing_is_false() {
        let dir = scratch("inode");
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        fs::write(&a, "x\n").unwrap();
        fs::write(&b, "x\n").unwrap();
        assert!(same_inode(&a, &a));
        assert!(!same_inode(&a, &b));
        assert!(!same_inode(&a, &dir.join("gone")));
        let link = dir.join("link.txt");
        fs::hard_link(&a, &link).unwrap();
        assert!(same_inode(&a, &link));
        wipe(&dir);
    }

    #[test]
    fn archive_head_extracts_and_counts() {
        let dir = scratch("archive");
        fs::create_dir_all(dir.join("course-engine")).unwrap();
        fs::write(dir.join("course-engine/keep.txt"), "body\n").unwrap();
        init_repo(&dir);
        let snap = dir.join("snap");
        archive_head(&dir, &snap).unwrap();
        assert!(snap.join("course-engine/keep.txt").is_file());
        assert_eq!(
            fs::read_to_string(snap.join("course-engine/keep.txt")).unwrap(),
            "body\n"
        );
        wipe(&dir);
    }

    #[test]
    fn json_set_n_items_and_plant_and_flip() {
        let dir = scratch("json");
        let pack = dir.join("pack.json");
        fs::write(
            &pack,
            serde_json::to_string(&json!({
                "n_items": 40,
                "items": [{"id": "q1", "stem": "s"}]
            }))
            .unwrap(),
        )
        .unwrap();
        json_set(&pack, &JsonJob::SetNItems(39)).unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&pack).unwrap()).unwrap();
        assert_eq!(v["n_items"], 39);
        json_set(&pack, &JsonJob::PlantCorrect("A".into())).unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&pack).unwrap()).unwrap();
        assert_eq!(v["items"][0]["correct"], "A");

        let keys = dir.join("keys.json");
        fs::write(
            &keys,
            serde_json::to_string(&json!({
                "keys": [{"correct": "D", "item_id": "q1"}]
            }))
            .unwrap(),
        )
        .unwrap();
        json_set(&keys, &JsonJob::FlipFirstKey).unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&keys).unwrap()).unwrap();
        assert_eq!(v["keys"][0]["correct"], "A");
        json_set(&keys, &JsonJob::FlipFirstKey).unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&keys).unwrap()).unwrap();
        assert_eq!(v["keys"][0]["correct"], "B");
        wipe(&dir);
    }

    #[test]
    fn json_set_empty_and_missing_target_are_red() {
        let dir = scratch("json_red");
        let empty = dir.join("empty.json");
        fs::write(&empty, " \n").unwrap();
        let err = json_set(&empty, &JsonJob::SetNItems(39)).unwrap_err();
        assert!(err.contains("empty"), "{err}");

        let no_items = dir.join("no_items.json");
        fs::write(&no_items, "{\"n_items\":40}\n").unwrap();
        let err = json_set(&no_items, &JsonJob::PlantCorrect("A".into())).unwrap_err();
        assert!(err.contains("no items"), "{err}");

        let no_keys = dir.join("no_keys.json");
        fs::write(&no_keys, "{\"n_items\":40}\n").unwrap();
        let err = json_set(&no_keys, &JsonJob::FlipFirstKey).unwrap_err();
        assert!(err.contains("no keys"), "{err}");
        wipe(&dir);
    }

    #[test]
    fn newest_bin_picks_latest_extensionless_executable() {
        let dir = scratch("bins");
        let deps = dir.join("deps");
        fs::create_dir_all(&deps).unwrap();
        let older = deps.join("s0_charter_pair-aaa");
        let newer = deps.join("s0_charter_pair-bbb");
        let skipped = deps.join("s0_charter_pair-ccc.d");
        fs::write(&older, b"old").unwrap();
        fs::write(&newer, b"new").unwrap();
        fs::write(&skipped, b"d").unwrap();
        for p in [&older, &newer] {
            let mut perm = fs::metadata(p).unwrap().permissions();
            perm.set_mode(0o755);
            fs::set_permissions(p, perm).unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(30));
        fs::write(&newer, b"new2").unwrap();
        let got = newest_bin(&deps, "s0_charter_pair").unwrap().unwrap();
        assert_eq!(got, newer);
        assert!(newest_bin(&deps, "missing").unwrap().is_none());
        wipe(&dir);
    }

    #[test]
    fn mtime_ns_moves_after_rewrite() {
        let dir = scratch("mtime");
        let p = dir.join("f");
        fs::write(&p, "a").unwrap();
        let before = mtime_ns(&p).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        fs::write(&p, "b").unwrap();
        let after = mtime_ns(&p).unwrap();
        assert!(after > before, "before={before} after={after}");
        wipe(&dir);
    }

    #[test]
    fn sha256_empty_matches_published() {
        let dir = scratch("sha");
        let p = dir.join("empty");
        fs::write(&p, b"").unwrap();
        assert_eq!(
            sha256_file(&p).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        wipe(&dir);
    }
}
