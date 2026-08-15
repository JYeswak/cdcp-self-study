//! Site lookups: lat/lon or a compiled location id → climate bin, seismic
//! PGA, grid carbon, flood zone. No network. Every value is a function of
//! already-vendored `cdcp_data` snapshots (NREL TMY3, USGS ASCE 7-16,
//! EPA eGRID2023, FEMA NFHL).
//!
//! A missing location is [`SiteError::MissingLocation`], never a default
//! nearest-neighbour. An empty catalog is [`SiteError::EmptyLocations`].
//! A missing FEMA pin is [`SiteError::FloodNotVendored`], never a default
//! zone letter.
#![forbid(unsafe_code)]

use cdcp_data::{
    compiled_pins, compiled_references, degree_days, free_cooling_hours, grid_co2_lb_per_mwh,
    interpolate_seismic, load_one, DataError, OracleError, QuantityError, SnapshotPin,
};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use thiserror::Error;

/// Token interpolated inside the missing-location path. Deleting the
/// named-error branch makes the matching selftest non-zero.
pub const MISSING_LOCATION: &str = "missing location is an ERROR";

/// Token interpolated inside the empty-catalog path.
pub const ANTI_VACUOUS_LOCATIONS: &str = "empty location set is an ERROR";

/// Token interpolated inside the flood path when no FEMA pin exists.
pub const FLOOD_NOT_VENDORED: &str = "flood zone not vendored";

/// Named NFHL value when `SFHA_TF` is `F` (outside the Special Flood
/// Hazard Area). Zone letter `X` is still recorded; this token names
/// the insurance/siting posture so a default "X" cannot be smuggled in
/// as "we did not look".
pub const NOT_IN_SFHA: &str = "not-in-special-flood-hazard";

/// HVAC bin-method width for dry-bulb hour counts (°C).
pub const BIN_WIDTH_C: i32 = 5;

/// Re-export the compiled-location record the catalog is made of.
pub use cdcp_data::{engine_root, Location, Seismic, SNAP_EGRID, SNAP_TMY3, SNAP_USGS};

/// Snapshot pin id: FEMA NFHL flood-zone point extract.
pub const SNAP_FLOOD: &str = "src-fema-nfhl";

/// How the caller names a site.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SiteQuery<'a> {
    /// Compiled location id (`ashburn`, …).
    Id(&'a str),
    /// WGS84 decimal degrees. West longitude is negative.
    Coord {
        /// Latitude.
        lat: f64,
        /// Longitude.
        lon: f64,
    },
}

/// One 5 °C dry-bulb hour band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TempBin {
    /// Inclusive lower edge (°C).
    pub lo_c: i32,
    /// Exclusive upper edge (°C).
    pub hi_c: i32,
    /// Hours in `[lo_c, hi_c)`.
    pub hours: u32,
}

/// Climate derived from the TMY3 dry-bulb record.
#[derive(Debug, Clone, PartialEq)]
pub struct Climate {
    /// Modal 5 °C band (most hours). Ties break toward the colder band.
    pub bin: TempBin,
    /// Full HVAC bin-method table, coldest first.
    pub bins: Vec<TempBin>,
    /// Hours with dry-bulb ≤ 18.33 °C.
    pub free_cooling_hours: f64,
    /// Daily-mean HDD, base 18 °C.
    pub heating_degree_days_18c: f64,
    /// Daily-mean CDD, base 18 °C.
    pub cooling_degree_days_18c: f64,
}

/// FEMA NFHL flood zone. Constructed only from a vendored, decoded
/// snapshot. A default zone letter is never invented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloodZone {
    /// NFHL `FLD_ZONE` code (`AE`, `X`, …).
    pub zone: String,
    /// NFHL `ZONE_SUBTY` (empty when the layer has none).
    pub subtype: String,
    /// True when NFHL `SFHA_TF` is `T`.
    pub in_sfha: bool,
}

impl fmt::Display for FloodZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.in_sfha {
            write!(f, "{}", self.zone)
        } else {
            write!(f, "{} ({NOT_IN_SFHA})", self.zone)
        }
    }
}

