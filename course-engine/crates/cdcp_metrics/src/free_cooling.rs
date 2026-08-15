//! Free-cooling hours consumed from `cdcp_data` / `cdcp_site`.
//!
//! This crate does **not** re-derive the economizer high-limit. Hours
//! enter as the integer count already computed by
//! [`cdcp_data::free_cooling_hours`] (which `cdcp_site::Climate` also
//! stores). Comparison is a [`crate::Ratio`], never an `f64` equality.

use crate::error::MetricsError;
use crate::ratio::Ratio;

/// Consume hours already computed by `cdcp_data::free_cooling_hours`
/// or `cdcp_site::Climate::free_cooling_hours`.
///
/// The upstream crates store a count in `f64`. This function accepts
/// that count at the crate boundary and returns a rational. A non-integer
/// or negative input is [`MetricsError::NonIntegerHours`].
pub fn take_free_cooling_hours(computed_hours: f64) -> Result<Ratio, MetricsError> {
    if !computed_hours.is_finite() || computed_hours < 0.0 {
        return Err(MetricsError::NonIntegerHours(computed_hours.to_string()));
    }
    if computed_hours > i64::MAX as f64 {
        return Err(MetricsError::Overflow);
    }
    let n = computed_hours as i64;
    if n as f64 != computed_hours {
        return Err(MetricsError::NonIntegerHours(computed_hours.to_string()));
    }
    Ok(Ratio::from_int(n))
}

/// Hours in a TMY3 record with dry-bulb at or below the threshold
/// `cdcp_data` already owns. Do not re-derive that threshold here.
pub fn free_cooling_hours(csv: &str, location_id: &str) -> Result<Ratio, MetricsError> {
    let hours = cdcp_data::free_cooling_hours(csv, location_id)
        .map_err(|e| MetricsError::FreeCooling(e.to_string()))?;
    take_free_cooling_hours(hours)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_count_becomes_ratio() {
        assert_eq!(take_free_cooling_hours(4.0).unwrap(), Ratio::from_int(4));
        assert_eq!(take_free_cooling_hours(0.0).unwrap(), Ratio::from_int(0));
    }

    #[test]
    fn non_integer_is_error() {
        let err = take_free_cooling_hours(4.5).unwrap_err();
        assert!(matches!(err, MetricsError::NonIntegerHours(_)));
    }

    #[test]
    fn negative_is_error() {
        let err = take_free_cooling_hours(-1.0).unwrap_err();
        assert!(matches!(err, MetricsError::NonIntegerHours(_)));
    }
}
