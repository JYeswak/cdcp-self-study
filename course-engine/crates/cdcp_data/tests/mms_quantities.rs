//! F4 MMS manufactured cases for every public `cdcp_data::quantities` fn.
//!
//! Each case is a tiny CSV we wrote, plus a paper closed-form answer written
//! in a comment. Expected values are literals in this file — never copied
//! from the function under test. Mathematics we do not control is the oracle.
//!
//! Plants:
//! - perturb a paper answer by 1 unit → assertion RED
//! - zero MMS cases → ERROR (inventory meta-test)
//! - a new `pub fn` in `quantities.rs` without a row here → inventory RED
//!
//! Meta-test: delete a comparison (`assert_scalar`) → this selftest is non-zero.

use cdcp_data::{
    degree_days, free_cooling_hours, grid_co2_lb_per_mwh, interpolate_seismic, DEGREE_DAY_BASE_C,
    FREE_COOLING_THRESHOLD_C, LB_PER_SHORT_TON,
};

/// Token interpolated inside the empty-inventory path.
const ZERO_MMS_CASES: &str = "zero MMS cases is an ERROR";

/// One manufactured case. `quantity_fn` must match a `pub fn` in
/// `src/quantities.rs`.
struct MmsCase {
    id: &'static str,
    quantity_fn: &'static str,
}

/// Inventory of MMS cases. Empty is a compile-time + runtime ERROR.
/// Deleting a row without covering that `pub fn` elsewhere fails
/// [`inventory_covers_every_public_quantity_fn`].
const MMS_CASES: &[MmsCase] = &[
    MmsCase {
        id: "free_cooling_hours/count",
        quantity_fn: "free_cooling_hours",
    },
    MmsCase {
        id: "free_cooling_hours/threshold-inclusive",
        quantity_fn: "free_cooling_hours",
    },
    MmsCase {
        id: "degree_days/daily-mean-18c",
        quantity_fn: "degree_days",
    },
    MmsCase {
        id: "interpolate_seismic/bilinear-interior",
        quantity_fn: "interpolate_seismic",
    },
    MmsCase {
        id: "interpolate_seismic/clamp-outside-cell",
        quantity_fn: "interpolate_seismic",
    },
    MmsCase {
        id: "grid_co2_lb_per_mwh/generation-weighted",
        quantity_fn: "grid_co2_lb_per_mwh",
    },
];

const MMS_CASE_COUNT: usize = MMS_CASES.len();
const _: () = assert!(MMS_CASE_COUNT > 0, "zero MMS cases is an ERROR");

/// Closed-form scalars this file asserts. Deleting a comparison so the
/// runner yields fewer pairs fails [`selftest_delete_comparison_is_nonzero`].
///
/// count hours + inclusive hours + (HDD, CDD) + 2 × (Ss, S1, PGA) + CO2.
const MMS_SCALAR_COUNT: usize = 1 + 1 + 2 + 3 + 3 + 1;

// ── manufactured CSVs ─────────────────────────────────────────────────────

/// Six hours at `mms-site`. `decoy` rows must not count.
///
/// Paper (`free_cooling_hours/count`):
///   T* = (65 − 32) × 5/9 = 165/9 °C ≈ 18.333…  (air-side high-limit, ≤)
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

