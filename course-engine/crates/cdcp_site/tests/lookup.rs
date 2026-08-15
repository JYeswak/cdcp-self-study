//! Live lookups against vendored TMY3 / USGS / eGRID / FEMA NFHL / EIA snapshots.

use cdcp_site::{
    compiled_site_locations, flood_pin_id, lookup_coord, lookup_flood, lookup_id,
    lookup_power_price, power_price_pin_id, SiteError, SiteQuery, SiteStore,
    ANTI_VACUOUS_LOCATIONS, NOT_IN_SFHA, SNAP_FLOOD, SNAP_PRICE,
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
    assert_eq!(profile.flood.zone, "X");
    assert!(!profile.flood.in_sfha);
    let hours: u32 = profile.climate.bins.iter().map(|b| b.hours).sum();
    assert_eq!(hours, 8760, "TMY3 is a non-leap 8760-hour record");
    let text = profile.to_string();
    assert!(text.contains("site ashburn"), "{text}");
    assert!(text.contains("pga="), "{text}");
    assert!(text.contains("flood_zone="), "{text}");
    assert!(text.contains(NOT_IN_SFHA), "{text}");
    assert_eq!(profile.power_price.value, 10.53);
    assert_eq!(profile.power_price.unit, "cents/kWh");
    assert_eq!(profile.power_price.sector, "industrial");
    assert_eq!(profile.power_price.state, "VA");
    assert_eq!(profile.power_price.period, "2026-05");
    assert!(text.contains("power_price="), "{text}");
    assert!(text.contains("cents/kWh"), "{text}");
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
        assert_eq!(by_id.flood, by_coord.flood);
        assert!(!by_id.flood.zone.is_empty(), "{} empty flood zone", loc.id);
        assert_eq!(by_id.power_price, by_coord.power_price);
        assert!(
            !by_id.power_price.unit.is_empty(),
            "{} bare power price",
            loc.id
        );
        assert!(
            by_id.power_price.value.is_finite() && by_id.power_price.value > 0.0,
            "{} power price {}",
            loc.id,
            by_id.power_price
        );
    }
}

#[test]
fn flood_lookup_returns_zone_not_not_vendored() {
    let pins = cdcp_data::compiled_pins().expect("pins");
    assert_eq!(
        flood_pin_id(&pins),
        Some(SNAP_FLOOD),
        "FEMA NFHL pin must be present so lookup_flood can decode"
    );
    let flood = lookup_flood(&engine(), SiteQuery::Id("ashburn")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(flood.zone, "X");
    assert!(!flood.in_sfha);
    assert_eq!(flood.subtype, "AREA OF MINIMAL FLOOD HAZARD");
    assert!(flood.to_string().contains(NOT_IN_SFHA), "{flood}");
}

#[test]
fn flood_missing_location_is_named_error() {
    let err = lookup_flood(&engine(), SiteQuery::Id("atlantis")).expect_err("missing");
    match err {
        SiteError::MissingLocation { ref id } => assert_eq!(id, "atlantis"),
        other => panic!("expected MissingLocation, got {other:?}"),
    }
}

#[test]
fn power_price_lookup_returns_typed_price_not_not_vendored() {
    let pins = cdcp_data::compiled_pins().expect("pins");
    assert_eq!(
        power_price_pin_id(&pins),
        Some(SNAP_PRICE),
        "EIA industrial-price pin must be present so lookup_power_price can decode"
    );
    let price =
        lookup_power_price(&engine(), SiteQuery::Id("ashburn")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(price.value, 10.53);
    assert_eq!(price.unit, "cents/kWh");
    assert_eq!(price.sector, "industrial");
    assert_eq!(price.state, "VA");
    assert!(price.to_string().contains("cents/kWh"), "{price}");
}

#[test]
fn power_price_missing_location_is_named_error() {
    let err = lookup_power_price(&engine(), SiteQuery::Id("atlantis")).expect_err("missing");
    match err {
        SiteError::MissingLocation { ref id } => assert_eq!(id, "atlantis"),
        other => panic!("expected MissingLocation, got {other:?}"),
    }
}
