//! Product CLI for `cdcp_metrics` (`bd-hardening-f-oracle-qly.9`).
//!
//! A well-formed PUE/WUE with an explicit Boundary prints the crate's
//! Display (kind + rational + boundary). A bare number or omitted
//! `[boundary]` is non-zero and names the schema token.

use assert_cmd::Command;
use cdcp_metrics::{parse_metric, BARE_NUMBER, MISSING_BOUNDARY};
use std::fs;
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

fn combined(assert: &assert_cmd::assert::Assert) -> String {
    let out = assert.get_output();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Closed-form PUE 6/5 with the parse.rs well-formed boundary.
const GOOD_PUE: &str = "\
kind = \"pue\"
value = { num = 6, den = 5 }
[boundary]
it_meter = \"ups-output\"
includes = [\"it-energy\", \"cooling\", \"lighting\", \"ups-losses\"]
excludes = [\"generator-testing\", \"office-hvac\"]
";

/// Site WUE 9/25 L/kWh. Hydro omitted is honest only for site scope.
const GOOD_WUE: &str = "\
kind = \"wue\"
value = { num = 9, den = 25 }
[boundary]
it_meter = \"ups-output\"
includes = [\"cooling-tower-evaporation\", \"blowdown\", \"humidification\"]
excludes = [\"fire-water\", \"energy-water\"]
water_scope = \"site\"
";

const NO_BOUNDARY: &str = "\
kind = \"pue\"
value = { num = 6, den = 5 }
";

#[test]
fn help_lists_metrics() {
    let assert = cdcp().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("PUE / WUE / CUE / ERE"),
        "cdcp --help must list the metrics verb: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().next() == Some("metrics")),
        "cdcp --help must list the `metrics` command: {stdout}"
    );
}

#[test]
fn metrics_help_lists_file_and_doc_flags() {
    let assert = cdcp().args(["metrics", "--help"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for flag in ["--file", "--doc"] {
        assert!(
            stdout.contains(flag),
            "cdcp metrics --help must list {flag}: {stdout}"
        );
    }
}

#[test]
fn well_formed_pue_with_boundary_prints_value() {
    let expected = parse_metric(GOOD_PUE)
        .expect("fixture PUE must parse")
        .to_string();
    let assert = cdcp()
        .args(["metrics", "--doc", GOOD_PUE])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(&expected),
        "well-formed PUE must print the crate Display ({expected}): {stdout}"
    );
    assert!(stdout.contains("pue"), "must name the kind: {stdout}");
    assert!(stdout.contains("6/5"), "must print the rational: {stdout}");
    assert!(
        stdout.contains("it_meter="),
        "must print the boundary, not a bare number: {stdout}"
    );
    assert!(
        !stdout.trim().eq("6/5") && !stdout.contains("1.2"),
        "must not collapse to a marketing float: {stdout}"
    );
}

#[test]
fn well_formed_wue_file_prints_value() {
    let dir = std::env::temp_dir().join(format!(
        "cdcp_cli_metrics_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("wue.toml");
    fs::write(&path, GOOD_WUE).expect("write wue");
    let expected = parse_metric(GOOD_WUE)
        .expect("fixture WUE must parse")
        .to_string();

    let assert = cdcp()
        .args(["metrics", "--file"])
        .arg(&path)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(&expected),
        "well-formed WUE must print the crate Display ({expected}): {stdout}"
    );
    assert!(stdout.contains("wue"), "must name the kind: {stdout}");
    assert!(stdout.contains("9/25"), "must print the rational: {stdout}");
    assert!(
        stdout.contains("water_scope=site"),
        "must print the WUE boundary: {stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bare_number_is_nonzero() {
    for raw in ["1.8", "18", "9/5", "1.54"] {
        let assert = cdcp().args(["metrics", "--doc", raw]).assert().failure();
        let out = combined(&assert);
        assert!(
            out.contains(BARE_NUMBER),
            "bare {raw} must name the schema token: {out}"
        );
    }
}

#[test]
fn omitted_boundary_is_nonzero() {
    let assert = cdcp()
        .args(["metrics", "--doc", NO_BOUNDARY])
        .assert()
        .failure();
    let out = combined(&assert);
    assert!(
        out.contains(MISSING_BOUNDARY),
        "omitted [boundary] must name the schema token: {out}"
    );
}

#[test]
fn metrics_without_input_is_nonzero() {
    let assert = cdcp().arg("metrics").assert().failure();
    let out = combined(&assert);
    assert!(
        out.contains("--file") && out.contains("--doc"),
        "bare metrics must name the required flags: {out}"
    );
}

/// Meta: delete the parse_metric call or add a socket → this selftest is non-zero.
#[test]
fn cli_metrics_source_calls_parse_metric_and_has_no_network() {
    let src = include_str!("../src/metrics.rs");
    assert!(
        src.contains("parse_metric("),
        "delete the parse_metric call → selftest non-zero"
    );
    assert!(
        src.contains("BARE_NUMBER"),
        "delete the bare-number token → selftest non-zero"
    );
    assert!(
        src.contains("MISSING_BOUNDARY"),
        "delete the missing-boundary token → selftest non-zero"
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
        assert!(
            !src.contains(needle),
            "metrics CLI must not mention {needle}"
        );
    }
}
