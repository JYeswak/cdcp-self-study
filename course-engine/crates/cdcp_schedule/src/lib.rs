//! Short-interval review + mastery thresholds — the learner law.
//!
//! This is **not** spaced repetition. [`INTERVAL_STEPS`] is `[1, 3]` with a
//! 3-day cap: a short-interval drill ladder, not Anki, not a forgetting curve.
//! Calling it SRS is an overclaim.
//!
//! Browser JS renders and persists; this crate (via `cdcp_wasm`) decides.
//!
//! Anti-vacuous: a ladder with zero steps, a zero step, or a mastery
//! threshold of 0 is an [`ScheduleError`] — never a silent default.
#![forbid(unsafe_code)]

use thiserror::Error;

/// Allowed interval steps in days. Cap is the last entry.
///
/// `[1, 3]` is short-interval review, not spaced repetition.
pub const INTERVAL_STEPS: [u32; 2] = [1, 3];

/// Fixed day length in milliseconds. No DST.
pub const DAY_MS: i64 = 86_400_000;

/// Practiced threshold in parts per thousand (800 = 0.80).
pub const PRACTICED_MILLI: u32 = 800;

/// Mastered per-attempt threshold in parts per thousand (900 = 0.90).
pub const MASTERED_MILLI: u32 = 900;

/// Minimum gap between two mastery-qualifying attempts.
pub const MASTERED_MIN_GAP_MS: i64 = DAY_MS;

/// Full-scale milli (1.0).
pub const RATIO_MILLI_MAX: u32 = 1000;

const fn steps_ok(steps: &[u32]) -> bool {
    if steps.is_empty() {
        return false;
    }
    let mut i = 0;
    let mut prev = 0u32;
    while i < steps.len() {
        if steps[i] == 0 {
            return false;
        }
        if i > 0 && steps[i] <= prev {
            return false;
        }
        prev = steps[i];
        i += 1;
    }
    true
}

const _: () = assert!(
    steps_ok(&INTERVAL_STEPS),
    "INTERVAL_STEPS must be non-empty, strictly increasing, every step > 0"
);
const _: () = assert!(PRACTICED_MILLI > 0, "practiced threshold 0 is an ERROR");
const _: () = assert!(MASTERED_MILLI > 0, "mastered threshold 0 is an ERROR");
const _: () = assert!(
    PRACTICED_MILLI <= MASTERED_MILLI,
    "practiced must not exceed mastered"
);
const _: () = assert!(DAY_MS > 0);
const _: () = assert!(MASTERED_MIN_GAP_MS > 0);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScheduleError {
    #[error("empty interval ladder is an ERROR — zero steps is not a schedule")]
    EmptyLadder,
    #[error("interval step 0 is an ERROR — a zero-day step is not a step")]
    ZeroStep,
    #[error("interval ladder must be strictly increasing")]
    NotIncreasing,
    #[error("mastery threshold 0 is an ERROR — a zero bar is not a bar")]
    ZeroThreshold,
    #[error("practiced threshold exceeds mastered threshold")]
    PracticedExceedsMastered,
}

/// One quiz attempt used by the mastered law.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewAttempt {
    pub ratio_milli: u32,
    pub at_ms: i64,
}

/// Validate an arbitrary ladder. Empty / zero / non-increasing is RED.
pub fn validate_steps(steps: &[u32]) -> Result<(), ScheduleError> {
    if steps.is_empty() {
        return Err(ScheduleError::EmptyLadder);
    }
    let mut prev: Option<u32> = None;
    for &s in steps {
        if s == 0 {
            return Err(ScheduleError::ZeroStep);
        }
        if let Some(p) = prev {
            if s <= p {
                return Err(ScheduleError::NotIncreasing);
            }
        }
        prev = Some(s);
    }
    Ok(())
}

/// Validate mastery thresholds. Either bar at 0 is RED.
pub fn validate_thresholds(practiced_milli: u32, mastered_milli: u32) -> Result<(), ScheduleError> {
    if practiced_milli == 0 || mastered_milli == 0 {
        return Err(ScheduleError::ZeroThreshold);
    }
    if practiced_milli > mastered_milli {
        return Err(ScheduleError::PracticedExceedsMastered);
    }
    Ok(())
}

