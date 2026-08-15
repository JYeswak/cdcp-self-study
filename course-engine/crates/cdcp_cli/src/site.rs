//! Product CLI for `cdcp_site` (`bd-hardening-f-oracle-qly.12`).
//!
//! BUILT ≠ WIRED: vendored FEMA NFHL (4up.6, 34acd3d) is already on
//! `SiteProfile.flood`. This verb prints climate / seismic / carbon /
//! flood from those snapshots. A missing flood snapshot or an empty
//! zone is a named ERROR, never a silent omit. No network.

use cdcp_site::{
    engine_root, lookup_coord, lookup_id, SiteQuery, FLOOD_NOT_VENDORED, MISSING_LOCATION,
};
use std::path::{Path, PathBuf};

/// `cdcp site --location <id>` / `cdcp site --lat --lon`.
pub(crate) fn run(
    root: Option<&Path>,
    location: Option<&str>,
    lat: Option<f64>,
    lon: Option<f64>,
) -> Result<(), String> {
    let resolved = resolve_root(root)?;
    let query = query(location, lat, lon)?;
    let profile = match query {
        SiteQuery::Id(id) => lookup_id(&resolved, id),
        SiteQuery::Coord { lat, lon } => lookup_coord(&resolved, lat, lon),
    }
    .map_err(|e| e.to_string())?;
    // First-class product field. Binding `.flood` makes dropping it from
    // SiteProfile a compile error here. An empty zone is the named
    // flood-not-vendored ERROR, not an omitted `flood_zone=` line.
    let flood = &profile.flood;
    if flood.zone.is_empty() {
        return Err(format!("{FLOOD_NOT_VENDORED}: {}", profile.location.id));
    }
    print!("{profile}");
    Ok(())
}

fn resolve_root(root: Option<&Path>) -> Result<PathBuf, String> {
    match root {
        Some(p) => Ok(p.to_path_buf()),
        None => {
            let start = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
            engine_root(&start).map_err(|e| e.to_string())
        }
    }
}

fn query<'a>(
    location: Option<&'a str>,
    lat: Option<f64>,
    lon: Option<f64>,
) -> Result<SiteQuery<'a>, String> {
    match (location, lat, lon) {
        (Some(id), None, None) => {
            let id = id.trim();
            if id.is_empty() {
                return Err(format!("{MISSING_LOCATION}: <empty --location>"));
            }
            Ok(SiteQuery::Id(id))
        }
        (None, Some(lat), Some(lon)) => Ok(SiteQuery::Coord { lat, lon }),
        (None, None, None) => {
            Err("site requires --location <id> or --lat <deg> --lon <deg>".into())
        }
        (Some(_), _, _) => Err("site: --location cannot be combined with --lat/--lon".into()),
        (None, Some(_), None) => Err("site: --lat requires --lon".into()),
        (None, None, Some(_)) => Err("site: --lon requires --lat".into()),
    }
}
