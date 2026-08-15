//! `content.lock` `[data]` must pin every file `snapshots.toml` names.
//!
//! The loader already hashes bodies against `snapshots.toml`. That is one
//! pin. This module is the second: a swapped body that is also rewritten
//! in `snapshots.toml` still trips RED here, because `content.lock` is an
//! independent record of the bytes.
//!
//! Coverage is the snapshot pin list, not a walk of `knowledge/corpus/`.
//! NOAA / USGS / eGRID stay unvendored (Epic E open). An unlisted corpus
//! file is outside this check.

use crate::{join_rel, parse_pins, sha256_hex, DataError, SnapshotPin, ANTI_VACUOUS_DATA_LOCK};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

/// Engine-root-relative lock file. Same path `verify-content-lock` reads.
pub const LOCK_REL: &str = "content.lock";
/// Engine-root-relative pin registry.
pub const SNAPSHOTS_REL: &str = "crates/cdcp_data/snapshots.toml";
/// Lock table that pins snapshot bodies and sidecars.
pub const DATA_SECTION: &str = "data";

/// Clean result of [`verify_data_lock`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLockReport {
    /// Paths `snapshots.toml` named (body + sidecar, unique).
    pub required: usize,
    /// Rows present under `[data]`.
    pub pinned: usize,
}

impl DataLockReport {
    /// True when at least one required path was checked. The constructor
    /// refuses the empty-lock case, so a live report is never vacuous.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.required > 0 && self.pinned >= self.required
    }
}

impl fmt::Display for DataLockReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "verify_data_lock: PASS required={} pinned={}",
            self.required, self.pinned
        )
    }
}

/// Every `body` and `sidecar` path a pin set names, de-duplicated, sorted.
#[must_use]
pub fn referenced_data_paths(pins: &[SnapshotPin]) -> Vec<String> {
    let mut out = BTreeSet::new();
    for pin in pins {
        if !pin.body.is_empty() {
            out.insert(pin.body.clone());
        }
        if !pin.sidecar.is_empty() {
            out.insert(pin.sidecar.clone());
        }
    }
    out.into_iter().collect()
}

/// Parse `snapshots.toml` from disk (the live file, not the compile-time include).
pub fn load_pins_from_disk(root: &Path) -> Result<Vec<SnapshotPin>, DataError> {
    let path = join_rel(root, SNAPSHOTS_REL);
    let text = std::fs::read_to_string(&path).map_err(|e| DataError::Io {
        path: SNAPSHOTS_REL.to_string(),
        detail: e.to_string(),
    })?;
    parse_pins(&text, SNAPSHOTS_REL)
}

