//! Answer-key position balance over the drawable single-select bank pool.
//!
//! Extracted from `cdcp_gate/src/gates/answer_key_skew.rs` by
//! `bd-engine-not-gate-ar39.16` (EXTRACT-THEN-DELETE). The assertion belongs
//! beside the bank product; `cdcp_gate` retains only the subcommand dispatcher.

use crate::Bank;
use std::path::Path;
use toml::Value;

pub const NAME: &str = "answer-key-skew";
pub const SUMMARY: &str = "approved answer-key distribution stays within registry band";
const POLICY: &str = "registries/answer_key_skew.toml";
const LETTERS: [&str; 4] = ["A", "B", "C", "D"];

/// The bank-product result mapped to process behavior by the gate dispatcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eval {
    Ok(String),
    Violation(Vec<String>),
    Error(String),
}

fn policy(root: &Path) -> Result<(f64, f64), String> {
    let path = root.join(POLICY);
    let raw =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let table: Value = raw
        .parse()
        .map_err(|e| format!("parse {}: {e}", path.display()))?;
    let row = table
        .get("tolerance")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(Value::as_table)
        .filter(|r| r.get("gate").and_then(Value::as_str) == Some(NAME))
        .ok_or_else(|| format!("{}: missing [[tolerance]] row", path.display()))?;
    let min = row
        .get("min_share")
        .and_then(Value::as_float)
        .ok_or_else(|| "tolerance.min_share must be a number".to_string())?;
    let max = row
        .get("max_share")
        .and_then(Value::as_float)
        .ok_or_else(|| "tolerance.max_share must be a number".to_string())?;
    if !(0.0..=1.0).contains(&min) || !(0.0..=1.0).contains(&max) || min > max {
        return Err("invalid answer-key tolerance band".to_string());
    }
    Ok((min, max))
}

fn summary(counts: &[usize; 4], n: usize) -> String {
    let pct = |count| count as f64 * 100.0 / n as f64;
    format!(
        "approved single-select={n}; A={} ({:.1}%), B={} ({:.1}%), C={} ({:.1}%), D={} ({:.1}%)",
        counts[0],
        pct(counts[0]),
        counts[1],
        pct(counts[1]),
        counts[2],
        pct(counts[2]),
        counts[3],
        pct(counts[3])
    )
}

/// Evaluate the approved single-select pool against the registry row.
pub fn evaluate(root: &Path) -> Eval {
    let (min, max) = match policy(root) {
        Ok(band) => band,
        Err(message) => return Eval::Error(message),
    };
    let bank = match Bank::load_dir(&root.join("bank/items")) {
        Ok(bank) => bank,
        Err(error) => return Eval::Error(format!("load bank/items: {error}")),
    };
    let mut counts = [0usize; 4];
    for item in bank
        .items
        .values()
        .filter(|item| item.is_approved() && item.kind.is_letter_form())
    {
        let Some(slot) = LETTERS.iter().position(|letter| *letter == item.correct) else {
            return Eval::Error(format!("{}: correct key is not A-D", item.id));
        };
        counts[slot] += 1;
    }
    let n = counts.iter().sum::<usize>();
    if n == 0 {
        return Eval::Error("zero approved single-select items (vacuous scan)".to_string());
    }
    let report = summary(&counts, n);
    let outside = LETTERS
        .iter()
        .zip(counts)
        .filter_map(|(letter, count)| {
            let share = count as f64 / n as f64;
            (share < min || share > max).then_some(*letter)
        })
        .collect::<Vec<_>>();
    if outside.is_empty() {
        Eval::Ok(format!(
            "{NAME}: PASS: {report}; band={:.1}%..{:.1}%",
            min * 100.0,
            max * 100.0
        ))
    } else {
        Eval::Violation(vec![format!(
            "{report}; band={:.1}%..{:.1}%; outside={}",
            min * 100.0,
            max * 100.0,
            outside.join(",")
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/answer_key_skew")
            .join(name)
    }

    #[test]
    fn known_bad_fixture_is_red() {
        match evaluate(&fixture("bad")) {
            Eval::Violation(items) => assert!(items[0].contains("outside=A,C,D")),
            other => panic!("known-bad fixture did not go RED: {other:?}"),
        }
    }

    #[test]
    fn uniform_fixture_is_green() {
        match evaluate(&fixture("uniform")) {
            Eval::Ok(text) => assert!(text.contains("A=1 (25.0%)")),
            other => panic!("uniform fixture did not pass: {other:?}"),
        }
    }

    #[test]
    fn zero_approved_single_select_is_an_error() {
        match evaluate(&fixture("retired")) {
            Eval::Error(message) => assert!(message.contains("zero approved single-select")),
            other => panic!("empty drawable pool was not an ERROR: {other:?}"),
        }
    }
}
