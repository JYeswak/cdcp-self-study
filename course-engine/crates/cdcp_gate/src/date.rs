//! Minimal civil-date arithmetic (no external crate; the workspace pins its deps
//! and this needs exactly two operations: "what is today" and "is this ISO date
//! in the past").
//!
//! There is deliberately NO environment override for "today". A gate whose clock
//! can be moved by an env var is a gate with a documented bypass; tests pass the
//! date in as an argument instead.

use std::time::{SystemTime, UNIX_EPOCH};

/// (year, month, day) — proleptic Gregorian.
pub type Ymd = (i32, u32, u32);

/// Howard Hinnant's `civil_from_days`.
fn civil_from_days(z: i64) -> Ymd {
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

/// Today's civil date in UTC.
pub fn today() -> Ymd {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_from_days(secs.div_euclid(86_400))
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(y) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Parse a strict `YYYY-MM-DD`. Rejects `2026-2-3`, `2026-13-01`, `2026-02-30`.
pub fn parse_ymd(s: &str) -> Result<Ymd, String> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return Err(format!("{s:?} is not a strict YYYY-MM-DD date"));
    }
    if !b
        .iter()
        .enumerate()
        .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    {
        return Err(format!("{s:?} is not a strict YYYY-MM-DD date"));
    }
    let y: i32 = s[0..4].parse().map_err(|_| format!("{s:?}: bad year"))?;
    let m: u32 = s[5..7].parse().map_err(|_| format!("{s:?}: bad month"))?;
    let d: u32 = s[8..10].parse().map_err(|_| format!("{s:?}: bad day"))?;
    if !(1..=12).contains(&m) {
        return Err(format!("{s:?}: month {m} out of range"));
    }
    let dim = days_in_month(y, m);
    if d < 1 || d > dim {
        return Err(format!("{s:?}: day {d} out of range for month {m}"));
    }
    Ok((y, m, d))
}

/// True when `a` is strictly before `b`.
pub fn before(a: Ymd, b: Ymd) -> bool {
    a < b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_day_zero_is_1970_01_01() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn known_days() {
        assert_eq!(civil_from_days(19_000), (2022, 1, 8));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn parses_strict_iso_only() {
        assert_eq!(parse_ymd("2026-08-13").unwrap(), (2026, 8, 13));
        for bad in [
            "2026-8-13",
            "26-08-13",
            "2026/08/13",
            "2026-13-01",
            "2026-02-30",
            "2025-02-29",
            "",
            "never",
            "2026-08-13 ",
        ] {
            assert!(parse_ymd(bad).is_err(), "{bad:?} must not parse");
        }
        assert_eq!(parse_ymd("2024-02-29").unwrap(), (2024, 2, 29));
    }

    #[test]
    fn ordering() {
        assert!(before((2026, 1, 1), (2026, 1, 2)));
        assert!(!before((2026, 1, 2), (2026, 1, 2)));
        assert!(before((2025, 12, 31), (2026, 1, 1)));
    }

    #[test]
    fn today_is_sane() {
        let (y, m, d) = today();
        assert!((2024..2100).contains(&y), "clock looks wrong: {y}");
        assert!((1..=12).contains(&m));
        assert!((1..=31).contains(&d));
    }
}