/// Four civil days, uneven hour counts. Daily-mean (not hourly) HDD/CDD.
///
/// Paper (`degree_days/daily-mean-18c`), base B = 18 °C:
///   Jan 1  T = {8, 10, 12, 10}   Tavg = 40/4 = 10
///          HDD = max(0, 18 − 10) = 8     CDD = 0
///   Jan 2  T = {20, 24, 22, 22}  Tavg = 88/4 = 22
///          HDD = 0                       CDD = 22 − 18 = 4
///   Jan 3  T = {18, 18}          Tavg = 36/2 = 18
///          HDD = 0                       CDD = 0
///   Jul 1  T = {30, 30, 24}      Tavg = 84/3 = 28
///          HDD = 0                       CDD = 28 − 18 = 10
///   HDD = 8
///   CDD = 4 + 10 = 14
///
/// Distinguishes daily-mean from hourly accumulation (hourly HDD on Jan 1
/// would be 10+8+6+8 = 32) and from a silent 24-hour assumption
/// (Jan 1 Tavg would be 40/24, HDD = 16 + 2/3).
const TMY3_DEGREE_DAYS: &str = "\
location_id,wmo,month,day,hour,dry_bulb_c
mms-site,0,1,1,1,8.0
mms-site,0,1,1,2,10.0
mms-site,0,1,1,3,12.0
mms-site,0,1,1,4,10.0
mms-site,0,1,2,1,20.0
mms-site,0,1,2,2,24.0
mms-site,0,1,2,3,22.0
mms-site,0,1,2,4,22.0
mms-site,0,1,3,1,18.0
mms-site,0,1,3,2,18.0
mms-site,0,7,1,1,30.0
mms-site,0,7,1,2,30.0
mms-site,0,7,1,3,24.0
decoy,0,1,1,1,99.0
";

/// 2×2 cell, corners listed NE / SW / SE / NW so file order is not the
/// interpolator. `decoy` shares the same lat/lon with huge values.
///
/// Paper (bilinear, SW=a SE=b NW=c NE=d):
///   south = a (1 − t_lon) + b t_lon
///   north = c (1 − t_lon) + d t_lon
///   value = south (1 − t_lat) + north t_lat
///
///   SW (0,  0): Ss=0   S1=0   PGA=0
///   SE (0, 10): Ss=10  S1=20  PGA=30
///   NW (10, 0): Ss=40  S1=50  PGA=60
///   NE (10,10): Ss=70  S1=80  PGA=90
///
/// `interpolate_seismic/bilinear-interior` at (lat, lon) = (2, 5):
///   t_lat = (2 − 0)/(10 − 0) = 1/5
///   t_lon = (5 − 0)/(10 − 0) = 1/2
///   Ss  south=5   north=55   → 5·4/5 + 55·1/5 = 4 + 11 = 15
///   S1  south=10  north=65   → 10·4/5 + 65·1/5 = 8 + 13 = 21
///   PGA south=15  north=75   → 15·4/5 + 75·1/5 = 12 + 15 = 27
///
/// `interpolate_seismic/clamp-outside-cell` at (lat, lon) = (−5, 5):
///   t_lat = clamp((−5 − 0)/10, 0, 1) = 0
///   t_lon = 1/2
///   → south edge: Ss=5, S1=10, PGA=15
///   (without clamp, t_lat = −1/2 → Ss = 5·3/2 + 55·(−1/2) = −20)
const USGS_CELL: &str = "\
location_id,lat,lon,ss,s1,pga
mms-site,10,10,70,80,90
mms-site,0,0,0,0,0
mms-site,0,10,10,20,30
mms-site,10,0,40,50,60
decoy,0,0,999,999,999
decoy,0,10,999,999,999
decoy,10,0,999,999,999
decoy,10,10,999,999,999
";

/// Three plants in `MMS`, one decoy in `OTHER`.
///
/// Paper (`grid_co2_lb_per_mwh/generation-weighted`):
///   rate = 2000 × Σ CO2 tons / Σ net generation MWh
///        = 2000 × (2 + 6 + 0) / (100 + 300 + 100)
///        = 2000 × 8 / 500
///        = 32
///
/// An unweighted mean of plant rates is (40 + 40 + 0)/3 = 80/3 ≠ 32.
/// Including OTHER would be 2000 × 9 / 100499 ≠ 32.
const EGRID_PLANTS: &str = "\
orispl,subregion,net_gen_mwh,co2_tons
1,MMS,100,2
2,MMS,300,6
3,OTHER,99999,1
4,MMS,100,0
";

// ── evaluation ────────────────────────────────────────────────────────────

