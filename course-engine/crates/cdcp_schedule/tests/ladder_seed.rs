//! Committed ladder + injected-time fixture for `cdcp_schedule`.
//!
//! The file is the pin. Inline asserts that do not load it cannot keep the
//! law honest: an empty `cases` / `known_bad` / `due` set is RED, a missing
//! file is RED, and disk bytes must match `include_str!`.

use cdcp_schedule::{
    due_at_ms, is_mastered, next_interval_days, next_interval_days_with, validate_steps,
    ReviewAttempt, ScheduleError, DAY_MS, INTERVAL_STEPS,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const RAW: &str = include_str!("fixtures/ladder_seed.json");
const FIXTURE_REL: &str = "tests/fixtures/ladder_seed.json";

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_REL)
}

fn load() -> Value {
    let path = fixture_path();
    let disk = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing committed fixture {}: {e} — empty scan set is an ERROR",
            path.display()
        )
    });
    assert!(
        !disk.trim().is_empty(),
        "{FIXTURE_REL} is empty — empty fixture is an ERROR, not a pass"
    );
    assert_eq!(
        disk, RAW,
        "{FIXTURE_REL} on disk must match include_str! bytes"
    );
    serde_json::from_str(&disk).unwrap_or_else(|e| panic!("{FIXTURE_REL} is not JSON: {e}"))
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    v.get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{FIXTURE_REL} missing array `{key}`"))
}

fn i64_field(v: &Value, key: &str) -> i64 {
    v.get(key)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("missing i64 `{key}` in {v}"))
}

fn u32_field(v: &Value, key: &str) -> u32 {
    let n = i64_field(v, key);
    u32::try_from(n).unwrap_or_else(|_| panic!("`{key}` out of u32 range: {n}"))
}

fn bool_field(v: &Value, key: &str) -> bool {
    v.get(key)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("missing bool `{key}` in {v}"))
}

fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string `{key}` in {v}"))
}

fn parse_error(name: &str) -> ScheduleError {
    match name {
        "EmptyLadder" => ScheduleError::EmptyLadder,
        "ZeroStep" => ScheduleError::ZeroStep,
        "NotIncreasing" => ScheduleError::NotIncreasing,
        "ZeroThreshold" => ScheduleError::ZeroThreshold,
        "PracticedExceedsMastered" => ScheduleError::PracticedExceedsMastered,
        other => panic!("unknown ScheduleError in fixture: {other}"),
    }
}

fn steps_of(v: &Value) -> Vec<u32> {
    v.get("steps")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("known_bad row missing `steps`: {v}"))
        .iter()
        .map(|s| {
            let n = s.as_u64().unwrap_or_else(|| panic!("step not u64: {s}"));
            u32::try_from(n).unwrap_or_else(|_| panic!("step out of u32: {n}"))
        })
        .collect()
}

#[test]
fn fixture_is_present_and_non_empty() {
    let v = load();
    assert_eq!(str_field(&v, "schema"), "cdcp_schedule.ladder_seed.v1");
    let ladder = arr(&v, "ladder");
    assert!(
        !ladder.is_empty(),
        "fixture ladder [] is an ERROR — that is the known-bad, not the compiled law"
    );
    let compiled: Vec<u32> = ladder
        .iter()
        .map(|s| u32::try_from(s.as_u64().expect("ladder step")).expect("ladder u32"))
        .collect();
    assert_eq!(compiled, INTERVAL_STEPS);
    assert_eq!(i64_field(&v, "day_ms"), DAY_MS);
    assert!(
        !arr(&v, "cases").is_empty(),
        "empty cases is an ERROR, not a pass"
    );
    assert!(
        !arr(&v, "due").is_empty(),
        "empty due table is an ERROR, not a pass"
    );
    assert!(
        !arr(&v, "known_bad").is_empty(),
        "empty known_bad is an ERROR, not a pass"
    );
    assert!(
        !arr(&v, "mastered").is_empty(),
        "empty mastered table is an ERROR, not a pass"
    );
}

