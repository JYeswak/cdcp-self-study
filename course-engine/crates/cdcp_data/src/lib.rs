//! Licence-gated, content-addressed snapshot loader.
//!
//! Product crate for E1 (`bd-hardening-e-data-4up.1`). Bodies are vendored
//! on disk. Identity and sha256 are compiled in from `snapshots.toml`.
//! This crate never opens a socket; pins are the only source of truth
//! for *which* bytes may be read.
//!
//! Rights are not re-derived here. [`load_one`] calls
//! [`cdcp_evidence::may_load`] and records
//! [`cdcp_evidence::ArtifactMeta::eligible_for_agent_index`]. A missing
//! licence line, `ai_ingestion=PROHIBITED`, or `redistribution` other
//! than `permitted` is a refusal, not a warning.
//!
//! OSHA / eCFR extracts (E3) live in [`osha`]: the 1910.147(a)(1)(ii)(D)
//! exclusion is a first-class fact, and concurrent maintainability is
//! the 1910.333 isolation constraint.
//!
//! The F3 differential harness lives in [`oracle`]: computed free-cooling
//! hours, seismic design values and grid carbon intensity versus published
//! references. Disagreement beyond a pre-declared tolerance is RED.
#![forbid(unsafe_code)]

mod data_lock;
mod oracle;
mod osha;
mod quantities;
pub use data_lock::{
    load_pins_from_disk, parse_data_section, referenced_data_paths, selftest_flip_one_byte,
    verify_data_lock, DataLockReport, DATA_SECTION, LOCK_REL, SNAPSHOTS_REL,
};
pub use oracle::{
    agrees, check_oracle, check_oracle_with, compiled_references, parse_references,
    perturb_one_tolerance, Comparison, Location, OracleError, OracleReport, PublishedRef, Quantity,
    ReferenceLedger, Tolerance, ANTI_VACUOUS_LOCATIONS, ANTI_VACUOUS_REFS, COMPILED_REFERENCES,
    COMPILED_REFERENCES_ORIGIN, DISAGREEMENT, SNAP_EGRID, SNAP_TMY3, SNAP_USGS,
};
pub use osha::{
    check_osha, check_osha_with, cites_147_as_electrical_loto_authority, IsolationConstraint,
    OshaFault, OshaReport, BACKFEED_TEST, CONTROL_DEVICES_NOT_ISOLATION, DEENERGIZE_FIRST,
    EXCLUSION_147, ISOLATION_CONSTRAINTS, SNAP_147, SNAP_269, SNAP_333,
};
pub use quantities::{
    degree_days, free_cooling_hours, grid_co2_lb_per_mwh, interpolate_seismic, QuantityError,
    Seismic, DEGREE_DAY_BASE_C, FREE_COOLING_THRESHOLD_C, LB_PER_SHORT_TON,
};

use cdcp_evidence::{may_load, parse_meta_toml, resolve_engine_root, LicenceFault};
use sha2::{Digest, Sha256};
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Token interpolated inside the missing-licence refusal path.
/// Deleting the [`may_load`] call makes the matching selftest non-zero.
pub const MISSING_LICENCE_REFUSAL: &str =
    "REFUSES to load an artifact whose .meta.toml lacks a licence line";

/// Token interpolated inside the hash comparison. Deleting the
/// recorded-vs-computed check makes the matching selftest non-zero.
pub const HASH_MISMATCH: &str = "sha256 mismatch";

/// Token interpolated inside [`load_registry`]. An empty pin list is
/// an ERROR, never a vacuous pass.
pub const ANTI_VACUOUS_EMPTY: &str = "zero registered snapshots is an ERROR";

/// Token interpolated inside [`load_registry`]. Registered-but-unloaded
/// is an ERROR, never a quiet empty success.
pub const ANTI_VACUOUS_NONE_LOADED: &str =
    "zero artifacts loaded where >=1 is registered is an ERROR";