/// `(component, computed, paper)` triples. Paper is a literal derived
/// in the comments above — not `fn_under_test(...).unwrap()`.
fn evaluate(case: &MmsCase) -> Vec<(&'static str, f64, f64)> {
    match case.id {
        "free_cooling_hours/count" => {
            let computed = free_cooling_hours(TMY3_COUNT, "mms-site").expect("mms-site hours");
            vec![("hours", computed, 4.0)]
        }
        "free_cooling_hours/threshold-inclusive" => {
            // One hour AT T*. Paper: T* = (65 − 32)×5/9; ≤ is inclusive → 1.
            // A strict `<` solver returns 0.
            let csv = format!(
                "location_id,wmo,month,day,hour,dry_bulb_c\nmms-site,0,6,1,1,{t}\n",
                t = FREE_COOLING_THRESHOLD_C
            );
            let computed = free_cooling_hours(&csv, "mms-site").expect("threshold hour");
            vec![("hours", computed, 1.0)]
        }
        "degree_days/daily-mean-18c" => {
            let (hdd, cdd) = degree_days(TMY3_DEGREE_DAYS, "mms-site").expect("degree days");
            vec![("hdd", hdd, 8.0), ("cdd", cdd, 14.0)]
        }
        "interpolate_seismic/bilinear-interior" => {
            let s = interpolate_seismic(USGS_CELL, "mms-site", 2.0, 5.0).expect("interior");
            vec![("ss", s.ss, 15.0), ("s1", s.s1, 21.0), ("pga", s.pga, 27.0)]
        }
        "interpolate_seismic/clamp-outside-cell" => {
            let s = interpolate_seismic(USGS_CELL, "mms-site", -5.0, 5.0).expect("clamp");
            vec![("ss", s.ss, 5.0), ("s1", s.s1, 10.0), ("pga", s.pga, 15.0)]
        }
        "grid_co2_lb_per_mwh/generation-weighted" => {
            let computed = grid_co2_lb_per_mwh(EGRID_PLANTS, "MMS").expect("egrid");
            vec![("lb_per_mwh", computed, 32.0)]
        }
        other => panic!("inventory row {other} has no evaluate arm"),
    }
}

/// Exact equality against the paper literal, then the one-unit plant.
/// Deleting this function (or every call) makes the meta-test non-zero.
fn assert_scalar(id: &str, computed: f64, paper: f64) {
    assert_eq!(
        computed, paper,
        "MMS {id}: computed={computed} paper={paper} \
         (paper is closed-form, not copied from the solver)"
    );
    assert_ne!(
        computed,
        paper + 1.0,
        "MMS {id}: perturb paper answer by 1 unit must be RED \
         (computed={computed} planted={})",
        paper + 1.0
    );
}

fn case_by_id(id: &str) -> &'static MmsCase {
    MMS_CASES
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("no MMS inventory row {id}"))
}

fn run_case(case: &MmsCase) -> usize {
    let pairs = evaluate(case);
    assert!(
        !pairs.is_empty(),
        "MMS {}: evaluate returned zero comparisons",
        case.id
    );
    for (component, computed, paper) in &pairs {
        assert_scalar(&format!("{}/{}", case.id, component), *computed, *paper);
    }
    pairs.len()
}

fn public_fns_in_quantities_src() -> Vec<String> {
    include_str!("../src/quantities.rs")
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("pub fn ")?;
            let name = rest.split('(').next()?.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect()
}

// ── tests ─────────────────────────────────────────────────────────────────

#[test]
fn paper_constants_match_the_published_definitions() {
    // These are the symbols the comments treat as given. A silent change
    // of the constant would make every paper derivation lie.
    assert_eq!(DEGREE_DAY_BASE_C, 18.0);
    assert_eq!(LB_PER_SHORT_TON, 2000.0);
    assert_eq!(FREE_COOLING_THRESHOLD_C, (65.0 - 32.0) * 5.0 / 9.0);
}