#[test]
fn ladder_cases_pin_next_interval() {
    let v = load();
    let cases = arr(&v, "cases");
    let mut saw = Vec::new();
    for case in cases {
        let id = str_field(case, "id");
        let current = i32::try_from(i64_field(case, "current_interval"))
            .unwrap_or_else(|_| panic!("{id}: current_interval out of i32"));
        let correct = bool_field(case, "correct");
        let want = u32_field(case, "next_interval");
        let got = next_interval_days(current, correct);
        assert_eq!(
            got, want,
            "case {id}: next_interval_days({current}, {correct})"
        );
        let via = next_interval_days_with(&INTERVAL_STEPS, current, correct)
            .expect("compiled ladder is valid");
        assert_eq!(
            via, want,
            "case {id}: next_interval_days_with compiled ladder"
        );
        saw.push((current, correct, want));
    }
    for required in [
        (0, false, 1u32),
        (0, true, 1),
        (1, true, 3),
        (3, true, 3),
        (3, false, 1),
        (1, false, 1),
    ] {
        assert!(
            saw.contains(&required),
            "fixture must pin {required:?}; have {saw:?}"
        );
    }
}

#[test]
fn known_bad_empty_and_zero_ladder_are_red() {
    let v = load();
    let rows = arr(&v, "known_bad");
    let mut saw_empty = false;
    let mut saw_zero = false;
    for row in rows {
        let id = str_field(row, "id");
        let steps = steps_of(row);
        let err_name = str_field(row, "error");
        let want = parse_error(err_name);
        assert_eq!(
            next_interval_days_with(&steps, 0, true),
            Err(parse_error(err_name)),
            "known_bad {id}: next_interval_days_with"
        );
        assert_eq!(
            validate_steps(&steps),
            Err(parse_error(err_name)),
            "known_bad {id}: validate_steps"
        );
        if steps.is_empty() {
            saw_empty = true;
            assert_eq!(want, ScheduleError::EmptyLadder);
        }
        if steps.iter().any(|&s| s == 0) {
            saw_zero = true;
            assert_eq!(want, ScheduleError::ZeroStep);
        }
    }
    assert!(saw_empty, "known_bad must include empty ladder []");
    assert!(saw_zero, "known_bad must include a zero step");
}

#[test]
fn due_at_uses_injected_now_ms() {
    let v = load();
    let fixture_now = i64_field(&v, "now_ms");
    for row in arr(&v, "due") {
        let id = str_field(row, "id");
        let interval = u32_field(row, "interval_days");
        let now = i64_field(row, "now_ms");
        let want = i64_field(row, "due_at_ms");
        let got = due_at_ms(interval, now);
        assert_eq!(got, want, "due {id}: due_at_ms({interval}, {now})");
        assert_eq!(
            due_at_ms(interval, now),
            due_at_ms(interval, now),
            "due {id}: same injected now_ms must be deterministic"
        );
        if interval == 0 {
            assert_eq!(got, now, "due {id}: interval 0 is identity on injected now");
        }
    }
    // A second injected clock must move due_at by the same delta — wall clock
    // is not consulted, so a frozen now_ms cannot pick up elapsed real time.
    let shifted = fixture_now + 12_345;
    assert_eq!(due_at_ms(1, shifted), shifted + DAY_MS);
    assert_eq!(due_at_ms(0, shifted), shifted);
}

#[test]
fn mastered_uses_injected_at_ms() {
    let v = load();
    for row in arr(&v, "mastered") {
        let id = str_field(row, "id");
        let want = bool_field(row, "mastered");
        let attempts: Vec<ReviewAttempt> = row
            .get("attempts")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("mastered {id}: missing attempts"))
            .iter()
            .map(|a| ReviewAttempt {
                ratio_milli: u32_field(a, "ratio_milli"),
                at_ms: i64_field(a, "at_ms"),
            })
            .collect();
        assert_eq!(is_mastered(&attempts), want, "mastered {id}");
    }
}

#[test]
fn crate_source_has_no_wall_clock() {
    let src = include_str!("../src/lib.rs");
    for needle in [
        "SystemTime",
        "Instant::now",
        "Utc::now",
        "Local::now",
        "unix_now",
        "std::time::now",
    ] {
        assert!(
            !src.contains(needle),
            "cdcp_schedule must not read the wall clock ({needle})"
        );
    }
    assert!(
        src.contains("now_ms"),
        "due_at must take a caller-supplied now_ms"
    );
    assert!(
        src.contains("at_ms"),
        "ReviewAttempt must take a caller-supplied at_ms"
    );
}