/// Token interpolated inside [`verify_data_lock`]. A non-empty pin list
/// whose lock section names nothing must not report like a lock that held.
pub const ANTI_VACUOUS_DATA_LOCK: &str =
    "snapshots.toml is non-empty but content.lock [data] lists zero files";

/// Compiled-in pin file, crate-relative. Tests assert it is non-empty.
pub const COMPILED_PINS: &str = include_str!("../snapshots.toml");

/// Origin label for [`COMPILED_PINS`].
pub const COMPILED_PINS_ORIGIN: &str = "crates/cdcp_data/snapshots.toml";

/// A build-time pin: identity, on-disk paths, recorded sha256.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotPin {
    /// `source_id` the sidecar must name.
    pub id: String,
    /// Engine-root-relative body path.
    pub body: String,
    /// Engine-root-relative `.meta.toml` path.
    pub sidecar: String,
    /// Recorded sha256 (lowercase hex). The body must match.
    pub sha256: String,
}

/// Bytes that passed licence + hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSnapshot {
    /// `source_id`.
    pub id: String,
    /// Absolute body path that was read.
    pub body_path: PathBuf,
    /// Computed sha256 (lowercase hex). Equals the pin.
    pub sha256: String,
    /// Body bytes.
    pub bytes: Vec<u8>,
    /// [`cdcp_evidence::ArtifactMeta::eligible_for_agent_index`].
    pub eligible_for_agent_index: bool,
}

/// Outcome of loading a non-empty pin set with no faults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadReport {
    /// Snapshots that loaded.
    pub loaded: Vec<LoadedSnapshot>,
}

impl LoadReport {
    /// True when at least one snapshot loaded (the constructor refuses
    /// the empty case, so this is always true for a live report).
    #[must_use]
    pub fn is_clean(&self) -> bool {
        !self.loaded.is_empty()
    }
}

impl fmt::Display for LoadReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "load_snapshots: PASS loaded={} faults=0",
            self.loaded.len()
        )?;
        for s in &self.loaded {
            writeln!(
                f,
                "  {} sha256={} eligible={}",
                s.id, s.sha256, s.eligible_for_agent_index
            )?;
        }
        Ok(())
    }
}