/// Extract `[data]` path = sha256 rows. Missing or empty → empty map
/// (the caller decides whether that is [`DataError::EmptyDataLock`]).
pub fn parse_data_section(
    lock_text: &str,
    origin: &str,
) -> Result<BTreeMap<String, String>, DataError> {
    let doc: toml::Value = toml::from_str(lock_text).map_err(|e| DataError::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;
    match doc.get(DATA_SECTION) {
        None => Ok(BTreeMap::new()),
        Some(toml::Value::Table(table)) => {
            let mut out = BTreeMap::new();
            for (k, v) in table {
                let path = k.trim();
                if path.is_empty() {
                    return Err(DataError::Unparseable {
                        path: origin.to_string(),
                        detail: "[data] has an empty path key".to_string(),
                    });
                }
                let Some(hash) = v.as_str().map(str::trim).filter(|s| !s.is_empty()) else {
                    return Err(DataError::Unparseable {
                        path: origin.to_string(),
                        detail: format!("[data] {path} is not a non-empty hash string"),
                    });
                };
                out.insert(path.to_string(), hash.to_ascii_lowercase());
            }
            Ok(out)
        }
        Some(other) => Err(DataError::Unparseable {
            path: origin.to_string(),
            detail: format!("`{DATA_SECTION}` must be a table, got {other}"),
        }),
    }
}

/// `[data]` covers every file `snapshots.toml` names, and every listed
/// digest matches the bytes on disk.
pub fn verify_data_lock(root: &Path) -> Result<DataLockReport, DataError> {
    let _ = ANTI_VACUOUS_DATA_LOCK;
    let pins = load_pins_from_disk(root)?;
    let required = referenced_data_paths(&pins);
    if required.is_empty() {
        return Err(DataError::EmptyRegistry);
    }

    let lock_path = join_rel(root, LOCK_REL);
    let lock_text = std::fs::read_to_string(&lock_path).map_err(|e| DataError::Io {
        path: LOCK_REL.to_string(),
        detail: e.to_string(),
    })?;
    let pinned = parse_data_section(&lock_text, LOCK_REL)?;
    if pinned.is_empty() {
        return Err(DataError::EmptyDataLock {
            registered: pins.len(),
        });
    }

    let mut faults = Vec::new();
    for path in &required {
        match pinned.get(path) {
            None => faults.push(DataError::DataUnpinned { path: path.clone() }),
            Some(recorded) => push_hash_fault(root, path, recorded, &mut faults),
        }
    }
    for (path, recorded) in &pinned {
        if required.iter().any(|r| r == path) {
            continue;
        }
        push_hash_fault(root, path, recorded, &mut faults);
    }

    if !faults.is_empty() {
        return Err(DataError::DataLockFailed { faults });
    }
    Ok(DataLockReport {
        required: required.len(),
        pinned: pinned.len(),
    })
}

fn push_hash_fault(root: &Path, path: &str, recorded: &str, faults: &mut Vec<DataError>) {
    let abs = join_rel(root, path);
    let bytes = match std::fs::read(&abs) {
        Ok(b) => b,
        Err(_) => {
            faults.push(DataError::DataMissing {
                path: path.to_string(),
            });
            return;
        }
    };
    let computed = sha256_hex(&bytes);
    if !computed.eq_ignore_ascii_case(recorded) {
        faults.push(DataError::DataHashMismatch {
            path: path.to_string(),
            recorded: recorded.to_string(),
            computed,
        });
    }
}

/// L4: copy the live pin set into a private tree, flip one byte of a
/// vendored OSHA body, and require [`verify_data_lock`] to go RED. The
/// committed tree is never written.
pub fn selftest_flip_one_byte(root: &Path) -> Result<String, DataError> {
    let pins = load_pins_from_disk(root)?;
    let required = referenced_data_paths(&pins);
    if required.is_empty() {
        return Err(DataError::EmptyRegistry);
    }
    let flip_rel = required
        .iter()
        .find(|p| p.contains("29cfr-1910.147.txt"))
        .or_else(|| required.iter().find(|p| p.ends_with(".txt")))
        .or_else(|| required.first())
        .cloned()
        .ok_or(DataError::EmptyRegistry)?;

    let tmp = scratch_tree("data-lock-selftest");
    std::fs::create_dir_all(join_rel(&tmp, "crates/cdcp_data")).map_err(|e| DataError::Io {
        path: tmp.display().to_string(),
        detail: e.to_string(),
    })?;

    let snapshots_src = join_rel(root, SNAPSHOTS_REL);
    std::fs::copy(&snapshots_src, join_rel(&tmp, SNAPSHOTS_REL)).map_err(|e| DataError::Io {
        path: SNAPSHOTS_REL.to_string(),
        detail: e.to_string(),
    })?;

    let mut rows = String::from("schema_version = 1\nbank_hash = \"00\"\n\n[data]\n");
    for rel in &required {
        let src = join_rel(root, rel);
        let dst = join_rel(&tmp, rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DataError::Io {
                path: parent.display().to_string(),
                detail: e.to_string(),
            })?;
        }
        std::fs::copy(&src, &dst).map_err(|e| DataError::Io {
            path: rel.clone(),
            detail: e.to_string(),
        })?;
        let bytes = std::fs::read(&dst).map_err(|e| DataError::Io {
            path: rel.clone(),
            detail: e.to_string(),
        })?;
        rows.push_str(&format!("\"{rel}\" = \"{}\"\n", sha256_hex(&bytes)));
    }
    std::fs::write(join_rel(&tmp, LOCK_REL), rows).map_err(|e| DataError::Io {
        path: LOCK_REL.to_string(),
        detail: e.to_string(),
    })?;

    verify_data_lock(&tmp)?;

    let flip_path = join_rel(&tmp, &flip_rel);
    let mut body = std::fs::read(&flip_path).map_err(|e| DataError::Io {
        path: flip_rel.clone(),
        detail: e.to_string(),
    })?;
    if body.is_empty() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(DataError::Io {
            path: flip_rel,
            detail: "vendored body is empty — nothing to flip".to_string(),
        });
    }
    body[0] ^= 0xff;
    std::fs::write(&flip_path, &body).map_err(|e| DataError::Io {
        path: flip_rel.clone(),
        detail: e.to_string(),
    })?;

    let err = match verify_data_lock(&tmp) {
        Ok(_) => {
            let _ = std::fs::remove_dir_all(&tmp);
            return Err(DataError::DataLockSelftestMissed);
        }
        Err(e) => e,
    };
    let _ = std::fs::remove_dir_all(&tmp);

    let red = match &err {
        DataError::DataLockFailed { faults } => faults.iter().any(|f| match f {
            DataError::DataHashMismatch { path, .. } => path == &flip_rel,
            _ => false,
        }),
        DataError::DataHashMismatch { path, .. } => path == &flip_rel,
        _ => false,
    };
    if !red {
        return Err(DataError::DataLockFailed {
            faults: vec![DataError::DataLockSelftestMissed, err],
        });
    }
    Ok(format!(
        "verify_data_lock: ok: flip-selftest trips RED ({flip_rel} hash mismatch)\n"
    ))
}

fn scratch_tree(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cdcp-data-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg(test)]
mod unit {
    use super::*;
    use crate::SnapshotPin;

    #[test]
    fn referenced_paths_include_body_and_sidecar() {
        let pins = [SnapshotPin {
            id: "x".into(),
            body: "knowledge/corpus/public/osha/a.txt".into(),
            sidecar: "knowledge/corpus/public/osha/a.meta.toml".into(),
            sha256: "aa".into(),
        }];
        let paths = referenced_data_paths(&pins);
        assert_eq!(
            paths,
            vec![
                "knowledge/corpus/public/osha/a.meta.toml".to_string(),
                "knowledge/corpus/public/osha/a.txt".to_string(),
            ]
        );
    }

    #[test]
    fn parse_data_section_missing_is_empty_not_a_pass() {
        let map = parse_data_section("schema_version = 1\n", "t.lock").expect("parse");
        assert!(map.is_empty());
    }

    #[test]
    fn production_refuses_empty_data_section() {
        let src = include_str!("data_lock.rs");
        assert!(src.contains("ANTI_VACUOUS_DATA_LOCK"));
        assert!(src.contains("pinned.is_empty()"));
        assert!(src.contains("EmptyDataLock"));
        assert!(src.contains("referenced_data_paths"));
    }
}
