//! Local release producer and stranger-facing staged-artifact verifier.
//!
//! This is intentionally local-only. It never tags, uploads, or publishes.
//! The producer refuses a dirty source tree, stages exactly one root-level
//! regular `cdcp` member, records identity beside the archive, and verifies
//! the archive before reporting it usable.

use cdcp_core::sha256_hex;
use cdcp_data::{compute_artifact_identity, verify_identity_manifest};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn build(
    root: &Path,
    out: &Path,
    target: Option<&str>,
    source: &str,
) -> Result<(), String> {
    let out_rel = relative_output(out)?;
    let target = match target {
        Some(target) => target.to_string(),
        None => host_target().ok_or_else(|| "cannot determine target triple".to_string())?,
    };
    let archive_rel = out_rel.join(format!("cdcp-{target}.tar.gz"));
    let archive = root.join(&archive_rel);
    let manifest = root.join(&out_rel).join("artifact-identity.toml");
    reject_existing(&archive, &manifest)?;

    let source_revision = git(
        root,
        &["rev-parse", "--verify", &format!("{source}^{{commit}}")],
    )?;
    let source_tree = git(
        root,
        &[
            "rev-parse",
            "--verify",
            &format!("{source_revision}^{{tree}}"),
        ],
    )?;
    ensure_clean_source(root, &source_revision)?;
    fs::create_dir_all(root.join(&out_rel))
        .map_err(|e| format!("cannot create {}: {e}", root.join(&out_rel).display()))?;

    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(root)
        .args([
            "build",
            "--locked",
            "--release",
            "-p",
            "cdcp_cli",
            "--target",
        ])
        .arg(&target);
    let status = cargo
        .status()
        .map_err(|e| format!("cannot execute cargo build: {e}"))?;
    if !status.success() {
        return Err(format!("cargo build failed with {status}"));
    }
    let built = root.join("target").join(&target).join("release/cdcp");
    if !built.is_file() {
        return Err(format!("release binary is missing: {}", built.display()));
    }

    let stage = root
        .join(&out_rel)
        .join(format!(".stage-{}", std::process::id()));
    fs::create_dir(&stage)
        .map_err(|e| format!("cannot create staging directory {}: {e}", stage.display()))?;
    let result = (|| {
        fs::copy(&built, stage.join("cdcp"))
            .map_err(|e| format!("cannot stage {}: {e}", built.display()))?;
        let status = Command::new("tar")
            .current_dir(root)
            .args(["-C"])
            .arg(&stage)
            .args(["-czf"])
            .arg(&archive)
            .arg("cdcp")
            .status()
            .map_err(|e| format!("cannot execute tar: {e}"))?;
        if !status.success() {
            return Err(format!("tar failed with {status}"));
        }
        let bytes = fs::read(&archive).map_err(|e| format!("cannot read archive: {e}"))?;
        let digest = sha256_hex(&bytes);
        fs::write(
            checksum_path(&archive),
            format!(
                "{digest}  {}\n",
                archive.file_name().unwrap().to_string_lossy()
            ),
        )
        .map_err(|e| format!("cannot write archive checksum: {e}"))?;
        let identity =
            compute_artifact_identity(root, &archive_rel, &source_revision, &["Cargo.lock"])?;
        if identity.sha256 != digest {
            return Err(format!(
                "archive digest changed during identity computation: tar={digest} identity={}",
                identity.sha256
            ));
        }
        fs::write(&manifest, render_manifest(&identity))
            .map_err(|e| format!("cannot write identity manifest: {e}"))?;
        verify(root, &archive, &manifest)?;
        println!(
            "release: PASS archive={} sha256={} source={} tree={} target={target}",
            archive.display(),
            digest,
            source_revision,
            source_tree
        );
        Ok(())
    })();
    let _ = fs::remove_dir_all(&stage);
    result
}

pub(crate) fn verify(root: &Path, archive: &Path, manifest: &Path) -> Result<(), String> {
    let checksum = checksum_path(archive);
    let expected = fs::read_to_string(&checksum)
        .map_err(|e| format!("missing checksum {}: {e}", checksum.display()))?
        .split_whitespace()
        .next()
        .ok_or_else(|| "checksum file is empty".to_string())?
        .to_string();
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("checksum file does not contain a 64-hex digest".to_string());
    }
    let actual = sha256_hex(&fs::read(archive).map_err(|e| format!("cannot read archive: {e}"))?);
    if !actual.eq_ignore_ascii_case(&expected) {
        return Err(format!(
            "archive sha256 mismatch: expected={expected} computed={actual}"
        ));
    }
    let listing = command_output("tar", &["-tzf", &archive.display().to_string()])?;
    if listing.trim() != "cdcp" {
        return Err(format!(
            "archive must contain exactly one root-level member named cdcp (got {})",
            listing.trim()
        ));
    }
    let verbose = command_output("tar", &["-tvzf", &archive.display().to_string()])?;
    if !verbose.lines().next().unwrap_or_default().starts_with('-') {
        return Err("archive member cdcp is not a regular file".to_string());
    }
    let identities = verify_identity_manifest(
        root,
        &fs::read_to_string(manifest)
            .map_err(|e| format!("cannot read identity manifest {}: {e}", manifest.display()))?,
    )?;
    if identities.len() != 1 {
        return Err(format!(
            "release identity manifest must contain one archive, got {}",
            identities.len()
        ));
    }
    println!(
        "release: verified archive={} sha256={actual} member=cdcp",
        archive.display()
    );
    Ok(())
}