/// Why a load could not succeed.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DataError {
    /// [`may_load`] refused the sidecar.
    #[error("refusing to load {id}: {fault}")]
    Refused {
        /// Snapshot id.
        id: String,
        /// D4 fault. Not re-derived here.
        fault: LicenceFault,
    },
    /// Body hash disagrees with the recorded pin and/or sidecar.
    #[error("{HASH_MISMATCH} for {id}: recorded={recorded} computed={computed}")]
    HashMismatch {
        /// Snapshot id.
        id: String,
        /// Hash the pin / sidecar recorded.
        recorded: String,
        /// Hash of the bytes just read.
        computed: String,
    },
    /// Pin list was empty.
    #[error("{ANTI_VACUOUS_EMPTY}")]
    EmptyRegistry,
    /// Every registered pin failed to load.
    #[error("{ANTI_VACUOUS_NONE_LOADED} (registered={registered})")]
    NoneLoaded {
        /// How many pins were registered.
        registered: usize,
        /// Per-pin faults.
        faults: Vec<DataError>,
    },
    /// Some pins loaded, at least one did not. Partial success is RED.
    #[error("registered {registered} snapshot(s), loaded {loaded}, refused {}", faults.len())]
    Partial {
        /// Successful loads.
        loaded: usize,
        /// Pin count.
        registered: usize,
        /// Per-pin faults.
        faults: Vec<DataError>,
    },
    /// Sidecar missing the `sha256` line.
    #[error("sidecar {origin} has no sha256 — recorded hash is required")]
    MissingRecordedHash {
        /// Sidecar origin.
        origin: String,
    },
    /// Pin `id` and sidecar `source_id` disagree.
    #[error("pin id {pin} != sidecar source_id {sidecar}")]
    IdMismatch {
        /// Pin id.
        pin: String,
        /// Sidecar source_id.
        sidecar: String,
    },
    /// Sidecar could not be parsed.
    #[error("unparseable {path}: {detail}")]
    Unparseable {
        /// Path that failed.
        path: String,
        /// Parser detail.
        detail: String,
    },
    /// Filesystem failure.
    #[error("cannot read {path}: {detail}")]
    Io {
        /// Path that failed.
        path: String,
        /// Underlying error.
        detail: String,
    },
    /// `snapshots.toml` named files but `content.lock` `[data]` listed none.
    #[error("{ANTI_VACUOUS_DATA_LOCK}")]
    EmptyDataLock {
        /// How many `[[snapshot]]` rows were registered.
        registered: usize,
    },
    /// A path `snapshots.toml` names is absent from `[data]`.
    #[error("[data] in snapshots.toml but not pinned in content.lock: {path}")]
    DataUnpinned {
        /// Engine-root-relative path.
        path: String,
    },
    /// `[data]` hash disagrees with the bytes on disk.
    #[error("[data] hash mismatch: {path} lock={recorded} live={computed}")]
    DataHashMismatch {
        /// Engine-root-relative path.
        path: String,
        /// Hash the lock recorded.
        recorded: String,
        /// Hash of the bytes just read.
        computed: String,
    },
    /// A `[data]` row's file is missing.
    #[error("[data] missing file: {path}")]
    DataMissing {
        /// Engine-root-relative path.
        path: String,
    },
    /// One or more `[data]` faults. Partial success is RED.
    #[error("content.lock [data] failed ({} fault(s))", faults.len())]
    DataLockFailed {
        /// Per-path faults.
        faults: Vec<DataError>,
    },
    /// Flip-selftest did not reach RED.
    #[error("expected RED on flipped vendored body but verify-data-lock was green")]
    DataLockSelftestMissed,
}

/// SHA-256 of `bytes` as lowercase hex.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Parse a pin registry. Zero `[[snapshot]]` rows is [`DataError::EmptyRegistry`].
pub fn parse_pins(text: &str, origin: &str) -> Result<Vec<SnapshotPin>, DataError> {
    let _ = ANTI_VACUOUS_EMPTY;
    let doc: toml::Value = toml::from_str(text).map_err(|e| DataError::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;
    let rows = match doc.get("snapshot") {
        Some(toml::Value::Array(items)) => items,
        Some(other) => {
            return Err(DataError::Unparseable {
                path: origin.to_string(),
                detail: format!("`snapshot` must be an array, got {other}"),
            });
        }
        None => {
            return Err(DataError::EmptyRegistry);
        }
    };
    if rows.is_empty() {
        return Err(DataError::EmptyRegistry);
    }
    let mut pins = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let id = toml_string(row, "id").ok_or_else(|| DataError::Unparseable {
            path: origin.to_string(),
            detail: format!("snapshot[{i}] missing `id`"),
        })?;
        let body = toml_string(row, "body").ok_or_else(|| DataError::Unparseable {
            path: origin.to_string(),
            detail: format!("snapshot[{i}] ({id}) missing `body`"),
        })?;
        let sidecar = toml_string(row, "sidecar").ok_or_else(|| DataError::Unparseable {
            path: origin.to_string(),
            detail: format!("snapshot[{i}] ({id}) missing `sidecar`"),
        })?;
        let sha256 = toml_string(row, "sha256").ok_or_else(|| DataError::Unparseable {
            path: origin.to_string(),
            detail: format!("snapshot[{i}] ({id}) missing `sha256`"),
        })?;
        pins.push(SnapshotPin {
            id,
            body,
            sidecar,
            sha256: sha256.to_ascii_lowercase(),
        });
    }
    Ok(pins)
}

/// Pins compiled into this crate. Empty is an ERROR.
pub fn compiled_pins() -> Result<Vec<SnapshotPin>, DataError> {
    parse_pins(COMPILED_PINS, COMPILED_PINS_ORIGIN)
}

