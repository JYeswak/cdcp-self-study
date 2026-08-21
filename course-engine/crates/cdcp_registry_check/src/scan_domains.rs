//! G2 input-set law for load-bearing scans.
//!
//! A scanner's declared root is part of its meaning.  Walking the engine root,
//! `target/`, or agent state such as `.beads/` silently changes the population
//! under test and can make a gate appear green while it has covered the wrong
//! bytes.  The registry below is deliberately a domain declaration, not an
//! exact count pin: legitimate bank edits change file counts.  We report the
//! declared root and the observed file count on every run instead.
#![forbid(unsafe_code)]

use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const REGISTRY_PATH: &str = "registries/scan_domains.toml";

const FORBIDDEN_ROOT_COMPONENTS: &[&str] = &[".beads", ".flywheel", ".git", "target"];

#[derive(Debug, Clone, Deserialize)]
pub struct Registry {
    pub schema_version: u32,
    #[serde(default)]
    pub scan: Vec<ScanDomain>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanDomain {
    pub id: String,
    pub root: String,
    pub consumer: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    pub id: String,
    pub stated_root: String,
    pub actual_files: usize,
}

pub fn load_registry(root: &Path) -> Result<Registry, String> {
    let path = root.join(REGISTRY_PATH);
    let text = fs::read_to_string(&path).map_err(|error| format!("read {REGISTRY_PATH}: {error}"))?;
    toml::from_str(&text).map_err(|error| format!("parse {REGISTRY_PATH}: {error}"))
}

/// Validate the schema and measure every declared input domain.
pub fn measure(root: &Path) -> Result<Vec<Measurement>, String> {
    let registry = load_registry(root)?;
    validate_registry(&registry)?;

    let engine = root
        .canonicalize()
        .map_err(|error| format!("canonicalize engine root {}: {error}", root.display()))?;
    let mut measurements = Vec::with_capacity(registry.scan.len());
    for row in registry.scan {
        let path = bounded_path(&engine, &row.root)?;
        let actual_files = count_files(&path)?;
        if actual_files == 0 {
            return Err(format!(
                "scan domain {} is empty: stated_root={} — empty input is ERROR, not PASS",
                row.id, row.root
            ));
        }
        measurements.push(Measurement {
            id: row.id,
            stated_root: row.root,
            actual_files,
        });
    }
    Ok(measurements)
}

pub fn validate_registry(registry: &Registry) -> Result<(), String> {
    if registry.schema_version != 1 {
        return Err(format!(
            "{REGISTRY_PATH}: schema_version {} unsupported (expected 1)",
            registry.schema_version
        ));
    }
    if registry.scan.is_empty() {
        return Err(format!(
            "{REGISTRY_PATH}: zero [[scan]] rows — empty registry is ERROR, not PASS"
        ));
    }

    let mut ids = BTreeSet::new();
    for (index, row) in registry.scan.iter().enumerate() {
        let label = if row.id.trim().is_empty() {
            format!("[[scan]] #{}", index + 1)
        } else {
            format!("[[scan]] {}", row.id.trim())
        };
        if row.id.trim().is_empty() {
            return Err(format!("{label}: missing id"));
        }
        if !ids.insert(row.id.trim().to_string()) {
            return Err(format!("{label}: duplicate id"));
        }
        if row.consumer.trim().is_empty() {
            return Err(format!("{label}: missing consumer"));
        }
        if row.reason.trim().is_empty() {
            return Err(format!("{label}: blank reason is SCHEMA ERROR"));
        }
        if row.root.trim().is_empty() {
            return Err(format!("{label}: missing root"));
        }
        validate_relative_root(&row.root).map_err(|error| format!("{label}: {error}"))?;
    }
    Ok(())
}

fn validate_relative_root(root: &str) -> Result<(), String> {
    let path = Path::new(root);
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(format!(
                    "root {root:?} escapes the engine or is absolute; scan roots must be bounded"
                ));
            }
            Component::CurDir => {
                return Err(format!(
                    "root {root:?} resolves to the repository root; scan a named product domain"
                ));
            }
            Component::Normal(value)
                if FORBIDDEN_ROOT_COMPONENTS
                    .iter()
                    .any(|forbidden| value == *forbidden) =>
            {
                return Err(format!(
                    "root {root:?} names untracked state {value:?}; scan product inputs only"
                ));
            }
            Component::Normal(_) => {}
        }
    }
    Ok(())
}

