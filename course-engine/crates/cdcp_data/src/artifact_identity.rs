//! Content identity for bytes that cross the release boundary.
//!
//! The release producer and installed-layer verifier consume this module. It
//! binds an artifact digest to the exact source commit, source tree, and
//! dependency blobs used to make it. The digest is always recomputed from the
//! bytes at the path under test; identity metadata is never inherited from a
//! previous build.
//!
//! This is deliberately product code rather than a human-only provenance
//! report: a caller must reject [`verify_artifact_identity`] errors before an
//! artifact can be staged or installed. It can be deleted when the release
//! producer and installed verifier enforce an equivalent contract without it.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

const GIT_OID_HEX_LEN: usize = 40;
const SHA256_HEX_LEN: usize = 64;

/// The normalized identity of one artifact's bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactIdentity {
    /// Engine-root-relative artifact path.
    pub artifact: String,
    /// SHA-256 of the bytes actually read.
    pub sha256: String,
    /// Full source commit object id.
    pub source_revision: String,
    /// Full Git tree object id for `source_revision`.
    pub tree_revision: String,
    /// Full object ids for dependency inputs, keyed by engine-relative path.
    pub dependency_revisions: BTreeMap<String, String>,
}

/// Expected identity read from a staged manifest or an installed package.
///
/// Revision fields may be abbreviated. They are normalized through Git before
/// comparison, so an abbreviated and full spelling of the same object is not
/// drift. The artifact digest is never abbreviated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactIdentityExpectation {
    /// Engine-root-relative artifact path.
    pub artifact: String,
    /// Expected SHA-256 of the artifact bytes.
    pub sha256: String,
    /// Commit reference that produced the artifact.
    pub source_revision: String,
    /// Expected source tree reference; may be abbreviated.
    pub tree_revision: String,
    /// Dependency paths and expected Git object references.
    pub dependency_revisions: BTreeMap<String, String>,
}

/// Parse the machine-consumed identity manifest used by release tooling.
pub fn parse_identity_manifest(text: &str) -> Result<Vec<ArtifactIdentityExpectation>, String> {
    let document: toml::Value = toml::from_str(text)
        .map_err(|e| format!("artifact identity manifest is invalid TOML: {e}"))?;
    if document
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        != Some(1)
    {
        return Err("artifact identity manifest schema_version must be 1".to_string());
    }
    let rows = document
        .get("artifacts")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "artifact identity manifest has no artifacts array".to_string())?;
    if rows.is_empty() {
        return Err("artifact identity manifest has zero artifacts".to_string());
    }
    rows.iter().map(parse_manifest_row).collect()
}

/// Recompute and verify every row in an identity manifest.
pub fn verify_identity_manifest(root: &Path, text: &str) -> Result<Vec<ArtifactIdentity>, String> {
    let expected = parse_identity_manifest(text)?;
    expected
        .iter()
        .map(|row| verify_artifact_identity(root, Path::new(&row.artifact), row))
        .collect()
}