/// Load the compiled-in pin set from `root`.
pub fn load_compiled(root: &Path) -> Result<LoadReport, DataError> {
    let pins = compiled_pins()?;
    load_registry(root, &pins)
}

/// Load every pin under `root`. Empty pin list is an ERROR. Zero
/// successful loads against a non-empty list is an ERROR. A single
/// refused pin fails the set — partial success is not a pass.
pub fn load_registry(root: &Path, pins: &[SnapshotPin]) -> Result<LoadReport, DataError> {
    let _ = ANTI_VACUOUS_EMPTY;
    let _ = ANTI_VACUOUS_NONE_LOADED;
    if pins.is_empty() {
        return Err(DataError::EmptyRegistry);
    }
    let mut loaded = Vec::new();
    let mut faults = Vec::new();
    for pin in pins {
        match load_one(root, pin) {
            Ok(s) => loaded.push(s),
            Err(e) => faults.push(e),
        }
    }
    if loaded.is_empty() {
        return Err(DataError::NoneLoaded {
            registered: pins.len(),
            faults,
        });
    }
    if !faults.is_empty() {
        return Err(DataError::Partial {
            loaded: loaded.len(),
            registered: pins.len(),
            faults,
        });
    }
    Ok(LoadReport { loaded })
}

/// Load one pin: parse sidecar → [`may_load`] → read body → verify sha256.
pub fn load_one(root: &Path, pin: &SnapshotPin) -> Result<LoadedSnapshot, DataError> {
    let sidecar_path = join_rel(root, &pin.sidecar);
    let origin = pin.sidecar.clone();
    let text = std::fs::read_to_string(&sidecar_path).map_err(|e| DataError::Io {
        path: origin.clone(),
        detail: e.to_string(),
    })?;
    let meta = parse_meta_toml(&text, &origin).map_err(|fault| DataError::Refused {
        id: pin.id.clone(),
        fault,
    })?;
    if meta.id() != pin.id {
        return Err(DataError::IdMismatch {
            pin: pin.id.clone(),
            sidecar: meta.id().to_string(),
        });
    }

    // D4 is the rights check. Do not re-derive the three-field split.
    let _ = MISSING_LICENCE_REFUSAL;
    may_load(&meta).map_err(|fault| DataError::Refused {
        id: pin.id.clone(),
        fault,
    })?;

    let recorded = sidecar_sha256(&text, &origin)?;
    if !hash_eq(&recorded, &pin.sha256) {
        return Err(DataError::HashMismatch {
            id: pin.id.clone(),
            recorded: pin.sha256.clone(),
            computed: recorded,
        });
    }

    let body_path = join_rel(root, &pin.body);
    let bytes = std::fs::read(&body_path).map_err(|e| DataError::Io {
        path: pin.body.clone(),
        detail: e.to_string(),
    })?;
    let computed = sha256_hex(&bytes);
    if !hash_eq(&computed, &pin.sha256) {
        return Err(DataError::HashMismatch {
            id: pin.id.clone(),
            recorded: pin.sha256.clone(),
            computed,
        });
    }

    let eligible_for_agent_index = meta.eligible_for_agent_index();
    Ok(LoadedSnapshot {
        id: pin.id.clone(),
        body_path,
        sha256: computed,
        bytes,
        eligible_for_agent_index,
    })
}

/// Walk up from `start` to the engine root (same anchor D4 uses).
pub fn engine_root(start: &Path) -> Result<PathBuf, DataError> {
    resolve_engine_root(start).map_err(|e| DataError::Io {
        path: start.display().to_string(),
        detail: e.to_string(),
    })
}

fn sidecar_sha256(text: &str, origin: &str) -> Result<String, DataError> {
    let doc: toml::Value = toml::from_str(text).map_err(|e| DataError::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;
    match toml_string(&doc, "sha256") {
        Some(h) => Ok(h.to_ascii_lowercase()),
        None => Err(DataError::MissingRecordedHash {
            origin: origin.to_string(),
        }),
    }
}

fn hash_eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn toml_string(v: &toml::Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

pub(crate) fn join_rel(root: &Path, rel: &str) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        p.push(part);
    }
    p
}

