//! Deterministic site quantities from vendored PD-GOV snapshots.
//!
//! No network. The caller supplies already-loaded snapshot bytes.

use std::collections::BTreeMap;

/// 65.0 °F expressed in °C. Air-side economizer high-limit used for
/// [`free_cooling_hours`].
pub const FREE_COOLING_THRESHOLD_C: f64 = (65.0 - 32.0) * 5.0 / 9.0;

/// EnergyPlus weather-file degree-day base matching STAT "18C baseline".
pub const DEGREE_DAY_BASE_C: f64 = 18.0;

/// Pounds per short ton. eGRID output rate is lb CO2 / MWh.
pub const LB_PER_SHORT_TON: f64 = 2000.0;

/// Why a quantity could not be computed.
#[derive(Debug, Clone, PartialEq)]
pub enum QuantityError {
    /// CSV could not be read as the expected table.
    Parse(String),
    /// Named location has no rows in the snapshot.
    MissingLocation(String),
    /// Named eGRID subregion has no generating plants.
    MissingSubregion(String),
    /// Seismic cell is degenerate or incomplete.
    Grid(String),
}

impl std::fmt::Display for QuantityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantityError::Parse(s) | QuantityError::Grid(s) => f.write_str(s),
            QuantityError::MissingLocation(id) => {
                write!(f, "no TMY3 / grid rows for location {id}")
            }
            QuantityError::MissingSubregion(id) => {
                write!(f, "no eGRID plants for subregion {id}")
            }
        }
    }
}

/// Hours in the TMY3 record with dry-bulb ≤ [`FREE_COOLING_THRESHOLD_C`].
pub fn free_cooling_hours(csv: &str, location_id: &str) -> Result<f64, QuantityError> {
    let temps = tmy3_temps(csv, location_id)?;
    let n = temps
        .iter()
        .filter(|t| **t <= FREE_COOLING_THRESHOLD_C)
        .count();
    Ok(n as f64)
}

/// Daily-mean heating and cooling degree-days at [`DEGREE_DAY_BASE_C`].
///
/// For each civil day: `Tavg` is the mean of that day's hourly dry-bulbs;
/// `HDD += max(0, base − Tavg)`, `CDD += max(0, Tavg − base)`.
pub fn degree_days(csv: &str, location_id: &str) -> Result<(f64, f64), QuantityError> {
    let days = tmy3_daily_means(csv, location_id)?;
    let mut hdd = 0.0;
    let mut cdd = 0.0;
    for tavg in days {
        hdd += (DEGREE_DAY_BASE_C - tavg).max(0.0);
        cdd += (tavg - DEGREE_DAY_BASE_C).max(0.0);
    }
    Ok((hdd, cdd))
}

/// Mapped seismic design values interpolated from a 4-corner cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seismic {
    /// Short-period spectral acceleration Ss (g).
    pub ss: f64,
    /// 1-second spectral acceleration S1 (g).
    pub s1: f64,
    /// Peak ground acceleration (g).
    pub pga: f64,
}

/// Bilinear interpolation of Ss / S1 / PGA at `lat`,`lon` from the
/// location's four vendored grid corners.
pub fn interpolate_seismic(
    csv: &str,
    location_id: &str,
    lat: f64,
    lon: f64,
) -> Result<Seismic, QuantityError> {
    let pts = seismic_points(csv, location_id)?;
    if pts.len() != 4 {
        return Err(QuantityError::Grid(format!(
            "location {location_id} has {} grid points, want 4",
            pts.len()
        )));
    }
    let lats: Vec<f64> = {
        let mut v: Vec<f64> = pts.iter().map(|p| p.0).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v
    };
    let lons: Vec<f64> = {
        let mut v: Vec<f64> = pts.iter().map(|p| p.1).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v
    };
    if lats.len() != 2 || lons.len() != 2 {
        return Err(QuantityError::Grid(format!(
            "location {location_id} is not a 2×2 lat/lon cell"
        )));
    }
    let lat0 = lats[0];
    let lat1 = lats[1];
    let lon0 = lons[0];
    let lon1 = lons[1];
    let at = |la: f64, lo: f64| -> Result<(f64, f64, f64), QuantityError> {
        pts.iter()
            .find(|(a, b, _, _, _)| (*a - la).abs() < 1e-9 && (*b - lo).abs() < 1e-9)
            .map(|p| (p.2, p.3, p.4))
            .ok_or_else(|| QuantityError::Grid(format!("missing corner {la},{lo}")))
    };
    let sw = at(lat0, lon0)?;
    let se = at(lat0, lon1)?;
    let nw = at(lat1, lon0)?;
    let ne = at(lat1, lon1)?;
    let t_lat = ((lat - lat0) / (lat1 - lat0)).clamp(0.0, 1.0);
    let t_lon = ((lon - lon0) / (lon1 - lon0)).clamp(0.0, 1.0);
    let blend = |a: f64, b: f64, c: f64, d: f64| {
        let south = a * (1.0 - t_lon) + b * t_lon;
        let north = c * (1.0 - t_lon) + d * t_lon;
        south * (1.0 - t_lat) + north * t_lat
    };
    Ok(Seismic {
        ss: blend(sw.0, se.0, nw.0, ne.0),
        s1: blend(sw.1, se.1, nw.1, ne.1),
        pga: blend(sw.2, se.2, nw.2, ne.2),
    })
}