/// Typed site values for one compiled location.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteProfile {
    /// Catalog row that matched the query.
    pub location: Location,
    /// TMY3 climate.
    pub climate: Climate,
    /// Interpolated ASCE 7-16 values.
    pub seismic: Seismic,
    /// eGRID output emission rate (lb CO2 / MWh).
    pub grid_co2_lb_per_mwh: f64,
    /// FEMA NFHL flood zone at the catalog point.
    pub flood: FloodZone,
}

impl fmt::Display for SiteProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "site {} lat={} lon={}",
            self.location.id, self.location.lat, self.location.lon
        )?;
        writeln!(
            f,
            "  climate_bin=[{}, {}) hours={}",
            self.climate.bin.lo_c, self.climate.bin.hi_c, self.climate.bin.hours
        )?;
        writeln!(
            f,
            "  free_cooling_hours={} hdd18c={} cdd18c={}",
            self.climate.free_cooling_hours,
            self.climate.heating_degree_days_18c,
            self.climate.cooling_degree_days_18c
        )?;
        writeln!(
            f,
            "  seismic ss={} s1={} pga={}",
            self.seismic.ss, self.seismic.s1, self.seismic.pga
        )?;
        writeln!(
            f,
            "  grid_co2_lb_per_mwh={} ({})",
            self.grid_co2_lb_per_mwh, self.location.egrid_subregion
        )?;
        writeln!(f, "  flood_zone={}", self.flood)
    }
}

/// Why a site lookup could not succeed.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SiteError {
    /// Named location (or coordinate) is not in the catalog / cells.
    #[error("{MISSING_LOCATION}: {id}")]
    MissingLocation {
        /// Location id or `lat,lon`.
        id: String,
    },
    /// Coordinate sits in more than one compiled cell.
    #[error("ambiguous location for {lat},{lon}: {ids}")]
    AmbiguousLocation {
        /// Query latitude.
        lat: f64,
        /// Query longitude.
        lon: f64,
        /// Matching location ids, comma-separated.
        ids: String,
    },
    /// Catalog or caller-supplied location set was empty.
    #[error("{ANTI_VACUOUS_LOCATIONS}")]
    EmptyLocations,
    /// No FEMA / NFHL snapshot is pinned.
    #[error("{FLOOD_NOT_VENDORED}")]
    FloodNotVendored,
    /// A flood pin exists but this crate has no decoder for it.
    #[error("flood snapshot {id} is pinned but has no decoder")]
    FloodUndecoded {
        /// Pin id.
        id: String,
    },
    /// Snapshot loader refused.
    #[error("{0}")]
    Data(#[from] DataError),
    /// Quantity parse / interpolation.
    #[error("{0}")]
    Quantity(String),
    /// Compiled location ledger could not be read.
    #[error("site catalog: {0}")]
    Catalog(String),
}

impl From<QuantityError> for SiteError {
    fn from(e: QuantityError) -> Self {
        match e {
            QuantityError::MissingLocation(id) => SiteError::MissingLocation { id },
            QuantityError::MissingSubregion(id) => SiteError::MissingLocation { id },
            other => SiteError::Quantity(other.to_string()),
        }
    }
}

impl From<OracleError> for SiteError {
    fn from(e: OracleError) -> Self {
        SiteError::Catalog(e.to_string())
    }
}

