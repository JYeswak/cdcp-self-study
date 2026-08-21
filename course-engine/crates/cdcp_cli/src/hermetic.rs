//! Hermetic cargo-test runner (bd-std-hermetic-tree-mo9w).
//!
//! This is intentionally product-owned rather than a shell gate.  It gives
//! each lane a private target directory and fingerprints the source snapshot
//! around the child process.  A child that passes while the source moved is a
//! distinct DRIFT result, not a test failure that can be mistaken for GREEN.

use cdcp_core::sha256_hex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) const DRIFT_PREFIX: &str = "DRIFT:";

/// Run cargo test against one source snapshot and one lane-owned target tree.
pub(crate) fn run(
    root_arg: Option<&Path>,
    lane_arg: Option<&str>,
    args: &[String],
) -> Result<(), String> {
    let root = resolve_workspace(root_arg)?;
    reject_overrides(args)?;
    reject_environment_overrides()?;

    let lane_env = std::env::var("CDCP_TEST_LANE").ok();
    let lane = sanitize_lane(lane_arg.or(lane_env.as_deref()).unwrap_or("default"));
    let target = root.join("target").join("cdcp-hermetic").join(&lane);
    ensure_not_symlink(&root.join("target"))?;
    ensure_not_symlink(&root.join("target").join("cdcp-hermetic"))?;
    ensure_not_symlink(&target)?;
    fs::create_dir_all(&target)
        .map_err(|e| format!("hermetic-test: create target {}: {e}", target.display()))?;
    ensure_not_symlink(&target)?;

    let before = fingerprint(&root)?;
    println!("hermetic-test: lane={lane} target_dir={}", target.display());
    println!(
        "hermetic-test: before head={} product_sha256={} files={}",
        before.head, before.product_sha, before.files
    );

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root)
        .arg("test")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .args(args)
        .env("CARGO_TARGET_DIR", &target)
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let status = cmd
        .status()
        .map_err(|e| format!("hermetic-test: start cargo: {e}"))?;
    let after = fingerprint(&root)?;
    println!(
        "hermetic-test: after  head={} product_sha256={} files={}",
        after.head, after.product_sha, after.files
    );
    if before != after {
        return Err(format!(
            "{DRIFT_PREFIX} source inputs changed during cargo test: before=head={} product_sha256={} files={}, after=head={} product_sha256={} files={}",
            before.head, before.product_sha, before.files, after.head, after.product_sha, after.files
        ));
    }
    if !status.success() {
        return Err(format!("hermetic-test: cargo test exited {status}"));
    }
    println!(
        "hermetic-test: GREEN (source snapshot stable; target_dir={})",
        target.display()
    );
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Fingerprint {
    head: String,
    product_sha: String,
    files: usize,
}

fn fingerprint(root: &Path) -> Result<Fingerprint, String> {
    let head = git(root, &["rev-parse", "HEAD"])?;
    let files = product_files(root)?;
    if files.is_empty() {
        return Err(
            "hermetic-test: product input set is empty — refusing to certify nothing".into(),
        );
    }
    let mut bytes = Vec::new();
    for rel in &files {
        let path = root.join(rel);
        let body =
            fs::read(&path).map_err(|e| format!("hermetic-test: read product input {rel}: {e}"))?;
        bytes.extend_from_slice(rel.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&body);
        bytes.push(0);
    }
    Ok(Fingerprint {
        head,
        product_sha: sha256_hex(&bytes),
        files: files.len(),
    })
}

fn product_files(root: &Path) -> Result<Vec<String>, String> {
    let out = git(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "bank/items",
            "knowledge",
            "tracks",
            "crates/*/src",
            "web",
            "install.sh",
        ],
    )?;
    let mut rows: Vec<String> = out
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    rows.sort();
    rows.dedup();
    Ok(rows)
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|e| format!("hermetic-test: start git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "hermetic-test: git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .trim_end_matches(['\n', '\r'])
        .to_string())
}

fn resolve_workspace(explicit: Option<&Path>) -> Result<PathBuf, String> {
    let start = explicit
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir().map_err(|e| e.to_string())?);
    let mut cur = fs::canonicalize(&start)
        .map_err(|e| format!("hermetic-test: resolve root {}: {e}", start.display()))?;
    if cur.is_file() {
        cur.pop();
    }
    loop {
        if cur.join("Cargo.toml").is_file() && cur.join(".git").exists() {
            return Ok(cur);
        }
        if !cur.pop() {
            break;
        }
    }
    Err(format!(
        "hermetic-test: no workspace root from {}",
        start.display()
    ))
}