/// Generation-weighted CO2 output emission rate (lb/MWh) for `subregion`.
///
/// `2000 × Σ plant CO2 tons / Σ plant net generation MWh`.
pub fn grid_co2_lb_per_mwh(csv: &str, subregion: &str) -> Result<f64, QuantityError> {
    let mut gen = 0.0;
    let mut tons = 0.0;
    let mut n = 0usize;
    for (i, line) in csv.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if t.starts_with("orispl") {
            continue;
        }
        let cols: Vec<&str> = t.split(',').collect();
        if cols.len() < 4 {
            return Err(QuantityError::Parse(format!(
                "eGRID plants line {}: want 4 columns",
                i + 1
            )));
        }
        if cols[1].trim() != subregion {
            continue;
        }
        let g: f64 = cols[2].trim().parse().map_err(|_| {
            QuantityError::Parse(format!("eGRID plants line {}: bad net_gen", i + 1))
        })?;
        let c: f64 = cols[3].trim().parse().map_err(|_| {
            QuantityError::Parse(format!("eGRID plants line {}: bad co2_tons", i + 1))
        })?;
        gen += g;
        tons += c;
        n += 1;
    }
    if n == 0 || gen == 0.0 {
        return Err(QuantityError::MissingSubregion(subregion.to_string()));
    }
    Ok(tons * LB_PER_SHORT_TON / gen)
}

fn tmy3_temps(csv: &str, location_id: &str) -> Result<Vec<f64>, QuantityError> {
    let mut out = Vec::new();
    for (i, line) in csv.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("location_id") {
            continue;
        }
        let cols: Vec<&str> = t.split(',').collect();
        if cols.len() < 6 {
            return Err(QuantityError::Parse(format!(
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
            .map_err(|_| QuantityError::Parse(format!("TMY3 line {}: bad dry_bulb_c", i + 1)))?;
        out.push(db);
    }
    if out.is_empty() {
        return Err(QuantityError::MissingLocation(location_id.to_string()));
    }
    Ok(out)
}

fn tmy3_daily_means(csv: &str, location_id: &str) -> Result<Vec<f64>, QuantityError> {
    let mut days: BTreeMap<(u32, u32), Vec<f64>> = BTreeMap::new();
    for (i, line) in csv.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("location_id") {
            continue;
        }
        let cols: Vec<&str> = t.split(',').collect();
        if cols.len() < 6 {
            return Err(QuantityError::Parse(format!(
                "TMY3 line {}: want 6 columns",
                i + 1
            )));
        }
        if cols[0].trim() != location_id {
            continue;
        }
        let month: u32 = cols[2]
            .trim()
            .parse()
            .map_err(|_| QuantityError::Parse(format!("TMY3 line {}: bad month", i + 1)))?;
        let day: u32 = cols[3]
            .trim()
            .parse()
            .map_err(|_| QuantityError::Parse(format!("TMY3 line {}: bad day", i + 1)))?;
        let db: f64 = cols[5]
            .trim()
            .parse()
            .map_err(|_| QuantityError::Parse(format!("TMY3 line {}: bad dry_bulb_c", i + 1)))?;
        days.entry((month, day)).or_default().push(db);
    }
    if days.is_empty() {
        return Err(QuantityError::MissingLocation(location_id.to_string()));
    }
    Ok(days
        .into_values()
        .map(|v| v.iter().sum::<f64>() / v.len() as f64)
        .collect())
}

fn seismic_points(
    csv: &str,
    location_id: &str,
) -> Result<Vec<(f64, f64, f64, f64, f64)>, QuantityError> {
    let mut out = Vec::new();
    for (i, line) in csv.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("location_id") {
            continue;
        }
        let cols: Vec<&str> = t.split(',').collect();
        if cols.len() < 6 {
            return Err(QuantityError::Parse(format!(
                "USGS grid line {}: want 6 columns",
                i + 1
            )));
        }
        if cols[0].trim() != location_id {
            continue;
        }
        let parse = |j: usize, name: &str| -> Result<f64, QuantityError> {
            cols[j]
                .trim()
                .parse()
                .map_err(|_| QuantityError::Parse(format!("USGS grid line {}: bad {name}", i + 1)))
        };
        out.push((
            parse(1, "lat")?,
            parse(2, "lon")?,
            parse(3, "ss")?,
            parse(4, "s1")?,
            parse(5, "pga")?,
        ));
    }
    if out.is_empty() {
        return Err(QuantityError::MissingLocation(location_id.to_string()));
    }
    Ok(out)
}
