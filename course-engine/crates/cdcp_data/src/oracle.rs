//! F3 differential harness: computed site quantities vs published references.
//!
//! Loads vendored PD-GOV snapshots through [`crate::load_one`] (which calls
//! `may_load`). No network. Disagreement beyond a pre-declared tolerance is
//! RED and names location, computed, reference, and delta.

use crate::quantities::{
    degree_days, free_cooling_hours, grid_co2_lb_per_mwh, interpolate_seismic, QuantityError,
};
use crate::{load_one, DataError, LoadedSnapshot, SnapshotPin};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use thiserror::Error;

/// Token interpolated inside the disagreement path. Deleting the
/// comparison makes the matching selftest non-zero.
pub const DISAGREEMENT: &str = "ORACLE RED";

/// Token interpolated inside the empty-reference path.
pub const ANTI_VACUOUS_REFS: &str = "zero reference locations compared is an ERROR";

/// Token interpolated inside the too-few-locations path.
pub const ANTI_VACUOUS_LOCATIONS: &str = "fewer than 5 reference locations is an ERROR";

/// Compiled-in published-reference ledger. Tests assert it is non-empty.
pub const COMPILED_REFERENCES: &str = include_str!("../references.toml");

/// Origin label for [`COMPILED_REFERENCES`].
pub const COMPILED_REFERENCES_ORIGIN: &str = "crates/cdcp_data/references.toml";

/// Snapshot pin id: NREL TMY3 dry-bulb extract.
pub const SNAP_TMY3: &str = "src-nrel-tmy3-drybulb";
/// Snapshot pin id: USGS ASCE 7-16 local grid.
pub const SNAP_USGS: &str = "src-usgs-asce7-16-grid";
/// Snapshot pin id: EPA eGRID2023 plant extract.
pub const SNAP_EGRID: &str = "src-epa-egrid2023-plants";

/// A quantity the harness knows how to compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quantity {
    /// Hours/year outdoor dry-bulb ≤ 18.33 °C.
    FreeCoolingHours,
    /// Daily-mean HDD, base 18 °C.
    HeatingDegreeDays18c,
    /// Daily-mean CDD, base 18 °C.
    CoolingDegreeDays18c,
    /// Mapped Ss (g).
    SeismicSs,
    /// Mapped S1 (g).
    SeismicS1,
    /// Mapped PGA (g).
    SeismicPga,
    /// eGRID output emission rate (lb CO2 / MWh).
    GridCo2LbPerMwh,
}

impl Quantity {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "free_cooling_hours" => Some(Self::FreeCoolingHours),
            "heating_degree_days_18c" => Some(Self::HeatingDegreeDays18c),
            "cooling_degree_days_18c" => Some(Self::CoolingDegreeDays18c),
            "seismic_ss" => Some(Self::SeismicSs),
            "seismic_s1" => Some(Self::SeismicS1),
            "seismic_pga" => Some(Self::SeismicPga),
            "grid_co2_lb_per_mwh" => Some(Self::GridCo2LbPerMwh),
            _ => None,
        }
    }

    /// Stable name used in RED findings and the reference ledger.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FreeCoolingHours => "free_cooling_hours",
            Self::HeatingDegreeDays18c => "heating_degree_days_18c",
            Self::CoolingDegreeDays18c => "cooling_degree_days_18c",
            Self::SeismicSs => "seismic_ss",
            Self::SeismicS1 => "seismic_s1",
            Self::SeismicPga => "seismic_pga",
            Self::GridCo2LbPerMwh => "grid_co2_lb_per_mwh",
        }
    }
}

/// One site the ledger names.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    /// Stable id (`ashburn`, …).
    pub id: String,
    /// Human name.
    pub name: String,
    /// Decimal degrees, WGS84.
    pub lat: f64,
    /// Decimal degrees, WGS84, negative west.
    pub lon: f64,
    /// TMY3 extract `location_id`.
    pub tmy3_id: String,
    /// EPA eGRID subregion acronym.
    pub egrid_subregion: String,
}