/// Validate the compiled-in schedule. Const-asserted; also a runtime belt.
pub fn validate_schedule() -> Result<(), ScheduleError> {
    validate_steps(&INTERVAL_STEPS)?;
    validate_thresholds(PRACTICED_MILLI, MASTERED_MILLI)?;
    Ok(())
}

/// Next interval in days for `steps`.
///
/// Law (matches the historical JS 1d/3d ladder when `steps == [1, 3]`):
/// - wrong → first step
/// - correct → first step strictly greater than `current`; else last step (cap)
pub fn next_interval_days_with(
    steps: &[u32],
    current_interval_days: i32,
    correct: bool,
) -> Result<u32, ScheduleError> {
    validate_steps(steps)?;
    let cur = if current_interval_days > 0 {
        current_interval_days as u32
    } else {
        0
    };
    if !correct {
        return Ok(steps[0]);
    }
    for &step in steps {
        if cur < step {
            return Ok(step);
        }
    }
    Ok(*steps.last().expect("validate_steps rejected empty"))
}

/// Next interval using the compiled-in [`INTERVAL_STEPS`].
pub fn next_interval_days(current_interval_days: i32, correct: bool) -> u32 {
    next_interval_days_with(&INTERVAL_STEPS, current_interval_days, correct)
        .expect("INTERVAL_STEPS is compile-time validated")
}

/// `due_at = now + interval_days * DAY_MS`. `interval_days == 0` → `now`.
pub fn due_at_ms(interval_days: u32, now_ms: i64) -> i64 {
    now_ms.saturating_add(i64::from(interval_days).saturating_mul(DAY_MS))
}

/// Convert a 0..=1 ratio to parts per thousand. Non-finite / negative → 0.
pub fn ratio_to_milli(ratio: f64) -> u32 {
    if !ratio.is_finite() || ratio <= 0.0 {
        return 0;
    }
    if ratio >= 1.0 {
        return RATIO_MILLI_MAX;
    }
    (ratio * 1000.0).round() as u32
}

/// practiced: `best_milli ≥ PRACTICED_MILLI` (800).
pub fn is_practiced_milli(best_milli: u32) -> bool {
    best_milli >= PRACTICED_MILLI
}

/// practiced from a 0..=1 ratio.
pub fn is_practiced_ratio(best_ratio: f64) -> bool {
    is_practiced_milli(ratio_to_milli(best_ratio))
}

/// mastered: ≥2 attempts with `ratio_milli ≥ MASTERED_MILLI` whose timestamps
/// are ≥ [`MASTERED_MIN_GAP_MS`] apart (earliest vs latest qualifying).
pub fn is_mastered(attempts: &[ReviewAttempt]) -> bool {
    let mut times: Vec<i64> = attempts
        .iter()
        .filter(|a| a.ratio_milli >= MASTERED_MILLI)
        .map(|a| a.at_ms)
        .collect();
    if times.len() < 2 {
        return false;
    }
    times.sort_unstable();
    times[times.len() - 1].saturating_sub(times[0]) >= MASTERED_MIN_GAP_MS
}

/// First step of the compiled ladder (the "again" / new-card interval).
pub fn first_step_days() -> u32 {
    INTERVAL_STEPS[0]
}

