//! Free-cooling hours are consumed from cdcp_data, not re-derived.

use cdcp_metrics::{free_cooling_hours, take_free_cooling_hours, Ratio};

/// Six hours at `mms-site`. `decoy` rows must not count.
///
/// Paper (`free_cooling_hours/count` — same fixture as cdcp_data MMS):
///   T* is owned by cdcp_data (air-side high-limit, ≤).
///   hour  T     T ≤ T*?
///      1  10    yes
///      2  18    yes
///      3  19    no
///      4   0    yes
///      5  30    no
///      6  −5    yes
///   hours = 4
const TMY3_COUNT: &str = "\
location_id,wmo,month,day,hour,dry_bulb_c
mms-site,0,1,1,1,10.0
mms-site,0,1,1,2,18.0
mms-site,0,1,1,3,19.0
mms-site,0,1,1,4,0.0
mms-site,0,1,1,5,30.0
mms-site,0,1,1,6,-5.0
decoy,0,1,1,1,-100.0
decoy,0,1,1,2,-100.0
";

#[test]
fn consumes_cdcp_data_count() {
    let hours = free_cooling_hours(TMY3_COUNT, "mms-site").expect("mms-site hours");
    assert_eq!(hours, Ratio::from_int(4));
}

#[test]
fn take_from_site_climate_count() {
    // cdcp_site::Climate.free_cooling_hours is this same f64 count.
    let hours = take_free_cooling_hours(5734.0).expect("ashburn-shaped count");
    assert_eq!(hours, Ratio::from_int(5734));
}

#[test]
fn missing_location_is_error() {
    let err = free_cooling_hours(TMY3_COUNT, "nowhere").expect_err("missing");
    let text = err.to_string();
    assert!(
        text.contains("nowhere") || text.contains("free-cooling"),
        "{text}"
    );
}

#[test]
fn production_calls_cdcp_data_and_does_not_rederive() {
    let src = include_str!("../src/free_cooling.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(
        src.contains("cdcp_data::free_cooling_hours"),
        "must consume cdcp_data::free_cooling_hours"
    );
    for needle in [
        "FREE_COOLING_THRESHOLD",
        "65.0",
        "18.33",
        "18.333",
        "(65.0 - 32.0)",
    ] {
        assert!(
            !src.contains(needle),
            "re-derived free-cooling threshold ({needle})"
        );
    }
}