/// One published number from a source we do not control.
#[derive(Debug, Clone, PartialEq)]
pub struct PublishedRef {
    /// Location id.
    pub location: String,
    /// Quantity.
    pub quantity: Quantity,
    /// Published value.
    pub value: f64,
    /// Unit label (display).
    pub unit: String,
    /// Source citation.
    pub source: String,
    /// Retrieval URL.
    pub url: String,
    /// `YYYY-MM-DD`.
    pub retrieved: String,
}

/// Pre-declared band for one quantity.
#[derive(Debug, Clone, PartialEq)]
pub struct Tolerance {
    /// Absolute band. Comparison is `|Δ| < abs`.
    pub abs: f64,
    /// Unit label.
    pub unit: String,
    /// Why this number, declared before seeing results.
    pub justification: String,
}

/// The compiled ledger: locations, references, tolerances.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceLedger {
    /// Sites.
    pub locations: Vec<Location>,
    /// Published rows.
    pub references: Vec<PublishedRef>,
    /// Per-quantity band.
    pub tolerances: BTreeMap<Quantity, Tolerance>,
}

impl ReferenceLedger {
    /// Distinct location ids that have at least one reference.
    #[must_use]
    pub fn referenced_locations(&self) -> BTreeSet<&str> {
        self.references
            .iter()
            .map(|r| r.location.as_str())
            .collect()
    }
}

/// One computed-vs-published pair.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    /// Location id.
    pub location: String,
    /// Quantity.
    pub quantity: Quantity,
    /// Value we computed from vendored data.
    pub computed: f64,
    /// Published reference.
    pub reference: f64,
    /// `computed − reference`.
    pub delta: f64,
    /// Declared open band.
    pub tolerance: f64,
    /// `|delta| < tolerance`.
    pub ok: bool,
}

/// Outcome of a clean run (every pair inside its band).
#[derive(Debug, Clone, PartialEq)]
pub struct OracleReport {
    /// Every pair that was compared. Never empty for a live report.
    pub comparisons: Vec<Comparison>,
}

impl OracleReport {
    /// True when every pair agreed.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        !self.comparisons.is_empty() && self.comparisons.iter().all(|c| c.ok)
    }
}

impl fmt::Display for OracleReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "oracle: PASS compared={} locations={}",
            self.comparisons.len(),
            self.comparisons
                .iter()
                .map(|c| c.location.as_str())
                .collect::<BTreeSet<_>>()
                .len()
        )?;
        for c in &self.comparisons {
            writeln!(
                f,
                "  {} {} computed={} reference={} delta={:.6} tol={}",
                c.location,
                c.quantity.as_str(),
                c.computed,
                c.reference,
                c.delta,
                c.tolerance
            )?;
        }
        Ok(())
    }
}

/// Why the oracle could not pass.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum OracleError {
    /// Snapshot loader refused.
    #[error("{0}")]
    Data(#[from] DataError),
    /// Quantity parse / lookup.
    #[error("{0}")]
    Quantity(String),
    /// Ledger could not be parsed.
    #[error("unparseable {path}: {detail}")]
    Unparseable {
        /// Origin.
        path: String,
        /// Parser detail.
        detail: String,
    },
    /// No reference rows.
    #[error("{ANTI_VACUOUS_REFS}")]
    EmptyReferences,
    /// Fewer than five distinct locations.
    #[error("{ANTI_VACUOUS_LOCATIONS} (n={n})")]
    TooFewLocations {
        /// How many distinct locations had a reference.
        n: usize,
    },
    /// A required snapshot was not in the load.
    #[error("oracle snapshot {id} did not load")]
    MissingSnapshot {
        /// Pin id.
        id: String,
    },
    /// A reference names a location the ledger does not define.
    #[error("reference location {id} has no [[location]] row")]
    UnknownLocation {
        /// Location id.
        id: String,
    },
    /// At least one pair is outside its band.
    #[error("{}", format_disagreements(.findings))]
    Disagreement {
        /// Failing pairs. Each names location, computed, reference, delta.
        findings: Vec<Comparison>,
    },
}

