//! Known-bad + anti-vacuous plants for cdcp_site.
//!
//! Plants:
//! - flip one vendored snapshot byte → load RED (hash mismatch)
//! - unknown location id → named MissingLocation, never a default
//! - lat/lon outside every compiled cell → named MissingLocation
//! - empty location set → ERROR
//!
//! Meta-tests: delete the named-error / empty-set / load_one paths → RED.

use cdcp_data::{compiled_pins, DataError, HASH_MISMATCH, SNAP_EGRID, SNAP_TMY3, SNAP_USGS};
use cdcp_site::{
    lookup_coord, lookup_id, require_locations, SiteError, SiteStore, ANTI_VACUOUS_LOCATIONS,
    MISSING_LOCATION,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQ: AtomicU64 = AtomicU64::new(1);

fn engine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("engine")
}

fn production_src() -> &'static str {
    include_str!("../src/lib.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests")
}

fn scratch(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cdcp-site-{}-{}-{}-{}",
        tag,
        std::process::id(),
        n,
        nanos
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn copy_rel(src_root: &Path, dst_root: &Path, rel: &str) {
    let src = src_root.join(rel);
    let dst = dst_root.join(rel);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).expect("parent");
    }
    fs::copy(&src, &dst).unwrap_or_else(|e| panic!("copy {rel}: {e}"));
}

#[test]
fn flip_one_snapshot_byte_is_red() {
    let src = engine();
    let dst = scratch("flip");
    let pins = compiled_pins().expect("pins");
    for want in [SNAP_TMY3, SNAP_USGS, SNAP_EGRID] {
        let pin = pins
            .iter()
            .find(|p| p.id == want)
            .unwrap_or_else(|| panic!("missing pin {want}"));
        copy_rel(&src, &dst, &pin.body);
        copy_rel(&src, &dst, &pin.sidecar);
    }
    let tmy3 = pins.iter().find(|p| p.id == SNAP_TMY3).expect("tmy3 pin");
    let body_path = dst.join(&tmy3.body);
    let mut body = fs::read(&body_path).expect("read tmy3");
    assert!(!body.is_empty(), "TMY3 body is empty — nothing to flip");
    body[0] ^= 0xff;
    fs::write(&body_path, &body).expect("write flipped tmy3");

    let err = SiteStore::load(&dst).expect_err("flipped byte must RED");
    match &err {
        SiteError::Data(DataError::HashMismatch {
            id,
            recorded,
            computed,
        }) => {
            assert_eq!(id, SNAP_TMY3);
            assert_ne!(recorded, computed);
        }
        other => panic!("expected Data(HashMismatch), got {other:?}"),
    }
    let text = err.to_string();
    assert!(text.contains(HASH_MISMATCH), "{text}");
    let _ = fs::remove_dir_all(&dst);
}

#[test]
fn unknown_id_is_named_error_never_a_default() {
    let err = lookup_id(&engine(), "atlantis").expect_err("missing");
    match &err {
        SiteError::MissingLocation { id } => assert_eq!(id, "atlantis"),
        other => panic!("expected MissingLocation, got {other:?}"),
    }
    assert!(err.to_string().contains(MISSING_LOCATION));
}

#[test]
fn coord_outside_catalog_is_named_error_never_nearest() {
    let err = lookup_coord(&engine(), 0.0, 0.0).expect_err("equator");
    match &err {
        SiteError::MissingLocation { id } => {
            assert!(id.contains('0'), "{id}");
        }
        other => panic!("expected MissingLocation, got {other:?}"),
    }
    assert!(err.to_string().contains(MISSING_LOCATION));
}

#[test]
fn empty_location_set_is_error() {
    let err = require_locations(&[]).expect_err("empty");
    assert!(matches!(err, SiteError::EmptyLocations), "{err:?}");
    assert!(err.to_string().contains(ANTI_VACUOUS_LOCATIONS));
}

#[test]
fn selftest_delete_named_errors_is_nonzero() {
    let src = production_src();
    assert!(
        src.contains("MISSING_LOCATION"),
        "delete the missing-location token → selftest non-zero"
    );
    assert!(
        src.contains("ANTI_VACUOUS_LOCATIONS"),
        "delete the empty-set token → selftest non-zero"
    );
    assert!(
        src.contains("locations.is_empty()"),
        "delete the empty-set check → selftest non-zero"
    );
    assert!(
        src.contains("load_one("),
        "delete load_one → flipped-byte plant cannot trip"
    );
    assert!(
        src.contains("FLOOD_NOT_VENDORED"),
        "delete the flood-not-vendored token → selftest non-zero"
    );
}
