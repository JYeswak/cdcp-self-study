//! Product CLI for `cdcp_site` (`bd-hardening-f-oracle-qly.7`).
//!
//! Live Ashburn lookup prints named climate / seismic / carbon quantities.
//! A missing location is non-zero and names the location. No network.

use assert_cmd::Command;
use cdcp_site::MISSING_LOCATION;
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

fn named_quantities() -> &'static [&'static str] {
    &[
        "site ashburn",
        "climate_bin=",
        "free_cooling_hours=",
        "seismic",
        "pga=",
        "grid_co2_lb_per_mwh=",
    ]
}

#[test]
fn help_lists_site() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("Climate / seismic / carbon"),
        "cdcp --help must list the site verb: {stdout}"
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
