//! One owner-engineer capstone scenario (K.1).
//!
//! Land is closed at the compiled Ashburn pin. The number the energy model
//! takes — TMY3 dry-bulb hours at or below the `cdcp_data` economizer
//! high-limit — is what justifies ordering air-side economizer as
//! first-order equipment. This module scores that number as
//! [`Item::NumericRange`]. It does not flatten the answer to A–D.
//!
//! The expected hours are an input ([`Ratio`]). Callers bind them from
//! `cdcp_site` + `cdcp_metrics::take_free_cooling_hours` so this crate
//! stays integer/rational and does not grow a filesystem or `f64` edge.
//! Scoring itself is [`crate::score`] / [`crate::score_digest_json`] —
//! the same dual-path payload `cdcp_wasm` already exports.

use crate::error::AssessError;
use crate::ratio::Ratio;
use crate::types::{Item, Quantity, Response, Tolerance, ToleranceKind};

/// First-class scenario name. Not a four-letter item id.
pub const ASHBURN_TMY3_FREE_COOLING_HOURS: &str = "ashburn-tmy3-free-cooling-hours";

/// Compiled location id the scenario is bound to.
pub const ASHBURN_LOCATION_ID: &str = "ashburn";

/// Hour unit on the wire. Empty is a bare number (schema ERROR).
pub const HOURS_UNITS: &str = "h";

/// Owner-engineer scenario: one typed item plus the site it was bound to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    /// [`ASHBURN_TMY3_FREE_COOLING_HOURS`].
    pub name: &'static str,
    /// [`ASHBURN_LOCATION_ID`].
    pub location_id: &'static str,
    /// Numeric-range item. Never a letter MCQ.
    pub item: Item,
}

/// Build the Ashburn free-cooling-hours item from an already-rational count.
///
/// Tolerance is absolute `0/1` hour: TMY3 hour counts are integers. An
/// off-by-one is zero credit, not "close enough." Negative hours are
/// [`AssessError::NegativeQuantity`], not a score of zero.
pub fn ashburn_tmy3_free_cooling_hours(hours: Ratio) -> Result<Scenario, AssessError> {
    if hours.is_negative() {
        return Err(AssessError::NegativeQuantity);
    }
    let item = Item::numeric_range(
        Quantity::new(hours, HOURS_UNITS)?,
        Tolerance::new(ToleranceKind::Absolute, Ratio::from_int(0))?,
    )?;
    Ok(Scenario {
        name: ASHBURN_TMY3_FREE_COOLING_HOURS,
        location_id: ASHBURN_LOCATION_ID,
        item,
    })
}

/// Same constructor from an integer hour count.
pub fn ashburn_tmy3_free_cooling_hours_int(hours: i64) -> Result<Scenario, AssessError> {
    ashburn_tmy3_free_cooling_hours(Ratio::from_int(hours))
}

/// Learner response in the same unit as the item (`h`).
pub fn hours_response(hours: Ratio) -> Result<Response, AssessError> {
    Response::numeric_range(Quantity::new(hours, HOURS_UNITS)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score;

    #[test]
    fn name_and_kind_are_not_letters() {
        let sc = ashburn_tmy3_free_cooling_hours_int(5734).unwrap();
        assert_eq!(sc.name, ASHBURN_TMY3_FREE_COOLING_HOURS);
        assert_eq!(sc.location_id, ASHBURN_LOCATION_ID);
        assert_eq!(sc.item.kind_name(), "numeric-range");
        match &sc.item {
            Item::NumericRange {
                expected,
                tolerance,
            } => {
                assert_eq!(expected.units.as_str(), HOURS_UNITS);
                assert_eq!(expected.value, Ratio::from_int(5734));
                assert_eq!(tolerance.kind, ToleranceKind::Absolute);
                assert_eq!(tolerance.magnitude, Ratio::from_int(0));
            }
            other => panic!("must not flatten to {other:?}"),
        }
    }

    #[test]
    fn exact_hours_full_credit_off_by_one_zero() {
        let sc = ashburn_tmy3_free_cooling_hours_int(5734).unwrap();
        let hit = hours_response(Ratio::from_int(5734)).unwrap();
        assert!(score(&sc.item, &hit).unwrap().is_full());
        for n in [5733_i64, 5735] {
            let miss = hours_response(Ratio::from_int(n)).unwrap();
            let s = score(&sc.item, &miss).unwrap();
            assert!(s.is_zero(), "{n} h must be zero credit");
        }
    }

    #[test]
    fn negative_hours_are_error_not_zero() {
        let err = ashburn_tmy3_free_cooling_hours_int(-1).unwrap_err();
        assert_eq!(err, AssessError::NegativeQuantity);
    }

    #[test]
    fn wrong_units_are_error_not_zero() {
        let sc = ashburn_tmy3_free_cooling_hours_int(5734).unwrap();
        let got =
            Response::numeric_range(Quantity::new(Ratio::from_int(5734), "h/yr").unwrap()).unwrap();
        assert!(matches!(
            score(&sc.item, &got),
            Err(AssessError::UnitMismatch { .. })
        ));
    }
}