fn format_disagreements(findings: &[Comparison]) -> String {
    let mut s = String::new();
    for (i, c) in findings.iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }
        s.push_str(&format!(
            "{DISAGREEMENT} location={} quantity={} computed={} reference={} delta={:.6} tolerance={}",
            c.location,
            c.quantity.as_str(),
            c.computed,
            c.reference,
            c.delta,
            c.tolerance
        ));
    }
    s
}

impl From<QuantityError> for OracleError {
    fn from(e: QuantityError) -> Self {
        OracleError::Quantity(e.to_string())
    }
}

/// `|computed − reference| < tolerance`. Open interval so a one-tolerance
/// perturbation of a match is RED.
#[must_use]
pub fn agrees(computed: f64, reference: f64, tolerance: f64) -> bool {
    (computed - reference).abs() < tolerance
}

/// Shift `computed` by one declared tolerance unit away from `reference`.
/// The matching known-bad test asserts the result is RED.
///
/// Binary64 cannot represent some declared bands (0.015 g) exactly, so
/// `computed + tolerance` can land *inside* the open interval. One ulp
/// outward is still a one-tolerance-unit plant, not a widened band.
#[must_use]
pub fn perturb_one_tolerance(computed: f64, reference: f64, tolerance: f64) -> f64 {
    let mut shifted = if computed >= reference {
        computed + tolerance
    } else {
        computed - tolerance
    };
    let mut guard = 0u32;
    while agrees(shifted, reference, tolerance) && shifted.is_finite() && guard < 8 {
        shifted = if computed >= reference {
            shifted.next_up()
        } else {
            shifted.next_down()
        };
        guard += 1;
    }
    shifted
}