fn parse_manifest_row(row: &toml::Value) -> Result<ArtifactIdentityExpectation, String> {
    let table = row
        .as_table()
        .ok_or_else(|| "artifact identity row is not a table".to_string())?;
    let string = |name: &str| {
        table
            .get(name)
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("artifact identity row is missing string field {name}"))
    };
    let dependency_revisions = table
        .get("dependency_revisions")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "artifact identity row has no dependency_revisions table".to_string())?
        .iter()
        .map(|(path, value)| {
            let revision = value
                .as_str()
                .ok_or_else(|| format!("dependency revision for {path} is not a string"))?;
            Ok((path.clone(), revision.to_string()))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    if dependency_revisions.is_empty() {
        return Err("artifact identity row has zero dependency revisions".to_string());
    }
    Ok(ArtifactIdentityExpectation {
        artifact: string("artifact")?,
        sha256: string("sha256")?,
        source_revision: string("source_revision")?,
        tree_revision: string("tree_revision")?,
        dependency_revisions,
    })
}

/// Recompute identity from the bytes at `artifact` and Git objects named by
/// `source_revision`.
pub fn compute_artifact_identity(
    root: &Path,
    artifact: &Path,
    source_revision: &str,
    dependency_paths: &[&str],
) -> Result<ArtifactIdentity, String> {
    let artifact_path = if artifact.is_absolute() {
        artifact.to_path_buf()
    } else {
        root.join(artifact)
    };
    let bytes = std::fs::read(&artifact_path)
        .map_err(|e| format!("cannot read artifact {}: {e}", artifact_path.display()))?;
    if bytes.is_empty() {
        return Err(format!("artifact is empty: {}", artifact_path.display()));
    }

    let source_revision = normalize_commit_ref(root, source_revision)?;
    let tree_revision = git_object(root, &format!("{source_revision}^{{tree}}"))?;
    let mut dependency_revisions = BTreeMap::new();
    for dependency in dependency_paths {
        let object = git_object(
            root,
            &format!("{source_revision}:{}", git_relative_path(root, dependency)?),
        )?;
        dependency_revisions.insert((*dependency).to_string(), object);
    }

    Ok(ArtifactIdentity {
        artifact: relative_artifact(root, &artifact_path),
        sha256: sha256_hex(&bytes),
        source_revision,
        tree_revision,
        dependency_revisions,
    })
}

/// Compare a freshly recomputed artifact identity with an expected record.
///
/// This function is fail-closed: a digest mismatch, source/tree mismatch, or
/// dependency mismatch is an error, never a warning. The returned identity is
/// the measurement taken from the bytes, which lets a caller print the actual
/// value it rejected.
pub fn verify_artifact_identity(
    root: &Path,
    artifact: &Path,
    expected: &ArtifactIdentityExpectation,
) -> Result<ArtifactIdentity, String> {
    let dependency_paths: Vec<&str> = expected
        .dependency_revisions
        .keys()
        .map(String::as_str)
        .collect();
    let actual =
        compute_artifact_identity(root, artifact, &expected.source_revision, &dependency_paths)?;
    let expected_source = normalize_commit_ref(root, &expected.source_revision)?;
    let expected_tree = normalize_object_ref(root, &expected.tree_revision)?;
    if actual.artifact != expected.artifact {
        return Err(format!(
            "artifact identity path mismatch: expected={} actual={}",
            expected.artifact, actual.artifact
        ));
    }
    require_sha256(&expected.sha256, "expected artifact sha256")?;
    if !actual.sha256.eq_ignore_ascii_case(&expected.sha256) {
        return Err(format!(
            "artifact identity digest mismatch for {}: expected={} computed={}",
            actual.artifact, expected.sha256, actual.sha256
        ));
    }
    if actual.source_revision != expected_source {
        return Err(format!(
            "artifact identity source revision mismatch for {}: expected={} computed={}",
            actual.artifact, expected_source, actual.source_revision
        ));
    }
    if actual.tree_revision != expected_tree {
        return Err(format!(
            "artifact identity tree revision mismatch for {}: expected={} computed={}",
            actual.artifact, expected_tree, actual.tree_revision
        ));
    }
    for (path, expected_ref) in &expected.dependency_revisions {
        let expected_object = normalize_object_ref(root, expected_ref)?;
        let actual_object = actual
            .dependency_revisions
            .get(path)
            .ok_or_else(|| format!("missing computed dependency revision for {path}"))?;
        if actual_object != &expected_object {
            return Err(format!(
                "artifact identity dependency mismatch for {path}: expected={} computed={}",
                expected_object, actual_object
            ));
        }
    }
    Ok(actual)
}

/// Resolve an abbreviated or full commit reference to its full object id.
pub fn normalize_commit_ref(root: &Path, reference: &str) -> Result<String, String> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Err("empty source revision is not an identity".to_string());
    }
    normalize_git_ref(root, &format!("{reference}^{{commit}}"), "source revision")
}

fn normalize_object_ref(root: &Path, reference: &str) -> Result<String, String> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Err("empty object revision is not an identity".to_string());
    }
    normalize_git_ref(root, reference, "object revision")
}

fn git_object(root: &Path, reference: &str) -> Result<String, String> {
    normalize_git_ref(root, reference, "Git object")
}

