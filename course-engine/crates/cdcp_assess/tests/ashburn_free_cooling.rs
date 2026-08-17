//! K.1 live bind: Ashburn TMY3 free-cooling hours from cdcp_site + cdcp_metrics,
//! scored as cdcp_assess numeric-range. The key is the lookup, not a letter.

use cdcp_assess::{
    ashburn_tmy3_free_cooling_hours, hours_response, score, score_digest, score_digest_json,
    AssessError, Item, Quantity, Ratio, Response, ASHBURN_LOCATION_ID,
    ASHBURN_TMY3_FREE_COOLING_HOURS, HOURS_UNITS,
};
use cdcp_metrics::take_free_cooling_hours;
use cdcp_site::{engine_root, lookup_id, SiteError};
use std::path::{Path, PathBuf};

/// Full-credit numeric-range ScoreReport pin (kind-level, not item body).
/// `{"earned":1,"full_credit":true,"kind":"numeric-range","out_of":1}`
/// Same pin as `cdcp_wasm` numeric-range dual-path.
const FULL_CREDIT_NUMERIC_PIN: &str =
    "610b51a19742bf708672567fe7d251cdf522db4736a624af52ab139ca84dcf0e";

fn engine() -> PathBuf {
    engine_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("engine root")
}

fn ashburn_hours() -> Ratio {
    let profile =
        lookup_id(&engine(), ASHBURN_LOCATION_ID).unwrap_or_else(|e| panic!("ashburn lookup: {e}"));
    assert_eq!(profile.location.id, ASHBURN_LOCATION_ID);
    let taken = take_free_cooling_hours(profile.climate.free_cooling_hours)
        .unwrap_or_else(|e| panic!("hours must be an integer count: {e}"));
    Ratio::new(taken.num(), taken.den()).expect("metrics ratio fits assess ratio")
}

#[test]
fn live_ashburn_key_is_5734_h_and_scores_full() {
    let hours = ashburn_hours();
    assert_eq!(hours, Ratio::from_int(5734), "vendored Ashburn TMY3 pin");
    let sc = ashburn_tmy3_free_cooling_hours(hours).expect("scenario");
    assert_eq!(sc.name, ASHBURN_TMY3_FREE_COOLING_HOURS);
    assert_eq!(sc.location_id, ASHBURN_LOCATION_ID);
    assert_eq!(sc.item.kind_name(), "numeric-range");
    match &sc.item {
        Item::NumericRange { expected, .. } => {
            assert_eq!(expected.units.as_str(), HOURS_UNITS);
            assert_eq!(expected.value, hours);
        }
        other => panic!("flattened to {other:?}"),
    }
    let ok = hours_response(hours).unwrap();
    let s = score(&sc.item, &ok).unwrap();
    assert!(s.is_full());
    assert_eq!((s.earned(), s.out_of()), (1, 1));
}

#[test]
fn live_key_is_not_four_letters() {
    let sc = ashburn_tmy3_free_cooling_hours(ashburn_hours()).unwrap();
    let json = serde_json::to_string(&sc.item).unwrap();
    assert!(json.contains("numeric-range"), "{json}");
    assert!(json.contains(HOURS_UNITS), "{json}");
    assert!(!json.contains("\"correct\":\"A\""), "{json}");
    assert!(
        !json.contains("\"options\":[\"A\",\"B\",\"C\",\"D\"]"),
        "{json}"
    );
}

#[test]
fn off_by_one_from_live_key_is_zero_credit() {
    let hours = ashburn_hours();
    let sc = ashburn_tmy3_free_cooling_hours(hours).unwrap();
    let miss = hours_response(hours.checked_add(Ratio::from_int(1)).unwrap()).unwrap();
    let s = score(&sc.item, &miss).unwrap();
    assert!(s.is_zero());
    assert_eq!((s.earned(), s.out_of()), (0, 1));
}

#[test]
fn digest_is_deterministic_and_json_dual_path_matches() {
    let hours = ashburn_hours();
    let sc = ashburn_tmy3_free_cooling_hours(hours).unwrap();
    let ok = hours_response(hours).unwrap();
    let a = score_digest(&sc.item, &ok).unwrap();
    let b = score_digest(&sc.item, &ok).unwrap();
    assert_eq!(a, b);
    assert_eq!(a, FULL_CREDIT_NUMERIC_PIN);
    let item_json = serde_json::to_string(&sc.item).unwrap();
    let resp_json = serde_json::to_string(&ok).unwrap();
    assert_eq!(a, score_digest_json(&item_json, &resp_json).unwrap());

    let miss = hours_response(Ratio::from_int(0)).unwrap();
    let zero = score_digest(&sc.item, &miss).unwrap();
    assert_ne!(zero, a);
    assert_eq!(zero.len(), 64);
}

#[test]
fn missing_location_cannot_mint_a_key() {
    let err = lookup_id(&engine(), "atlantis").expect_err("missing");
    match err {
        SiteError::MissingLocation { ref id } => assert_eq!(id, "atlantis"),
        other => panic!("expected MissingLocation, got {other:?}"),
    }
}

#[test]
fn unit_mismatch_from_live_item_is_error() {
    let sc = ashburn_tmy3_free_cooling_hours(ashburn_hours()).unwrap();
    let got =
        Response::numeric_range(Quantity::new(Ratio::from_int(5734), "kWh").unwrap()).unwrap();
    assert!(matches!(
        score(&sc.item, &got),
        Err(AssessError::UnitMismatch { .. })
    ));
}
