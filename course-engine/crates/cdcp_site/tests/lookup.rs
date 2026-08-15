//! Live lookups against vendored TMY3 / USGS / eGRID snapshots.

use cdcp_site::{
    compiled_site_locations, flood_pin_id, lookup_coord, lookup_flood, lookup_id, SiteError,
    SiteQuery, SiteStore, ANTI_VACUOUS_LOCATIONS, FLOOD_NOT_VENDORED,
};
use std::path::PathBuf;

fn engine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("engine")
}

#[test]
fn compiled_catalog_is_non_empty() {
    let locs = compiled_site_locations().expect("catalog");
    assert!(
        !locs.is_empty(),
        "{ANTI_VACUOUS_LOCATIONS}: compiled catalog was empty"
    );
    assert!(
        locs.iter().any(|l| l.id == "ashburn"),
        "ashburn must be a compiled location: {:?}",
        locs.iter().map(|l| l.id.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn lookup_id_ashburn_returns_typed_values() {
    let profile = lookup_id(&engine(), "ashburn").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(profile.location.id, "ashburn");
    assert_eq!(profile.climate.free_cooling_hours, 5734.0);
    assert!(profile.seismic.pga.is_finite() && profile.seismic.pga > 0.0);
    assert!(profile.grid_co2_lb_per_mwh.is_finite() && profile.grid_co2_lb_per_mwh > 0.0);
    assert!(profile.climate.bin.hours > 0);
    let hours: u32 = profile.climate.bins.iter().map(|b| b.hours).sum();
    assert_eq!(hours, 8760, "TMY3 is a non-leap 8760-hour record");
    let text = profile.to_string();
    assert!(text.contains("site ashburn"), "{text}");
    assert!(text.contains("pga="), "{text}");
}

#[test]
fn lookup_coord_matches_compiled_sites() {
    let store = SiteStore::load(&engine()).expect("store");
    assert!(!store.locations().is_empty(), "{ANTI_VACUOUS_LOCATIONS}");
    for loc in store.locations() {
        let by_id = store
            .lookup(SiteQuery::Id(&loc.id))
            .unwrap_or_else(|e| panic!("{}: {e}", loc.id));
        let by_coord = lookup_coord(&engine(), loc.lat, loc.lon)
            .unwrap_or_else(|e| panic!("coord {} {}: {e}", loc.lat, loc.lon));
        assert_eq!(by_id.location.id, loc.id);
        assert_eq!(by_coord.location.id, loc.id);
        assert_eq!(
            by_id.climate.free_cooling_hours,
            by_coord.climate.free_cooling_hours
        );
        assert_eq!(by_id.seismic.pga, by_coord.seismic.pga);
        assert_eq!(by_id.grid_co2_lb_per_mwh, by_coord.grid_co2_lb_per_mwh);
    }
}

#[test]
fn flood_is_named_error_not_a_default_zone() {
    let pins = cdcp_data::compiled_pins().expect("pins");
    assert!(
        flood_pin_id(&pins).is_none(),
        "FEMA is not vendored; a flood pin would need a decoder, not a default zone"
    );
    let err = lookup_flood(&engine(), SiteQuery::Id("ashburn")).expect_err("flood");
    assert!(matches!(err, SiteError::FloodNotVendored), "{err:?}");
    assert!(err.to_string().contains(FLOOD_NOT_VENDORED));
}