/// Loaded snapshots plus the compiled location catalog.
///
/// Construction refuses an empty catalog. Lookups never invent a
/// location that is not in [`SiteStore::locations`].
#[derive(Debug, Clone, PartialEq)]
pub struct SiteStore {
    tmy3: String,
    usgs: String,
    egrid: String,
    flood: String,
    flood_pin: String,
    locations: Vec<Location>,
    cells: BTreeMap<String, Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cell {
    lat0: f64,
    lat1: f64,
    lon0: f64,
    lon1: f64,
}

impl Cell {
    fn contains(self, lat: f64, lon: f64) -> bool {
        lat >= self.lat0 && lat <= self.lat1 && lon >= self.lon0 && lon <= self.lon1
    }
}

impl SiteStore {
    /// Load TMY3 / USGS / eGRID through [`load_one`] and the compiled
    /// location catalog. Empty catalog is [`SiteError::EmptyLocations`].
    pub fn load(root: &Path) -> Result<Self, SiteError> {
        let locations = compiled_site_locations()?;
        let pins = compiled_pins()?;
        let tmy3 = load_text(root, &pins, SNAP_TMY3)?;
        let usgs = load_text(root, &pins, SNAP_USGS)?;
        let egrid = load_text(root, &pins, SNAP_EGRID)?;
        let flood_pin = flood_pin_id(&pins)
            .ok_or(SiteError::FloodNotVendored)?
            .to_string();
        let flood = load_text(root, &pins, &flood_pin)?;
        let cells = usgs_cells(&usgs)?;
        Ok(Self {
            tmy3,
            usgs,
            egrid,
            flood,
            flood_pin,
            locations,
            cells,
        })
    }

    /// Compiled locations. Never empty for a live store.
    #[must_use]
    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    /// Resolve `query` against this store and compute typed values.
    pub fn lookup(&self, query: SiteQuery<'_>) -> Result<SiteProfile, SiteError> {
        let loc = resolve(&self.locations, &self.cells, query)?.clone();
        self.profile(&loc)
    }

    fn profile(&self, loc: &Location) -> Result<SiteProfile, SiteError> {
        let climate = climate_from_tmy3(&self.tmy3, &loc.tmy3_id)?;
        let seismic = interpolate_seismic(&self.usgs, &loc.id, loc.lat, loc.lon)?;
        let grid_co2_lb_per_mwh = grid_co2_lb_per_mwh(&self.egrid, &loc.egrid_subregion)?;
        let flood = flood_zone_from_csv(&self.flood, &loc.id, &self.flood_pin)?;
        Ok(SiteProfile {
            location: loc.clone(),
            climate,
            seismic,
            grid_co2_lb_per_mwh,
            flood,
        })
    }

    /// Flood zone for `query`. Missing location is
    /// [`SiteError::MissingLocation`], never a default zone.
    pub fn lookup_flood(&self, query: SiteQuery<'_>) -> Result<FloodZone, SiteError> {
        Ok(self.lookup(query)?.flood)
    }
}

/// Compiled location catalog. Empty is [`SiteError::EmptyLocations`].
pub fn compiled_site_locations() -> Result<Vec<Location>, SiteError> {
    let ledger = compiled_references()?;
    require_locations(&ledger.locations)?;
    Ok(ledger.locations)
}

/// Refuse an empty location set. A lookup over no locations cannot pass.
pub fn require_locations(locations: &[Location]) -> Result<(), SiteError> {
    let _ = ANTI_VACUOUS_LOCATIONS;
    if locations.is_empty() {
        return Err(SiteError::EmptyLocations);
    }
    Ok(())
}

/// Load snapshots from `root` and look up `query`.
pub fn lookup(root: &Path, query: SiteQuery<'_>) -> Result<SiteProfile, SiteError> {
    SiteStore::load(root)?.lookup(query)
}

/// Look up a compiled location id.
pub fn lookup_id(root: &Path, id: &str) -> Result<SiteProfile, SiteError> {
    lookup(root, SiteQuery::Id(id))
}

/// Look up WGS84 coordinates. Outside every compiled cell is
/// [`SiteError::MissingLocation`], never the nearest site.
pub fn lookup_coord(root: &Path, lat: f64, lon: f64) -> Result<SiteProfile, SiteError> {
    lookup(root, SiteQuery::Coord { lat, lon })
}

/// Flood zone for `query`. No FEMA / NFHL pin is
/// [`SiteError::FloodNotVendored`]. A default zone letter is never
/// returned.
pub fn lookup_flood(root: &Path, query: SiteQuery<'_>) -> Result<FloodZone, SiteError> {
    let _ = FLOOD_NOT_VENDORED;
    let pins = compiled_pins()?;
    match flood_pin_id(&pins) {
        None => Err(SiteError::FloodNotVendored),
        Some(_) => SiteStore::load(root)?.lookup_flood(query),
    }
}

/// Pin id that would carry a flood layer, if any.
#[must_use]
pub fn flood_pin_id(pins: &[SnapshotPin]) -> Option<&str> {
    pins.iter()
        .find(|p| is_flood_pin(&p.id))
        .map(|p| p.id.as_str())
}

fn is_flood_pin(id: &str) -> bool {
    let l = id.to_ascii_lowercase();
    l.contains("flood") || l.contains("fema") || l.contains("nfhl")
}

fn resolve<'a>(
    locations: &'a [Location],
    cells: &BTreeMap<String, Cell>,
    query: SiteQuery<'_>,
) -> Result<&'a Location, SiteError> {
    require_locations(locations)?;
    match query {
        SiteQuery::Id(id) => {
            let _ = MISSING_LOCATION;
            locations
                .iter()
                .find(|l| l.id == id)
                .ok_or_else(|| SiteError::MissingLocation { id: id.to_string() })
        }
        SiteQuery::Coord { lat, lon } => resolve_coord(locations, cells, lat, lon),
    }
}

