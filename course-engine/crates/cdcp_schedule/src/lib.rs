//! Short-interval review + mastery thresholds — the learner law.
//!
//! This is **not** spaced repetition. [`INTERVAL_STEPS`] is `[1, 3]` with a
//! 3-day cap: a short-interval drill ladder, not Anki, not a forgetting curve.
//! Calling it SRS is an overclaim.
//!
//! Browser JS renders and persists; this crate (via `cdcp_wasm`) decides.
//! Persisted cards carry [`STATE_VERSION`]. Unversioned records migrate
//! (`0` → current). An unknown version is [`ScheduleError::UnknownVersion`].
//!
//! Anti-vacuous: a ladder with zero steps, a zero step, a mastery
//! threshold of 0, or an unknown state version is an [`ScheduleError`] —
//! never a silent default.
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

/// Persisted schedule-state version this crate writes today.
///
/// Matches the historical JS `cdcp.srs.v1` `schema_version: 1`.
pub const STATE_VERSION: u32 = 1;

/// Unversioned historical records (missing version field).
///
/// The one migration: [`STATE_VERSION_UNVERSIONED`] → [`STATE_VERSION`].
pub const STATE_VERSION_UNVERSIONED: u32 = 0;

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

#[derive(Debug, Error, PartialEq, Eq, Clone, Copy)]
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
    #[error("unknown schedule state version {0} is an ERROR")]
    UnknownVersion(u32),
    #[error("negative interval is an ERROR — due_at cannot subtract days")]
    NegativeInterval,
    #[error("due_at overflow is an ERROR — now_ms + interval_days * DAY_MS does not fit i64")]
    DueOverflow,
}

/// One persisted review-card record. Version is the record schema, not the ladder.
///
/// Timestamps are millisecond instants (`due_at` / `updated_at` on the JS wire).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleCard {
    pub version: u32,
    pub interval_days: u32,
    pub due_at_ms: i64,
    pub reps: u32,
    pub lapses: u32,
    pub updated_at_ms: i64,
}

impl ScheduleCard {
    /// A card stamped with [`STATE_VERSION`].
    pub fn new(
        interval_days: u32,
        due_at_ms: i64,
        reps: u32,
        lapses: u32,
        updated_at_ms: i64,
    ) -> Self {
        Self {
            version: STATE_VERSION,
            interval_days,
            due_at_ms,
            reps,
            lapses,
            updated_at_ms,
        }
    }
}

/// Accept or migrate a persisted version number.
///
/// - [`STATE_VERSION_UNVERSIONED`] (`0`) → [`STATE_VERSION`] (the one rule)
/// - [`STATE_VERSION`] → unchanged
/// - anything else → [`ScheduleError::UnknownVersion`]
pub fn migrate_state_version(from: u32) -> Result<u32, ScheduleError> {
    match from {
        STATE_VERSION_UNVERSIONED => Ok(STATE_VERSION),
        STATE_VERSION => Ok(STATE_VERSION),
        other => Err(ScheduleError::UnknownVersion(other)),
    }
}