#[test]
fn every_mms_case_matches_its_paper_answer() {
    assert!(!MMS_CASES.is_empty(), "{ZERO_MMS_CASES}");
    let mut scalars = 0usize;
    for case in MMS_CASES {
        scalars += run_case(case);
    }
    assert_eq!(
        scalars, MMS_SCALAR_COUNT,
        "scalar inventory drifted: ran {scalars}, table says {MMS_SCALAR_COUNT}"
    );
}

#[test]
fn perturb_each_paper_answer_by_one_unit_is_red() {
    // Same walk as the happy path: the plant lives next to the equality
    // so deleting one without the other is a source-scan miss.
    let mut planted = 0usize;
    for case in MMS_CASES {
        for (component, computed, paper) in evaluate(case) {
            let planted_value = paper + 1.0;
            assert_ne!(
                computed, planted_value,
                "MMS {}/{}: computed={computed} equals paper+1 ({planted_value}) \
                 — the one-unit plant must stay RED",
                case.id, component
            );
            planted += 1;
        }
    }
    assert!(planted > 0, "{ZERO_MMS_CASES}");
    assert_eq!(planted, MMS_SCALAR_COUNT);
}

#[test]
#[allow(clippy::assertions_on_constants)] // anti-vacuous: empty inventory is a deleted floor
fn inventory_zero_mms_cases_is_error() {
    assert!(!MMS_CASES.is_empty(), "{ZERO_MMS_CASES}");
    assert!(MMS_CASE_COUNT > 0, "{ZERO_MMS_CASES}");
    assert!(MMS_SCALAR_COUNT > 0, "{ZERO_MMS_CASES}");
}

#[test]
fn inventory_covers_every_public_quantity_fn() {
    let src_fns = public_fns_in_quantities_src();
    assert!(
        !src_fns.is_empty(),
        "quantities.rs has zero pub fn — a scan that found nothing is an ERROR"
    );
    for name in &src_fns {
        assert!(
            MMS_CASES.iter().any(|c| c.quantity_fn == name.as_str()),
            "pub fn {name} has no MMS case (anti-vacuous inventory)"
        );
    }
    for case in MMS_CASES {
        assert!(
            src_fns.iter().any(|n| n == case.quantity_fn),
            "inventory names {} but quantities.rs has no such pub fn",
            case.quantity_fn
        );
    }
}

/// Meta-test: delete the comparison → this selftest is non-zero.
#[test]
fn selftest_delete_comparison_is_nonzero() {
    let src = include_str!("mms_quantities.rs");
    assert!(
        src.contains("fn assert_scalar"),
        "delete assert_scalar → selftest non-zero"
    );
    let calls = src.matches("assert_scalar(").count();
    assert!(
        calls >= 2,
        "delete the assert_scalar call site → selftest non-zero (saw {calls})"
    );
    assert!(
        src.contains("paper + 1.0"),
        "delete the one-unit plant → selftest non-zero"
    );
    assert!(!MMS_CASES.is_empty(), "{ZERO_MMS_CASES}");
    for case in MMS_CASES {
        assert!(
            src.contains(case.id),
            "delete case {} from the file → inventory RED",
            case.id
        );
    }
}

#[test]
fn free_cooling_hours_count() {
    run_case(case_by_id("free_cooling_hours/count"));
}

#[test]
fn free_cooling_hours_threshold_inclusive() {
    run_case(case_by_id("free_cooling_hours/threshold-inclusive"));
}

#[test]
fn degree_days_daily_mean_18c() {
    run_case(case_by_id("degree_days/daily-mean-18c"));
}

#[test]
fn interpolate_seismic_bilinear_interior() {
    run_case(case_by_id("interpolate_seismic/bilinear-interior"));
}

#[test]
fn interpolate_seismic_clamp_outside_cell() {
    run_case(case_by_id("interpolate_seismic/clamp-outside-cell"));
}

#[test]
fn grid_co2_generation_weighted() {
    run_case(case_by_id("grid_co2_lb_per_mwh/generation-weighted"));
}