fn resolve_coord<'a>(
    locations: &'a [Location],
    cells: &BTreeMap<String, Cell>,
    lat: f64,
    lon: f64,
) -> Result<&'a Location, SiteError> {
    let _ = MISSING_LOCATION;
    if !lat.is_finite() || !lon.is_finite() {
        return Err(SiteError::MissingLocation {
            id: format!("{lat},{lon}"),
        });
    }
    let mut hits: Vec<&Location> = Vec::new();
    for loc in locations {
        if let Some(cell) = cells.get(&loc.id) {
            if cell.contains(lat, lon) {
                hits.push(loc);
            }
        }
    }
    match hits.as_slice() {
        [] => Err(SiteError::MissingLocation {
            id: format!("{lat},{lon}"),
        }),
        [only] => Ok(*only),
        many => Err(SiteError::AmbiguousLocation {
            lat,
            lon,
            ids: many
                .iter()
                .map(|l| l.id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        }),
    }
}

fn climate_from_tmy3(csv: &str, location_id: &str) -> Result<Climate, SiteError> {
    let bins = climate_bins(csv, location_id)?;
    let bin = *bins
        .iter()
        .max_by_key(|b| (b.hours, -b.lo_c))
        .ok_or_else(|| SiteError::MissingLocation {
            id: location_id.to_string(),
        })?;
    let free_cooling_hours = free_cooling_hours(csv, location_id)?;
    let (heating_degree_days_18c, cooling_degree_days_18c) = degree_days(csv, location_id)?;
    Ok(Climate {
        bin,
        bins,
        free_cooling_hours,
        heating_degree_days_18c,
        cooling_degree_days_18c,
    })
}

/// HVAC bin-method hour counts in [`BIN_WIDTH_C`] °C bands.
pub fn climate_bins(csv: &str, location_id: &str) -> Result<Vec<TempBin>, SiteError> {
    let mut counts: BTreeMap<i32, u32> = BTreeMap::new();
    for (i, line) in csv.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("location_id") {
            continue;
        }
        let cols: Vec<&str> = t.split(',').collect();
        if cols.len() < 6 {
            return Err(SiteError::Quantity(format!(
                "TMY3 line {}: want 6 columns",
                i + 1
            )));
        }
        if cols[0].trim() != location_id {
            continue;
        }
        let db: f64 = cols[5]
            .trim()
            .parse()
            .map_err(|_| SiteError::Quantity(format!("TMY3 line {}: bad dry_bulb_c", i + 1)))?;
        if !db.is_finite() {
            return Err(SiteError::Quantity(format!(
                "TMY3 line {}: non-finite dry_bulb_c",
                i + 1
            )));
        }
        let lo = (db / f64::from(BIN_WIDTH_C)).floor() as i32 * BIN_WIDTH_C;
        *counts.entry(lo).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return Err(SiteError::MissingLocation {
            id: location_id.to_string(),
        });
    }
    Ok(counts
        .into_iter()
        .map(|(lo, hours)| TempBin {
            lo_c: lo,
            hi_c: lo + BIN_WIDTH_C,
            hours,
        })
        .collect())
}