fn relative_output(out: &Path) -> Result<PathBuf, String> {
    if out.is_absolute() || out.components().any(|c| c.as_os_str() == "..") {
        return Err("release output must be repository-relative and contain no '..'".to_string());
    }
    Ok(out.to_path_buf())
}

fn reject_existing(archive: &Path, manifest: &Path) -> Result<(), String> {
    if archive.exists() {
        return Err(format!(
            "refusing to overwrite existing archive: {}",
            archive.display()
        ));
    }
    if manifest.exists() || checksum_path(archive).exists() {
        return Err(format!(
            "refusing to overwrite existing release metadata beside {}",
            archive.display()
        ));
    }
    Ok(())
}

fn ensure_clean_source(root: &Path, source: &str) -> Result<(), String> {
    let diff = Command::new("git")
        .current_dir(root)
        .args(["diff", "--quiet", source, "--", "."])
        .status()
        .map_err(|e| format!("cannot inspect source worktree: {e}"))?;
    if !diff.success() {
        return Err(format!(
            "source worktree differs from {source}; refusing to label a dirty build"
        ));
    }
    let status = git(
        root,
        &["status", "--porcelain", "--untracked-files=all", "--"],
    )?;
    if !status.trim().is_empty() {
        return Err(
            "source worktree has untracked files; refusing to label a dirty build".to_string(),
        );
    }
    Ok(())
}

fn checksum_path(archive: &Path) -> PathBuf {
    let name = archive
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    archive.with_file_name(format!("{name}.sha256"))
}

fn host_target() -> Option<String> {
    command_output("rustc", &["-vV"]).ok().and_then(|text| {
        text.lines()
            .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
    })
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    command_output_in("git", root, args)
}

fn command_output(program: &str, args: &[&str]) -> Result<String, String> {
    command_output_in(program, Path::new("."), args)
}

fn command_output_in(program: &str, cwd: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| format!("cannot execute {program}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn render_manifest(identity: &cdcp_data::ArtifactIdentity) -> String {
    let mut out = String::from("schema_version = 1\n\n[[artifacts]]\n");
    out.push_str(&format!("artifact = \"{}\"\n", identity.artifact));
    out.push_str(&format!("sha256 = \"{}\"\n", identity.sha256));
    out.push_str(&format!(
        "source_revision = \"{}\"\n",
        identity.source_revision
    ));
    out.push_str(&format!("tree_revision = \"{}\"\n", identity.tree_revision));
    out.push_str("dependency_revisions = { ");
    for (i, (path, revision)) in identity.dependency_revisions.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{path}\" = \"{revision}\""));
    }
    out.push_str(" }\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct Fixture {
        root: PathBuf,
        dir: TempDir,
        archive: PathBuf,
        manifest: PathBuf,
    }

    fn fixture() -> Fixture {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("cdcp_cli is crates/cdcp_cli")
            .to_path_buf();
        let dir = tempfile::tempdir().unwrap();
        let stage = dir.path().join("stage");
        fs::create_dir(&stage).unwrap();
        fs::copy(std::env::current_exe().unwrap(), stage.join("cdcp")).unwrap();
        let archive = dir.path().join("cdcp-test.tar.gz");
        let status = Command::new("tar")
            .args(["-C"])
            .arg(&stage)
            .args(["-czf"])
            .arg(&archive)
            .arg("cdcp")
            .status()
            .unwrap();
        assert!(status.success());
        let digest = sha256_hex(&fs::read(&archive).unwrap());
        fs::write(
            checksum_path(&archive),
            format!("{digest}  cdcp-test.tar.gz\n"),
        )
        .unwrap();
        let source = git(&root, &["rev-parse", "HEAD"]).unwrap();
        let identity = compute_artifact_identity(&root, &archive, &source, &["Cargo.lock"])
            .expect("identity fixture");
        let manifest = dir.path().join("identity.toml");
        fs::write(&manifest, render_manifest(&identity)).unwrap();
        Fixture {
            root,
            dir,
            archive,
            manifest,
        }
    }

    #[test]
    fn release_verify_accepts_one_member_archive() {
        let fixture = fixture();
        verify(&fixture.root, &fixture.archive, &fixture.manifest).unwrap();
        assert!(fixture.dir.path().is_dir());
    }

    #[test]
    fn release_verify_rejects_one_character_identity_digest_mismatch() {
        let fixture = fixture();
        let text = fs::read_to_string(&fixture.manifest).unwrap();
        let digest = text
            .lines()
            .find_map(|line| {
                line.strip_prefix("sha256 = \"")
                    .map(|v| v.trim_end_matches('"'))
            })
            .unwrap();
        let replacement = if digest.starts_with('0') { '1' } else { '0' };
        let mut bad = text.clone();
        bad = bad.replacen(digest, &format!("{replacement}{}", &digest[1..]), 1);
        fs::write(&fixture.manifest, bad).unwrap();
        let error = verify(&fixture.root, &fixture.archive, &fixture.manifest).unwrap_err();
        assert!(error.contains("digest mismatch"), "{error}");
    }
}
