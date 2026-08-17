//! `due_at_ms(now_ms, interval_days)` — injected time only.
//!
//! Overflow and negative interval are ERROR. An empty known-bad set is
//! itself an ERROR (a never-checked deliverable reports like a pass).

use cdcp_schedule::{due_at_ms, ScheduleError, DAY_MS};

/// Cases that MUST stay RED. Empty = vacuous pass.
const KNOWN_BAD_DUE: &[(i64, i64, ScheduleError)] = &[
    (1_700_000_000_000, -1, ScheduleError::NegativeInterval),
    (0, -5, ScheduleError::NegativeInterval),
    (0, i64::MIN, ScheduleError::NegativeInterval),
    (i64::MAX, 1, ScheduleError::DueOverflow),
    (0, i64::MAX, ScheduleError::DueOverflow),
    (i64::MAX - DAY_MS + 1, 1, ScheduleError::DueOverflow),
];

#[test]
fn known_bad_set_is_non_empty() {
    assert!(
        !KNOWN_BAD_DUE.is_empty(),
        "empty known_bad is an ERROR, not a pass"
    );
}

#[test]
fn happy_path_uses_injected_now_and_compiled_day() {
    let t0 = 1_700_000_000_000i64;
    assert_eq!(due_at_ms(t0, 0).unwrap(), t0);
    assert_eq!(due_at_ms(t0, 1).unwrap(), t0 + DAY_MS);
    assert_eq!(due_at_ms(t0, 3).unwrap(), t0 + 3 * DAY_MS);
}

#[test]
fn known_bad_negative_and_overflow_are_red() {
    let mut saw_neg = false;
    let mut saw_overflow = false;
    for &(now, interval, ref want) in KNOWN_BAD_DUE {
        let got = due_at_ms(now, interval);
        assert_eq!(
            got,
            Err(*want),
            "due_at_ms({now}, {interval}) must be {want:?}"
        );
        match want {
            ScheduleError::NegativeInterval => saw_neg = true,
            ScheduleError::DueOverflow => saw_overflow = true,
            other => panic!("known-bad table has unexpected error {other:?}"),
        }
    }
    assert!(saw_neg, "known-bad must include a negative interval");
    assert!(saw_overflow, "known-bad must include an overflow");
}

#[test]
fn crate_source_does_not_saturate_or_read_wall_clock() {
    let src = include_str!("../src/lib.rs");
    for needle in [
        "SystemTime",
        "Instant::now",
        "Utc::now",
        "Local::now",
        "saturating_add",
        "saturating_mul",
    ] {
        assert!(
            !src.contains(needle),
            "due_at must not saturate or read the wall clock ({needle})"
        );
    }
    assert!(
        src.contains("checked_add") && src.contains("checked_mul"),
        "due_at must use checked arithmetic so overflow is ERROR"
    );
}