fn bounded_path(engine: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_relative_root(relative)?;
    let path = engine.join(relative);
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("scan root {relative:?} is unreadable: {error}"))?;
    if canonical == engine {
        return Err(format!(
            "scan root {relative:?} resolves to the repository root; input set is unbounded"
        ));
    }
    if !canonical.starts_with(engine) {
        return Err(format!(
            "scan root {relative:?} resolves outside the engine; input set is unbounded"
        ));
    }
    Ok(canonical)
}

fn count_files(path: &Path) -> Result<usize, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("stat {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "scan domain contains symlink {} — refusing an unbounded input set",
            path.display()
        ));
    }
    if metadata.is_file() {
        return Ok(1);
    }
    let mut count = 0;
    let entries = fs::read_dir(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("read {} entry: {error}", path.display()))?;
        let child = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| format!("stat {} entry: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "scan domain contains symlink {} — refusing an unbounded input set",
                child.display()
            ));
        } else if metadata.is_dir() {
            count += count_files(&child)?;
        } else if metadata.is_file() {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn engine_root() -> PathBuf {
        cdcp_root::walk_engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    #[test]
    fn live_registry_reports_nonempty_domains_and_counts() {
        let measurements = measure(&engine_root()).unwrap();
        assert!(measurements.len() >= 10);
        assert!(measurements.iter().all(|row| row.actual_files > 0));
        assert!(measurements.iter().any(|row| {
            row.id == "bank-items" && row.stated_root == "bank/items" && row.actual_files >= 900
        }));
    }

    #[test]
    fn repository_root_and_untracked_state_are_rejected() {
        for root in [".", "target", "target/debug", ".beads", ".beads/.br_history", ".git"] {
            let error = validate_relative_root(root).unwrap_err();
            assert!(
                error.contains("repository root") || error.contains("untracked state"),
                "root {root:?} had an unhelpful error: {error}"
            );
        }
    }

    #[test]
    fn escape_and_symlinked_outside_root_are_rejected() {
        assert!(validate_relative_root("../").is_err());
        assert!(validate_relative_root("/tmp").is_err());

        let temp = tempfile::tempdir().unwrap();
        let engine = temp.path().join("engine");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&engine).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret"), "not a product input").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, engine.join("linked")).unwrap();
        #[cfg(unix)]
        assert!(bounded_path(&engine, "linked").is_err());
    }

    #[test]
    fn empty_registry_and_blank_reason_are_schema_errors() {
        let empty = Registry {
            schema_version: 1,
            scan: Vec::new(),
        };
        assert!(validate_registry(&empty).unwrap_err().contains("zero [[scan]]"));

        let blank = Registry {
            schema_version: 1,
            scan: vec![ScanDomain {
                id: "x".into(),
                root: "bank/items".into(),
                consumer: "test".into(),
                reason: "".into(),
            }],
        };
        assert!(validate_registry(&blank).unwrap_err().contains("blank reason"));
    }

    #[test]
    fn zero_files_is_error_not_a_clean_scan() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("registries")).unwrap();
        fs::create_dir_all(root.join("empty")).unwrap();
        let mut registry = fs::File::create(root.join(REGISTRY_PATH)).unwrap();
        writeln!(
            registry,
            "schema_version = 1\n\n[[scan]]\nid = \"empty\"\nroot = \"empty\"\nconsumer = \"test\"\nreason = \"known-bad empty-domain specimen\""
        )
        .unwrap();
        assert!(measure(root).unwrap_err().contains("empty: stated_root=empty"));
    }
}