fn git_relative_path(root: &Path, engine_relative: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-prefix"])
        .output()
        .map_err(|e| format!("cannot execute git while resolving dependency path: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cannot resolve dependency path prefix: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(format!("{prefix}{engine_relative}"))
}

fn normalize_git_ref(root: &Path, reference: &str, label: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", reference])
        .output()
        .map_err(|e| format!("cannot execute git while resolving {label}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cannot resolve {label} {reference:?}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let value = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_ascii_lowercase();
    if value.len() != GIT_OID_HEX_LEN || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!(
            "Git returned a non-full object id for {label} {reference:?}: {value:?}"
        ));
    }
    Ok(value)
}

fn relative_artifact(root: &Path, artifact: &Path) -> String {
    artifact
        .strip_prefix(root)
        .unwrap_or(artifact)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn require_sha256(value: &str, label: &str) -> Result<(), String> {
    let value = value.trim();
    if value.len() != SHA256_HEX_LEN || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!(
            "{label} must be exactly {SHA256_HEX_LEN} lowercase/uppercase hex characters"
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn git(dir: &Path, args: &[&str], date: &str) -> String {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {args:?}");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn fixture() -> (TempDir, String, String, String) {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q"], "2026-08-21T00:00:00Z");
        git(
            dir.path(),
            &["config", "user.email", "test@example.com"],
            "2026-08-21T00:00:00Z",
        );
        git(
            dir.path(),
            &["config", "user.name", "test"],
            "2026-08-21T00:00:00Z",
        );
        fs::create_dir_all(dir.path().join("dist")).unwrap();
        fs::write(dir.path().join("Cargo.lock"), "dependency=one\n").unwrap();
        fs::write(dir.path().join("dist/app.bin"), b"installed bytes\n").unwrap();
        git(dir.path(), &["add", "."], "2026-08-21T00:01:00Z");
        git(
            dir.path(),
            &["commit", "-qm", "fixture"],
            "2026-08-21T00:01:00Z",
        );
        let full = git(dir.path(), &["rev-parse", "HEAD"], "2026-08-21T00:01:00Z");
        let tree = git(
            dir.path(),
            &["rev-parse", "HEAD^{tree}"],
            "2026-08-21T00:01:00Z",
        );
        let dependency = git(
            dir.path(),
            &["rev-parse", "HEAD:Cargo.lock"],
            "2026-08-21T00:01:00Z",
        );
        (dir, full, tree, dependency)
    }

    fn expectation(
        root: &Path,
        source: &str,
        tree: &str,
        dependency: &str,
    ) -> ArtifactIdentityExpectation {
        let bytes = fs::read(root.join("dist/app.bin")).unwrap();
        ArtifactIdentityExpectation {
            artifact: "dist/app.bin".to_string(),
            sha256: sha256_hex(&bytes),
            source_revision: source.to_string(),
            tree_revision: tree.to_string(),
            dependency_revisions: BTreeMap::from([(
                "Cargo.lock".to_string(),
                dependency.to_string(),
            )]),
        }
    }

    #[test]
    fn abbreviated_and_full_same_commit_are_not_drift() {
        let (dir, full, tree, dependency) = fixture();
        let abbreviated = format!("{}", &full[..12]);
        let tree_abbreviated = &tree[..12];
        let dependency_abbreviated = &dependency[..12];
        let expected = expectation(
            dir.path(),
            &abbreviated,
            tree_abbreviated,
            dependency_abbreviated,
        );
        let actual = verify_artifact_identity(dir.path(), Path::new("dist/app.bin"), &expected)
            .expect("abbreviated and full spellings name the same identity");
        assert_eq!(actual.source_revision, full);
        assert_eq!(actual.tree_revision, tree);
        assert_eq!(actual.dependency_revisions["Cargo.lock"], dependency);
    }

    #[test]
    fn one_character_digest_mismatch_is_rejected() {
        let (dir, full, tree, dependency) = fixture();
        let mut expected = expectation(dir.path(), &full, &tree, &dependency);
        let replacement = if expected.sha256.starts_with('0') {
            '1'
        } else {
            '0'
        };
        expected.sha256.replace_range(..1, &replacement.to_string());
        let error =
            verify_artifact_identity(dir.path(), Path::new("dist/app.bin"), &expected).unwrap_err();
        assert!(error.contains("digest mismatch"), "{error}");
        assert!(
            error.contains("expected=") && error.contains("computed="),
            "{error}"
        );
    }

    #[test]
    fn changed_installed_bytes_are_recomputed_and_rejected() {
        let (dir, full, tree, dependency) = fixture();
        let expected = expectation(dir.path(), &full, &tree, &dependency);
        fs::write(
            dir.path().join("dist/app.bin"),
            b"different installed bytes\n",
        )
        .unwrap();
        let error =
            verify_artifact_identity(dir.path(), Path::new("dist/app.bin"), &expected).unwrap_err();
        assert!(error.contains("digest mismatch"), "{error}");
    }

    #[test]
    fn empty_artifact_is_an_error() {
        let (dir, full, tree, dependency) = fixture();
        fs::write(dir.path().join("dist/app.bin"), []).unwrap();
        let expected = expectation(dir.path(), &full, &tree, &dependency);
        let error =
            verify_artifact_identity(dir.path(), Path::new("dist/app.bin"), &expected).unwrap_err();
        assert!(error.contains("artifact is empty"), "{error}");
    }

    #[test]
    fn manifest_round_trip_recomputes_installed_bytes() {
        let (dir, full, tree, dependency) = fixture();
        let expected = expectation(dir.path(), &full, &tree, &dependency);
        let manifest = format!(
            "schema_version = 1\n\n[[artifacts]]\nartifact = \"{}\"\nsha256 = \"{}\"\nsource_revision = \"{}\"\ntree_revision = \"{}\"\ndependency_revisions = {{ \"Cargo.lock\" = \"{}\" }}\n",
            expected.artifact,
            expected.sha256,
            expected.source_revision,
            expected.tree_revision,
            expected.dependency_revisions["Cargo.lock"]
        );
        let actual = verify_identity_manifest(dir.path(), &manifest).unwrap();
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].sha256, expected.sha256);
    }
}