/// Apply [`migrate_state_version`] to a card. Field values are identity.
pub fn migrate_card(card: ScheduleCard) -> Result<ScheduleCard, ScheduleError> {
    Ok(ScheduleCard {
        version: migrate_state_version(card.version)?,
        interval_days: card.interval_days,
        due_at_ms: card.due_at_ms,
        reps: card.reps,
        lapses: card.lapses,
        updated_at_ms: card.updated_at_ms,
    })
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
///
/// Pinned by `tests/fixtures/ladder_seed.json` (`(current, correct) → next`).
pub fn next_interval_days(current_interval_days: i32, correct: bool) -> u32 {
    next_interval_days_with(&INTERVAL_STEPS, current_interval_days, correct)
        .expect("INTERVAL_STEPS is compile-time validated")
}

/// `due_at = now_ms + interval_days * DAY_MS`. `interval_days == 0` → `now_ms`.
///
/// `now_ms` is caller-supplied. This crate does not read the wall clock.
/// Negative `interval_days` is [`ScheduleError::NegativeInterval`].
/// A product that does not fit in `i64` is [`ScheduleError::DueOverflow`]
/// (never saturating wrap — that would be a second due).
/// Pinned by `tests/fixtures/ladder_seed.json` (`due` / `due_known_bad` rows).
pub fn due_at_ms(now_ms: i64, interval_days: i64) -> Result<i64, ScheduleError> {
    if interval_days < 0 {
        return Err(ScheduleError::NegativeInterval);
    }
    let delta = interval_days
        .checked_mul(DAY_MS)
        .ok_or(ScheduleError::DueOverflow)?;
    now_ms.checked_add(delta).ok_or(ScheduleError::DueOverflow)
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
        assert_eq!(validate_thresholds(0, 0), Err(ScheduleError::ZeroThreshold));
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
        assert_eq!(due_at_ms(t0, 1).unwrap(), t0 + DAY_MS);
        assert_eq!(due_at_ms(t0, 3).unwrap(), t0 + 3 * DAY_MS);
        assert_eq!(due_at_ms(t0, 0).unwrap(), t0);
    }

    #[test]
    fn known_bad_negative_interval_is_error() {
        let t0 = 1_700_000_000_000i64;
        assert_eq!(due_at_ms(t0, -1), Err(ScheduleError::NegativeInterval));
        assert_eq!(
            due_at_ms(t0, i64::MIN),
            Err(ScheduleError::NegativeInterval)
        );
        assert!(ScheduleError::NegativeInterval
            .to_string()
            .contains("negative interval"));
    }

    #[test]
    fn known_bad_due_overflow_is_error() {
        assert_eq!(due_at_ms(i64::MAX, 1), Err(ScheduleError::DueOverflow));
        assert_eq!(due_at_ms(0, i64::MAX), Err(ScheduleError::DueOverflow));
        assert!(ScheduleError::DueOverflow
            .to_string()
            .contains("due_at overflow"));
        // interval 0 is identity even at the i64 edge — not overflow.
        assert_eq!(due_at_ms(i64::MAX, 0).unwrap(), i64::MAX);
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

    #[test]
    fn unversioned_state_migrates_to_v1_identity_on_fields() {
        assert_eq!(STATE_VERSION, 1);
        assert_eq!(STATE_VERSION_UNVERSIONED, 0);
        assert_eq!(migrate_state_version(0), Ok(1));
        let raw = ScheduleCard {
            version: 0,
            interval_days: 3,
            due_at_ms: 42,
            reps: 1,
            lapses: 2,
            updated_at_ms: 41,
        };
        let got = migrate_card(raw).unwrap();
        assert_eq!(got.version, STATE_VERSION);
        assert_eq!(got.interval_days, 3);
        assert_eq!(got.due_at_ms, 42);
        assert_eq!(got.reps, 1);
        assert_eq!(got.lapses, 2);
        assert_eq!(got.updated_at_ms, 41);
    }

    #[test]
    fn current_state_version_is_identity() {
        let card = ScheduleCard::new(1, 7, 0, 0, 7);
        assert_eq!(migrate_card(card), Ok(card));
        assert_eq!(migrate_state_version(STATE_VERSION), Ok(STATE_VERSION));
    }

    #[test]
    fn known_bad_unknown_state_version_is_error() {
        assert_eq!(
            migrate_state_version(2),
            Err(ScheduleError::UnknownVersion(2))
        );
        assert_eq!(
            migrate_state_version(99),
            Err(ScheduleError::UnknownVersion(99))
        );
        let future = ScheduleCard {
            version: 2,
            ..ScheduleCard::new(1, 0, 0, 0, 0)
        };
        assert_eq!(migrate_card(future), Err(ScheduleError::UnknownVersion(2)));
        assert!(ScheduleError::UnknownVersion(2)
            .to_string()
            .contains("unknown schedule state version"));
    }
}