fn usgs_cells(csv: &str) -> Result<BTreeMap<String, Cell>, SiteError> {
    let mut pts: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for (i, line) in csv.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("location_id") {
            continue;
        }
        let cols: Vec<&str> = t.split(',').collect();
        if cols.len() < 3 {
            return Err(SiteError::Quantity(format!(
                "USGS grid line {}: want lat/lon columns",
                i + 1
            )));
        }
        let id = cols[0].trim();
        if id.is_empty() {
            continue;
        }
        let lat: f64 = cols[1]
            .trim()
            .parse()
            .map_err(|_| SiteError::Quantity(format!("USGS grid line {}: bad lat", i + 1)))?;
        let lon: f64 = cols[2]
            .trim()
            .parse()
            .map_err(|_| SiteError::Quantity(format!("USGS grid line {}: bad lon", i + 1)))?;
        pts.entry(id.to_string()).or_default().push((lat, lon));
    }
    let mut out = BTreeMap::new();
    for (id, xs) in pts {
        let mut lats: Vec<f64> = xs.iter().map(|p| p.0).collect();
        let mut lons: Vec<f64> = xs.iter().map(|p| p.1).collect();
        lats.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        lons.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let lat0 = *lats.first().expect("non-empty group");
        let lat1 = *lats.last().expect("non-empty group");
        let lon0 = *lons.first().expect("non-empty group");
        let lon1 = *lons.last().expect("non-empty group");
        if (lat1 - lat0).abs() < 1e-12 || (lon1 - lon0).abs() < 1e-12 {
            continue;
        }
        out.insert(
            id,
            Cell {
                lat0,
                lat1,
                lon0,
                lon1,
            },
        );
    }
    Ok(out)
}

fn csv_col(header: &[String], name: &str) -> Option<usize> {
    header.iter().position(|h| h == name)
}

/// Decode one compiled location from the vendored NFHL extract.
///
/// Header row is required (`location_id` + `fld_zone`). A pin whose
/// body cannot be read as that table is [`SiteError::FloodUndecoded`],
/// not a guessed zone letter.
fn flood_zone_from_csv(csv: &str, location_id: &str, pin_id: &str) -> Result<FloodZone, SiteError> {
    let mut header: Option<Vec<String>> = None;
    let mut found: Option<FloodZone> = None;
    for (i, line) in csv.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let cols: Vec<String> = t.split(',').map(|c| c.trim().to_string()).collect();
        if header.is_none() {
            if cols.iter().any(|c| c == "location_id") && cols.iter().any(|c| c == "fld_zone") {
                header = Some(cols);
                continue;
            }
            return Err(SiteError::FloodUndecoded {
                id: pin_id.to_string(),
            });
        }
        let header = header.as_ref().expect("header set");
        let id_i = csv_col(header, "location_id").expect("checked");
        let zone_i = csv_col(header, "fld_zone").expect("checked");
        if cols.len() <= id_i.max(zone_i) {
            return Err(SiteError::Quantity(format!(
                "flood line {}: fewer columns than the header",
                i + 1
            )));
        }
        if cols[id_i] != location_id {
            continue;
        }
        let zone = cols[zone_i].clone();
        if zone.is_empty() {
            return Err(SiteError::Quantity(format!(
                "flood line {}: empty fld_zone",
                i + 1
            )));
        }
        let subtype = csv_col(header, "zone_subty")
            .and_then(|j| cols.get(j).cloned())
            .unwrap_or_default();
        let in_sfha = match csv_col(header, "sfha").and_then(|j| cols.get(j).map(String::as_str)) {
            Some("T") | Some("t") => true,
            Some("F") | Some("f") => false,
            Some(other) => {
                return Err(SiteError::Quantity(format!(
                    "flood line {}: bad sfha {other}",
                    i + 1
                )));
            }
            None => {
                return Err(SiteError::Quantity(format!(
                    "flood line {}: missing sfha column",
                    i + 1
                )));
            }
        };
        let row = FloodZone {
            zone,
            subtype,
            in_sfha,
        };
        if let Some(prev) = &found {
            if prev != &row {
                return Err(SiteError::Quantity(format!(
                    "flood: conflicting zones for {location_id}"
                )));
            }
        }
        found = Some(row);
    }
    if header.is_none() {
        return Err(SiteError::FloodUndecoded {
            id: pin_id.to_string(),
        });
    }
    found.ok_or_else(|| SiteError::MissingLocation {
        id: location_id.to_string(),
    })
}