fn reject_overrides(args: &[String]) -> Result<(), String> {
    for arg in args {
        if arg == "--config"
            || arg.starts_with("--config=")
            || arg == "--target-dir"
            || arg.starts_with("--target-dir=")
            || arg == "--manifest-path"
            || arg.starts_with("--manifest-path=")
        {
            return Err(format!("hermetic-test: refusing caller override {arg}; target, config, and manifest are wrapper-owned"));
        }
    }
    Ok(())
}

fn reject_environment_overrides() -> Result<(), String> {
    for key in [
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_HOME",
        "CARGO_CONFIG_PATH",
        "RUSTC_WRAPPER",
    ] {
        if std::env::var_os(key).is_some() {
            return Err(format!("hermetic-test: refusing environment override {key}; invoke with a clean test environment"));
        }
    }
    Ok(())
}

fn ensure_not_symlink(path: &Path) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return Err(format!(
                "hermetic-test: refusing symlinked target path {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn sanitize_lane(raw: &str) -> String {
    let mut out: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if out.is_empty() {
        out = "default".into();
    }
    out.truncate(64);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn temp_root(label: &str) -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("cdcp-hermetic-{label}-{}-{n}", std::process::id()));
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("crates/demo/src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=[]\n").unwrap();
        fs::write(root.join("crates/demo/src/lib.rs"), "pub fn stable() {}\n").unwrap();
        root
    }

    fn fake_cargo(root: &Path, mutate: bool) {
        let bin = root.join("fake-bin");
        fs::create_dir_all(&bin).unwrap();
        let path = bin.join("cargo");
        let body = if mutate {
            format!(
                "#!/bin/sh\nsleep 0.05\nprintf 'mutated\\n' >> '{}'\nexit 0\n",
                root.join("crates/demo/src/lib.rs").display()
            )
        } else {
            "#!/bin/sh\nexit 0\n".into()
        };
        fs::write(&path, body).unwrap();
        let mut mode = fs::metadata(&path).unwrap().permissions();
        mode.set_mode(0o755);
        fs::set_permissions(&path, mode).unwrap();
        let git = bin.join("git");
        fs::write(&git, "#!/bin/sh\ncase \"$1 $2\" in\n  'rev-parse HEAD') echo deadbeef;;\n  'ls-files -z') printf 'crates/demo/src/lib.rs\\0';;\n  *) exit 2;;\nesac\n").unwrap();
        let mut git_mode = fs::metadata(&git).unwrap().permissions();
        git_mode.set_mode(0o755);
        fs::set_permissions(&git, git_mode).unwrap();
        let old = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{old}", bin.display()));
    }

    #[test]
    fn stable_source_returns_green_and_names_lane_target() {
        let _guard = env_guard();
        let root = temp_root("green");
        fake_cargo(&root, false);
        std::env::remove_var("CARGO_HOME");
        std::env::remove_var("CARGO_TARGET_DIR");
        std::env::remove_var("CARGO_BUILD_TARGET_DIR");
        std::env::remove_var("CARGO_CONFIG_PATH");
        std::env::remove_var("RUSTC_WRAPPER");
        let out = run(Some(&root), Some("pane-3"), &[]);
        assert!(
            out.is_ok(),
            "known-good runner refused stable source: {out:?}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mid_run_product_mutation_returns_distinct_drift() {
        let _guard = env_guard();
        let root = temp_root("drift");
        fake_cargo(&root, true);
        std::env::remove_var("CARGO_HOME");
        std::env::remove_var("CARGO_TARGET_DIR");
        std::env::remove_var("CARGO_BUILD_TARGET_DIR");
        std::env::remove_var("CARGO_CONFIG_PATH");
        std::env::remove_var("RUSTC_WRAPPER");
        let out = run(Some(&root), Some("pane-3"), &[]);
        let msg = out.expect_err("mid-run product mutation must be DRIFT, not GREEN");
        assert!(msg.starts_with(DRIFT_PREFIX), "wrong verdict: {msg}");
        assert!(
            msg.contains("product_sha256"),
            "DRIFT must name the source fingerprints: {msg}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_target_and_config_overrides() {
        assert!(reject_overrides(&["--target-dir=/tmp/x".into()]).is_err());
        assert!(reject_overrides(&["--config".into(), "foo".into()]).is_err());
    }

    #[test]
    fn rejects_symlinked_target() {
        let root = temp_root("symlink");
        fs::create_dir_all(root.join("target/cdcp-hermetic")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/tmp", root.join("target/cdcp-hermetic/pane-3")).unwrap();
        let err = ensure_not_symlink(&root.join("target/cdcp-hermetic/pane-3")).unwrap_err();
        assert!(err.contains("symlinked"));
        let _ = fs::remove_dir_all(root);
    }
}