/// Parse a reference ledger. Empty `[[reference]]` is [`OracleError::EmptyReferences`].
pub fn parse_references(text: &str, origin: &str) -> Result<ReferenceLedger, OracleError> {
    let _ = ANTI_VACUOUS_REFS;
    let _ = ANTI_VACUOUS_LOCATIONS;
    let doc: toml::Value = toml::from_str(text).map_err(|e| OracleError::Unparseable {
        path: origin.to_string(),
        detail: e.to_string(),
    })?;

    let mut tolerances = BTreeMap::new();
    if let Some(tol_tbl) = doc.get("tolerance").and_then(|v| v.as_table()) {
        for (k, v) in tol_tbl {
            let Some(q) = Quantity::parse(k) else {
                return Err(OracleError::Unparseable {
                    path: origin.to_string(),
                    detail: format!("unknown tolerance key {k}"),
                });
            };
            let abs = v
                .get("abs")
                .and_then(|x| x.as_float())
                .or_else(|| v.get("abs").and_then(|x| x.as_integer()).map(|i| i as f64));
            let Some(abs) = abs else {
                return Err(OracleError::Unparseable {
                    path: origin.to_string(),
                    detail: format!("tolerance.{k} missing abs"),
                });
            };
            let unit = toml_string(v, "unit").unwrap_or_default();
            let justification = toml_string(v, "justification").unwrap_or_default();
            tolerances.insert(
                q,
                Tolerance {
                    abs,
                    unit,
                    justification,
                },
            );
        }
    }

    let loc_rows = match doc.get("location") {
        Some(toml::Value::Array(a)) => a.as_slice(),
        _ => &[],
    };
    let mut locations = Vec::new();
    for (i, row) in loc_rows.iter().enumerate() {
        let id = toml_string(row, "id").ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("location[{i}] missing id"),
        })?;
        let name = toml_string(row, "name").unwrap_or_else(|| id.clone());
        let lat = toml_f64(row, "lat").ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("location[{i}] ({id}) missing lat"),
        })?;
        let lon = toml_f64(row, "lon").ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("location[{i}] ({id}) missing lon"),
        })?;
        let tmy3_id = toml_string(row, "tmy3_id").unwrap_or_else(|| id.clone());
        let egrid_subregion =
            toml_string(row, "egrid_subregion").ok_or_else(|| OracleError::Unparseable {
                path: origin.to_string(),
                detail: format!("location[{i}] ({id}) missing egrid_subregion"),
            })?;
        locations.push(Location {
            id,
            name,
            lat,
            lon,
            tmy3_id,
            egrid_subregion,
        });
    }

    let ref_rows = match doc.get("reference") {
        Some(toml::Value::Array(a)) => a.as_slice(),
        None => return Err(OracleError::EmptyReferences),
        Some(_) => {
            return Err(OracleError::Unparseable {
                path: origin.to_string(),
                detail: "`reference` must be an array".into(),
            });
        }
    };
    if ref_rows.is_empty() {
        return Err(OracleError::EmptyReferences);
    }
    let mut references = Vec::new();
    for (i, row) in ref_rows.iter().enumerate() {
        let location = toml_string(row, "location").ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("reference[{i}] missing location"),
        })?;
        let qname = toml_string(row, "quantity").ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("reference[{i}] missing quantity"),
        })?;
        let quantity = Quantity::parse(&qname).ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("reference[{i}] unknown quantity {qname}"),
        })?;
        let value = toml_f64(row, "value").ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("reference[{i}] missing value"),
        })?;
        let retrieved = toml_string(row, "retrieved").ok_or_else(|| OracleError::Unparseable {
            path: origin.to_string(),
            detail: format!("reference[{i}] missing retrieved"),
        })?;
        if retrieved.is_empty() {
            return Err(OracleError::Unparseable {
                path: origin.to_string(),
                detail: format!("reference[{i}] empty retrieved"),
            });
        }
        references.push(PublishedRef {
            location,
            quantity,
            value,
            unit: toml_string(row, "unit").unwrap_or_default(),
            source: toml_string(row, "source").unwrap_or_default(),
            url: toml_string(row, "url").unwrap_or_default(),
            retrieved,
        });
    }

    let n_loc = references
        .iter()
        .map(|r| r.location.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    if n_loc < 5 {
        return Err(OracleError::TooFewLocations { n: n_loc });
    }

    Ok(ReferenceLedger {
        locations,
        references,
        tolerances,
    })
}

/// Pins compiled into this crate as a reference ledger.
pub fn compiled_references() -> Result<ReferenceLedger, OracleError> {
    parse_references(COMPILED_REFERENCES, COMPILED_REFERENCES_ORIGIN)
}

/// Run the live compiled ledger against snapshots loaded from `root`.
pub fn check_oracle(root: &Path) -> Result<OracleReport, OracleError> {
    let ledger = compiled_references()?;
    let pins = crate::compiled_pins()?;
    check_oracle_with(root, &ledger, &pins)
}