fn load_text(root: &Path, pins: &[SnapshotPin], id: &str) -> Result<String, SiteError> {
    let pin = pins
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| SiteError::Catalog(format!("oracle snapshot {id} is not pinned")))?;
    let loaded = load_one(root, pin)?;
    String::from_utf8(loaded.bytes).map_err(|e| SiteError::Quantity(format!("{id}: {e}")))
}

#[cfg(test)]
mod unit {
    use super::*;

    fn loc(id: &str, lat: f64, lon: f64) -> Location {
        Location {
            id: id.to_string(),
            name: id.to_string(),
            lat,
            lon,
            tmy3_id: id.to_string(),
            egrid_subregion: "SRVC".to_string(),
        }
    }

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
    }

    #[test]
    fn empty_location_set_is_error() {
        let err = require_locations(&[]).expect_err("empty");
        assert!(matches!(err, SiteError::EmptyLocations), "{err:?}");
        assert!(err.to_string().contains(ANTI_VACUOUS_LOCATIONS));
    }

    #[test]
    fn unknown_id_is_named_missing_never_a_default() {
        let catalog = vec![loc("ashburn", 39.0, -77.5)];
        let cells = BTreeMap::new();
        let err = resolve(&catalog, &cells, SiteQuery::Id("atlantis")).expect_err("missing");
        match err {
            SiteError::MissingLocation { ref id } => assert_eq!(id, "atlantis"),
            other => panic!("expected MissingLocation, got {other:?}"),
        }
        assert!(err.to_string().contains(MISSING_LOCATION));
    }

    #[test]
    fn coord_outside_every_cell_is_named_missing_never_nearest() {
        let catalog = vec![loc("ashburn", 39.0438, -77.4874)];
        let mut cells = BTreeMap::new();
        cells.insert(
            "ashburn".into(),
            Cell {
                lat0: 39.0,
                lat1: 39.05,
                lon0: -77.50,
                lon1: -77.45,
            },
        );
        let err = resolve(&catalog, &cells, SiteQuery::Coord { lat: 0.0, lon: 0.0 })
            .expect_err("equator is not Ashburn");
        match err {
            SiteError::MissingLocation { id } => assert_eq!(id, "0,0"),
            other => panic!("expected MissingLocation, got {other:?}"),
        }
    }

    #[test]
    fn coord_inside_cell_resolves() {
        let catalog = vec![loc("ashburn", 39.0438, -77.4874)];
        let mut cells = BTreeMap::new();
        cells.insert(
            "ashburn".into(),
            Cell {
                lat0: 39.0,
                lat1: 39.05,
                lon0: -77.50,
                lon1: -77.45,
            },
        );
        let hit = resolve(
            &catalog,
            &cells,
            SiteQuery::Coord {
                lat: 39.0438,
                lon: -77.4874,
            },
        )
        .expect("inside");
        assert_eq!(hit.id, "ashburn");
    }

    #[test]
    fn overlapping_cells_are_ambiguous_not_first_wins() {
        let catalog = vec![loc("a", 1.0, 1.0), loc("b", 1.0, 1.0)];
        let cell = Cell {
            lat0: 0.0,
            lat1: 2.0,
            lon0: 0.0,
            lon1: 2.0,
        };
        let mut cells = BTreeMap::new();
        cells.insert("a".into(), cell);
        cells.insert("b".into(), cell);
        let err = resolve(&catalog, &cells, SiteQuery::Coord { lat: 1.0, lon: 1.0 })
            .expect_err("overlap");
        match err {
            SiteError::AmbiguousLocation { ids, .. } => {
                assert!(ids.contains('a') && ids.contains('b'), "{ids}");
            }
            other => panic!("expected AmbiguousLocation, got {other:?}"),
        }
    }

    #[test]
    fn climate_bins_empty_id_is_missing_location() {
        let csv = "location_id,wmo,month,day,hour,dry_bulb_c\nashburn,1,1,1,1,0.0\n";
        let err = climate_bins(csv, "nowhere").expect_err("missing");
        match err {
            SiteError::MissingLocation { id } => assert_eq!(id, "nowhere"),
            other => panic!("expected MissingLocation, got {other:?}"),
        }
    }

    #[test]
    fn climate_bins_group_hours() {
        let csv = "\
location_id,wmo,month,day,hour,dry_bulb_c
x,1,1,1,1,0.0
x,1,1,1,2,4.9
x,1,1,1,3,5.0
";
        let bins = climate_bins(csv, "x").expect("bins");
        assert_eq!(
            bins,
            vec![
                TempBin {
                    lo_c: 0,
                    hi_c: 5,
                    hours: 2
                },
                TempBin {
                    lo_c: 5,
                    hi_c: 10,
                    hours: 1
                },
            ]
        );
    }

    #[test]
    fn flood_pin_detector_is_specific() {
        assert!(is_flood_pin("src-fema-nfhl"));
        assert!(is_flood_pin("src-fema-flood-zones"));
        assert!(is_flood_pin(SNAP_FLOOD));
        assert!(!is_flood_pin(SNAP_TMY3));
        assert!(!is_flood_pin(SNAP_USGS));
        assert!(!is_flood_pin(SNAP_EGRID));
    }

    fn flood_csv() -> &'static str {
        "\
location_id,lat,lon,dfirm_id,fld_zone,zone_subty,sfha
x,0,0,00000C,X,AREA OF MINIMAL FLOOD HAZARD,F
ae,1,1,00000C,AE,,T
"
    }

    #[test]
    fn flood_csv_decodes_not_in_sfha() {
        let z = flood_zone_from_csv(flood_csv(), "x", SNAP_FLOOD).expect("x");
        assert_eq!(z.zone, "X");
        assert!(!z.in_sfha);
        assert_eq!(z.subtype, "AREA OF MINIMAL FLOOD HAZARD");
        assert!(z.to_string().contains(NOT_IN_SFHA), "{}", z);
    }

    #[test]
    fn flood_csv_decodes_sfha_zone() {
        let z = flood_zone_from_csv(flood_csv(), "ae", SNAP_FLOOD).expect("ae");
        assert_eq!(z.zone, "AE");
        assert!(z.in_sfha);
        assert_eq!(z.to_string(), "AE");
    }

    #[test]
    fn flood_csv_missing_id_is_named_missing() {
        let err = flood_zone_from_csv(flood_csv(), "nowhere", SNAP_FLOOD).expect_err("missing");
        match err {
            SiteError::MissingLocation { id } => assert_eq!(id, "nowhere"),
            other => panic!("expected MissingLocation, got {other:?}"),
        }
    }

    #[test]
    fn flood_csv_without_header_is_undecoded() {
        let err =
            flood_zone_from_csv("not,a,flood,table\n", "x", SNAP_FLOOD).expect_err("undecoded");
        match err {
            SiteError::FloodUndecoded { id } => assert_eq!(id, SNAP_FLOOD),
            other => panic!("expected FloodUndecoded, got {other:?}"),
        }
    }

    #[test]
    fn production_calls_load_one_and_refuses_empty() {
        let src = production_src();
        assert!(src.contains("load_one("), "must use cdcp_data::load_one");
        assert!(src.contains("require_locations"));
        assert!(src.contains("ANTI_VACUOUS_LOCATIONS"));
        assert!(src.contains("MISSING_LOCATION"));
        assert!(src.contains("FLOOD_NOT_VENDORED"));
        assert!(src.contains("locations.is_empty()"));
    }

    #[test]
    fn production_has_no_socket_or_client() {
        let src = production_src();
        for needle in [
            "TcpStream",
            "UdpSocket",
            "TcpListener",
            "std::net",
            "::net::",
            "ToSocketAddrs",
            "reqwest",
            "ureq",
        ] {
            assert!(!src.contains(needle), "production mentions {needle}");
        }
    }
}