/// Cap of the compiled ladder (last step).
pub fn cap_days() -> u32 {
    INTERVAL_STEPS[INTERVAL_STEPS.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_schedule_is_valid() {
        validate_schedule().unwrap();
        assert_eq!(INTERVAL_STEPS, [1, 3]);
        assert_eq!(PRACTICED_MILLI, 800);
        assert_eq!(MASTERED_MILLI, 900);
        assert_eq!(DAY_MS, 86_400_000);
        assert_eq!(MASTERED_MIN_GAP_MS, DAY_MS);
    }

    #[test]
    fn anti_vacuous_empty_ladder_is_error() {
        assert_eq!(
            next_interval_days_with(&[], 0, true),
            Err(ScheduleError::EmptyLadder)
        );
        assert_eq!(validate_steps(&[]), Err(ScheduleError::EmptyLadder));
    }

    #[test]
    fn anti_vacuous_zero_step_is_error() {
        assert_eq!(validate_steps(&[0, 3]), Err(ScheduleError::ZeroStep));
        assert_eq!(
            next_interval_days_with(&[0], 0, true),
            Err(ScheduleError::ZeroStep)
        );
    }

    #[test]
    fn anti_vacuous_zero_threshold_is_error() {
        assert_eq!(
            validate_thresholds(0, 900),
            Err(ScheduleError::ZeroThreshold)
        );
        assert_eq!(
            validate_thresholds(800, 0),
            Err(ScheduleError::ZeroThreshold)
        );
        assert_eq!(
            validate_thresholds(0, 0),
            Err(ScheduleError::ZeroThreshold)
        );
    }

    #[test]
    fn next_interval_matches_historical_1d_3d_ladder() {
        assert_eq!(next_interval_days(0, false), 1);
        assert_eq!(next_interval_days(0, true), 1);
        assert_eq!(next_interval_days(1, true), 3);
        assert_eq!(next_interval_days(3, true), 3);
        assert_eq!(next_interval_days(3, false), 1);
        assert_eq!(next_interval_days(1, false), 1);
        assert_eq!(next_interval_days(-5, true), 1);
    }

    #[test]
    fn known_bad_moved_cap_changes_verdict() {
        // If INTERVAL_STEPS were [1, 99], 1d + correct would become 99, not 3.
        // This is the mechanism the wasm export uses: the ladder is data.
        let moved = next_interval_days_with(&[1, 99], 1, true).unwrap();
        assert_eq!(moved, 99);
        assert_ne!(moved, next_interval_days(1, true));
        assert_eq!(next_interval_days(1, true), 3);
        assert_eq!(
            next_interval_days_with(&[1, 3], 1, true).unwrap(),
            next_interval_days(1, true)
        );
    }

    #[test]
    fn due_at_arithmetic() {
        let t0 = 1_700_000_000_000i64;
        assert_eq!(due_at_ms(1, t0), t0 + DAY_MS);
        assert_eq!(due_at_ms(3, t0), t0 + 3 * DAY_MS);
        assert_eq!(due_at_ms(0, t0), t0);
    }

    #[test]
    fn practiced_boundaries() {
        assert!(!is_practiced_milli(799));
        assert!(is_practiced_milli(800));
        assert!(is_practiced_milli(1000));
        assert!(!is_practiced_ratio(0.79));
        assert!(is_practiced_ratio(0.80));
        assert!(!is_practiced_ratio(f64::NAN));
    }

    #[test]
    fn mastered_spacing_law() {
        let t0 = 1_700_000_000_000i64;
        assert!(!is_mastered(&[]));
        assert!(!is_mastered(&[ReviewAttempt {
            ratio_milli: 900,
            at_ms: t0
        }]));
        assert!(!is_mastered(&[
            ReviewAttempt {
                ratio_milli: 900,
                at_ms: t0
            },
            ReviewAttempt {
                ratio_milli: 900,
                at_ms: t0 + DAY_MS - 1
            },
        ]));
        assert!(is_mastered(&[
            ReviewAttempt {
                ratio_milli: 900,
                at_ms: t0
            },
            ReviewAttempt {
                ratio_milli: 900,
                at_ms: t0 + DAY_MS
            },
        ]));
        // intervening low score does not break a spaced qualifying pair
        assert!(is_mastered(&[
            ReviewAttempt {
                ratio_milli: 900,
                at_ms: t0
            },
            ReviewAttempt {
                ratio_milli: 500,
                at_ms: t0 + 1000
            },
            ReviewAttempt {
                ratio_milli: 1000,
                at_ms: t0 + DAY_MS + 1000
            },
        ]));
        // 890 milli = 0.89 — practiced, not mastered
        assert!(!is_mastered(&[
            ReviewAttempt {
                ratio_milli: 890,
                at_ms: t0
            },
            ReviewAttempt {
                ratio_milli: 890,
                at_ms: t0 + DAY_MS
            },
        ]));
    }

    #[test]
    fn ratio_to_milli_rounds() {
        assert_eq!(ratio_to_milli(0.8), 800);
        assert_eq!(ratio_to_milli(0.9), 900);
        assert_eq!(ratio_to_milli(0.0), 0);
        assert_eq!(ratio_to_milli(1.0), 1000);
        assert_eq!(ratio_to_milli(1.2), 1000);
        assert_eq!(ratio_to_milli(-0.1), 0);
    }
}