/// Same check with a caller-supplied ledger and pin list (tests inject plants).
pub fn check_oracle_with(
    root: &Path,
    ledger: &ReferenceLedger,
    pins: &[SnapshotPin],
) -> Result<OracleReport, OracleError> {
    let _ = DISAGREEMENT;
    if ledger.references.is_empty() {
        return Err(OracleError::EmptyReferences);
    }
    let n_loc = ledger.referenced_locations().len();
    if n_loc < 5 {
        return Err(OracleError::TooFewLocations { n: n_loc });
    }

    let mut loaded: BTreeMap<String, LoadedSnapshot> = BTreeMap::new();
    for want in [SNAP_TMY3, SNAP_USGS, SNAP_EGRID] {
        let pin =
            pins.iter()
                .find(|p| p.id == want)
                .ok_or_else(|| OracleError::MissingSnapshot {
                    id: want.to_string(),
                })?;
        let s = load_one(root, pin)?;
        loaded.insert(s.id.clone(), s);
    }
    let tmy3 = snapshot_text(&loaded, SNAP_TMY3)?;
    let usgs = snapshot_text(&loaded, SNAP_USGS)?;
    let egrid = snapshot_text(&loaded, SNAP_EGRID)?;

    let by_id: BTreeMap<&str, &Location> = ledger
        .locations
        .iter()
        .map(|l| (l.id.as_str(), l))
        .collect();

    let mut comparisons = Vec::new();
    let mut findings = Vec::new();
    for pref in &ledger.references {
        let loc = by_id.get(pref.location.as_str()).copied().ok_or_else(|| {
            OracleError::UnknownLocation {
                id: pref.location.clone(),
            }
        })?;
        let tol = ledger
            .tolerances
            .get(&pref.quantity)
            .map(|t| t.abs)
            .ok_or_else(|| OracleError::Unparseable {
                path: COMPILED_REFERENCES_ORIGIN.to_string(),
                detail: format!("no tolerance declared for {}", pref.quantity.as_str()),
            })?;
        let computed = compute(pref.quantity, loc, tmy3, usgs, egrid)?;
        let delta = computed - pref.value;
        let ok = agrees(computed, pref.value, tol);
        let pair = Comparison {
            location: pref.location.clone(),
            quantity: pref.quantity,
            computed,
            reference: pref.value,
            delta,
            tolerance: tol,
            ok,
        };
        if !ok {
            findings.push(pair.clone());
        }
        comparisons.push(pair);
    }

    if !findings.is_empty() {
        return Err(OracleError::Disagreement { findings });
    }
    if comparisons.is_empty() {
        return Err(OracleError::EmptyReferences);
    }
    Ok(OracleReport { comparisons })
}

fn compute(
    q: Quantity,
    loc: &Location,
    tmy3: &str,
    usgs: &str,
    egrid: &str,
) -> Result<f64, OracleError> {
    match q {
        Quantity::FreeCoolingHours => Ok(free_cooling_hours(tmy3, &loc.tmy3_id)?),
        Quantity::HeatingDegreeDays18c => Ok(degree_days(tmy3, &loc.tmy3_id)?.0),
        Quantity::CoolingDegreeDays18c => Ok(degree_days(tmy3, &loc.tmy3_id)?.1),
        Quantity::SeismicSs => Ok(interpolate_seismic(usgs, &loc.id, loc.lat, loc.lon)?.ss),
        Quantity::SeismicS1 => Ok(interpolate_seismic(usgs, &loc.id, loc.lat, loc.lon)?.s1),
        Quantity::SeismicPga => Ok(interpolate_seismic(usgs, &loc.id, loc.lat, loc.lon)?.pga),
        Quantity::GridCo2LbPerMwh => Ok(grid_co2_lb_per_mwh(egrid, &loc.egrid_subregion)?),
    }
}

fn snapshot_text<'a>(
    loaded: &'a BTreeMap<String, LoadedSnapshot>,
    id: &str,
) -> Result<&'a str, OracleError> {
    let s = loaded
        .get(id)
        .ok_or_else(|| OracleError::MissingSnapshot { id: id.to_string() })?;
    std::str::from_utf8(&s.bytes).map_err(|e| OracleError::Quantity(format!("{id}: {e}")))
}

fn toml_string(v: &toml::Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn toml_f64(v: &toml::Value, key: &str) -> Option<f64> {
    let x = v.get(key)?;
    x.as_float().or_else(|| x.as_integer().map(|i| i as f64))
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn agrees_is_open_so_one_tolerance_from_a_match_is_red() {
        assert!(agrees(10.0, 10.0, 1.0));
        assert!(!agrees(11.0, 10.0, 1.0));
        assert!(agrees(10.5, 10.0, 1.0));
    }

    #[test]
    fn perturb_one_tolerance_from_a_match_is_outside() {
        let c = 10.0;
        let r = 10.0;
        let tol = 1.0;
        let bad = perturb_one_tolerance(c, r, tol);
        assert!(!agrees(bad, r, tol), "planted {bad} must be RED");
    }
}