#[cfg(test)]
mod unit {
    use super::*;

    fn production_src() -> &'static str {
        include_str!("lib.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests")
    }

    #[test]
    fn crate_forbids_unsafe() {
        let src = include_str!("lib.rs");
        assert!(src.contains("#![forbid(unsafe_code)]"));
        assert!(!production_src().contains("unsafe "));
        assert!(
            !include_str!("osha.rs").contains("unsafe "),
            "no unsafe token in osha.rs"
        );
        assert!(
            !include_str!("oracle.rs").contains("unsafe "),
            "no unsafe token in oracle.rs"
        );
        assert!(
            !include_str!("quantities.rs").contains("unsafe "),
            "no unsafe token in quantities.rs"
        );
    }

    #[test]
    fn production_calls_may_load_and_does_not_reimplement_split() {
        let src = production_src();
        assert!(
            src.contains("may_load("),
            "E1 must call cdcp_evidence::may_load"
        );
        assert!(
            src.contains("eligible_for_agent_index"),
            "E1 must consult eligible_for_agent_index"
        );
        assert!(
            src.contains("MISSING_LICENCE_REFUSAL"),
            "delete the licence-refusal token interpolation → selftest non-zero"
        );
        assert!(
            !src.contains("fn has_licence_or_rights")
                && !src.contains("fn redistribution_is_permitted")
                && !src.contains("fn evaluate_artifact"),
            "do not reimplement the three-field split"
        );
    }

    #[test]
    fn production_names_both_hashes_on_mismatch() {
        let src = production_src();
        assert!(src.contains("HASH_MISMATCH"));
        assert!(src.contains("recorded") && src.contains("computed"));
        assert!(src.contains("hash_eq"));
    }

    #[test]
    fn production_rejects_empty_and_none_loaded() {
        let src = production_src();
        assert!(src.contains("ANTI_VACUOUS_EMPTY"));
        assert!(src.contains("ANTI_VACUOUS_NONE_LOADED"));
        assert!(src.contains("pins.is_empty()"));
        assert!(src.contains("loaded.is_empty()"));
    }

    #[test]
    fn production_has_no_socket_or_client() {
        let src = production_src();
        let extra = [include_str!("oracle.rs"), include_str!("quantities.rs")];
        for needle in [
            "TcpStream",
            "UdpSocket",
            "TcpListener",
            "std::net",
            "::net::",
            "ToSocketAddrs",
        ] {
            assert!(!src.contains(needle), "production mentions {needle}");
            for (i, extra_src) in extra.iter().enumerate() {
                assert!(!extra_src.contains(needle), "module {i} mentions {needle}");
            }
        }
    }

    #[test]
    fn compiled_pins_are_non_empty() {
        let pins = compiled_pins().expect("compiled pins");
        assert!(!pins.is_empty(), "{ANTI_VACUOUS_EMPTY}");
        assert!(
            pins.iter().any(|p| p.id == "src-nist-sp800-123"),
            "NIST is the live permitted body; compiled pins: {pins:?}"
        );
    }

    #[test]
    fn parse_pins_empty_array_is_error() {
        let err = parse_pins("schema = \"x\"\nsnapshot = []\n", "empty.toml")
            .expect_err("empty snapshot array");
        assert!(matches!(err, DataError::EmptyRegistry), "{err:?}");
        assert!(err.to_string().contains(ANTI_VACUOUS_EMPTY));
    }

    #[test]
    fn parse_pins_missing_table_is_error() {
        let err = parse_pins("schema = \"x\"\n", "none.toml").expect_err("no snapshot table");
        assert!(matches!(err, DataError::EmptyRegistry), "{err:?}");
    }
}
