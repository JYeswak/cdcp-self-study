//! Product CLI for `cdcp_site` (`bd-hardening-f-oracle-qly.13`).
//!
//! Live Ashburn lookup prints named climate / seismic / carbon / flood
//! / power-price quantities. The power-price line carries units
//! (cents/kWh); a bare number is an ERROR. A missing location is
//! non-zero and names the location. No network.

use assert_cmd::Command;
use cdcp_site::{MISSING_LOCATION, NOT_IN_SFHA};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve course-engine workspace root")
}

fn cdcp() -> Command {
    let mut cmd = Command::cargo_bin("cdcp").expect("cdcp binary");
    cmd.current_dir(workspace_root());
    cmd
}

fn named_quantities() -> [&'static str; 10] {
    [
        "site ashburn",
        "climate_bin=",
        "free_cooling_hours=",
        "seismic",
        "pga=",
        "grid_co2_lb_per_mwh=",
        "flood_zone=",
        NOT_IN_SFHA,
        "power_price=",
        "cents/kWh",
    ]
}

/// clap about for `site`. The three-hazard prefix is not enough: the
/// verb also prints flood and power price. Pinning the five-name
/// string makes a revert to the leftover about fail this file.
const SITE_ABOUT: &str = "Climate / seismic / carbon / flood / power price";

#[test]
fn help_lists_site() {
    let assert = cdcp().env("CDCP_DEV", "1").arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(SITE_ABOUT),
        "cdcp --help must name flood and power price on site: {stdout}"
    );
    // clap prints the command name as its own token in the Commands list.
    // "site" alone is too weak: oracle-check's about text already says "site".
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("site")),
        "cdcp --help must list the `site` command: {stdout}"
    );
}

#[test]
fn site_help_lists_location_and_coord_flags() {
    let assert = cdcp().args(["site", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(SITE_ABOUT),
        "cdcp site --help must name flood and power price: {stdout}"
    );
    for flag in ["--location", "--lat", "--lon"] {
        assert!(
            stdout.contains(flag),
            "cdcp site --help must list {flag}: {stdout}"
        );
    }
}

#[test]
fn site_location_ashburn_prints_named_quantities() {
    let assert = cdcp()
        .args(["site", "--location", "ashburn"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for needle in named_quantities() {
        assert!(
            stdout.contains(needle),
            "live Ashburn lookup must print {needle}: {stdout}"
        );
    }
}

/// The print path must carry units. `power_price=10.53` with no unit
/// is the leftover this bead exists to make unrepresentable.
#[test]
fn site_location_ashburn_power_price_is_not_a_bare_number() {
    let assert = cdcp()
        .args(["site", "--location", "ashburn"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("power_price="))
        .unwrap_or_else(|| panic!("live Ashburn lookup must print power_price=: {stdout}"));
    assert!(
        line.contains("cents/kWh"),
        "power-price line must carry units: {line}"
    );
    let rest = line
        .split_once('=')
        .map(|(_, r)| r.trim())
        .expect("power_price=");
    assert!(
        rest.parse::<f64>().is_err(),
        "bare number on power_price line is an ERROR: {line}"
    );
}

#[test]
fn site_lat_lon_ashburn_prints_named_quantities() {
    let assert = cdcp()
        .args(["site", "--lat", "39.0438", "--lon=-77.4874"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for needle in named_quantities() {
        assert!(
            stdout.contains(needle),
            "Ashburn coord lookup must print {needle}: {stdout}"
        );
    }
}

#[test]
fn site_missing_location_is_nonzero_and_names_it() {
    let assert = cdcp()
        .args(["site", "--location", "atlantis"])
        .assert()
        .failure();
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        out.contains("atlantis"),
        "missing location must name the id: {out}"
    );
    assert!(
        out.contains(MISSING_LOCATION),
        "missing location must use the named-error token: {out}"
    );
}

#[test]
fn site_missing_coord_is_nonzero_and_names_it() {
    let assert = cdcp()
        .args(["site", "--lat", "0", "--lon", "0"])
        .assert()
        .failure();
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        out.contains("0,0") || out.contains("0, 0"),
        "missing coord must name the location: {out}"
    );
    assert!(
        out.contains(MISSING_LOCATION),
        "missing coord must use the named-error token: {out}"
    );
}

#[test]
fn site_without_query_is_nonzero() {
    let assert = cdcp().arg("site").assert().failure();
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&assert.get_output().stdout),
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
    assert!(
        out.contains("--location") && out.contains("--lat"),
        "bare site must name the required flags: {out}"
    );
}

/// Meta: delete the lookup calls or add a socket → this selftest is non-zero.
#[test]
fn cli_site_source_calls_lookup_and_has_no_network() {
    let src = include_str!("../src/site.rs");
    assert!(
        src.contains("lookup_id("),
        "delete the lookup_id call → selftest non-zero"
    );
    assert!(
        src.contains("lookup_coord("),
        "delete the lookup_coord call → selftest non-zero"
    );
    assert!(
        src.contains("MISSING_LOCATION"),
        "delete the missing-location token → selftest non-zero"
    );
    assert!(
        src.contains("profile.flood") && src.contains("FLOOD_NOT_VENDORED"),
        "delete the flood field / named-error token → selftest non-zero"
    );
    assert!(
        src.contains("profile.power_price") && src.contains("BARE_PRICE_NUMBER"),
        "delete the power_price field / named-error token → selftest non-zero"
    );
    assert!(
        src.contains("price.unit"),
        "delete the unit check → a bare number can reach stdout"
    );
    for needle in [
        "TcpStream",
        "UdpSocket",
        "TcpListener",
        "std::net",
        "::net::",
        "reqwest",
        "ureq",
        "hyper::",
    ] {
        assert!(!src.contains(needle), "site CLI must not mention {needle}");
    }
}
